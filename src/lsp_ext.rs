use lsp_types::{notification::Notification, request::Request, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

pub enum PreprocessedDocument {}

impl Request for PreprocessedDocument {
    type Params = PreprocessedDocumentParams;
    type Result = String;
    const METHOD: &'static str = "sourcepawn-lsp/preprocessedDocument";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PreprocessedDocumentParams {
    pub text_document: Option<TextDocumentIdentifier>,
}

pub enum ServerStatusNotification {}

impl Notification for ServerStatusNotification {
    type Params = ServerStatusParams;
    const METHOD: &'static str = "sourcepawn-lsp/serverStatus";
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct ServerStatusParams {
    pub health: Health,
    pub quiescent: bool,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Health {
    Ok,
    Warning,
    Error,
}

pub enum SpcompStatusNotification {}

impl Notification for SpcompStatusNotification {
    type Params = SpcompStatusParams;
    const METHOD: &'static str = "sourcepawn-lsp/spcompStatus";
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct SpcompStatusParams {
    pub quiescent: bool,
}
