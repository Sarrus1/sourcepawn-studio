use std::hash::Hash;

use anyhow::{anyhow, bail, Context};
use base_db::{RE_CHEVRON, RE_QUOTE};
use fxhash::FxHashMap;
use lsp_types::{Diagnostic, Position, Range};
use sourcepawn_lexer::{Delta, Literal, Operator, PreprocDir, SourcepawnLexer, Symbol, TokenKind};
use vfs::FileId;

use errors::{ExpansionError, IncludeNotFoundError, MacroNotFoundError};
use evaluator::IfCondition;
use macros::expand_identifier;

pub mod db;
mod errors;
pub(crate) mod evaluator;
mod macros;
mod preprocessor_operator;
mod result;

pub use errors::EvaluationError;
pub use result::PreprocessingResult;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConditionState {
    NotActivated,
    Activated,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offset {
    pub range: lsp_types::Range,
    pub diff: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeLessSymbol {
    pub(crate) token_kind: TokenKind,
    text: String,
    pub(crate) delta: Delta,
}

impl Hash for RangeLessSymbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.token_kind.hash(state);
        self.text.hash(state);
        self.delta.hash(state);
    }
}

impl From<Symbol> for RangeLessSymbol {
    fn from(symbol: Symbol) -> Self {
        Self {
            token_kind: symbol.token_kind,
            text: symbol.inline_text(),
            delta: symbol.delta,
        }
    }
}

impl RangeLessSymbol {
    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn to_symbol(&self, prev_range: Range) -> Symbol {
        let range = Range::new(
            Position::new(prev_range.end.line, prev_range.end.character),
            Position::new(
                prev_range.end.line.saturating_add_signed(self.delta.line),
                prev_range
                    .end
                    .character
                    .saturating_add_signed(self.delta.col),
            ),
        );
        Symbol::new(self.token_kind, Some(&self.text), range, self.delta)
    }
}

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    idx: usize,
    lexer: SourcepawnLexer<'a>,
    macros: FxHashMap<String, Macro>,
    expansion_stack: Vec<Symbol>,
    skip_line_start_col: u32,
    skipped_lines: Vec<lsp_types::Range>,
    macro_not_found_errors: Vec<MacroNotFoundError>,
    evaluation_errors: Vec<EvaluationError>,
    include_not_found_errors: Vec<IncludeNotFoundError>,
    evaluated_define_symbols: Vec<Symbol>,
    file_id: FileId,
    current_line: String,
    prev_end: u32,
    conditions_stack: Vec<ConditionState>,
    out: Vec<String>,
    offsets: FxHashMap<u32, Vec<Offset>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Macro {
    pub(crate) file_id: FileId,
    pub(crate) idx: usize,
    pub(crate) params: Option<Vec<i8>>,
    pub(crate) nb_params: i8,
    pub(crate) body: Vec<RangeLessSymbol>,
    pub(crate) disabled: bool,
}

impl Macro {
    pub fn default(file_id: FileId) -> Self {
        Self {
            file_id,
            idx: 0,
            params: None,
            nb_params: 0,
            body: vec![],
            disabled: false,
        }
    }
}

