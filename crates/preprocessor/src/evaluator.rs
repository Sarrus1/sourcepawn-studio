use sourcepawn_lexer::{Comment, Literal, Operator, Symbol, TextRange, TokenKind};

use super::{
    errors::{EvaluationError, ExpansionError, MacroNotFoundError},
    macros::expand_identifier,
    preprocessor_operator::PreOperator,
};
use crate::{linebreak_count, offset::SourceMap, MacroStore};

#[derive(Debug, Default)]
struct OperatorStack {
    stack: Vec<(PreOperator, TextRange)>,
}

impl OperatorStack {
    pub fn push(&mut self, operator: PreOperator, range: TextRange) {
        self.stack.push((operator, range));
    }

    pub fn pop(&mut self) -> Option<(PreOperator, TextRange)> {
        self.stack.pop()
    }

    pub fn top(&mut self) -> Option<&(PreOperator, TextRange)> {
        self.stack.last()
    }
}

#[derive(Debug, Default)]
pub(crate) struct OutputStack {
    stack: Vec<i32>,
}

impl OutputStack {
    pub fn push(&mut self, value: i32) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<i32> {
        self.stack.pop()
    }

    pub fn top(&mut self) -> Option<&i32> {
        self.stack.last()
    }
}

#[derive(Debug)]
pub struct IfCondition<'a> {
    pub symbols: Vec<Symbol>,
    pub(super) macro_not_found_errors: Vec<MacroNotFoundError>,
    macro_store: &'a mut MacroStore,
    expansion_stack: Vec<Symbol>,
    line_continuation_count: u32,
    source_map: &'a mut SourceMap,
}

impl<'a> IfCondition<'a> {
    pub(super) fn new(macro_store: &'a mut MacroStore, source_map: &'a mut SourceMap) -> Self {
        Self {
            symbols: vec![],
            macro_not_found_errors: vec![],
            macro_store,
            expansion_stack: vec![],
            line_continuation_count: Default::default(),
            source_map,
        }
    }

