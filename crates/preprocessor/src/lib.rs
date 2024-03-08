use std::{hash::Hash, sync::Arc};

use anyhow::{bail, Context};
use base_db::{RE_CHEVRON, RE_QUOTE};
use deepsize::DeepSizeOf;
use fxhash::{FxHashMap, FxHashSet};
use lsp_types::{Diagnostic, Position, Range};
use smol_str::SmolStr;
use sourcepawn_lexer::{Literal, Operator, PreprocDir, SourcepawnLexer, Symbol, TokenKind};
use stdx::hashable_hash_map::HashableHashMap;
use symbol::RangeLessSymbol;
use vfs::FileId;

use errors::{ExpansionError, PreprocessorErrors, UnresolvedIncludeError};
use evaluator::IfCondition;
use macros::expand_identifier;

pub mod db;
mod errors;
pub(crate) mod evaluator;
mod macros;
mod offset;
mod preprocessor_operator;
mod result;
mod symbol;

pub use errors::{EvaluationError, PreprocessorError};
pub use offset::Offset;
pub use result::PreprocessingResult;

#[cfg(test)]
mod test;

/// State of a preprocessor condition.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ConditionState {
    /// The condition is not activated and could be activated by an else/elseif directive.
    NotActivated,

    /// The condition has been activated, all related else/elseif directives should be skipped.
    Activated,

    /// The condition is active and the preprocessor should process the code.
    Active,
}

pub type MacrosMap = FxHashMap<SmolStr, Arc<Macro>>;
pub type HMacrosMap = HashableHashMap<SmolStr, Arc<Macro>>;
pub type ArgsMap = FxHashMap<u32, Vec<(Range, Range)>>;

#[derive(Debug)]
pub struct SourcepawnPreprocessor<'a, F>
where
    F: FnMut(&mut MacrosMap, String, FileId, bool) -> anyhow::Result<()>,
{
    /// The index of the current macro in the file.
    idx: u32,
    lexer: SourcepawnLexer<'a>,
    input: &'a str,
    macros: MacrosMap,
    expansion_stack: Vec<Symbol>,
    skip_line_start_col: u32,
    skipped_lines: Vec<lsp_types::Range>,
    errors: PreprocessorErrors,
    file_id: FileId,
    current_line: String,
    prev_end: u32,
    conditions_stack: Vec<ConditionState>,
    out: Vec<String>,
    offsets: FxHashMap<u32, Vec<Offset>>,
    args_maps: ArgsMap,
    include_file: &'a mut F,
    disabled_macros: FxHashSet<Arc<Macro>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Macro {
    pub(crate) file_id: FileId,
    pub(crate) idx: u32,
    pub(crate) params: Option<Vec<i8>>,
    pub(crate) nb_params: i8,
    pub(crate) body: Vec<RangeLessSymbol>,
}

impl DeepSizeOf for Macro {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        std::mem::size_of::<FileId>()
            + self.idx.deep_size_of_children(context)
            + self.params.deep_size_of_children(context)
            + self.nb_params.deep_size_of_children(context)
            + self.body.deep_size_of_children(context)
    }
}

impl Macro {
    pub fn default(file_id: FileId) -> Self {
        Self {
            file_id,
            idx: 0,
            params: None,
            nb_params: 0,
            body: vec![],
        }
    }
}

/// Parse status of `using __intrinsics__.Handle;`.
/// This is used to handle the `using __intrinsics__.Handle;` in handles.inc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IntrinsicsParseStatus {
    Using,
    Dot,
    Intrinsics,
    Handle,
}

