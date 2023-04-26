use anyhow::anyhow;
use sourcepawn_lexer::Operator;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreOperator {
    /// Unary `!`.
    Not,

    /// Unary `~`.
    Tilde,

    /// Unary `-`.
    Negate,

    /// Unary `+`.
    Confirm,

    /// Binary `*`.
    Star,

    /// Binary `/`.
    Slash,

    /// Binary `%`.
    Percent,

    /// Binary `-`.
    Minus,

    /// Binary `+`.
    Plus,

    /// Binary `<<`.
    Shl,

    /// Binary `>>`.
    Shr,

    /// Binary `>>>`.
    Ushr,

    /// Binary `&`.
    Ampersand,

    /// Binary `^`.
    Bitxor,

    /// Binary `|`.
    Bitor,

    /// Binary `<`.
    Lt,

    /// Binary `<=`.
    Le,

    /// Binary `>`.
    Gt,

    /// Binary `>=`.
    Ge,

    /// Binary `==`.
    Equals,

    /// Binary `!=`.
    NotEquals,

    /// Binary `&&`.
    And,

    /// Binary `||`.
    Or,

    /// Ternary `?`.
    Qmark,

    /// Unary `defined`.
    Defined,

    /// Left parenthesis.
    LParen,

    /// Right parenthesis.
    RParen,
}

impl PreOperator {
    pub fn convert(op: &Operator) -> anyhow::Result<Self> {
        let res = match op {
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
            _ => return Err(anyhow!("Operator {:?} is not a preprocessor operator.", op)),
        };

        Ok(res)
    }

    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            PreOperator::Not
                | PreOperator::Tilde
                | PreOperator::Negate
                | PreOperator::Confirm
                | PreOperator::Defined
        )
    }

    pub fn priority(&self) -> i32 {
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

    pub fn process_op(&self, stack: &mut Vec<i32>) {
        if self.is_unary() {
            let right = stack.pop().unwrap_or(0);
            let result: i32 = match self {
                PreOperator::Not => (!to_bool(right)).into(),
                PreOperator::Tilde => !right,
                PreOperator::Negate => -right,
                PreOperator::Confirm => right,
                _ => unreachable!(),
            };
            stack.push(result);
            return;
        }
        let right = stack.pop().unwrap_or(0); //TODO: Handle error here
        let left = stack.pop().unwrap_or(0);
        let result: i32 = match self {
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
            PreOperator::Bitor => left | right,
            PreOperator::Bitxor => left ^ right,
            PreOperator::Ampersand => left & right,
            PreOperator::Shl => left << right,
            PreOperator::Shr => left >> right,
            PreOperator::Ushr => (left as u32 >> right as u32) as i32,
            PreOperator::Percent => left % right,
            PreOperator::Defined => todo!(),
            PreOperator::Qmark => todo!(),
            PreOperator::Not
            | PreOperator::Tilde
            | PreOperator::Negate
            | PreOperator::Confirm
            | PreOperator::LParen
            | PreOperator::RParen => unreachable!(),
        };
        stack.push(result);
    }
}

fn to_bool<T: std::cmp::PartialEq<i32>>(value: T) -> bool {
    value != 0
}
