use std::collections::HashMap;

use sourcepawn_lexer::{SourcePawnLexer, Symbol, TokenKind};

pub fn preprocess_input(input: &str) {
    let mut lexer = SourcePawnLexer::new(input);
    while let Some(symbol) = lexer.next() {
        match symbol.token_kind {
            TokenKind::MIf => {
                let mut if_condition = IfCondition::default();
                while let Some(symbol) = lexer.next() {
                    if symbol.token_kind == TokenKind::Newline {
                        break;
                    }
                    if_condition.symbols.push(symbol);
                }
                println!("{:?}", if_condition.evaluate());
            }
            _ => println!("{:?}", symbol),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::preprocess_input;

    #[test]
    fn basic() {
        let input = r#"#if false || true && false"#;
        preprocess_input(input);
    }
}

#[derive(Default, Debug)]
pub struct IfCondition {
    symbols: Vec<Symbol>,
}

impl IfCondition {
    fn evaluate(&self) -> bool {
        let mut output_queue: Vec<Symbol> = vec![];
        let mut operator_stack: Vec<Symbol> = vec![];

        for symbol in &self.symbols {
            match symbol.token_kind {
                TokenKind::Identifier => {
                    // if let Some(value) = macros.get(name) {
                    //     let value = value.parse::<bool>().unwrap_or(false);
                    //     stack.push(value);
                    // } else {
                    //     panic!("Undefined macro: {}", name);
                    // }
                }
                // TokenKind::Or | TokenKind::And => operator = Some(symbol.text.clone()),
                TokenKind::Or
                | TokenKind::And
                | TokenKind::Equals
                | TokenKind::NotEquals
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Le
                | TokenKind::Ge => {
                    let precedence = match symbol.text.as_str() {
                        "==" | "!=" => 2,
                        "<" | ">" | "<=" | ">=" => 3,
                        "&&" => 4,
                        "||" => 5,
                        _ => panic!("Invalid operator: {:?}", &symbol),
                    };
                    while let Some(top) = operator_stack.last() {
                        let top_precedence = match top.text.as_str() {
                            "==" | "!=" => 2,
                            "<" | ">" | "<=" | ">=" => 3,
                            "&&" => 4,
                            "||" => 5,
                            _ => panic!("Invalid operator: {:?}", &top),
                        };
                        if top_precedence >= precedence {
                            output_queue.push(operator_stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                    operator_stack.push(symbol.clone());
                }
                // TokenKind::Equals
                // | TokenKind::NotEquals
                // | TokenKind::Lt
                // | TokenKind::Gt
                // | TokenKind::Le
                // | TokenKind::Ge => {
                //     let right = stack.pop().unwrap_or(false);
                //     let left = stack.pop().unwrap_or(false);
                //     let result = match symbol.text.as_str() {
                //         "==" => left == right,
                //         "!=" => left != right,
                //         "<" => left < right,
                //         ">" => left > right,
                //         "<=" => left <= right,
                //         ">=" => left >= right,
                //         _ => panic!("Invalid operator: {}", symbol.text),
                //     };
                //     stack.push(result);
                // }
                _ => output_queue.push(symbol.clone()),
            }

            while let Some(op) = operator_stack.pop() {
                output_queue.push(op);
            }

            let mut stack: Vec<bool> = vec![];

            for token in output_queue {
                match token {
                    Token::Literal(value) => {
                        let value = value.parse::<bool>().unwrap_or(false);
                        stack.push(value);
                    }
                    Token::Operator(op) => {
                        let right = stack.pop().unwrap_or(false);
                        let left = stack.pop().unwrap_or(false);
                        let result = match op.as_str() {
                            "==" => left == right,
                            "!=" => left != right,
                            "<" => left < right,
                            ">" => left > right,
                            "<=" => left <= right,
                            ">=" => left >= right,
                            "&&" => left && right,
                            "||" => left || right,
                            _ => panic!("Invalid operator: {}", op),
                        };
                        stack.push(result);
                    }
                    _ => panic!("Invalid token in output queue: {:?}", token),
                }
            }

            stack.pop().unwrap_or(false)
        }

        stack.pop().unwrap_or(false)
    }
}
