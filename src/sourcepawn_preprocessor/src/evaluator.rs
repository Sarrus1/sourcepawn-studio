use fxhash::FxHashMap;
use sourcepawn_lexer::{Literal, Operator, Symbol, TokenKind};

use crate::{macros::expand_symbol, preprocessor::Macro, preprocessor_operator::PreOperator};

#[derive(Debug)]
pub struct IfCondition<'a> {
    pub symbols: Vec<Symbol>,
    macros: &'a FxHashMap<String, Macro>,
    expansion_stack: Vec<Symbol>,
}

impl<'a> IfCondition<'a> {
    pub(crate) fn new(macros: &'a FxHashMap<String, Macro>) -> Self {
        Self {
            symbols: vec![],
            macros,
            expansion_stack: vec![],
        }
    }

    pub fn evaluate(&mut self) -> bool {
        let val = self.yard();
        val != 0
    }

    fn yard(&mut self) -> i32 {
        let mut output_queue: Vec<i32> = vec![];
        let mut operator_stack: Vec<PreOperator> = vec![];
        let mut may_be_unary = true;
        let mut looking_for_defined = false;
        let mut symbol_iter = self
            .symbols
            .clone() // FIXME: This is horrible.
            .into_iter()
            .peekable();
        while symbol_iter.peek().is_some() || !self.expansion_stack.is_empty() {
            let symbol = if !self.expansion_stack.is_empty() {
                self.expansion_stack.pop().unwrap()
            } else {
                symbol_iter.next().unwrap()
            };
            match &symbol.token_kind {
                TokenKind::LParen => {
                    operator_stack.push(PreOperator::LParen);
                    if !looking_for_defined {
                        may_be_unary = true;
                    }
                }
                TokenKind::Identifier => {
                    if looking_for_defined {
                        output_queue.push(self.macros.contains_key(&symbol.text()).into());
                        looking_for_defined = false;
                        may_be_unary = false;
                    } else {
                        // TODO: Handle function-like macros.
                        expand_symbol(
                            &mut symbol_iter,
                            &self.macros,
                            &symbol,
                            &mut self.expansion_stack,
                        )
                    }
                }
                TokenKind::RParen => {
                    while let Some(top) = operator_stack.last() {
                        if PreOperator::LParen == *top {
                            operator_stack.pop();
                            may_be_unary = false;
                            break;
                        } else {
                            operator_stack.pop().unwrap().process_op(&mut output_queue);
                        }
                    }
                }
                TokenKind::Defined => {
                    looking_for_defined = true;
                }
                TokenKind::Operator(op) => {
                    let mut cur_op = PreOperator::from(op);
                    if may_be_unary && is_unary(op) {
                        cur_op = match op {
                            Operator::Not => PreOperator::Not,
                            Operator::Tilde => PreOperator::Tilde,
                            Operator::Minus => PreOperator::Negate,
                            Operator::Plus => PreOperator::Confirm,
                            _ => unreachable!(),
                        };
                    }
                    while let Some(top) = operator_stack.last() {
                        if top == &PreOperator::LParen {
                            break;
                        }
                        if (!cur_op.is_unary() && top.priority() <= cur_op.priority())
                            || (cur_op.is_unary() && top.priority() < cur_op.priority())
                        {
                            operator_stack.pop().unwrap().process_op(&mut output_queue);
                        } else {
                            break;
                        }
                    }
                    operator_stack.push(cur_op);
                    may_be_unary = true;
                }
                TokenKind::True => {
                    output_queue.push(1);
                    may_be_unary = false;
                }
                TokenKind::False => {
                    output_queue.push(0);
                    may_be_unary = false;
                }
                TokenKind::Literal(lit) => match lit {
                    Literal::IntegerLiteral
                    | Literal::BinaryLiteral
                    | Literal::HexLiteral
                    | Literal::OctodecimalLiteral
                    | Literal::CharLiteral => {
                        output_queue.push(symbol.to_int().unwrap_or(0) as i32);
                        may_be_unary = false;
                    }
                    _ => {
                        unimplemented!(
                            "Literal {:?} is not supported in expression evaluation.",
                            lit
                        )
                    }
                },
                TokenKind::Comment(_) | TokenKind::Newline | TokenKind::Eof => (),
                _ => todo!("TokenKind: {:?}", &symbol.token_kind),
            }
        }
        while !operator_stack.is_empty() {
            operator_stack.pop().unwrap().process_op(&mut output_queue);
        }

        *output_queue.last().unwrap()
    }
}

fn is_unary(op: &Operator) -> bool {
    matches!(
        op,
        Operator::Not | Operator::Tilde | Operator::Minus | Operator::Plus
    )
}
