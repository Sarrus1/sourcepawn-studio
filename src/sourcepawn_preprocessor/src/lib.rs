use fxhash::FxHashMap;
use preprocessor_operator::PreOperator;
use sourcepawn_lexer::{Literal, Operator, PreprocDir, SourcepawnLexer, Symbol, TokenKind};

mod preprocessor_operator;

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
                match symbol.token_kind {
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
                continue;
            }
            match &symbol.token_kind {
                TokenKind::PreprocDir(dir) => match dir {
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
                        let line_diff =
                            if_condition.symbols.last().unwrap().range.end_line - line_nb;
                        for _ in 0..line_diff {
                            self.out.push(String::new());
                        }
                        self.prev_end = 0;
                    }
                    PreprocDir::MDefine => {
                        self.push_ws(&symbol);
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
                },
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
        self.out.push(self.current_line.clone());
    }
}

#[derive(Debug)]
pub struct IfCondition<'a> {
    symbols: Vec<Symbol>,
    defines_map: &'a FxHashMap<String, Vec<Symbol>>,
}

impl<'a> IfCondition<'a> {
    pub fn new(defines_map: &'a FxHashMap<String, Vec<Symbol>>) -> Self {
        Self {
            symbols: vec![],
            defines_map,
        }
    }

    pub fn evaluate(&self) -> bool {
        let val = self.yard();
        val != 0
    }

    fn yard(&self) -> i32 {
        let mut output_queue: Vec<i32> = vec![];
        let mut operator_stack: Vec<PreOperator> = vec![];
        let mut may_be_unary = true;
        let mut looking_for_defined = false;
        for symbol in &self.symbols {
            match &symbol.token_kind {
                TokenKind::LParen => {
                    operator_stack.push(PreOperator::LParen);
                    if !looking_for_defined {
                        may_be_unary = true;
                    }
                }
                TokenKind::Identifier => {
                    if looking_for_defined {
                        output_queue.push(self.defines_map.contains_key(&symbol.text()).into());
                        looking_for_defined = false;
                        may_be_unary = false;
                    } else {
                        todo!("Identifier: {:?}", symbol.text());
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
                            &operator_stack.pop().unwrap().process_op(&mut output_queue);
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
                    Literal::IntegerLiteral => {
                        output_queue.push(symbol.text().parse().unwrap());
                        may_be_unary = false;
                    }
                    _ => todo!("Literal: {:?}", lit),
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

#[cfg(test)]
mod test {
    use fxhash::FxHashMap;
    use sourcepawn_lexer::{SourcepawnLexer, Symbol, TokenKind};

    use crate::{IfCondition, SourcepawnPreprocessor};

    #[test]
    fn no_preprocessor_directives() {
        let input = r#"
        int foo;
        int bar;
        "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), input);
    }

    fn evaluate_if_condition(input: &str) -> bool {
        let mut lexer = SourcepawnLexer::new(input);
        let defines_map: FxHashMap<String, Vec<Symbol>> = FxHashMap::default();
        let mut if_condition = IfCondition::new(&defines_map);
        if let Some(symbol) = lexer.next() {
            if TokenKind::PreprocDir(sourcepawn_lexer::PreprocDir::MIf) == symbol.token_kind {
                while lexer.in_preprocessor() {
                    if let Some(symbol) = lexer.next() {
                        if_condition.symbols.push(symbol);
                    } else {
                        break;
                    }
                }
            }
        }

        if_condition.evaluate()
    }

    #[test]
    fn if_directive_simple_true() {
        let input = r#"#if 1"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_false() {
        let input = r#"#if 0"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_true_with_ws() {
        let input = r#"#if 1 "#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_true_parenthesis() {
        let input = r#"#if (1)"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_binary_true() {
        let input = r#"#if 1+1"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_binary_false() {
        let input = r#"#if 1-1"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_unary_false() {
        let input = r#"#if !1"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_equality_true() {
        let input = r#"#if 1 == 1"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_difference_true() {
        let input = r#"#if 1 != 0"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_equality_false() {
        let input = r#"#if 1 == 0"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_difference_false() {
        let input = r#"#if 1 != 1"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_complexe_expression_1() {
        let input = r#"#if (1 + 1) && (0 + 0)"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_complexe_expression_2() {
        let input = r#"#if (true && 1) || (true + 1)"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_defined() {
        let input = r#"#define FOO
#if defined FOO
    int foo;
#endif"#;
        let output = r#"#define FOO

    int foo;
      "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_1() {
        let input = r#"#define FOO
#if defined FOO && defined BAR
    int foo;
    int bar;
#endif"#;
        let output = r#"#define FOO



      "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_2() {
        let input = r#"#define FOO
#define BAR
#if defined FOO && defined BAR
    int foo;
    int bar;
#endif"#;
        let output = r#"#define FOO
#define BAR

    int foo;
    int bar;
      "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_3() {
        let input = r#"#define FOO
#define BAR
#if defined FOO
    int foo;
    #if defined BAR
    int bar;
    #endif
#endif"#;
        let output = r#"#define FOO
#define BAR

    int foo;

    int bar;
          
      "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_4() {
        let input = r#"#define FOO
#if defined FOO
    int foo;
    #if defined BAZ
    int bar;
    #else
    int baz;
    #endif
#endif"#;
        let output = r#"#define FOO

    int foo;


         
    int baz;
          
      "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }
}
