use std::collections::HashMap;

use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};

use crate::spitem::get_items_from_position;

use super::FeatureRequest;

pub fn provide_rename(request: FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
    let items = get_items_from_position(
        &request.store,
        request.params.text_document_position.position,
        request
            .params
            .text_document_position
            .text_document
            .uri
            .clone(),
    );
    if items.len() != 1 {
        return None;
    }
    let item = items[0].read().unwrap();

    let mut changes = HashMap::new();
    changes.insert(
        (*item.uri()).clone(),
        vec![TextEdit {
            range: item.range()?,
            new_text: request.params.new_name.clone(),
        }],
    );
    for reference in item.references()? {
        let edit = TextEdit {
            range: reference.range,
            new_text: request.params.new_name.clone(),
        };
        if let Some(uri_changes) = changes.get_mut(&reference.uri) {
            uri_changes.push(edit)
        } else {
            changes.insert((*reference.uri).clone(), vec![edit]);
        }
    }

    Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}
