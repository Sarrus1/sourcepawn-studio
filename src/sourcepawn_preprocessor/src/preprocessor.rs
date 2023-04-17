use fxhash::FxHashMap;
use sourcepawn_lexer::{PreprocDir, SourcepawnLexer, Symbol, TokenKind};

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
        while let Some(symbol) = self.lexer.next() {
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
                _ => todo!(),
            },
            TokenKind::Newline => {
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
            }
            _ => (),
        }
    }

    fn push_ws(&mut self, symbol: &Symbol) {
        let ws_diff = symbol.range.start_col - self.prev_end;
        self.current_line.push_str(&" ".repeat(ws_diff));
    }

    fn push_current_line(&mut self) {
        self.out.push(self.current_line.clone());
    }
}
