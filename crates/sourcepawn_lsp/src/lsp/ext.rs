use ide::WideEncoding;
use lsp_types::PositionEncodingKind;
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

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Hover {
    #[serde(flatten)]
    pub hover: lsp_types::Hover,
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
