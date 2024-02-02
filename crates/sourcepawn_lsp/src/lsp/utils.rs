//! Utilities for LSP-related boilerplate code.
use std::{mem, ops::Range, sync::Arc};

use lsp_types::request::Request;

use crate::{
    line_index::{LineEndings, LineIndex, PositionEncoding},
    lsp_ext, GlobalState,
};

use super::from_proto;

impl GlobalState {
    pub(crate) fn show_message(
        &mut self,
        typ: lsp_types::MessageType,
        message: String,
        show_open_log_button: bool,
    ) {
        self.send_notification::<lsp_types::notification::ShowMessage>(
            lsp_types::ShowMessageParams { typ, message },
        )
        // TODO: Hardcoded false
        // match self.config.open_server_logs() && show_open_log_button  {
        // match false  {
        //     true => self.send_request::<lsp_types::request::ShowMessageRequest>(
        //         lsp_types::ShowMessageRequestParams {
        //             typ,
        //             message,
        //             actions: Some(vec![lsp_types::MessageActionItem {
        //                 title: "Open server logs".to_owned(),
        //                 properties: Default::default(),
        //             }]),
        //         },
        //         |this, resp| {
        //             let lsp_server::Response { error: None, result: Some(result), .. } = resp
        //             else { return };
        //             if let Ok(Some(_item)) = crate::from_json::<
        //                 <lsp_types::request::ShowMessageRequest as lsp_types::request::Request>::Result,
        //             >(
        //                 lsp_types::request::ShowMessageRequest::METHOD, &result
        //             ) {
        //                 this.send_notification::<lsp_ext::OpenServerLogs>(());
        //             }
        //         },
        //     ),
        //     false => self.send_notification::<lsp_types::notification::ShowMessage>(
        //         lsp_types::ShowMessageParams {
        //             typ,
        //             message,
        //         },
        //     ),
        // }
    }
}

pub(crate) fn apply_document_changes(
    encoding: PositionEncoding,
    file_contents: impl FnOnce() -> String,
    mut content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
) -> String {
    // Skip to the last full document change, as it invalidates all previous changes anyways.
    let mut start = content_changes
        .iter()
        .rev()
        .position(|change| change.range.is_none())
        .map(|idx| content_changes.len() - idx - 1)
        .unwrap_or(0);

    let mut text: String = match content_changes.get_mut(start) {
        // peek at the first content change as an optimization
        Some(lsp_types::TextDocumentContentChangeEvent {
            range: None, text, ..
        }) => {
            let text = mem::take(text);
            start += 1;

            // The only change is a full document update
            if start == content_changes.len() {
                return text;
            }
            text
        }
        Some(_) => file_contents(),
        // we received no content changes
        None => return file_contents(),
    };

    let mut line_index = LineIndex {
        // the index will be overwritten in the bottom loop's first iteration
        index: Arc::new(ide::LineIndex::new(&text)),
        // We don't care about line endings here.
        endings: LineEndings::Unix,
        encoding,
    };

    // The changes we got must be applied sequentially, but can cross lines so we
    // have to keep our line index updated.
    // Some clients (e.g. Code) sort the ranges in reverse. As an optimization, we
    // remember the last valid line in the index and only rebuild it if needed.
    // The VFS will normalize the end of lines to `\n`.
    let mut index_valid = !0u32;
    for change in content_changes {
        // The None case can't happen as we have handled it above already
        if let Some(range) = change.range {
            if index_valid <= range.end.line {
                *Arc::make_mut(&mut line_index.index) = ide::LineIndex::new(&text);
            }
            index_valid = range.start.line;
            if let Ok(range) = from_proto::text_range(&line_index, range) {
                text.replace_range(Range::<usize>::from(range), &change.text);
            }
        }
    }
    text
}
