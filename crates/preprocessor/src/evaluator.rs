use fxhash::FxHashMap;
use lsp_types::{Position, Range};
use sourcepawn_lexer::{Literal, Operator, Symbol, TokenKind};
use vfs::FileId;

use super::{
    errors::{EvaluationError, ExpansionError, MacroNotFoundError},
    macros::expand_identifier,
    preprocessor_operator::PreOperator,
};
use crate::{MacrosMap, Offset};

#[derive(Debug)]
pub struct IfCondition<'a> {
    pub symbols: Vec<Symbol>,
    pub(super) macro_not_found_errors: Vec<MacroNotFoundError>,
    macro_store: &'a mut MacrosMap,
    expansion_stack: Vec<Symbol>,
    line_nb: u32,
    offsets: &'a mut FxHashMap<u32, Vec<Offset>>,
}

impl<'a> IfCondition<'a> {
    pub(super) fn new(
        macro_store: &'a mut MacrosMap,
        line_nb: u32,
        offsets: &'a mut FxHashMap<u32, Vec<Offset>>,
    ) -> Self {
        Self {
            symbols: vec![],
            macro_not_found_errors: vec![],
            macro_store,
            expansion_stack: vec![],
            line_nb,
            offsets,
        }
    }

    pub(super) fn evaluate(&mut self) -> Result<bool, EvaluationError> {
        let mut output_queue: Vec<i32> = Vec::new();
        let mut operator_stack: Vec<(PreOperator, Range)> = Vec::new();
        let mut may_be_unary = true;
        let mut looking_for_defined = false;
        let mut current_symbol_range = Range::new(
            Position::new(self.line_nb, 0),
            Position::new(self.line_nb, 1000),
        );
        let mut symbol_iter = self
            .symbols
            .clone() // TODO: This is horrible.
            .into_iter()
            .peekable();
        while let Some(symbol) = if !self.expansion_stack.is_empty() {
            self.expansion_stack.pop()
        } else {
            let symbol = symbol_iter.next();
            if let Some(symbol) = &symbol {
                current_symbol_range = symbol.range;
            }
            symbol
        } {
            match &symbol.token_kind {
                TokenKind::LParen => {
                    operator_stack.push((PreOperator::LParen, symbol.range));
                    if !looking_for_defined {
                        may_be_unary = true;
                    }
                }
                TokenKind::RParen => {
                    while let Some((top, _)) = operator_stack.last() {
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
                                        current_symbol_range,
                                    )
                                })?;
                            op.process_op(&range, &mut output_queue)?;
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
                            current_symbol_range,
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
                    while let Some((top, _)) = operator_stack.last() {
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
                                    current_symbol_range,
                                )
                            })?;
                            op.process_op(&range, &mut output_queue)?;
                        } else {
                            break;
                        }
                    }
                    operator_stack.push((cur_op, symbol.range));
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
                        return Err(EvaluationError::new(
                            format!(
                            "Literal {:?} is not supported in preprocessor expression evaluation.",
                            lit
                        ),
                            current_symbol_range,
                        ))
                    }
                },
                TokenKind::Comment(_) | TokenKind::Newline | TokenKind::Eof => (),
                TokenKind::PreprocDir(_) => {
                    return Err(EvaluationError::new(
                        "Preprocessor directives are not supported in preprocessor expression evaluation."
                            .to_string(),
                        current_symbol_range,
                    ))
                }
                _ => {
                    if looking_for_defined {
                        if let Some(macro_) = self.macro_store.get(&symbol.text()) {
                            self.offsets.entry(symbol.range.start.line).or_default().push(Offset {
                                        file_id: macro_.file_id,
                                        range: symbol.range,
                                        diff: 0, // FIXME: This is the default value, we should calculate it.
                                        idx: macro_.idx
                                    });
                            output_queue.push(1);
                        } else {
                            output_queue.push(0);
                        }
                        looking_for_defined = false;
                        may_be_unary = false;
                    } else {
                        // Skip the macro if it is disabled and reenable it.
                        let mut attr: Option<(u32, FileId)> = None;
                        if let Some(macro_) = self.macro_store.get_mut(&symbol.text()) {
                            attr = (macro_.idx, macro_.file_id).into();
                            if macro_.disabled {
                                macro_.disabled = false;
                                continue;
                            }
                        };
                        match expand_identifier(
                            &mut symbol_iter,
                            self.macro_store,
                            &symbol,
                            &mut self.expansion_stack,
                            false
                        ) {
                            Ok(_) => {
                                if let Some((idx, file_id)) = attr {
                                    self.offsets.entry(symbol.range.start.line).or_default().push(Offset {
                                        file_id,
                                        range: symbol.range,
                                        diff: 0, // FIXME: This is the default value, we should calculate it.
                                        idx
                                    })
                                    ;
                                }
                            }, // No need to keep track of expanded macros here, we do that when calling expand_symbol.
                            Err(ExpansionError::MacroNotFound(err)) => {
                                self.macro_not_found_errors.push(err.clone());
                                return Err(EvaluationError::new(
                                    err.to_string(),
                                    current_symbol_range,
                                ));
                            }
                            Err(ExpansionError::Parse(err)) => {
                                return Err(EvaluationError::new(
                                    err.to_string(),
                                    current_symbol_range,
                                ));
                            }
                        }
                    }
                }
            }
        }
        while let Some((op, range)) = operator_stack.pop() {
            op.process_op(&range, &mut output_queue)?;
        }

        let res = *output_queue.last().ok_or_else(|| {
            EvaluationError::new(
                "Invalid preprocessor condition, expected a result.".to_string(),
                current_symbol_range,
            )
        })?;

        Ok(res != 0)
    }
}

fn is_unary(op: &Operator) -> bool {
    matches!(
        op,
        Operator::Not | Operator::Tilde | Operator::Minus | Operator::Plus
    )
}
