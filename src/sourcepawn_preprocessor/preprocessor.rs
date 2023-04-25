use std::sync::Arc;

use anyhow::{anyhow, Context};
use fxhash::FxHashMap;
use lsp_types::{Diagnostic, Position, Url};
use sourcepawn_lexer::{Literal, Operator, PreprocDir, Range, SourcepawnLexer, Symbol, TokenKind};

use crate::store::Store;

use super::{evaluator::IfCondition, macros::expand_symbol};

#[derive(Debug, Clone)]
enum ConditionState {
    NotActivated,
    Activated,
    Active,
}

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    pub(crate) lexer: SourcepawnLexer<'a>,
    pub(crate) macros: FxHashMap<String, Macro>,
    pub(crate) expansion_stack: Vec<Symbol>,
    skip_line_start_col: u32,
    skipped_lines: Vec<lsp_types::Range>,
    document_uri: Arc<Url>,
    current_line: String,
    prev_end: usize,
    conditions_stack: Vec<ConditionState>,
    out: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Macro {
    pub(crate) args: Option<Vec<i8>>,
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
            prev_end: 0,
            conditions_stack: vec![],
            out: vec![],
            macros: FxHashMap::default(),
            expansion_stack: vec![],
        }
    }

    pub fn get_disabled_diagnostics(&self) -> Vec<Diagnostic> {
        let mut ranges: Vec<lsp_types::Range> = vec![];
        for range in self.skipped_lines.iter() {
            if let Some(old_range) = ranges.pop() {
                if old_range.end.line == range.start.line - 1 {
                    ranges.push(lsp_types::Range::new(old_range.start, range.end));
                    continue;
                } else {
                    ranges.push(old_range);
                }
            } else {
                ranges.push(*range);
            }
        }
        ranges
            .iter()
            .map(|range| Diagnostic {
                range: *range,
                message: "Code disabled by the preprocessor.".to_string(),
                severity: Some(lsp_types::DiagnosticSeverity::HINT),
                tags: Some(vec![lsp_types::DiagnosticTag::UNNECESSARY]),
                ..Default::default()
            })
            .collect()
    }

    pub fn preprocess_input(&mut self, store: &mut Store) -> anyhow::Result<String> {
        while let Some(symbol) = if !self.expansion_stack.is_empty() {
            self.expansion_stack.pop()
        } else {
            self.lexer.next()
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
                TokenKind::PreprocDir(dir) => self.process_directive(store, dir, &symbol)?,
                TokenKind::Newline => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    self.current_line = "".to_string();
                    self.prev_end = 0;
                }
                TokenKind::Identifier => match self.macros.get(&symbol.text()) {
                    Some(_) => {
                        expand_symbol(
                            &mut self.lexer,
                            &self.macros,
                            &symbol,
                            &mut self.expansion_stack,
                        )?;
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
        let line_nb = symbol.range.start_line;
        let mut if_condition = IfCondition::new(&self.macros);
        while self.lexer.in_preprocessor() {
            if let Some(symbol) = self.lexer.next() {
                if_condition.symbols.push(symbol);
            } else {
                break;
            }
        }
        if if_condition.evaluate().unwrap_or(false) {
            self.conditions_stack.push(ConditionState::Active);
        } else {
            self.skip_line_start_col = symbol.range.end_col as u32;
            self.conditions_stack.push(ConditionState::NotActivated);
        }
        if let Some(last_symbol) = if_condition.symbols.last() {
            let line_diff = last_symbol.range.end_line - line_nb;
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
                self.skip_line_start_col = symbol.range.end_col as u32;
                self.conditions_stack.push(ConditionState::Activated);
            }
        }

        Ok(())
    }

    fn process_directive(
        &mut self,
        store: &mut Store,
        dir: &PreprocDir,
        symbol: &Symbol,
    ) -> anyhow::Result<()> {
        match dir {
            PreprocDir::MIf | PreprocDir::MElseif => self.process_if_directive(symbol),
            PreprocDir::MDefine => {
                self.push_symbol(symbol);
                let mut macro_name = String::new();
                let mut macro_ = Macro {
                    args: None,
                    body: vec![],
                };
                enum State {
                    Start,
                    Args,
                    Body,
                }
                let mut args = vec![-1, 10];
                let mut found_args = false;
                let mut state = State::Start;
                let mut args_idx = 0;
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        self.push_ws(&symbol);
                        self.prev_end = symbol.range.end_col;
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
                                    state = State::Args;
                                } else {
                                    macro_.body.push(symbol);
                                    state = State::Body;
                                }
                            }
                            State::Args => {
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
                                        found_args = true;
                                        args[symbol.to_int().context(format!(
                                            "Could not convert {:?} to an int value.",
                                            symbol.text()
                                        ))?
                                            as usize] = args_idx;
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
                                macro_.body.push(symbol);
                            }
                        }
                    }
                }
                if found_args {
                    macro_.args = Some(args);
                }
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
                self.macros.insert(macro_name, macro_);
            }
            PreprocDir::MEndif => {
                self.conditions_stack.pop();
            }
            PreprocDir::MElse => self.process_else_directive(symbol)?,
            PreprocDir::MInclude => {
                self.push_symbol(symbol);
                let mut delta = 0;
                while self.lexer.in_preprocessor() {
                    if let Some(mut symbol) = self.lexer.next() {
                        match symbol.token_kind {
                            TokenKind::Literal(Literal::StringLiteral) => {
                                // Rewrite the symbol to be a single line.
                                delta += symbol.range.end_line - symbol.range.start_line;
                                let text = symbol.inline_text();
                                symbol = Symbol::new(
                                    symbol.token_kind.clone(),
                                    Some(&text),
                                    Range {
                                        start_line: symbol.range.start_line,
                                        end_line: symbol.range.start_line,
                                        start_col: symbol.range.start_col,
                                        end_col: text.len(),
                                    },
                                    symbol.delta,
                                );

                                let mut path = text[1..text.len() - 1].trim().to_string();
                                if let Some(include_uri) =
                                    store.resolve_import(&mut path, &self.document_uri)
                                {
                                    if let Some(include_macros) =
                                        store.preprocess_document_by_uri(Arc::new(include_uri))
                                    {
                                        self.macros.extend(include_macros);
                                    }
                                }
                            }
                            TokenKind::Eof | TokenKind::Newline => continue, // Ignore the EOF here so that it does not duplicate the current line.
                            _ => (),
                        }
                        self.push_symbol(&symbol);
                    }
                }
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
                for _ in 0..delta {
                    self.out.push(String::new());
                }
            }
            _ => self.push_symbol(symbol),
        }

        Ok(())
    }

    fn process_negative_condition(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        match &symbol.token_kind {
            TokenKind::PreprocDir(dir) => match dir {
                PreprocDir::MEndif => {
                    self.conditions_stack.pop();
                }
                PreprocDir::MElse => self.process_else_directive(symbol)?,
                PreprocDir::MElseif => {
                    let last = self
                        .conditions_stack
                        .pop()
                        .context("Expect if before else clause.")?;
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
                    Position::new(symbol.range.start_line as u32, self.skip_line_start_col),
                    Position::new(
                        symbol.range.start_line as u32,
                        symbol.range.start_col as u32,
                    ),
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
        self.prev_end = symbol.range.end_col;
        self.current_line.push_str(&symbol.text());
    }
}
