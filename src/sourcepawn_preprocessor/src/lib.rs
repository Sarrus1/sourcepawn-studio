use sourcepawn_lexer::{Literal, Operator, PreprocDir, SourcepawnLexer, Symbol, TokenKind};

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    lexer: SourcepawnLexer<'a>,
    current_line: String,
    prev_end: usize,
    conditions_stack: Vec<bool>,
    out: Vec<String>,
}

impl<'a> SourcepawnPreprocessor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            current_line: "".to_string(),
            prev_end: 0,
            conditions_stack: vec![],
            out: vec![],
        }
    }
    pub fn preprocess_input(&mut self) -> String {
        while let Some(symbol) = self.lexer.next() {
            match symbol.token_kind {
                TokenKind::PreprocDir(PreprocDir::MIf) => {
                    let mut if_condition = IfCondition::default();
                    while self.lexer.in_preprocessor() {
                        if let Some(symbol) = self.lexer.next() {
                            if_condition.symbols.push(symbol);
                        }
                    }
                    eprintln!("Evaluate {}", if_condition.evaluate());
                }
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

    fn push_ws(&mut self, symbol: &Symbol) {
        let ws_diff = symbol.range.start_col - self.prev_end;
        self.current_line.push_str(&" ".repeat(ws_diff));
    }

    fn push_current_line(&mut self) {
        if self.conditions_stack.is_empty() {
            self.out.push(self.current_line.clone());
        }
    }
}
#[cfg(test)]
mod test {
    use crate::SourcepawnPreprocessor;

    #[test]
    fn no_preprocessor_directives() {
        let input = r#"
        int foo;
        int bar;
        "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), input);
    }

    #[test]
    fn if_directive() {
        let input = r#"
        #if (1+1 == 2) && (1+1==2)
        "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        preprocessor.preprocess_input();
    }
}

#[derive(Default, Debug)]
pub struct IfCondition {
    symbols: Vec<Symbol>,
}

impl IfCondition {
    pub fn evaluate(&self) -> bool {
        let val = self.yard();
        eprintln!("Val: {}", val);
        val != 0
    }

    fn yard(&self) -> i32 {
        let mut output_queue: Vec<i32> = vec![];
        let mut operator_stack: Vec<PreOperator> = vec![];
        let mut may_be_unary = true;
        for symbol in &self.symbols {
            match &symbol.token_kind {
                TokenKind::LParen => {
                    operator_stack.push(PreOperator::LParen);
                    may_be_unary = true;
                }
                TokenKind::RParen => {
                    while let Some(top) = operator_stack.last() {
                        if PreOperator::LParen == *top {
                            operator_stack.pop();
                            may_be_unary = false;
                            break;
                        } else {
                            process_op(&mut output_queue, &operator_stack.pop().unwrap());
                        }
                    }
                }
                TokenKind::Operator(op) => {
                    let mut cur_op = PreOperator::from_op(op);
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
                            process_op(&mut output_queue, &operator_stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                    operator_stack.push(cur_op);
                    may_be_unary = true;
                }
                // TokenKind::True | TokenKind::False | TokenKind::Literal(_) => {
                //     output_queue.push(symbol.clone())
                // }
                TokenKind::True => {
                    output_queue.push(1);
                    may_be_unary = false;
                }
                TokenKind::False => {
                    output_queue.push(0);
                    may_be_unary = false;
                }
                TokenKind::Literal(lit) => match lit {
                    Literal::IntegerLiteral => {
                        output_queue.push(symbol.text().parse().unwrap());
                        may_be_unary = false;
                    }
                    _ => todo!("Literal: {:?}", lit),
                },
                TokenKind::Comment(_) | TokenKind::Newline => (),
                _ => todo!("TokenKind: {:?}", &symbol.token_kind),
            }
        }
        eprintln!("Output: {:?}", output_queue);
        while !operator_stack.is_empty() {
            process_op(&mut output_queue, &operator_stack.pop().unwrap());
        }

        *output_queue.last().unwrap()
    }
}

fn to_bool<T: std::cmp::PartialEq<i32>>(value: T) -> bool {
    value != 0
}

fn is_unary(op: &Operator) -> bool {
    matches!(
        op,
        Operator::Not | Operator::Tilde | Operator::Minus | Operator::Plus
    )
}

fn process_op(stack: &mut Vec<i32>, op: &PreOperator) {
    let right = stack.pop().unwrap_or(0);
    let left = stack.pop().unwrap_or(0);
    let result: i32 = match op {
        PreOperator::Equals => (left == right).into(),
        PreOperator::NotEquals => (left != right).into(),
        PreOperator::Lt => (left < right).into(),
        PreOperator::Gt => (left > right).into(),
        PreOperator::Le => (left <= right).into(),
        PreOperator::Ge => (left >= right).into(),
        PreOperator::Plus => left + right,
        PreOperator::Minus => left - right,
        PreOperator::Slash => left / right,
        PreOperator::Star => left * right,
        PreOperator::And => (to_bool(left) && to_bool(right)).into(),
        PreOperator::Or => (to_bool(left) || to_bool(right)).into(),
        _ => todo!("Operator: {:?}", op),
    };
    stack.push(result);
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PreOperator {
    Not,
    Tilde,

    /// Unary `-`.
    Negate,

    /// Unary `+`.
    Confirm,
    Star,
    Slash,
    Percent,
    Minus,
    Plus,
    Shl,
    Shr,
    Ushr,
    Ampersand,
    Bitxor,
    Bitor,
    Lt,
    Le,
    Gt,
    Ge,
    Equals,
    NotEquals,
    And,
    Or,
    Qmark,
    Defined,
    LParen,
    RParen,
}

impl PreOperator {
    fn from_op(op: &Operator) -> Self {
        match op {
            Operator::Not => PreOperator::Not,
            Operator::Tilde => PreOperator::Tilde,
            Operator::Star => PreOperator::Star,
            Operator::Slash => PreOperator::Slash,
            Operator::Percent => PreOperator::Percent,
            Operator::Minus => PreOperator::Minus,
            Operator::Plus => PreOperator::Plus,
            Operator::Shl => PreOperator::Shl,
            Operator::Shr => PreOperator::Shr,
            Operator::Ushr => PreOperator::Ushr,
            Operator::Ampersand => PreOperator::Ampersand,
            Operator::Bitxor => PreOperator::Bitxor,
            Operator::Bitor => PreOperator::Bitor,
            Operator::Lt => PreOperator::Lt,
            Operator::Le => PreOperator::Le,
            Operator::Gt => PreOperator::Gt,
            Operator::Ge => PreOperator::Ge,
            Operator::Equals => PreOperator::Equals,
            Operator::NotEquals => PreOperator::NotEquals,
            Operator::And => PreOperator::And,
            Operator::Or => PreOperator::Or,
            _ => todo!("Operator: {:?}", op),
        }
    }

    fn is_unary(&self) -> bool {
        matches!(
            self,
            PreOperator::Not
                | PreOperator::Tilde
                | PreOperator::Negate
                | PreOperator::Confirm
                | PreOperator::Defined
        )
    }

    fn priority(&self) -> i32 {
        match self {
            PreOperator::Not
            | PreOperator::Tilde
            | PreOperator::Negate
            | PreOperator::Confirm
            | PreOperator::Defined => 2,
            PreOperator::Star | PreOperator::Slash | PreOperator::Percent => 3,
            PreOperator::Minus | PreOperator::Plus => 4,
            PreOperator::Shl | PreOperator::Shr | PreOperator::Ushr => 5,
            PreOperator::Ampersand => 6,
            PreOperator::Bitxor => 7,
            PreOperator::Bitor => 8,
            PreOperator::Lt | PreOperator::Le | PreOperator::Gt | PreOperator::Ge => 9,
            PreOperator::Equals | PreOperator::NotEquals => 10,
            PreOperator::And => 11,
            PreOperator::Or => 12,
            PreOperator::Qmark => 13,
            PreOperator::LParen | PreOperator::RParen => panic!("Invalid operator: {:?}", &self),
        }
    }
}
