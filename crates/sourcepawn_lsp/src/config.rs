//! Config used by the language server.
//!
//! We currently get this config from `initialize` LSP request, which is not the
//! best way to do it, but was the simplest thing we could implement.

use ide::{DiagnosticsConfig, HoverConfig, HoverDocFormat};
use itertools::Itertools;
use lsp_types::{ClientCapabilities, MarkupKind};
use paths::AbsPathBuf;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

use crate::{line_index::PositionEncoding, lsp::ext::negotiated_encoding};

macro_rules! try_ {
    ($expr:expr) => {
        || -> _ { Some($expr) }()
    };
}
macro_rules! try_or {
    ($expr:expr, $or:expr) => {
        try_!($expr).unwrap_or($or)
    };
}

macro_rules! try_or_def {
    ($expr:expr) => {
        try_!($expr).unwrap_or_default()
    };
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ConfigData {
    pub include_directories: Vec<PathBuf>,
    pub spcomp_path: PathBuf,
    pub linter_arguments: Vec<String>,
    pub disable_syntax_linter: bool,
}

impl ConfigData {
    pub fn from_json(value: serde_json::Value, errors: &mut Vec<serde_json::Error>) -> Self {
        let options = match serde_json::from_value(value) {
            Ok(new_options) => new_options,
            Err(err) => {
                errors.push(err);
                None
            }
        };

        options.unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    /// The workspace roots as registered by the LSP client
    workspace_roots: Vec<PathBuf>,
    caps: lsp_types::ClientCapabilities,
    root_path: AbsPathBuf,
    data: ConfigData,
    is_visual_studio_code: bool,
}

impl Config {
    pub fn new(
        root_path: AbsPathBuf,
        caps: ClientCapabilities,
        workspace_roots: Vec<PathBuf>,
        is_visual_studio_code: bool,
    ) -> Self {
        Config {
            caps,
            data: ConfigData::default(),
            root_path,
            workspace_roots,
            is_visual_studio_code,
        }
    }

    pub fn update(&mut self, json: serde_json::Value) -> Result<(), serde_json::Error> {
        tracing::info!("updating config from JSON: {:#}", json);
        if json.is_null() || json.as_object().map_or(false, |it| it.is_empty()) {
            return Ok(());
        }
        let mut error = None;
        self.data = match serde_json::from_value(json) {
            Ok(data) => data,
            Err(err) => {
                error = Some(err);
                ConfigData::default()
            }
        };
        tracing::debug!("deserialized config data: {:#?}", self.data);

        // TODO: Implement this.
        // self.validate(&mut errors);
        if let Some(error) = error {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn position_encoding(&self) -> PositionEncoding {
        negotiated_encoding(&self.caps)
    }

    pub fn root_path(&self) -> &AbsPathBuf {
        &self.root_path
    }

    pub fn caps(&self) -> &lsp_types::ClientCapabilities {
        &self.caps
    }

    pub fn publish_diagnostics(&self) -> bool {
        // TODO: Implement this config
        // self.data.diagnostics_enable
        true
    }

    pub fn diagnostics(&self) -> DiagnosticsConfig {
        DiagnosticsConfig {
            enabled: true,
            disable_experimental: false,
            disabled: HashSet::default(),
        }
    }

    pub fn include_directories(&self) -> Vec<AbsPathBuf> {
        // FIXME: Instead of dropping invalid paths, we should report them to the user.
        self.data
            .include_directories
            .clone()
            .into_iter()
            .flat_map(AbsPathBuf::try_from)
            .collect_vec()
    }

    pub fn semantic_tokens_refresh(&self) -> bool {
        try_or_def!(
            self.caps
                .workspace
                .as_ref()?
                .semantic_tokens
                .as_ref()?
                .refresh_support?
        )
    }

    pub fn hover(&self) -> HoverConfig {
        HoverConfig {
            // TODO: Impl these configs
            // links_in_hover: self.data.hover_links_enable,
            links_in_hover: true,
            // documentation: self.data.hover_documentation_enable,
            documentation: true,
            format: {
                let is_markdown = try_or_def!(self
                    .caps
                    .text_document
                    .as_ref()?
                    .hover
                    .as_ref()?
                    .content_format
                    .as_ref()?
                    .as_slice())
                .contains(&MarkupKind::Markdown);
                if is_markdown {
                    HoverDocFormat::Markdown
                } else {
                    HoverDocFormat::PlainText
                }
            },
            // keywords: self.data.hover_documentation_keywords_enable,
            keywords: true,
        }
    }
}
