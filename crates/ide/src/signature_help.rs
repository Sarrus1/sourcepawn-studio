use base_db::FilePosition;
use hir::{DefResolution, HasSource, Semantics};
use ide_db::{Documentation, RootDatabase};
use preprocessor::u_pos_to_s_pos;
use syntax::{utils::lsp_position_to_ts_point, TSKind};

#[derive(Debug)]
pub struct SignatureHelp {
    pub doc: Option<Documentation>,
    pub signature: String,
    pub active_parameter: Option<u32>,
    pub parameters: Vec<String>,
}

pub(crate) fn signature_help(
    db: &RootDatabase,
    FilePosition {
        file_id,
        mut position,
    }: FilePosition,
) -> Option<SignatureHelp> {
    let sema = &Semantics::new(db);
    let tree = sema.parse(file_id);
    let preprocessing_results = sema.preprocess_file(file_id);

    // TODO: If the range is some we are in a macro call, try to resolve it.
    if u_pos_to_s_pos(
        preprocessing_results.args_map(),
        preprocessing_results.offsets(),
        &mut position,
    )
    .is_some()
    {
        return None;
    }
    let point = lsp_position_to_ts_point(&position);
    let root_node = tree.root_node();

    let node = root_node.descendant_for_point_range(point, point)?;
    let mut parent = node.parent()?;

    for depth in 0..3 {
        if TSKind::from(&parent) != TSKind::call_arguments {
            if depth == 2 {
                // Not in a call expression
                return None;
            }
            parent = node.parent()?;
        } else {
            break;
        }
    }
    let active_parameter = parent
        .children(&mut parent.walk())
        .filter(|c| TSKind::from(c) == TSKind::anon_COMMA && c.end_position() <= point)
        .count() as u32;
    let call_expression = parent.parent()?;
    let callee = call_expression.child_by_field_name("function")?;
    let def = match TSKind::from(&callee) {
        TSKind::identifier => sema.find_def(file_id, &callee)?,
        TSKind::field_access => {
            let field = callee.child_by_field_name("field")?;
            sema.find_def(file_id, &field)?
        }
        _ => return None,
    };

    let DefResolution::Function(func) = def else {
        return None;
    };

    let file_id = def.file_id(db);
    let tree = sema.parse(file_id);
    let source = sema.preprocessed_text(file_id);
    let node = func.source(db, &tree)?;
    SignatureHelp {
        doc: Documentation::from_node(node.value, source.as_bytes()),
        signature: func.render(db)?,
        active_parameter: active_parameter.into(),
        parameters: func.parameters(db),
    }
    .into()
}
