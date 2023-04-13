use sourcepawn_lexer::{SourcepawnLexer, Symbol, TokenKind};

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    lexer: SourcepawnLexer<'a>,
    current_line: String,
    prev_end: usize,
}

impl<'a> SourcepawnPreprocessor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            current_line: "".to_string(),
            prev_end: 0,
        }
    }
    pub fn preprocess_input(&mut self) -> String {
        let mut out: Vec<String> = vec![];
        while let Some(symbol) = self.lexer.next() {
            match symbol.token_kind {
                // TokenKind::MIf => {
                //     let mut if_condition = IfCondition::default();
                //     while let Some(symbol) = lexer.next() {
                //         if symbol.token_kind == TokenKind::Newline {
                //             break;
                //         }
                //         if_condition.symbols.push(symbol);
                //     }
                //     println!("{:?}", if_condition.evaluate());
                // }
                TokenKind::Newline => {
                    self.push_ws(&symbol);
                    out.push(self.current_line.clone());
                    self.current_line = "".to_string();
                    self.prev_end = 0;
                }
                TokenKind::Eof => {
                    self.push_ws(&symbol);
                    out.push(self.current_line.clone());
                    break;
                }
                _ => {
                    self.push_ws(&symbol);
                    self.prev_end = symbol.range.end_col;
                    self.current_line.push_str(&symbol.text());
                }
            }
        }

        out.join("\n")
    }

    fn push_ws(&mut self, symbol: &Symbol) {
        let ws_diff = symbol.range.start_col - self.prev_end;
        self.current_line.push_str(&" ".repeat(ws_diff));
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
}

#[derive(Default, Debug)]
pub struct IfCondition {
    symbols: Vec<Symbol>,
}

// impl IfCondition {
//     fn evaluate(&self) -> bool {
//         let mut output_queue: Vec<Symbol> = vec![];
//         let mut operator_stack: Vec<Symbol> = vec![];

//         for symbol in &self.symbols {
//             match symbol.token_kind {
//                 TokenKind::Identifier => {
//                     // if let Some(value) = macros.get(name) {
//                     //     let value = value.parse::<bool>().unwrap_or(false);
//                     //     stack.push(value);
//                     // } else {
//                     //     panic!("Undefined macro: {}", name);
//                     // }
//                 }
//                 // TokenKind::Or | TokenKind::And => operator = Some(symbol.text.clone()),
//                 TokenKind::Or
//                 | TokenKind::And
//                 | TokenKind::Equals
//                 | TokenKind::NotEquals
//                 | TokenKind::Lt
//                 | TokenKind::Gt
//                 | TokenKind::Le
//                 | TokenKind::Ge => {
//                     let precedence = match symbol.text.as_str() {
//                         "==" | "!=" => 2,
//                         "<" | ">" | "<=" | ">=" => 3,
//                         "&&" => 4,
//                         "||" => 5,
//                         _ => panic!("Invalid operator: {:?}", &symbol),
//                     };
//                     while let Some(top) = operator_stack.last() {
//                         let top_precedence = match top.text.as_str() {
//                             "==" | "!=" => 2,
//                             "<" | ">" | "<=" | ">=" => 3,
//                             "&&" => 4,
//                             "||" => 5,
//                             _ => panic!("Invalid operator: {:?}", &top),
//                         };
//                         if top_precedence >= precedence {
//                             output_queue.push(operator_stack.pop().unwrap());
//                         } else {
//                             break;
//                         }
//                     }
//                     operator_stack.push(symbol.clone());
//                 }
//                 // TokenKind::Equals
//                 // | TokenKind::NotEquals
//                 // | TokenKind::Lt
//                 // | TokenKind::Gt
//                 // | TokenKind::Le
//                 // | TokenKind::Ge => {
//                 //     let right = stack.pop().unwrap_or(false);
//                 //     let left = stack.pop().unwrap_or(false);
//                 //     let result = match symbol.text.as_str() {
//                 //         "==" => left == right,
//                 //         "!=" => left != right,
//                 //         "<" => left < right,
//                 //         ">" => left > right,
//                 //         "<=" => left <= right,
//                 //         ">=" => left >= right,
//                 //         _ => panic!("Invalid operator: {}", symbol.text),
//                 //     };
//                 //     stack.push(result);
//                 // }
//                 _ => output_queue.push(symbol.clone()),
//             }

//             while let Some(op) = operator_stack.pop() {
//                 output_queue.push(op);
//             }

//             let mut stack: Vec<bool> = vec![];

//             for token in output_queue {
//                 match token {
//                     Token::Literal(value) => {
//                         let value = value.parse::<bool>().unwrap_or(false);
//                         stack.push(value);
//                     }
//                     Token::Operator(op) => {
//                         let right = stack.pop().unwrap_or(false);
//                         let left = stack.pop().unwrap_or(false);
//                         let result = match op.as_str() {
//                             "==" => left == right,
//                             "!=" => left != right,
//                             "<" => left < right,
//                             ">" => left > right,
//                             "<=" => left <= right,
//                             ">=" => left >= right,
//                             "&&" => left && right,
//                             "||" => left || right,
//                             _ => panic!("Invalid operator: {}", op),
//                         };
//                         stack.push(result);
//                     }
//                     _ => panic!("Invalid token in output queue: {:?}", token),
//                 }
//             }

//             stack.pop().unwrap_or(false)
//         }

//         stack.pop().unwrap_or(false)
//     }
// }
