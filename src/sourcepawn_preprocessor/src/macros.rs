use sourcepawn_lexer::{Literal, Operator, Range, Symbol, TokenKind};

use crate::SourcepawnPreprocessor;

impl<'a> SourcepawnPreprocessor<'a> {
    pub(crate) fn expand_macro(&mut self, symbol: &Symbol) {
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
                        let mut consecutive_percent = 0;
                        for (i, child) in macro_.body.iter().enumerate() {
                            match &child.token_kind {
                                TokenKind::Operator(Operator::Percent) => {
                                    // Count consecutive % tokens.
                                    // Keep every odd number and if a literal is found, pop the stack to remove it
                                    // and insert the argument instead.
                                    // This allows to preserve the spacing between the last token and the % when
                                    // there is an escaped %.
                                    consecutive_percent += 1;
                                    if consecutive_percent % 2 == 1 {
                                        stack.push((child.clone(), child.delta, d + 1))
                                    }
                                }
                                TokenKind::Literal(Literal::IntegerLiteral) => {
                                    if consecutive_percent == 1 {
                                        stack.pop();
                                        let arg_idx = child.to_int().unwrap() as usize;
                                        for (i, child) in args[arg_idx].iter().enumerate() {
                                            stack.push((
                                                child.clone(),
                                                if i == 0 { symbol.delta } else { child.delta },
                                                d + 1,
                                            ));
                                        }
                                    } else {
                                        stack.push((child.clone(), child.delta, d + 1));
                                    }
                                    consecutive_percent = 0;
                                }
                                _ => {
                                    stack.push((
                                        child.clone(),
                                        if i == 0 { symbol.delta } else { child.delta },
                                        d + 1,
                                    ));
                                    consecutive_percent = 0;
                                }
                            }
                        }
                    }
                }
                TokenKind::Literal(Literal::StringLiteral)
                | TokenKind::Literal(Literal::CharLiteral) => {
                    let text = symbol.inline_text();
                    self.expansion_stack.push(Symbol::new(
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
                    self.expansion_stack.push(symbol);
                }
            }
        }
    }
}
