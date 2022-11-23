use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification},
    CompletionItem, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams,
};
use std::{collections::HashMap, io};
use tree_sitter::Parser;

use crate::{fileitem::FileItem, parser::parse_document, spitem::to_completion};

pub struct Store {
    /// Any documents the server has handled, indexed by their URL
    documents: HashMap<String, FileItem>,

    parser: Parser,
}

impl Store {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_sourcepawn::language())
            .expect("Error loading SourcePawn grammar");

        Store {
            documents: HashMap::new(),
            parser,
        }
    }

    pub fn handle_open_document(
        &mut self,
        _connection: &lsp_server::Connection,
        n: lsp_server::Notification,
    ) -> Result<(), io::Error> {
        let params: DidOpenTextDocumentParams = n.extract(DidOpenTextDocument::METHOD).unwrap();
        let uri_string = params.text_document.uri.path();
        let text = params.text_document.text;
        let mut file_item = FileItem {
            uri: uri_string.to_string(),
            text,
            ..Default::default()
        };
        match parse_document(&mut self.parser, &mut file_item) {
            Err(err) => eprintln!("Failed to parse {} because of {}", uri_string, err),
            Ok(()) => {}
        }
        self.documents.insert(uri_string.to_string(), file_item);

        Ok(())
    }

    pub fn handle_change_document(
        &mut self,
        _connection: &lsp_server::Connection,
        n: lsp_server::Notification,
    ) -> Result<(), io::Error> {
        let params: DidChangeTextDocumentParams = n.extract(DidChangeTextDocument::METHOD).unwrap();
        let uri_string = params.text_document.uri.path().to_string();
        let text = params.content_changes[0].text.to_string();
        let mut file_item = self.documents.get_mut(&uri_string).unwrap();
        file_item.text = text;
        file_item.sp_items.clear();
        match parse_document(&mut self.parser, &mut file_item) {
            Err(err) => eprintln!("Failed to parse {} because of {}", uri_string, err),
            Ok(()) => {}
        }

        Ok(())
    }

    pub fn provide_completions(&self, params: &CompletionParams) -> CompletionResponse {
        let mut results: Vec<CompletionItem> = Vec::new();
        for (_, file_item) in self.documents.iter() {
            for sp_item in file_item.sp_items.iter() {
                let res = to_completion(sp_item, params);
                if res.is_some() {
                    results.push(res.unwrap());
                }
            }
        }
        CompletionResponse::Array(results)
    }
}
