use fxhash::FxHashMap;
use sourcepawn_lexer::{Literal, PreprocDir, SourcepawnLexer, Symbol, TokenKind, Range};

use crate::evaluator::IfCondition;

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    lexer: SourcepawnLexer<'a>,
    current_line: String,
    prev_end: usize,
    conditions_stack: Vec<bool>,
    out: Vec<String>,
    defines_map: FxHashMap<String, Vec<Symbol>>,
}

impl<'a> SourcepawnPreprocessor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            current_line: "".to_string(),
            prev_end: 0,
            conditions_stack: vec![],
            out: vec![],
            defines_map: FxHashMap::default(),
        }
    }

    pub fn preprocess_input(&mut self) -> String {
        let mut expansion_stack = vec![];
        loop {
            let symbol = if !expansion_stack.is_empty() {
                expansion_stack.pop().unwrap()
            } else if let Some(symbol) = self.lexer.next() {
                symbol
            } else {
                break;
            };
            if !self.conditions_stack.is_empty() && !*self.conditions_stack.last().unwrap() {
                self.process_negative_condition(&symbol);
                continue;
            }
            match &symbol.token_kind {
                TokenKind::PreprocDir(dir) => self.process_directive(dir, &symbol),
                TokenKind::Newline => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    self.current_line = "".to_string();
                    self.prev_end = 0;
                }
                TokenKind::Identifier => match self.defines_map.get(&symbol.text()) {
                    Some(_) => {
                        self.expand_define(&mut expansion_stack, &symbol);
                    }
                    None => {
                        self.push_ws(&symbol);
                        self.prev_end = symbol.range.end_col;
                        self.current_line.push_str(&symbol.text());
                    }
                },
                TokenKind::Eof => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    break;
                }
                _ => {
                    self.push_ws(&symbol);
                    self.prev_end = symbol.range.end_col;
                    self.current_line.push_str(&symbol.text());
                }
            }
        }

        self.out.join("\n")
    }

    fn expand_define(&self, expansion_stack: &mut Vec<Symbol>, symbol: &Symbol) {
        let depth = 0;
        let mut stack = vec![(symbol, symbol.delta, depth)];
        while let Some((symbol, delta, d)) = stack.pop() {
            match &symbol.token_kind {
                TokenKind::Identifier => {
                    for (i, child) in self
                        .defines_map
                        .get(&symbol.text())
                        .unwrap()
                        .iter()
                        .enumerate()
                    {
                        stack.push((
                            child,
                            if i == 0 { symbol.delta } else { child.delta },
                            d + 1,
                        ));
                    }
                }
                TokenKind::Literal(Literal::StringLiteral)
                | TokenKind::Literal(Literal::CharLiteral) => {
                    let text = symbol.inline_text();
                    expansion_stack.push(Symbol::new(
                        symbol.token_kind.clone(),
                        Some(&text),
                        Range {
                            start_line: symbol.range.start_line,
                            end_line: symbol.range.start_line,
                            start_col: symbol.range.start_col,
                            end_col: text.len(),
                        },
                        symbol.delta,
                    ));
                }
                TokenKind::Newline | TokenKind::LineContinuation | TokenKind::Comment(_) => (),
                _ => {
                    let mut symbol = symbol.clone();
                    symbol.delta = delta;
                    expansion_stack.push(symbol);
                }
            }
        }
    }

    fn process_directive(&mut self, dir: &PreprocDir, symbol: &Symbol) {
        match dir {
            PreprocDir::MIf => {
                let line_nb = symbol.range.start_line;
                let mut if_condition = IfCondition::new(&self.defines_map);
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        if_condition.symbols.push(symbol);
                    } else {
                        break;
                    }
                }
                self.conditions_stack.push(if_condition.evaluate());
                let line_diff = if_condition.symbols.last().unwrap().range.end_line - line_nb;
                for _ in 0..line_diff {
                    self.out.push(String::new());
                }
                self.prev_end = 0;
            }
            PreprocDir::MDefine => {
                self.push_ws(symbol);
                self.prev_end = symbol.range.end_col;
                self.current_line.push_str(&symbol.text());
                let mut define_name = String::new();
                let mut define_value = vec![];
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        self.push_ws(&symbol);
                        self.prev_end = symbol.range.end_col;
                        if symbol.token_kind != TokenKind::Newline {
                            self.current_line.push_str(&symbol.text());
                        }
                        if define_name.is_empty() {
                            if TokenKind::Identifier == symbol.token_kind {
                                define_name = symbol.text();
                            } else {
                                // We are looking for the define's name.
                                continue;
                            }
                        } else {
                            define_value.push(symbol);
                        }
                    } else {
                        break;
                    }
                }
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
                self.defines_map.insert(define_name, define_value);
            }
            PreprocDir::MEndif => {
                self.conditions_stack.pop();
            }
            _ => todo!(),
        }
    }

    fn process_negative_condition(&mut self, symbol: &Symbol) {
        match &symbol.token_kind {
            TokenKind::PreprocDir(dir) => match dir {
                PreprocDir::MEndif => {
                    self.conditions_stack.pop();
                }
                PreprocDir::MElse => {
                    let last = self.conditions_stack.pop().unwrap();
                    self.conditions_stack.push(!last);
                }
                // TODO: Handle #elseif.
                _ => todo!(),
            },
            TokenKind::Newline => {
                // Keep the newline to keep the line numbers in sync.
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
            }
            // Skip any token that is not a directive or a newline.
            _ => (),
        }
    }

    fn push_ws(&mut self, symbol: &Symbol) {
        self.current_line
            .push_str(&" ".repeat(symbol.delta.col.abs() as usize));
    }

    fn push_current_line(&mut self) {
        self.out.push(self.current_line.clone());
    }
}
