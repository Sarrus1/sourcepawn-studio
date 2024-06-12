use ide::WideEncoding;
use lsp_types::{
    notification::Notification, request::Request, PositionEncodingKind, TextDocumentIdentifier, Url,
};
use serde::{Deserialize, Serialize};

use crate::line_index::PositionEncoding;

pub fn negotiated_encoding(caps: &lsp_types::ClientCapabilities) -> PositionEncoding {
    let client_encodings = match &caps.general {
        Some(general) => general.position_encodings.as_deref().unwrap_or_default(),
        None => &[],
    };

    for enc in client_encodings {
        if enc == &PositionEncodingKind::UTF8 {
            return PositionEncoding::Utf8;
        } else if enc == &PositionEncodingKind::UTF32 {
            return PositionEncoding::Wide(WideEncoding::Utf32);
        }
        // NB: intentionally prefer just about anything else to utf-16.
    }

    PositionEncoding::Wide(WideEncoding::Utf16)
}

pub enum PreprocessedDocument {}

impl Request for PreprocessedDocument {
    type Params = PreprocessedDocumentParams;
    type Result = String;
    const METHOD: &'static str = "sourcepawn-studio/preprocessedDocument";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PreprocessedDocumentParams {
    pub text_document: Option<TextDocumentIdentifier>,
}

pub enum SyntaxTree {}

impl Request for SyntaxTree {
    type Params = SyntaxTreeParams;
    type Result = String;
    const METHOD: &'static str = "sourcepawn-studio/syntaxTree";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeParams {
    pub text_document: Option<TextDocumentIdentifier>,
}

pub enum ItemTree {}

impl Request for ItemTree {
    type Params = ItemTreeParams;
    type Result = String;
    const METHOD: &'static str = "sourcepawn-studio/itemTree";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemTreeParams {
    pub text_document: Option<TextDocumentIdentifier>,
}

pub enum AnalyzerStatus {}

impl Request for AnalyzerStatus {
    type Params = AnalyzerStatusParams;
    type Result = String;
    const METHOD: &'static str = "sourcepawn-studio/analyzerStatus";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerStatusParams {
    pub text_document: Option<TextDocumentIdentifier>,
}

pub enum ProjectMainPath {}

impl Request for ProjectMainPath {
    type Params = ProjectMainPathParams;
    type Result = Url;
    const METHOD: &'static str = "sourcepawn-studio/projectMainPath";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMainPathParams {
    pub uri: Option<Url>,
}

pub enum ProjectsGraphviz {}

impl Request for ProjectsGraphviz {
    type Params = ProjectsGraphvizParams;
    type Result = String;
    const METHOD: &'static str = "sourcepawn-studio/projectsGraphviz";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsGraphvizParams {
    pub text_document: Option<TextDocumentIdentifier>,
}

pub enum ServerStatusNotification {}

impl Notification for ServerStatusNotification {
    type Params = ServerStatusParams;
    const METHOD: &'static str = "sourcepawn-studio/serverStatus";
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
    const METHOD: &'static str = "sourcepawn-studio/spcompStatus";
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct SpcompStatusParams {
    pub quiescent: bool,
}

pub enum HoverRequest {}

impl Request for HoverRequest {
    type Params = lsp_types::HoverParams;
    type Result = Option<Hover>;
    const METHOD: &'static str = lsp_types::request::HoverRequest::METHOD;
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Hover {
    #[serde(flatten)]
    pub hover: lsp_types::Hover,
    pub actions: Vec<CommandLinkGroup>,
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct CommandLinkGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub commands: Vec<CommandLink>,
}

// LSP v3.15 Command does not have a `tooltip` field, vscode supports one.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct CommandLink {
    #[serde(flatten)]
    pub command: lsp_types::Command,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ClientCommandOptions {
    pub commands: Vec<String>,
}
