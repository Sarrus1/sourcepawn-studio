use fxhash::FxHashMap;
use sourcepawn_lexer::{Literal, Operator, PreprocDir, Range, SourcepawnLexer, Symbol, TokenKind};

use crate::evaluator::IfCondition;

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    lexer: SourcepawnLexer<'a>,
    current_line: String,
    prev_end: usize,
    conditions_stack: Vec<bool>,
    out: Vec<String>,
    macros: FxHashMap<String, Macro>,
}

#[derive(Debug, Clone)]
pub(crate) struct Macro {
    pub(crate) args: Option<Vec<i8>>,
    pub(crate) body: Vec<Symbol>,
}

impl<'a> SourcepawnPreprocessor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            current_line: "".to_string(),
            prev_end: 0,
            conditions_stack: vec![],
            out: vec![],
            macros: FxHashMap::default(),
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
                TokenKind::Identifier => match self.macros.get(&symbol.text()) {
                    Some(_) => {
                        self.expand_macro(&mut expansion_stack, &symbol);
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

    fn expand_macro(&mut self, expansion_stack: &mut Vec<Symbol>, symbol: &Symbol) {
        let depth = 0;
        let mut stack: Vec<(Symbol, sourcepawn_lexer::Delta, i32)> =
            vec![(symbol.clone(), symbol.delta, depth)];

        while let Some((symbol, delta, d)) = stack.pop() {
            if d == 5 {
                continue;
            }
            match &symbol.token_kind {
                TokenKind::Identifier => {
                    let macro_ = self.macros.get(&symbol.text()).unwrap();
                    if macro_.args.is_none() {
                        for (i, child) in macro_.body.iter().enumerate() {
                            stack.push((
                                child.clone(),
                                if i == 0 { symbol.delta } else { child.delta },
                                d + 1,
                            ));
                        }
                    } else {
                        // Parse the arguments of the macro and prepare to expand them when iterating over the body.
                        let mut paren_depth = 0;
                        let mut entered_args = false;
                        let mut arg_idx = 0;
                        let mut args: Vec<Vec<Symbol>> = vec![];
                        for _ in 0..10 {
                            args.push(vec![]);
                        }
                        while let Some(sub_symbol) = self.lexer.next() {
                            match &sub_symbol.token_kind {
                                TokenKind::LParen => {
                                    entered_args = true;
                                    paren_depth += 1;
                                }
                                TokenKind::RParen => {
                                    paren_depth -= 1;
                                    if entered_args && paren_depth == 0 {
                                        break;
                                    }
                                }
                                TokenKind::Comma => {
                                    if paren_depth == 1 {
                                        arg_idx += 1;
                                    }
                                }
                                _ => args[arg_idx].push(sub_symbol),
                            }
                        }
                        // TODO: Handle escaped.
                        let mut found_percent = false;
                        for (i, child) in macro_.body.iter().enumerate() {
                            match &child.token_kind {
                                TokenKind::Operator(Operator::Percent) => {
                                    found_percent = true;
                                }
                                TokenKind::Literal(Literal::IntegerLiteral) => {
                                    if found_percent {
                                        let arg_idx = child.to_int().unwrap() as usize;
                                        for (i, child) in args[arg_idx].iter().enumerate() {
                                            stack.push((
                                                child.clone(),
                                                if i == 0 { symbol.delta } else { child.delta },
                                                d + 1,
                                            ));
                                        }
                                        found_percent = false;
                                    } else {
                                        stack.push((child.clone(), child.delta, d + 1));
                                    }
                                }
                                _ => stack.push((
                                    child.clone(),
                                    if i == 0 { symbol.delta } else { child.delta },
                                    d + 1,
                                )),
                            }
                        }
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
                let mut if_condition = IfCondition::new(&self.macros);
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
                        if symbol.token_kind != TokenKind::Newline {
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
                                        args[symbol.to_int().unwrap() as usize] = args_idx;
                                    }
                                    TokenKind::Comma => {
                                        args_idx += 1;
                                    }
                                    TokenKind::Operator(Operator::Percent) => (),
                                    _ => unimplemented!("Unexpected token in macro args"),
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
            .push_str(&" ".repeat(symbol.delta.col.unsigned_abs() as usize));
    }

    fn push_current_line(&mut self) {
        self.out.push(self.current_line.clone());
    }
}
