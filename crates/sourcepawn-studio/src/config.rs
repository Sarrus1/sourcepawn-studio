#![allow(unused)]
//! Config used by the language server.
//!
//! We currently get this config from `initialize` LSP request, which is not the
//! best way to do it, but was the simplest thing we could implement.

use ide::{DiagnosticsConfig, HoverConfig, HoverDocFormat};
use itertools::Itertools;
use lsp_types::{ClientCapabilities, MarkupKind};
use paths::AbsPathBuf;
use serde::de::DeserializeOwned;
use std::iter;
use std::{collections::HashSet, fmt, path::PathBuf};

use crate::lsp;
use crate::{line_index::PositionEncoding, lsp::ext::negotiated_encoding};

macro_rules! try_ {
    ($expr:expr) => {
        || -> _ { Some($expr) }()
    };
}

#[allow(unused)]
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

config_data! {
    struct ConfigData {
        /// Warm up caches on project load.
        cachePriming_enable: bool = "true",
        /// How many worker threads to handle priming caches. The default `0` means to pick automatically.
        cachePriming_numThreads: ParallelCachePrimingNumThreads = "0",
        /// Number of projects above which cachePriming will only be done for the current project.
        /// The more projects you have in one workspace, the more RAM cachePriming will consume.
        /// Lower this setting or disable cachePriming with `#SourcePawnLanguageServer.cachePriming.enable#` to lower
        /// the server's RAM usage.
        /// `0` for unlimited projects.
        cachePriming_projectsThreshold: usize = "5",
        /// Linter arguments that will be passed to spcomp.
        /// Note that the compilation target, include directories and output path are already handled by the server.
        compiler_arguments: Vec<String> = "[]",
        /// Compute spcomp diagnostics on save.
        compiler_onSave: bool = "true",
        /// Path to the SourcePawn compiler (spcomp).
        compiler_path: Option<String> = "null",

        /// Name of the game we want the events for, as it appears on the Alliedmodders website.
        /// For example, "Counter-Strike: Global Offensive" or "Team Fortress 2".
        eventsGameName: Option<String> = "null",

        /// Whether to show `Debug` action. Only applies when
        /// `#SourcePawnLanguageServer.hover.actions.enable#` is set.
        hover_actions_debug_enable: bool           = "true",
        /// Whether to show HoverActions in Sourcepawn files.
        hover_actions_enable: bool          = "true",
        /// Whether to show `Go to Type Definition` action. Only applies when
        /// `#SourcePawnLanguageServer.hover.actions.enable#` is set.
        hover_actions_gotoTypeDef_enable: bool     = "true",
        /// Whether to show `Implementations` action. Only applies when
        /// `#SourcePawnLanguageServer.hover.actions.enable#` is set.
        hover_actions_implementations_enable: bool = "true",
        /// Whether to show `References` action. Only applies when
        /// `#SourcePawnLanguageServer.hover.actions.enable#` is set.
        hover_actions_references_enable: bool      = "false",
        /// Whether to show `Run` action. Only applies when
        /// `#SourcePawnLanguageServer.hover.actions.enable#` is set.
        hover_actions_run_enable: bool             = "true",

        /// Include directories paths for the compiler and the linter.
        includeDirectories: Vec<PathBuf> = "[]",

        /// Disable the language server's syntax linter. This is independant from spcomp.
        linter_disable: bool = "false",
        /// How many worker threads in the main loop. The default `null` means to pick automatically.
        numThreads: Option<usize> = "null",
    }
}

impl Default for ConfigData {
    fn default() -> Self {
        ConfigData::from_json(serde_json::Value::Null, &mut Vec::new())
    }
}

#[allow(unused)]
#[derive(Debug, Default, Clone)]
pub struct Config {
    /// The workspace roots as registered by the LSP client
    workspace_roots: Vec<PathBuf>,
    caps: lsp_types::ClientCapabilities,
    root_path: AbsPathBuf,
    data: ConfigData,
    is_visual_studio_code: bool,
}