impl<'a, F> SourcepawnPreprocessor<'a, F>
where
    F: FnMut(&mut MacrosMap, String, FileId, bool) -> anyhow::Result<()>,
{
    pub fn new(file_id: FileId, input: &'a str, include_file: &'a mut F) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            input,
            file_id,
            include_file,
            idx: Default::default(),
            current_line: Default::default(),
            skip_line_start_col: Default::default(),
            skipped_lines: Default::default(),
            errors: Default::default(),
            prev_end: Default::default(),
            conditions_stack: Default::default(),
            out: Default::default(),
            macros: FxHashMap::default(),
            expansion_stack: Default::default(),
            offsets: FxHashMap::default(),
            args_maps: FxHashMap::default(),
            disabled_macros: FxHashSet::default(),
        }
    }

    pub fn set_macros(&mut self, macros: MacrosMap) {
        self.macros.extend(macros);
    }

    fn remove_macro(&mut self, name: &str) {
        self.macros.remove(name);
    }

    pub fn insert_macro(&mut self, name: SmolStr, mut macro_: Macro) {
        macro_.idx = self.idx;
        self.idx += 1;
        self.macros.insert(name, macro_.into());
    }

    pub fn disable_macro(&mut self, macro_: Arc<Macro>) {
        self.disabled_macros.insert(macro_);
    }

    pub fn enable_macro(&mut self, macro_: Arc<Macro>) {
        self.disabled_macros.remove(&macro_);
    }

    pub fn is_macro_disabled(&self, macro_: &Arc<Macro>) -> bool {
        self.disabled_macros.contains(macro_)
    }

    pub fn result(self) -> PreprocessingResult {
        let inactive_ranges = self.get_inactive_ranges();
        let mut res = PreprocessingResult::new(
            self.out.join("\n").into(),
            self.macros,
            self.offsets,
            self.args_maps,
            self.errors,
            inactive_ranges,
        );
        res.shrink_to_fit();
        res
    }

    pub fn error_result(self) -> PreprocessingResult {
        let inactive_ranges = self.get_inactive_ranges();
        let mut res = PreprocessingResult::new(
            self.input.to_owned().into(),
            self.macros,
            self.offsets,
            self.args_maps,
            self.errors,
            inactive_ranges,
        );
        res.shrink_to_fit();
        res
    }

    pub fn add_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        self.get_macro_not_found_diagnostics(diagnostics);
        self.get_evaluation_error_diagnostics(diagnostics);
        self.get_include_not_found_diagnostics(diagnostics);
    }

    fn get_inactive_ranges(&self) -> Vec<lsp_types::Range> {
        let mut ranges: Vec<lsp_types::Range> = vec![];
        for range in self.skipped_lines.iter() {
            if let Some(old_range) = ranges.pop() {
                if old_range.end.line == range.start.line - 1 {
                    ranges.push(lsp_types::Range::new(old_range.start, range.end));
                    continue;
                } else {
                    ranges.push(old_range);
                    ranges.push(*range);
                }
            } else {
                ranges.push(*range);
            }
        }
        for range in ranges.iter_mut() {
            range.start.character = 0;
        }

        ranges
    }

    fn get_macro_not_found_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(
            self.errors
                .macro_not_found_errors
                .iter()
                .map(|err| Diagnostic {
                    range: err.range,
                    message: format!("Macro {} not found.", err.macro_name),
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    ..Default::default()
                }),
        );
    }

    fn get_include_not_found_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(
            self.errors
                .unresolved_include_errors
                .iter()
                .map(|err| Diagnostic {
                    range: err.range,
                    message: format!("Include \"{}\" not found.", err.include_text),
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    ..Default::default()
                }),
        );
    }

    fn get_evaluation_error_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(self.errors.evaluation_errors.iter().map(|err| Diagnostic {
            range: err.range,
            message: format!("Preprocessor condition is invalid: {}", err.text),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            ..Default::default()
        }));
    }

    pub fn preprocess_input(mut self) -> PreprocessingResult {
        let _ = (self.include_file)(
            &mut self.macros,
            "sourcemod".to_string(),
            self.file_id,
            false,
        );
        let mut intrinsics_parse_status = None;
        let mut col_offset: Option<i32> = None;
        let mut expanded_symbol: Option<(Symbol, u32, FileId)> = None;
        while let Some(symbol) = if !self.expansion_stack.is_empty() {
            let symbol = self.expansion_stack.pop().unwrap();
            col_offset = Some(
                col_offset.map_or(symbol.inline_text().len() as i32, |offset| {
                    offset + symbol.delta.col + symbol.inline_text().len() as i32
                }),
            );
            Some(symbol)
        } else {
            let symbol = self.lexer.next();
            if let Some((expanded_symbol, idx, file_id)) = expanded_symbol.take() {
                if let Some(symbol) = symbol.clone() {
                    self.offsets
                        .entry(symbol.range.start.line)
                        .or_default()
                        .push(Offset {
                            idx,
                            file_id,
                            range: expanded_symbol.range,
                            diff: (col_offset.take().unwrap_or(0)
                                - (expanded_symbol.range.end.character
                                    - expanded_symbol.range.start.character)
                                    as i32),
                        });
                }
            }

            symbol
        } {
            if matches!(
                self.conditions_stack
                    .last()
                    .unwrap_or(&ConditionState::Active),
                ConditionState::Activated | ConditionState::NotActivated
            ) {
                if self.process_negative_condition(&symbol).is_err() {
                    return self.error_result();
                }
                continue;
            }
            match &symbol.token_kind {
                TokenKind::Unknown => return self.error_result(),
                TokenKind::PreprocDir(dir) => {
                    if self.process_directive(dir, &symbol).is_err() {
                        return self.error_result();
                    }
                }
                TokenKind::Newline => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    self.current_line = "".to_string();
                    self.prev_end = 0;
                }
                TokenKind::Identifier => {
                    // This is a hack to handle `using __intrinsics__.Handle;` in handles.inc, which
                    // is not a part of sourcemod as of 060c832f89709e6a6222cf039071061dcc0a36da.
                    // see: https://github.com/alliedmodders/sourcemod/commit/060c832f89709e6a6222cf039071061dcc0a36da
                    if intrinsics_parse_status == Some(IntrinsicsParseStatus::Dot) {
                        if symbol.text() == "Handle" {
                            self.current_line.push_str("methodmap Handle __nullable__ {public native ~Handle();public native void Close();};")
                        }
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Handle);
                        continue;
                    }
                    match self.macros.get(&symbol.text()) {
                        // TODO: Evaluate the performance dropoff of supporting macro expansion when overriding reserved keywords.
                        // This might only be a problem for a very small subset of users.
                        Some(macro_) => {
                            // Skip the macro if it is disabled and reenable it.
                            if self.is_macro_disabled(macro_) {
                                self.enable_macro(macro_.clone());
                                self.push_symbol(&symbol);
                                continue;
                            }
                            let idx = macro_.idx;
                            let file_id: FileId = macro_.file_id;
                            match expand_identifier(
                                &mut self.lexer,
                                &mut self.macros,
                                &symbol,
                                &mut self.expansion_stack,
                                true,
                                &mut self.disabled_macros,
                            ) {
                                Ok(args_map) => {
                                    extend_args_map(&mut self.args_maps, args_map);
                                    expanded_symbol = Some((symbol.clone(), idx, file_id));
                                    continue;
                                }
                                Err(ExpansionError::MacroNotFound(err)) => {
                                    self.errors.macro_not_found_errors.push(err.clone());
                                    return self.error_result();
                                }
                                Err(ExpansionError::Parse(_)) => {
                                    return self.error_result();
                                }
                            }
                        }
                        None => {
                            self.push_symbol(&symbol);
                        }
                    }
                }
                TokenKind::Using => {
                    if intrinsics_parse_status.take().is_none() {
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Using);
                    }
                }
                TokenKind::Intrinsics => {
                    if intrinsics_parse_status.take() == Some(IntrinsicsParseStatus::Using) {
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Intrinsics);
                    }
                }
                TokenKind::Semicolon => {
                    if intrinsics_parse_status.take() != Some(IntrinsicsParseStatus::Handle) {
                        self.push_symbol(&symbol);
                    }
                }
                TokenKind::Dot => match intrinsics_parse_status {
                    Some(IntrinsicsParseStatus::Intrinsics) => {
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Dot);
                    }
                    _ => self.push_symbol(&symbol),
                },
                TokenKind::Eof => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    break;
                }
                _ => self.push_symbol(&symbol),
            }
        }

        self.result()
    }

    fn process_if_directive(&mut self, symbol: &Symbol) {
        let line_nb = symbol.range.start.line;
        let mut if_condition = IfCondition::new(
            &mut self.macros,
            symbol.range.start.line,
            &mut self.offsets,
            &mut self.args_maps,
            &mut self.disabled_macros,
        );
        while self.lexer.in_preprocessor() {
            if let Some(symbol) = self.lexer.next() {
                if_condition.symbols.push(symbol);
            } else {
                break;
            }
        }
        let if_condition_eval = match if_condition.evaluate() {
            Ok(res) => res,
            Err(err) => {
                self.errors.evaluation_errors.push(err);
                // Default to false when we fail to evaluate a condition.
                false
            }
        };

        if if_condition_eval {
            self.conditions_stack.push(ConditionState::Active);
        } else {
            self.skip_line_start_col = symbol.range.end.character;
            self.conditions_stack.push(ConditionState::NotActivated);
        }
        self.errors
            .macro_not_found_errors
            .extend(if_condition.macro_not_found_errors);
        if let Some(last_symbol) = if_condition.symbols.last() {
            let line_diff = last_symbol.range.end.line - line_nb;
            for _ in 0..line_diff {
                self.out.push(String::new());
            }
        }

        self.prev_end = 0;
    }

    fn process_else_directive(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        let last = self
            .conditions_stack
            .pop()
            .context("Expect if before else clause.")?;
        match last {
            ConditionState::NotActivated => {
                self.conditions_stack.push(ConditionState::Active);
            }
            ConditionState::Active | ConditionState::Activated => {
                self.skip_line_start_col = symbol.range.end.character;
                self.conditions_stack.push(ConditionState::Activated);
            }
        }

        Ok(())
    }

    fn process_endif_directive(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        self.conditions_stack
            .pop()
            .context("Expect if before endif clause")?;
        // Skip the endif if it is in a nested condition.
        if let Some(last) = self.conditions_stack.last() {
            if *last != ConditionState::Active {
                self.skipped_lines.push(lsp_types::Range::new(
                    Position::new(symbol.range.start.line, self.skip_line_start_col),
                    Position::new(symbol.range.start.line, symbol.range.end.character),
                ));
            }
        }

        Ok(())
    }

    fn process_directive(&mut self, dir: &PreprocDir, symbol: &Symbol) -> anyhow::Result<()> {
        match dir {
            PreprocDir::MIf => self.process_if_directive(symbol),
            PreprocDir::MElseif => {
                let last = self
                    .conditions_stack
                    .pop()
                    .context("Expect if before elseif clause.")?;
                match last {
                    ConditionState::NotActivated => self.process_if_directive(symbol),
                    ConditionState::Active | ConditionState::Activated => {
                        self.conditions_stack.push(ConditionState::Activated);
                    }
                }
            }
            PreprocDir::MDefine => {
                self.push_symbol(symbol);
                let mut macro_name = SmolStr::default();
                let mut macro_ = Macro::default(self.file_id);
                enum State {
                    Start,
                    Params,
                    Body,
                }
                let mut args = vec![-1; 10];
                let mut found_params = false;
                let mut state = State::Start;
                let mut args_idx = 0;
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        if symbol.token_kind == TokenKind::Eof {
                            break;
                        }
                        self.push_ws(&symbol);
                        self.prev_end = symbol.range.end.character;
                        if !matches!(symbol.token_kind, TokenKind::Newline | TokenKind::Eof) {
                            self.current_line.push_str(&symbol.text());
                        }
                        match state {
                            State::Start => {
                                if macro_name.is_empty()
                                    && TokenKind::Identifier == symbol.token_kind
                                {
                                    macro_name = symbol.text();
                                } else if symbol.delta.col == 0
                                    && symbol.token_kind == TokenKind::LParen
                                {
                                    state = State::Params;
                                } else {
                                    macro_.body.push(symbol.into());
                                    state = State::Body;
                                }
                            }
                            State::Params => {
                                if symbol.delta.col > 0 {
                                    macro_.body.push(symbol.into());
                                    state = State::Body;
                                    continue;
                                }
                                match &symbol.token_kind {
                                    TokenKind::RParen => {
                                        state = State::Body;
                                    }
                                    TokenKind::Literal(Literal::IntegerLiteral) => {
                                        found_params = true;
                                        let idx = symbol.to_int().context(format!(
                                            "Could not convert {:?} to an int value.",
                                            symbol.text()
                                        ))?
                                            as usize;
                                        if idx >= args.len() {
                                            bail!(
                                                "Argument index out of bounds for macro {}",
                                                symbol.text()
                                            );
                                        }
                                        args[idx] = args_idx;
                                    }
                                    TokenKind::Comma => {
                                        args_idx += 1;
                                    }
                                    TokenKind::Operator(Operator::Percent) => (),
                                    _ => {
                                        bail!("Unexpected symbol {} in macro args", symbol.text())
                                    }
                                }
                            }
                            State::Body => {
                                macro_.body.push(symbol.into());
                            }
                        }
                    }
                }
                if found_params {
                    macro_.nb_params = args.iter().filter(|&n| *n != -1).count() as i8;
                    macro_.params = Some(args);
                }
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
                self.insert_macro(macro_name, macro_);
            }
            PreprocDir::MUndef => {
                self.push_symbol(symbol);
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        self.push_ws(&symbol);
                        self.prev_end = symbol.range.end.character;
                        if !matches!(symbol.token_kind, TokenKind::Newline | TokenKind::Eof) {
                            self.current_line.push_str(&symbol.text());
                        }
                        if symbol.token_kind == TokenKind::Identifier {
                            self.remove_macro(&symbol.text());
                            break;
                        }
                    }
                }
            }
            PreprocDir::MEndif => self.process_endif_directive(symbol)?,
            PreprocDir::MElse => self.process_else_directive(symbol)?,
            PreprocDir::MInclude => self.process_include_directive(symbol, false),
            PreprocDir::MTryinclude => self.process_include_directive(symbol, true),
            _ => self.push_symbol(symbol),
        }

        Ok(())
    }

    fn process_include_directive(&mut self, symbol: &Symbol, is_try: bool) {
        let text = symbol.inline_text().trim().to_string();
        let delta = symbol.range.end.line - symbol.range.start.line;
        let symbol = Symbol::new(
            symbol.token_kind,
            Some(&text),
            Range::new(
                Position::new(symbol.range.start.line, symbol.range.start.character),
                Position::new(symbol.range.start.line, text.len() as u32),
            ),
            symbol.delta,
        );

        if let Some(path) = RE_CHEVRON.captures(&text).and_then(|c| c.get(1)) {
            match (self.include_file)(
                &mut self.macros,
                path.as_str().to_string(),
                self.file_id,
                false,
            ) {
                Ok(_) => (),
                Err(_) => {
                    if !is_try {
                        // TODO: Emit a warning here for #tryinclude?
                        let mut range = symbol.range;
                        range.start.character = path.start() as u32;
                        range.end.character = path.end() as u32;
                        self.errors
                            .unresolved_include_errors
                            .push(UnresolvedIncludeError::new(
                                path.as_str().to_string(),
                                range,
                            ))
                    }
                }
            }
        };
        if let Some(path) = RE_QUOTE.captures(&text).and_then(|c| c.get(1)) {
            match (self.include_file)(
                &mut self.macros,
                path.as_str().to_string(),
                self.file_id,
                true,
            ) {
                Ok(_) => (),
                Err(_) => {
                    if !is_try {
                        // TODO: Emit a warning here for #tryinclude?
                        let mut range = symbol.range;
                        range.start.character = path.start() as u32;
                        range.end.character = path.end() as u32;
                        self.errors
                            .unresolved_include_errors
                            .push(UnresolvedIncludeError::new(
                                path.as_str().to_string(),
                                range,
                            ))
                    }
                }
            }
        };

        self.push_symbol(&symbol);
        if delta > 0 {
            self.push_current_line();
            self.current_line = "".to_string();
            self.prev_end = 0;
            for _ in 0..delta - 1 {
                self.out.push(String::new());
            }
        }
    }

    fn process_negative_condition(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        match &symbol.token_kind {
            TokenKind::PreprocDir(dir) => match dir {
                PreprocDir::MIf => {
                    // Keep track of any nested if statements to ensure we properly pop when reaching an endif.
                    self.conditions_stack.push(ConditionState::Activated);
                }
                PreprocDir::MEndif => self.process_endif_directive(symbol)?,
                PreprocDir::MElse => self.process_else_directive(symbol)?,
                PreprocDir::MElseif => {
                    let last = self
                        .conditions_stack
                        .pop()
                        .context("Expect if before elseif clause.")?;
                    match last {
                        ConditionState::NotActivated => self.process_if_directive(symbol),
                        ConditionState::Active | ConditionState::Activated => {
                            self.conditions_stack.push(ConditionState::Activated);
                        }
                    }
                }
                _ => (),
            },
            TokenKind::Newline => {
                // Keep the newline to keep the line numbers in sync.
                self.push_current_line();
                self.skipped_lines.push(lsp_types::Range::new(
                    Position::new(symbol.range.start.line, self.skip_line_start_col),
                    Position::new(symbol.range.start.line, symbol.range.start.character),
                ));
                self.current_line = "".to_string();
                self.prev_end = 0;
            }
            // Skip any token that is not a directive or a newline.
            _ => (),
        }

        Ok(())
    }

    fn push_ws(&mut self, symbol: &Symbol) {
        self.current_line
            .push_str(&" ".repeat(symbol.delta.col.unsigned_abs() as usize));
    }

    fn push_current_line(&mut self) {
        self.out.push(self.current_line.clone());
    }

    fn push_symbol(&mut self, symbol: &Symbol) {
        if symbol.token_kind == TokenKind::Eof {
            self.push_current_line();
            return;
        }
        self.push_ws(symbol);
        self.prev_end = symbol.range.end.character;
        self.current_line.push_str(&symbol.text());
    }
}

fn extend_args_map(args_map: &mut ArgsMap, buffer: Option<Vec<Vec<(Range, Range)>>>) {
    let Some(buffer) = buffer else { return };
    buffer
        .into_iter()
        .filter(|it| !it.is_empty())
        .for_each(|it| {
            it.into_iter()
                .for_each(|it| args_map.entry(it.0.start.line).or_default().push(it))
        })
}
