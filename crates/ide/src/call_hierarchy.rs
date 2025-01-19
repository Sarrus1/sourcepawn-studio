use base_db::FilePosition;
use fxhash::FxHashMap;
use hir::{DefResolution, Function, FunctionType, HasSource, Semantics};
use hir_def::DefDatabase;
use ide_db::{CallItem, IncomingCallItem, OutgoingCallItem, RootDatabase, SymbolKind};
use itertools::Itertools;
use lazy_static::lazy_static;
use line_index::TextRange;
use preprocessor::db::PreprocDatabase;
use smol_str::ToSmolStr;
use streaming_iterator::StreamingIterator;
use syntax::{utils::ts_range_to_text_range, TSKind};
use tree_sitter::QueryCursor;

pub(crate) fn call_hierarchy_prepare(
    db: &RootDatabase,
    mut fpos: FilePosition,
) -> Option<Vec<CallItem>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(fpos.file_id);
    let tree = sema.parse(fpos.file_id);
    let root_node = tree.root_node();

    if sema.find_macro_def(&fpos).is_some() {
        return None;
    }

    fpos.offset = preprocessing_results
        .source_map()
        .closest_s_position_always(fpos.offset);

    let node =
        root_node.descendant_for_byte_range(fpos.raw_offset_usize(), fpos.raw_offset_usize())?;

    let def = sema.find_def(fpos.file_id, &node)?;
    let DefResolution::Function(func) = def else {
        return None;
    };

    vec![func_to_call_item(sema, func)?].into()
}

fn func_to_call_item(sema: &Semantics<RootDatabase>, func: Function) -> Option<CallItem> {
    let def: DefResolution = func.into();
    let file_id = def.file_id(sema.db);
    let tree = sema.parse(file_id);
    let source_node = func.source(sema.db, &tree)?.value;
    let preprocessing_result = sema.preprocess_file(file_id);
    let source = preprocessing_result.preprocessed_text();
    let name_node = source_node.child_by_field_name("name");
    let res = CallItem {
        name: source_node.utf8_text(source.as_bytes()).ok()?.to_smolstr(),
        kind: match func.kind(sema.db) {
            FunctionType::Function => SymbolKind::Function,
            FunctionType::Constructor => SymbolKind::Constructor,
            FunctionType::Destructor => SymbolKind::Destructor,
            FunctionType::Method | FunctionType::Getter | FunctionType::Setter => {
                SymbolKind::Method
            }
        },
        deprecated: func.is_deprecated(sema.db),
        details: func.signature(&source_node, &source),
        file_id,
        full_range: preprocessing_result
            .source_map()
            .closest_u_range_always(ts_range_to_text_range(&source_node.range())),
        focus_range: name_node.map(|n| {
            preprocessing_result
                .source_map()
                .closest_u_range_always(ts_range_to_text_range(&n.range()))
        }),
        data: Some(func),
    };
    Some(res)
}