#[derive(Debug)]
pub struct ConfigError {
    errors: Vec<(String, serde_json::Error)>,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let errors = self.errors.iter().format_with("\n", |(key, e), f| {
            f(key)?;
            f(&": ")?;
            f(e)
        });
        write!(
            f,
            "invalid config value{}:\n{}",
            if self.errors.len() == 1 { "" } else { "s" },
            errors
        )
    }
}

impl std::error::Error for ConfigError {}

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

    pub fn update(&mut self, json: serde_json::Value) -> Result<(), ConfigError> {
        tracing::info!("updating config from JSON: {:#}", json);
        if json.is_null() || json.as_object().is_some_and(|it| it.is_empty()) {
            return Ok(());
        }
        let mut errors = Vec::new();
        self.data = ConfigData::from_json(json, &mut errors);
        tracing::debug!("deserialized config data: {:#?}", self.data);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError { errors })
        }
    }

    #[allow(unused)]
    pub fn json_schema() -> serde_json::Value {
        ConfigData::json_schema()
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

    pub fn workspace_roots(&self) -> &[PathBuf] {
        &self.workspace_roots
    }

    pub fn is_visual_studio_code(&self) -> bool {
        self.is_visual_studio_code
    }

    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut ConfigData {
        &mut self.data
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
            .includeDirectories
            .clone()
            .into_iter()
            .flat_map(AbsPathBuf::try_from)
            .collect_vec()
    }

    pub fn prime_caches_num_threads(&self) -> u8 {
        match self.data.cachePriming_numThreads {
            0 => num_cpus::get_physical().try_into().unwrap_or(u8::MAX),
            n => n,
        }
    }

    pub fn hover_actions(&self) -> HoverActionsConfig {
        let enable = self.experimental("hoverActions") && self.data.hover_actions_enable;
        HoverActionsConfig {
            implementations: enable && self.data.hover_actions_implementations_enable,
            references: enable && self.data.hover_actions_references_enable,
            run: enable && self.data.hover_actions_run_enable,
            debug: enable && self.data.hover_actions_debug_enable,
            goto_type_def: enable && self.data.hover_actions_gotoTypeDef_enable,
        }
    }

    pub fn events_game_name(&self) -> Option<&str> {
        self.data.eventsGameName.as_deref()
    }

    pub fn client_commands(&self) -> ClientCommandsConfig {
        let commands = try_or!(
            self.caps.experimental.as_ref()?.get("commands")?,
            &serde_json::Value::Null
        );
        let commands: Option<lsp::ext::ClientCommandOptions> =
            serde_json::from_value(commands.clone()).ok();
        let commands = commands.map(|it| it.commands).unwrap_or_default();

        let get = |name: &str| commands.iter().any(|it| it == name);

        ClientCommandsConfig {
            // run_single: get("sourcepawn-vscode.runSingle"),
            // debug_single: get("sourcepawn-vscode.debugSingle"),
            // show_reference: get("sourcepawn-vscode.showReferences"),
            goto_location: get("sourcepawn-vscode.gotoLocation"),
            // trigger_parameter_hints: get("editor.action.triggerParameterHints"),
        }
    }

    #[allow(unused)]
    pub fn main_loop_num_threads(&self) -> usize {
        self.data.numThreads.unwrap_or(num_cpus::get_physical()) // TODO: Use this config.
    }

    pub fn prefill_caches(&self) -> bool {
        self.data.cachePriming_enable
    }

    pub fn files_to_prime_below_threshold(&self, nb_projects: usize) -> bool {
        self.data.cachePriming_projectsThreshold == 0
            || nb_projects <= self.data.cachePriming_projectsThreshold
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

    pub fn completion_label_details_support(&self) -> bool {
        try_!(self
            .caps
            .text_document
            .as_ref()?
            .completion
            .as_ref()?
            .completion_item
            .as_ref()?
            .label_details_support
            .as_ref()?)
        .is_some()
    }

    pub fn location_link(&self) -> bool {
        try_or_def!(self.caps.text_document.as_ref()?.definition?.link_support?)
    }

    #[allow(unused)]
    pub fn semantics_tokens_augments_syntax_tokens(&self) -> bool {
        try_!(
            self.caps
                .text_document
                .as_ref()?
                .semantic_tokens
                .as_ref()?
                .augments_syntax_tokens?
        )
        .unwrap_or(false)
    }

    fn experimental(&self, index: &'static str) -> bool {
        try_or_def!(self.caps.experimental.as_ref()?.get(index)?.as_bool()?)
    }

    pub fn server_status_notification(&self) -> bool {
        self.experimental("serverStatusNotification")
    }

    pub fn compiler_path(&self) -> Option<&str> {
        self.data.compiler_path.as_deref()
    }

    pub fn compiler_arguments(&self) -> Vec<String> {
        self.data.compiler_arguments.clone()
    }

    pub fn compiler_on_save(&self) -> bool {
        self.data.compiler_onSave
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

type ParallelCachePrimingNumThreads = u8;

pub struct ClientCommandsConfig {
    // pub run_single: bool,
    // pub debug_single: bool,
    // pub show_reference: bool,
    pub goto_location: bool,
    // pub trigger_parameter_hints: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverActionsConfig {
    pub implementations: bool,
    pub references: bool,
    pub run: bool,
    pub debug: bool,
    pub goto_type_def: bool,
}

impl HoverActionsConfig {
    pub const NO_ACTIONS: Self = Self {
        implementations: false,
        references: false,
        run: false,
        debug: false,
        goto_type_def: false,
    };

    pub fn any(&self) -> bool {
        self.implementations || self.references || self.runnable() || self.goto_type_def
    }

    pub fn none(&self) -> bool {
        !self.any()
    }

    pub fn runnable(&self) -> bool {
        self.run || self.debug
    }
}

macro_rules! _config_data {
    (struct $name:ident {
        $(
            $(#[doc=$doc:literal])*
            $field:ident $(| $alias:ident)*: $ty:ty = $default:expr,
        )*
    }) => {
        #[allow(non_snake_case)]
        #[derive(Debug, Clone, serde::Serialize)]
        pub struct $name { $(
            #[cfg(test)]
            pub $field: $ty,
            #[cfg(not(test))]
            $field: $ty,
        )* }
        impl $name {
            fn from_json(mut json: serde_json::Value, error_sink: &mut Vec<(String, serde_json::Error)>) -> $name {
                $name {$(
                    $field: get_field(
                        &mut json,
                        error_sink,
                        stringify!($field),
                        None$(.or(Some(stringify!($alias))))*,
                        $default,
                    ),
                )*}
            }

            fn json_schema() -> serde_json::Value {
                schema(&[
                    $({
                        let field = stringify!($field);
                        let ty = stringify!($ty);

                        (field, ty, &[$($doc),*], $default)
                    },)*
                ])
            }

            #[cfg(test)]
            fn manual() -> String {
                manual(&[
                    $({
                        let field = stringify!($field);
                        let ty = stringify!($ty);

                        (field, ty, &[$($doc),*], $default)
                    },)*
                ])
            }
        }

        #[test]
        fn fields_are_sorted() {
            [$(stringify!($field)),*].windows(2).for_each(|w| assert!(w[0] <= w[1], "{} <= {} does not hold", w[0], w[1]));
        }
    };
}
use _config_data as config_data;

fn get_field<T: DeserializeOwned>(
    json: &mut serde_json::Value,
    error_sink: &mut Vec<(String, serde_json::Error)>,
    field: &'static str,
    alias: Option<&'static str>,
    default: &str,
) -> T {
    // XXX: check alias first, to work around the VS Code where it pre-fills the
    // defaults instead of sending an empty object.
    alias
        .into_iter()
        .chain(iter::once(field))
        .filter_map(move |field| {
            let mut pointer = field.replace('_', "/");
            pointer.insert(0, '/');
            json.pointer_mut(&pointer)
                .map(|it| serde_json::from_value(it.take()).map_err(|e| (e, pointer)))
        })
        .find(Result::is_ok)
        .and_then(|res| match res {
            Ok(it) => Some(it),
            Err((e, pointer)) => {
                tracing::warn!("Failed to deserialize config field at {}: {:?}", pointer, e);
                error_sink.push((pointer, e));
                None
            }
        })
        .unwrap_or_else(|| {
            serde_json::from_str(default).unwrap_or_else(|e| panic!("{e} on: `{default}`"))
        })
}

fn schema(fields: &[(&'static str, &'static str, &[&str], &str)]) -> serde_json::Value {
    let map = fields
        .iter()
        .map(|(field, ty, doc, default)| {
            let name = field.replace('_', ".");
            let name = format!("SourcePawnLanguageServer.{name}");
            let props = field_props(field, ty, doc, default);
            (name, props)
        })
        .collect::<serde_json::Map<_, _>>();
    map.into()
}

fn field_props(field: &str, ty: &str, doc: &[&str], default: &str) -> serde_json::Value {
    let doc = doc_comment_to_string(doc);
    let doc = doc.trim_end_matches('\n');
    assert!(
        doc.ends_with('.') && doc.starts_with(char::is_uppercase),
        "bad docs for {field}: {doc:?}"
    );
    let default = default
        .parse::<serde_json::Value>()
        .unwrap_or_else(|_| String::new().into());

    let mut map = serde_json::Map::default();
    macro_rules! set {
        ($($key:literal: $value:tt),*$(,)?) => {{$(
            map.insert($key.into(), serde_json::json!($value));
        )*}};
    }
    set!("markdownDescription": doc);
    set!("default": default);

    match ty {
        "bool" => set!("type": "boolean"),
        "usize" => set!("type": "integer", "minimum": 0),
        "String" => set!("type": "string"),
        "Vec<String>" => set! {
            "type": "array",
            "items": { "type": "string" },
        },
        "Vec<PathBuf>" => set! {
            "type": "array",
            "items": { "type": "string" },
        },
        "FxHashSet<String>" => set! {
            "type": "array",
            "items": { "type": "string" },
            "uniqueItems": true,
        },
        "FxHashMap<Box<str>, Box<[Box<str>]>>" => set! {
            "type": "object",
        },
        "FxHashMap<String, SnippetDef>" => set! {
            "type": "object",
        },
        "FxHashMap<String, String>" => set! {
            "type": "object",
        },
        "FxHashMap<Box<str>, usize>" => set! {
            "type": "object",
        },
        "Option<usize>" => set! {
            "type": ["null", "integer"],
            "minimum": 0,
        },
        "Option<String>" => set! {
            "type": ["null", "string"],
        },
        "Option<PathBuf>" => set! {
            "type": ["null", "string"],
        },
        "Option<bool>" => set! {
            "type": ["null", "boolean"],
        },
        "Option<Vec<String>>" => set! {
            "type": ["null", "array"],
            "items": { "type": "string" },
        },
        "ParallelCachePrimingNumThreads" => set! {
            "type": "number",
            "minimum": 0,
            "maximum": 255
        },
        _ => panic!("missing entry for {ty}: {default}"),
    }

    map.into()
}

#[cfg(test)]
fn manual(fields: &[(&'static str, &'static str, &[&str], &str)]) -> String {
    fields
        .iter()
        .map(|(field, _ty, doc, default)| {
            let name = format!("SourcePawnLanguageServer.{}", field.replace('_', "."));
            let field = field.replace('_', ".");
            let doc = doc_comment_to_md_doc(&doc_comment_to_string(doc));
            format!(
                r#"## {field}

**{name}**

{doc}
_Default_: `{default}`

"#
            )
        })
        .fold(String::new(), |acc, it| acc + &it)
}

fn doc_comment_to_string(doc: &[&str]) -> String {
    doc.iter()
        .map(|it| it.strip_prefix(' ').unwrap_or(it))
        .fold(String::new(), |acc, it| acc + it + "\n")
}

fn doc_comment_to_md_doc(input: &str) -> String {
    let pattern_start = "`#";
    let pattern_end = "#`";

    if let Some(start_index) = input.find(pattern_start) {
        if let Some(end_index) = input.find(pattern_end) {
            let key = &input[start_index + pattern_start.len()..end_index];
            let link = format!(
                "[`{}`](#{})",
                key,
                key.replace("SourcePawnLanguageServer", "").replace('.', "")
            );
            return input.replace(&format!("{}{}{}", pattern_start, key, pattern_end), &link);
        }
    }

    input.to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use test_utils::{ensure_file_contents, project_root};

    use super::*;

    static CONFIG_TEMPLATE_PATH: &str = "editors/code/package_template.json";

    #[test]
    fn generate_package_json_config() {
        let s = Config::json_schema();
        let mut schema = format!("{s:#}");

        // Transform the asciidoc form link to markdown style.
        //
        // https://link[text] => [text](https://link)
        let url_matches = schema.match_indices("https://");
        let mut url_offsets = url_matches.map(|(idx, _)| idx).collect::<Vec<usize>>();
        url_offsets.reverse();
        for idx in url_offsets {
            let link = &schema[idx..];
            // matching on whitespace to ignore normal links
            if let Some(link_end) = link.find([' ', '[']) {
                if link.chars().nth(link_end) == Some('[') {
                    if let Some(link_text_end) = link.find(']') {
                        let link_text = link[link_end..(link_text_end + 1)].to_string();

                        schema.replace_range((idx + link_end)..(idx + link_text_end + 1), "");
                        schema.insert(idx, '(');
                        schema.insert(idx + link_end + 1, ')');
                        schema.insert_str(idx, &link_text);
                    }
                }
            }
        }

        let package_template_json_path = project_root().join(CONFIG_TEMPLATE_PATH);
        let package_json_path = project_root().join("editors/code/package.json");
        let mut package_json = fs::read_to_string(package_template_json_path).unwrap();

        // Parse the package.json and insert the schema.
        let mut old: Value = serde_json::from_str(&package_json).unwrap();
        let old_config = old
            .get_mut("contributes")
            .unwrap()
            .get_mut("configuration")
            .unwrap()
            .get_mut("properties")
            .unwrap()
            .as_object_mut()
            .unwrap();

        let p = remove_ws(&package_json);
        let s = remove_ws(&schema);
        if !p.contains(&s) {
            let new: Value = serde_json::from_str(&schema).unwrap();
            for i in new.as_object().unwrap() {
                old_config.insert(i.0.clone(), i.1.clone());
            }
            package_json = serde_json::to_string_pretty(&old).unwrap();
            ensure_file_contents(&package_json_path, &package_json)
        }
    }

    #[test]
    fn generate_config_documentation() {
        let docs_path = project_root().join("docs/docs/configuration/generated_settings.md");
        let expected = format!(
            r#"---
id: lsp-settings-reference
title: Server Settings Reference
---

{}"#,
            ConfigData::manual()
        );
        ensure_file_contents(&docs_path, &expected);
    }

    #[test]
    fn generate_vscode_config_documentation() {
        let docs_path = project_root().join("docs/docs/configuration/generated_vscode_settings.md");

        let package_template_json_path = project_root().join(CONFIG_TEMPLATE_PATH);
        let mut package_json = fs::read_to_string(package_template_json_path).unwrap();

        // Parse the package.json and insert the schema.
        let mut config = serde_json::from_str::<Value>(&package_json).unwrap();
        let mut config = config
            .get_mut("contributes")
            .unwrap()
            .get_mut("configuration")
            .unwrap()
            .get_mut("properties")
            .unwrap()
            .as_object_mut()
            .unwrap();

        let expected = config
            .iter()
            .map(|(field, value)| {
                let name = field.replace('_', ".");
                let field = field.replace("sourcepawn.", "").replace('_', ".");
                let doc = value.get("description").unwrap().as_str().unwrap();
                let default = value
                    .get("default")
                    .map(|it| it.to_string())
                    .unwrap_or_default();
                format!(
                    r#"## {field}

**{name}**

{doc}

_Default_: `{default}`

"#
                )
            })
            .fold(String::new(), |acc, it| acc + &it);

        let expected = format!(
            r#"---
id: vscode-settings-reference
title: VSCode Settings Reference
---

{}"#,
            expected
        );
        ensure_file_contents(&docs_path, &expected);
    }

    fn remove_ws(text: &str) -> String {
        text.replace(char::is_whitespace, "")
    }
}
