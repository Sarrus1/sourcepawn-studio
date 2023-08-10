use anyhow::{anyhow, Context};
use fxhash::FxHashMap;
use lazy_static::lazy_static;
use lsp_types::{Diagnostic, Position, Range, Url};
use regex::Regex;
use sourcepawn_lexer::{Literal, Operator, PreprocDir, SourcepawnLexer, Symbol, TokenKind};
use std::sync::Arc;

use errors::{EvaluationError, ExpansionError, IncludeNotFoundError, MacroNotFoundError};
use evaluator::IfCondition;
use macros::expand_symbol;

mod errors;
pub(crate) mod evaluator;
mod macros;
mod preprocessor_operator;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConditionState {
    NotActivated,
    Activated,
    Active,
}

#[derive(Debug, Clone)]
pub struct Offset {
    pub col: u32,
    pub diff: i32,
}

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    pub lexer: SourcepawnLexer<'a>,
    pub macros: FxHashMap<String, Macro>,
    pub expansion_stack: Vec<Symbol>,
    skip_line_start_col: u32,
    skipped_lines: Vec<lsp_types::Range>,
    pub(self) macro_not_found_errors: Vec<MacroNotFoundError>,
    pub(self) evaluation_errors: Vec<EvaluationError>,
    pub(self) include_not_found_errors: Vec<IncludeNotFoundError>,
    pub evaluated_define_symbols: Vec<Symbol>,
    document_uri: Arc<Url>,
    current_line: String,
    prev_end: u32,
    conditions_stack: Vec<ConditionState>,
    out: Vec<String>,
    pub offsets: FxHashMap<u32, Vec<Offset>>,
}

#[derive(Debug, Clone, Default)]
pub struct Macro {
    pub(crate) params: Option<Vec<i8>>,
    pub(crate) nb_params: i8,
    pub(crate) body: Vec<Symbol>,
}

impl<'a> SourcepawnPreprocessor<'a> {
    pub fn new(document_uri: Arc<Url>, input: &'a str) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            document_uri,
            current_line: "".to_string(),
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

    pub fn preprocess_input<F>(&mut self, include_file: &mut F) -> anyhow::Result<String>
    where
        F: FnMut(&mut FxHashMap<String, Macro>, String, &Url, bool) -> anyhow::Result<()>,
    {
        let _ = include_file(
            &mut self.macros,
            "sourcemod".to_string(),
            &self.document_uri,
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
                        .or_insert_with(Vec::new)
                        .push(Offset {
                            col: expanded_symbol.range.start.character,
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
                TokenKind::Identifier => match self.macros.get(&symbol.text()) {
                    // TODO: Evaluate the performance dropoff of supporting macro expansion when overriding reserved keywords.
                    // This might only be a problem for a very small subset of users.
                    Some(_) => {
                        match expand_symbol(
                            &mut self.lexer,
                            &self.macros,
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
                                return Err(anyhow!("{}", err));
                            }
                            Err(ExpansionError::Parse(err)) => {
                                return Err(anyhow!("{}", err));
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

        Ok(self.out.join("\n"))
    }

    fn process_if_directive(&mut self, symbol: &Symbol) {
        let line_nb = symbol.range.start.line;
        let mut if_condition = IfCondition::new(&self.macros, symbol.range.start.line);
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
        F: FnMut(&mut FxHashMap<String, Macro>, String, &Url, bool) -> anyhow::Result<()>,
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
                let mut macro_ = Macro::default();
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
                                    macro_.body.push(symbol);
                                    state = State::Body;
                                }
                            }
                            State::Params => {
                                if symbol.delta.col > 0 {
                                    macro_.body.push(symbol);
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
                                            return Err(anyhow!(
                                                "Argument index out of bounds for macro {}",
                                                symbol.text()
                                            ));
                                        }
                                        args[idx] = args_idx;
                                    }
                                    TokenKind::Comma => {
                                        args_idx += 1;
                                    }
                                    TokenKind::Operator(Operator::Percent) => (),
                                    _ => {
                                        return Err(anyhow!(
                                            "Unexpected symbol {} in macro args",
                                            symbol.text()
                                        ))
                                    }
                                }
                            }
                            State::Body => {
                                if symbol.token_kind == TokenKind::Identifier {
                                    self.evaluated_define_symbols.push(symbol.clone());
                                }
                                macro_.body.push(symbol);
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
                self.macros.insert(macro_name, macro_);
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
                            self.macros.remove(&symbol.text());
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
                    symbol.token_kind.clone(),
                    Some(&text),
                    Range::new(
                        Position::new(symbol.range.start.line, symbol.range.start.character),
                        Position::new(symbol.range.start.line, text.len() as u32),
                    ),
                    symbol.delta,
                );
                lazy_static! {
                    static ref RE1: Regex = Regex::new(r"<([^>]+)>").unwrap();
                    static ref RE2: Regex = Regex::new("\"([^>]+)\"").unwrap();
                }
                if let Some(caps) = RE1.captures(&text) {
                    if let Some(path) = caps.get(1) {
                        match include_file(
                            &mut self.macros,
                            path.as_str().to_string(),
                            &self.document_uri,
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
                if let Some(caps) = RE2.captures(&text) {
                    if let Some(path) = caps.get(1) {
                        match include_file(
                            &mut self.macros,
                            path.as_str().to_string(),
                            &self.document_uri,
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