impl<'a> SourcepawnPreprocessor<'a> {
    pub fn new(file_id: FileId, input: &'a str) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            file_id,
            idx: 0,
            current_line: String::new(),
            skip_line_start_col: 0,
            skipped_lines: vec![],
            macro_not_found_errors: vec![],
            include_not_found_errors: vec![],
            evaluation_errors: vec![],
            evaluated_define_symbols: vec![],
            prev_end: 0,
            conditions_stack: vec![],
            out: vec![],
            macros: FxHashMap::default(),
            expansion_stack: vec![],
            offsets: FxHashMap::default(),
        }
    }

    pub fn set_macros(&mut self, macros_map: FxHashMap<String, Macro>) {
        self.macros.extend(macros_map);
    }

    fn remove_macro(&mut self, name: &str) {
        self.macros.remove(name);
    }

    pub fn insert_macro(&mut self, name: String, mut macro_: Macro) {
        macro_.idx = self.idx;
        self.idx += 1;
        self.macros.insert(name, macro_);
    }

    pub fn result(self) -> PreprocessingResult {
        PreprocessingResult::new(
            self.out.join("\n"),
            self.macros,
            self.offsets,
            self.evaluation_errors,
        )
    }

    pub fn add_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        self.get_disabled_diagnostics(diagnostics);
        self.get_macro_not_found_diagnostics(diagnostics);
        self.get_evaluation_error_diagnostics(diagnostics);
        self.get_include_not_found_diagnostics(diagnostics);
    }

    fn get_disabled_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
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
        diagnostics.extend(ranges.iter().map(|range| Diagnostic {
            range: *range,
            message: "Code disabled by the preprocessor.".to_string(),
            severity: Some(lsp_types::DiagnosticSeverity::HINT),
            tags: Some(vec![lsp_types::DiagnosticTag::UNNECESSARY]),
            ..Default::default()
        }));
    }

    fn get_macro_not_found_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(self.macro_not_found_errors.iter().map(|err| Diagnostic {
            range: err.range,
            message: format!("Macro {} not found.", err.macro_name),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            ..Default::default()
        }));
    }

    fn get_include_not_found_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(self.include_not_found_errors.iter().map(|err| Diagnostic {
            range: err.range,
            message: format!("Include \"{}\" not found.", err.include_text),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            ..Default::default()
        }));
    }

    fn get_evaluation_error_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(self.evaluation_errors.iter().map(|err| Diagnostic {
            range: err.range,
            message: format!("Preprocessor condition is invalid: {}", err.text),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            ..Default::default()
        }));
    }

    pub fn preprocess_input<F>(
        mut self,
        include_file: &mut F,
    ) -> anyhow::Result<PreprocessingResult>
    where
        F: FnMut(&mut FxHashMap<String, Macro>, String, FileId, bool) -> anyhow::Result<()>,
    {
        let _ = include_file(
            &mut self.macros,
            "sourcemod".to_string(),
            self.file_id,
            false,
        );
        let mut col_offset: Option<i32> = None;
        let mut expanded_symbol: Option<Symbol> = None;
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
            if let Some(expanded_symbol) = expanded_symbol.take() {
                if let Some(symbol) = symbol.clone() {
                    self.offsets
                        .entry(symbol.range.start.line)
                        .or_default()
                        .push(Offset {
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
                self.process_negative_condition(&symbol)?;
                continue;
            }
            match &symbol.token_kind {
                TokenKind::Unknown => Err(anyhow!("Unknown token: {:#?}", symbol.range))?,
                TokenKind::PreprocDir(dir) => self.process_directive(include_file, dir, &symbol)?,
                TokenKind::Newline => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    self.current_line = "".to_string();
                    self.prev_end = 0;
                }
                TokenKind::Identifier => match self.macros.get_mut(&symbol.text()) {
                    // TODO: Evaluate the performance dropoff of supporting macro expansion when overriding reserved keywords.
                    // This might only be a problem for a very small subset of users.
                    Some(macro_) => {
                        // Skip the macro if it is disabled and reenable it.
                        if macro_.disabled {
                            macro_.disabled = false;
                            self.push_symbol(&symbol);
                            continue;
                        }
                        match expand_identifier(
                            &mut self.lexer,
                            &mut self.macros,
                            &symbol,
                            &mut self.expansion_stack,
                            true,
                        ) {
                            Ok(expanded_macros) => {
                                expanded_symbol = Some(symbol.clone());
                                self.evaluated_define_symbols.extend(expanded_macros);
                                continue;
                            }
                            Err(ExpansionError::MacroNotFound(err)) => {
                                self.macro_not_found_errors.push(err.clone());
                                bail!("{}", err);
                            }
                            Err(ExpansionError::Parse(err)) => {
                                bail!("{}", err);
                            }
                        }
                    }
                    None => {
                        self.push_symbol(&symbol);
                    }
                },
                TokenKind::Eof => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    break;
                }
                _ => self.push_symbol(&symbol),
            }
        }

        Ok(self.result())
    }

    fn process_if_directive(&mut self, symbol: &Symbol) {
        let line_nb = symbol.range.start.line;
        let mut if_condition = IfCondition::new(&mut self.macros, symbol.range.start.line);
        while self.lexer.in_preprocessor() {
            if let Some(symbol) = self.lexer.next() {
                if symbol.token_kind == TokenKind::Identifier {
                    self.evaluated_define_symbols.push(symbol.clone());
                }
                if_condition.symbols.push(symbol);
            } else {
                break;
            }
        }
        let if_condition_eval = match if_condition.evaluate() {
            Ok(res) => res,
            Err(err) => {
                self.evaluation_errors.push(err);
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
        self.macro_not_found_errors
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

    fn process_directive<F>(
        &mut self,
        include_file: &mut F,
        dir: &PreprocDir,
        symbol: &Symbol,
    ) -> anyhow::Result<()>
    where
        F: FnMut(&mut FxHashMap<String, Macro>, String, FileId, bool) -> anyhow::Result<()>,
    {
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
                let mut macro_name = String::new();
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
                                    if symbol.token_kind == TokenKind::Identifier {
                                        self.evaluated_define_symbols.push(symbol.clone());
                                    }
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
                                if symbol.token_kind == TokenKind::Identifier {
                                    self.evaluated_define_symbols.push(symbol.clone());
                                }
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
            PreprocDir::MInclude | PreprocDir::MTryinclude => {
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
                // FIXME: The logic here is wrong.
                if let Some(caps) = RE_CHEVRON.captures(&text) {
                    if let Some(path) = caps.get(1) {
                        match include_file(
                            &mut self.macros,
                            path.as_str().to_string(),
                            self.file_id,
                            false,
                        ) {
                            Ok(_) => (),
                            Err(_) => {
                                self.include_not_found_errors
                                    .push(IncludeNotFoundError::new(
                                        path.as_str().to_string(),
                                        symbol.range,
                                    ))
                            }
                        }
                    }
                };
                if let Some(caps) = RE_QUOTE.captures(&text) {
                    if let Some(path) = caps.get(1) {
                        match include_file(
                            &mut self.macros,
                            path.as_str().to_string(),
                            self.file_id,
                            true,
                        ) {
                            Ok(_) => (),
                            Err(_) => {
                                self.include_not_found_errors
                                    .push(IncludeNotFoundError::new(
                                        path.as_str().to_string(),
                                        symbol.range,
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
            _ => self.push_symbol(symbol),
        }

        Ok(())
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
            TokenKind::Identifier => {
                // Keep track of the identifiers, so that they can be seen by the semantic analyzer.
                self.evaluated_define_symbols.push(symbol.clone());
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
