mod defaults;
mod documentation;
mod includes;
mod item;

use std::panic::AssertUnwindSafe;

use base_db::FilePosition;
use hir::{DefResolution, Field, Function, HasSource, LocalDef, Property, Semantics};
use hir_def::{DefDatabase, FieldId, FunctionKind};
use ide_db::{Documentation, RootDatabase, SymbolKind};
pub use item::{CompletionItem, CompletionKind};
use itertools::Itertools;
use lazy_static::lazy_static;
use lsp_types::Url;
use paths::AbsPathBuf;
use preprocessor::db::PreprocDatabase;
use regex::Regex;
use smol_str::{SmolStr, ToSmolStr};
use syntax::TSKind;
use vfs::FileId;

use crate::{
    completion::{
        defaults::get_default_completions,
        documentation::{get_doc_completion, is_documentation_start},
        includes::{get_include_completions, is_include_statement},
    },
    events::{event_name, events_completions},
    hover::{render_def, Render},
};

pub fn completions(
    db: &RootDatabase,
    pos: FilePosition,
    trigger_character: Option<char>,
    include_directories: Vec<AbsPathBuf>,
    file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Url>,
    events_game_name: Option<&str>,
) -> Option<Vec<CompletionItem>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(pos.file_id);
    let preprocessed_text = preprocessing_results.preprocessed_text();

    let tree = sema.parse(pos.file_id);

    let u_pos = pos.offset;
    let s_pos = preprocessing_results
        .source_map()
        .closest_s_position_always(u_pos);
    let raw_s_pos: usize = s_pos.into();
    let split_line = preprocessed_text
        .split_at_checked(raw_s_pos)
        .unwrap_or((&preprocessed_text, ""));
    let split_line = (
        split_line.0.rsplit('\n').next()?,
        split_line.1.split('\n').next()?,
    );
    if let Some(include_st) = is_include_statement(split_line.0, split_line.1) {
        return get_include_completions(
            db,
            include_st,
            pos.file_id,
            include_directories,
            file_id_to_url,
        );
    }
    if trigger_character == Some('/') || trigger_character == Some('<') {
        // We are past the include statement check, so we can return early.
        return None;
    }

    if is_documentation_start(split_line.0, split_line.1) {
        return get_doc_completion(db, pos, &preprocessed_text);
    }
    if trigger_character == Some('*') {
        // We are past the doc comment check, so we can return early.
        return None;
    }

    lazy_static! {
        pub static ref NEW_REGEX: Regex = Regex::new(r"new\s+$").unwrap();
    }

    let token = if NEW_REGEX.is_match(split_line.0) && trigger_character == Some(' ') {
        "foo()"
    } else if trigger_character == Some(' ') {
        // The space trigger character is only for constructors.
        return None;
    } else {
        "foo"
    };

    let new_source_code = [
        &preprocessed_text[..raw_s_pos],
        token,
        &preprocessed_text[raw_s_pos..],
    ]
    .concat();
    // TODO: Use the edit to update the tree
    // tree.edit(&edit);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_sourcepawn::language())
        .unwrap();
    let new_tree = parser.parse(new_source_code.as_bytes(), None)?;

    let root_node = new_tree.root_node();
    // get the node before the cursor
    let node = root_node
        .descendant_for_byte_range(raw_s_pos.saturating_add(1), raw_s_pos.saturating_add(1))?;

    // Check if we are in an event such as "EventHook"
    if event_name(&node, &new_source_code).is_some() {
        return events_completions(events_game_name).into();
    }
    if trigger_character == Some('"') {
        return None;
    }

    if matches!(
        TSKind::from(node),
        TSKind::comment | TSKind::string_literal | TSKind::char_literal
    ) {
        // No completions in comments or strings
        return None;
    }

    let mut container = node.parent()?;
    // If the node does not have a parent we are at the root, nothing to resolve.
    while !matches!(
        TSKind::from(container),
        TSKind::function_definition
            | TSKind::enum_struct_method
            | TSKind::r#enum
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
            | TSKind::typedef
            | TSKind::source_file
            | TSKind::field_access
            | TSKind::new_expression
            | TSKind::array_scope_access
            | TSKind::scope_access
            | TSKind::comment
            | TSKind::string_literal
    ) {
        if let Some(candidate) = container.parent() {
            container = candidate;
        } else {
            break;
        }
    }
    let mut add_defaults = false;
    let mut local_context = true;

    log::debug!("completion container kind: {:?}", container.kind());
    log::debug!("completion node: {:?}", node.kind());
    let defs = match TSKind::from(container) {
        TSKind::function_definition
        | TSKind::methodmap_method
        | TSKind::methodmap_property_getter
        | TSKind::methodmap_property_method
        | TSKind::methodmap_property_setter
        | TSKind::enum_struct_method
        | TSKind::methodmap_method_constructor
        | TSKind::methodmap_method_destructor
            if !is_triggered_by_scope_or_field_access(trigger_character) =>
        {
            add_defaults = true;
            if let Some(res) = in_function_completion(container, tree, sema, pos) {
                res
            } else {
                // If we can't resolve the function we are in, the AST is probably broken
                // return all global defs
                sema.defs_in_scope(pos.file_id)
                    .into_iter()
                    .filter(|it| !matches!(it, DefResolution::Local(_) | DefResolution::Global(_)))
                    .collect_vec()
            }
        }
        TSKind::field_access if is_triggered_by_scope_or_field_access(trigger_character) => {
            field_access_completions(container, sema, pos, false)?
        }
        TSKind::scope_access | TSKind::array_scope_access
            if is_triggered_by_scope_or_field_access(trigger_character) =>
        {
            field_access_completions(container, sema, pos, true)?
        }
        TSKind::new_expression if !is_triggered_by_scope_or_field_access(trigger_character) => sema
            .defs_in_scope(pos.file_id)
            .into_iter()
            .filter_map(|it| match it {
                DefResolution::Methodmap(it) => Some(it),
                _ => None,
            })
            .flat_map(|it| db.methodmap_data(it.id()).constructor().cloned())
            .map(Function::from)
            .map(|it| it.into())
            .collect_vec(),
        TSKind::comment | TSKind::string_literal => return None,
        _ if !is_triggered_by_scope_or_field_access(trigger_character) => {
            local_context = false;
            sema.defs_in_scope(pos.file_id)
                .into_iter()
                .filter(|it| !matches!(it, DefResolution::Local(_)))
                .collect_vec()
        }
        _ => Default::default(),
    };

    let mut res = Vec::new();

    defs.into_iter().for_each(|def| match &def {
        DefResolution::Function(it) => {
            let data = sema.db.function_data(it.id());
            match data.kind {
                FunctionKind::Def | FunctionKind::Native => {
                    res.push(CompletionItem {
                        label: it.name(db).to_string().into(),
                        kind: SymbolKind::Function.into(),
                        data: Some(def),
                        deprecated: data.deprecated,
                        ..Default::default()
                    });
                }
                FunctionKind::Forward => {
                    res.push(CompletionItem {
                        label: it.name(db).to_string().into(),
                        kind: SymbolKind::Forward.into(),
                        data: Some(def.clone()),
                        deprecated: data.deprecated,
                        ..Default::default()
                    });
                    if local_context {
                        // Add the snippet only in a global context
                        return;
                    }
                    res.push(CompletionItem {
                        label: it.name(db).to_string().into(),
                        kind: CompletionKind::Snippet,
                        data: Some(def),
                        deprecated: data.deprecated,
                        ..Default::default()
                    });
                }
            }
        }
        DefResolution::Macro(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Macro.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::EnumStruct(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Struct.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Methodmap(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Methodmap.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Property(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Property.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Enum(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Enum.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Variant(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Variant.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Typedef(it) => {
            let Some(name) = it.name(db) else {
                return;
            };
            let name = name.to_smolstr();
            res.push(CompletionItem {
                label: name.clone(),
                kind: SymbolKind::Typedef.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });

            if local_context {
                // Add the snippet only in a global context
                return;
            }
            res.push(CompletionItem {
                label: name,
                kind: CompletionKind::Snippet,
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Typeset(it) => {
            let name: SmolStr = it.name(db).to_string().into();
            res.push(CompletionItem {
                label: name.clone(),
                kind: SymbolKind::Typeset.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });

            if local_context {
                // Add the snippets only in a global context
                return;
            }
            res.extend(it.children(db).into_iter().flat_map(|child| {
                Some(CompletionItem {
                    label: name.clone(),
                    kind: CompletionKind::Snippet,
                    data: Some(child.into()),
                    deprecated: child.is_deprecated(db),
                    ..Default::default()
                })
            }))
        }
        DefResolution::Functag(it) => {
            let Some(name) = it.name(db) else {
                return;
            };
            res.push(CompletionItem {
                label: { name.to_smolstr() },
                kind: SymbolKind::Functag.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });

            if local_context {
                // Add the snippet only in a global context
                return;
            }
            res.push(CompletionItem {
                label: name.to_smolstr(),
                kind: CompletionKind::Snippet,
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Funcenum(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Funcenum.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });

            if local_context {
                // Add the snippets only in a global context
                return;
            }
            res.extend(it.children(db).into_iter().flat_map(|child| {
                Some(CompletionItem {
                    label: it.name(db).to_string().into(),
                    kind: CompletionKind::Snippet,
                    data: Some(child.into()),
                    deprecated: child.is_deprecated(db),
                    ..Default::default()
                })
            }));
        }
        DefResolution::Struct(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Struct.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::StructField(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Field.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Field(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Field.into(),
                data: Some(def.clone()),
                deprecated: it.is_deprecated(db),
                ..Default::default()
            });
        }
        DefResolution::Global(it) => {
            res.push(CompletionItem {
                label: it.name(db).to_string().into(),
                kind: SymbolKind::Global.into(),
                data: Some(def),
                ..Default::default()
            });
        }
        DefResolution::Local(LocalDef { name, .. }) => {
            res.push(CompletionItem {
                label: {
                    let Some(name) = name else {
                        return;
                    };
                    name.to_smolstr()
                },
                kind: SymbolKind::Local.into(),
                data: Some(def),
                ..Default::default()
            });
        }
        DefResolution::File(_) => (),
    });

    if add_defaults {
        res.extend(get_default_completions(local_context));
    }

    res.into()
}

fn is_triggered_by_scope_or_field_access(trigger_character: Option<char>) -> bool {
    // A ':' triggered a completion but it was not for a scope access. Do not suggest anything here.
    // https://github.com/Sarrus1/sourcepawn-studio/issues/442
    // https://github.com/Sarrus1/sourcepawn-studio/issues/443
    let Some(trigger_character) = trigger_character else {
        return false;
    };
    trigger_character == '.' || trigger_character == ':'
}

/// Get the target node of a field access node, given the parent of the field node.
///
/// # Justification
/// Consider the following example: `foo.bar.baz`.
/// This is parsed such as:
/// ```
///     foo.bar.baz
///       /     \
///   foo.bar   baz
///   /    \
/// foo    bar
/// ```
///
/// This function takes the `foo` node and searches for the `bar` node.
/// The `bar` node will be used to resolve the type of the target of `bar.baz`, in order
/// to determine what are the possible completions that can take the place of baz.
/// See https://github.com/Sarrus1/sourcepawn-studio/issues/443.
///
/// Without this function, completion breaks for nested field access. It would not be needed
/// if we represented the AST such as below, because we would only have to go to the parent.
/// However this would introduce other complications when lowering.
/// ```
///     foo.bar.baz
///       /     \
///     foo   bar.baz
///            /    \
///          bar    baz
/// ```
///
/// # Arguments
/// * `target` - The closest ancestor `field_access` node.
fn get_previous_field_access_node(target: Option<tree_sitter::Node>) -> Option<tree_sitter::Node> {
    let target = target?;
    let field_name = match TSKind::from(target) {
        TSKind::identifier => return Some(target),
        TSKind::scope_access => "scope",
        TSKind::array_indexed_access => "array",
        TSKind::field_access => "field",
        TSKind::call_expression => "function",
        _ => return None,
    };
    get_previous_field_access_node(target.child_by_field_name(field_name))
}

fn field_access_completions(
    container: tree_sitter::Node,
    sema: &Semantics<RootDatabase>,
    pos: FilePosition,
    is_scope: bool,
) -> Option<Vec<DefResolution>> {
    let tree = sema.parse(pos.file_id);
    let source = sema.preprocessed_text(pos.file_id);
    let target = container.child_by_field_name(if is_scope { "scope" } else { "target" })?;
    let target = tree
        .root_node()
        .descendant_for_byte_range(target.start_byte(), target.end_byte());
    let target = get_previous_field_access_node(target)?;
    let def = sema.find_type_def(pos.file_id, target)?;
    let def = match def {
        DefResolution::Function(def) => def.return_type_def(sema.db)?,
        _ => def,
    };
    let target_text = target.utf8_text(source.as_bytes()).ok()?;
    Some(match def {
        DefResolution::Methodmap(it) if target_text == "this" => {
            let data = sema.db.methodmap_data(it.id());
            let mut res = data
                .methods()
                .map(Function::from)
                .map(|it| it.into())
                .collect_vec();
            res.extend(data.properties().map(Property::from).map(|it| it.into()));
            res
        }
        DefResolution::Methodmap(it)
            if !is_scope && target_text == it.name(sema.db).to_string() =>
        {
            let data = sema.db.methodmap_data(it.id());
            data.static_methods()
                .map(Function::from)
                .map(|it| it.into())
                .collect_vec()
        }
        DefResolution::Methodmap(it) => {
            let data = sema.db.methodmap_data(it.id());
            let mut res = data
                .methods()
                .map(Function::from)
                .map(|it| it.into())
                .collect_vec();
            res.extend(data.properties().map(Property::from).map(|it| it.into()));
            res
        }
        DefResolution::EnumStruct(it) if target_text == "this" => {
            let data = sema.db.enum_struct_data(it.id());
            let mut res = data
                .methods()
                .map(Function::from)
                .map(|it| it.into())
                .collect_vec();
            res.extend(
                data.fields()
                    .map(|id| FieldId {
                        parent: it.id(),
                        local_id: id,
                    })
                    .map(Field::from)
                    .map(|it| it.into()),
            );
            res
        }
        DefResolution::EnumStruct(it) => {
            let data = sema.db.enum_struct_data(it.id());
            let mut res = data
                .methods()
                .map(Function::from)
                .map(|it| it.into())
                .collect_vec();
            res.extend(
                data.fields()
                    .map(|id| FieldId {
                        parent: it.id(),
                        local_id: id,
                    })
                    .map(Field::from)
                    .map(|it| it.into()),
            );
            res
        }
        _ => return None,
    })
}

fn in_function_completion(
    container: tree_sitter::Node,
    tree: base_db::Tree,
    sema: &Semantics<RootDatabase>,
    pos: FilePosition,
) -> Option<Vec<DefResolution>> {
    // Trick to get the function node in the original tree.
    // We get the byte range of the name node, as it is the same in both trees.
    let name = if TSKind::from(container) == TSKind::methodmap_property_method {
        container
            .children(&mut container.walk())
            .find_map(|child| {
                if matches!(
                    TSKind::from(child),
                    TSKind::methodmap_property_getter | TSKind::methodmap_property_setter
                ) {
                    child.child_by_field_name("name")
                } else {
                    None
                }
            })?
    } else {
        container.child_by_field_name("name")?
    };
    let name = tree
        .root_node()
        .descendant_for_byte_range(name.start_byte(), name.end_byte())?;
    let container = if TSKind::from(container) == TSKind::methodmap_property_method {
        container
    } else {
        name.parent()?
    };
    let def = sema.find_def(pos.file_id, &name)?;
    let DefResolution::Function(def) = def else {
        return None;
    };
    let body_node = container.child_by_field_name("body")?;
    Some(sema.defs_in_function_scope(pos, def.id(), body_node))
}

pub fn resolve_completion(
    db: &RootDatabase,
    def: DefResolution,
    mut item: lsp_types::CompletionItem,
) -> Option<lsp_types::CompletionItem> {
    if item.kind == Some(lsp_types::CompletionItemKind::SNIPPET) {
        match def {
            DefResolution::Typedef(it) => item.insert_text = it.as_snippet(db),
            DefResolution::Function(it) => {
                // Forward
                item.insert_text = it.as_snippet(db);
            }
            DefResolution::Functag(it) => item.insert_text = it.as_snippet(db),
            _ => unreachable!("unexpected completion kind: {:?}", item.kind),
        }
        item.insert_text_format = Some(lsp_types::InsertTextFormat::SNIPPET);
    }
    let file_id = def.file_id(db);
    let source = db.preprocessed_text(file_id);
    let tree = db.parse(file_id);
    if let Some(def_node) = def.clone().source(db, &tree).map(|it| it.value) {
        if let Some(docs) = Documentation::from_node(def_node, source.as_bytes()) {
            item.documentation = Some(docs.into());
        }
    }
    if let Some(Render::String(render)) = render_def(db, def) {
        item.detail = Some(render);
    }

    item.into()
}