    pub(super) fn evaluate(&mut self) -> Result<bool, EvaluationError> {
        let mut output_stack = OutputStack::default();
        let mut operator_stack = OperatorStack::default();
        let mut may_be_unary = true;
        let mut looking_for_defined = false;
        let mut symbol_iter = self
            .symbols
            .clone() // TODO: This is horrible.
            .into_iter()
            .peekable();
        while let Some(symbol) = if !self.expansion_stack.is_empty() {
            self.expansion_stack.pop()
        } else {
            symbol_iter.next()
        } {
            match &symbol.token_kind {
                TokenKind::LineContinuation | TokenKind::Newline => self.line_continuation_count += 1,
                TokenKind::Comment(Comment::BlockComment) => {
                   self.line_continuation_count +=  linebreak_count(symbol.text().as_str()) as u32;
                }
                TokenKind::LParen => {
                    operator_stack.push(PreOperator::LParen, symbol.range);
                    if !looking_for_defined {
                        may_be_unary = true;
                    }
                }
                TokenKind::RParen => {
                    while let Some((top, _)) = operator_stack.top() {
                        if PreOperator::LParen == *top {
                            operator_stack.pop();
                            may_be_unary = false;
                            break;
                        } else {
                            let (op, range) = operator_stack
                                .pop()
                                .ok_or_else(|| {
                                    EvaluationError::new(
                                        "Invalid preprocessor condition, expected an operator before ) token."
                                            .to_string(),
                                        symbol.range,
                                    )
                                })?;
                            op.process_op(&range, &mut output_stack)?;
                        }
                    }
                }
                TokenKind::Defined => {
                    looking_for_defined = true;
                }
                TokenKind::Operator(op) => {
                    let mut cur_op = PreOperator::convert(op).ok().ok_or_else(|| {
                        EvaluationError::new(
                            "Invalid preprocessor condition, expected a result.".to_string(),
                            symbol.range,
                        )
                    })?;
                    if may_be_unary && is_unary(op) {
                        cur_op = match op {
                            Operator::Not => PreOperator::Not,
                            Operator::Tilde => PreOperator::Tilde,
                            Operator::Minus => PreOperator::Negate,
                            Operator::Plus => PreOperator::Confirm,
                            _ => unreachable!(),
                        };
                    }
                    while let Some((top, _)) = operator_stack.top() {
                        if top == &PreOperator::LParen {
                            break;
                        }
                        if (!cur_op.is_unary() && top.priority() <= cur_op.priority())
                            || (cur_op.is_unary() && top.priority() < cur_op.priority())
                        {
                            let (op, range) = operator_stack.pop().ok_or_else(|| {
                                EvaluationError::new(
                                    "Invalid preprocessor condition, expected an operator."
                                        .to_string(),
                                    symbol.range,
                                )
                            })?;
                            op.process_op(&range, &mut output_stack)?;
                        } else {
                            break;
                        }
                    }
                    operator_stack.push(cur_op, symbol.range);
                    may_be_unary = true;
                }
                TokenKind::True => {
                    output_stack.push(1);
                    may_be_unary = false;
                }
                TokenKind::False => {
                    output_stack.push(0);
                    may_be_unary = false;
                }
                TokenKind::Literal(lit) => match lit {
                    Literal::IntegerLiteral
                    | Literal::BinaryLiteral
                    | Literal::HexLiteral
                    | Literal::OctodecimalLiteral
                    | Literal::CharLiteral => {
                        output_stack.push(symbol.to_int().unwrap_or(0) as i32);
                        may_be_unary = false;
                    }
                    _ => {
                        return Err(EvaluationError::new(
                            format!(
                            "Literal {:?} is not supported in preprocessor expression evaluation.",
                            lit
                        ),
                            symbol.range,
                        ))
                    }
                },
                TokenKind::Comment(_) | TokenKind::Eof => (),
                TokenKind::PreprocDir(_) => {
                    return Err(EvaluationError::new(
                        "Preprocessor directives are not supported in preprocessor expression evaluation."
                            .to_string(),
                        symbol.range,
                    ))
                }
                _ => {
                    if looking_for_defined {
                        if let Some(macro_) = self.macro_store.get(&symbol.text()) {
                            self.source_map.push_expanded_symbol(symbol.range, symbol.range.start().into(), symbol.range.end().into(), macro_); // FIXME: This is wrong.
                            output_stack.push(1);
                        } else {
                            output_stack.push(0);
                        }
                        looking_for_defined = false;
                        may_be_unary = false;
                    } else {
                        // Skip the macro if it is disabled and reenable it.
                        if let Some(macro_) = self.macro_store.get(&symbol.text()).cloned() {
                            if self.macro_store.is_macro_disabled(&macro_) {
                                self.macro_store.enable_macro(&macro_);
                                continue;
                            }
                        };
                        match expand_identifier(
                            &mut symbol_iter,
                            self.macro_store,
                            &symbol,
                            &mut self.expansion_stack,
                            false,
                        ) {
                            Ok(r_paren_offset) => {
                                if let Some(macro_) = self.macro_store.get(&symbol.text()) {
                                    let s_range = if let Some(r_paren_offset) = r_paren_offset{
                                        TextRange::new(symbol.range.start(), r_paren_offset)
                                    } else {
                                        symbol.range
                                    };
                                    self.source_map.push_expanded_symbol(s_range, symbol.range.start().into(), symbol.range.end().into(), macro_);
                                }
                            }, // No need to keep track of expanded macros here, we do that when calling expand_symbol.
                            Err(ExpansionError::MacroNotFound(err)) => {
                                self.macro_not_found_errors.push(err.clone());
                                return Err(EvaluationError::new(
                                    "Unresolved macro".into(), // The error is already propagated in `macro_not_found_errors`.
                                    symbol.range,
                                ));
                            }
                            Err(ExpansionError::Parse(err)) => {
                                return Err(EvaluationError::new(
                                    err.to_string(),
                                    symbol.range,
                                ));
                            }
                        }
                    }
                }
            }
        }
        while let Some((op, range)) = operator_stack.pop() {
            op.process_op(&range, &mut output_stack)?;
        }

        let res = *output_stack.top().ok_or_else(|| {
            EvaluationError::new(
                "Invalid preprocessor condition, expected a result.".to_string(),
                TextRange::default(), // FIXME: Default range
            )
        })?;

        Ok(res != 0)
    }

    pub fn line_continuation_count(&self) -> u32 {
        self.line_continuation_count
    }
}

fn is_unary(op: &Operator) -> bool {
    matches!(
        op,
        Operator::Not | Operator::Tilde | Operator::Minus | Operator::Plus
    )
}