pub(crate) fn call_hierarchy_incoming(
    db: &RootDatabase,
    func: Function,
) -> Option<Vec<IncomingCallItem>> {
    // TODO: There is quite a lot of back and forth between u_range and s_range in this function, in order to comply with
    // the Semantics API. While this is not ideal, it's really a low priority to optimize it, as call hierarchy request are
    // rare anyways.
    let sema = Semantics::new(db);
    let def: DefResolution = func.into();
    let source_file_id = def.file_id(db);
    let source_tree = db.parse(source_file_id);
    let source_preprocessing_results = db.preprocess_file(source_file_id);
    let source_node = func.source(db, &source_tree)?.value;
    let name_source_node = source_node.child_by_field_name("name")?;
    let u_name_range = source_preprocessing_results
        .source_map()
        .closest_u_range_always(ts_range_to_text_range(&name_source_node.range()));
    let source_offset = u_name_range.start();

    let (_, references) = sema.find_references_from_pos(FilePosition {
        file_id: source_file_id,
        offset: source_offset,
    })?;
    let mut res: FxHashMap<Function, Vec<TextRange>> = FxHashMap::default();
    let _ = references
        .into_iter()
        .flat_map(|frange| {
            if frange.file_id == def.file_id(db) && frange.range == u_name_range {
                // Do not include the function definition
                return None;
            }
            let file_id = frange.file_id;
            let tree = sema.parse(file_id);
            let preprocessing_results = sema.preprocess_file(file_id);
            let offset: u32 = preprocessing_results
                .source_map()
                .closest_s_position_always(frange.range.start())
                .into();
            let node = tree
                .root_node()
                .descendant_for_byte_range(offset as usize, offset as usize)?;

            let mut container = node.parent()?;
            while !matches!(
                TSKind::from(container),
                TSKind::function_definition
                    | TSKind::enum_struct_method
                    | TSKind::methodmap_native
                    | TSKind::methodmap_native_constructor
                    | TSKind::methodmap_native_destructor
                    | TSKind::methodmap_method
                    | TSKind::methodmap_method_constructor
                    | TSKind::methodmap_method_destructor
                    | TSKind::methodmap_property_getter
                    | TSKind::methodmap_property_setter
                    | TSKind::methodmap_property_native
                    | TSKind::methodmap_property_method
            ) {
                if let Some(candidate) = container.parent() {
                    container = candidate;
                } else {
                    break;
                }
            }
            // Make sure we did not reach the top of the tree.
            let _ = container.parent()?;
            let DefResolution::Function(func) =
                sema.find_def(file_id, &container.child_by_field_name("name")?)?
            else {
                return None;
            };
            res.entry(func)
                .and_modify(|v| v.push(frange.range))
                .or_insert(vec![frange.range]);
            Some(())
        })
        .collect_vec();

    res.into_iter()
        .flat_map(|(func, ranges)| {
            Some(IncomingCallItem {
                call_item: func_to_call_item(&sema, func)?,
                ranges,
            })
        })
        .collect_vec()
        .into()
}

pub(crate) fn call_hierarchy_outgoing(
    db: &RootDatabase,
    func: Function,
) -> Option<Vec<OutgoingCallItem>> {
    let sema = Semantics::new(db);
    let def: DefResolution = func.into();
    let file_id = def.file_id(db);
    let tree = db.parse(file_id);
    let preprocessing_results = db.preprocess_file(file_id);
    let source = preprocessing_results.preprocessed_text();
    let source_node = func.source(db, &tree)?.value;
    lazy_static! {
        static ref CALL_QUERY: tree_sitter::Query = tree_sitter::Query::new(
            &tree_sitter_sourcepawn::language(),
            "(call_expression) @call_expression"
        )
        .expect("Could not build identifier query.");
    }

    let mut res: FxHashMap<Function, Vec<TextRange>> = FxHashMap::default();

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.captures(&CALL_QUERY, source_node, source.as_bytes());
    while let Some((match_, _)) = matches.next() {
        for c in match_.captures {
            let Some(mut node) = c.node.child_by_field_name("function") else {
                continue;
            };
            match TSKind::from(&node) {
                TSKind::identifier => (),
                TSKind::field_access => {
                    let Some(node_) = node.child_by_field_name("field") else {
                        continue;
                    };
                    node = node_;
                }
                _ => continue,
            }
            let u_range = preprocessing_results
                .source_map()
                .closest_u_range_always(ts_range_to_text_range(&node.range()));
            let Some(DefResolution::Function(func)) = sema.find_def(file_id, &node) else {
                continue;
            };
            res.entry(func)
                .and_modify(|v| v.push(u_range))
                .or_insert(vec![u_range]);
        }
    }

    res.into_iter()
        .flat_map(|(func, ranges)| {
            Some(OutgoingCallItem {
                call_item: func_to_call_item(&sema, func)?,
                ranges,
            })
        })
        .collect_vec()
        .into()
}
