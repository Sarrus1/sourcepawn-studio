#[allow(unused_macros)]
macro_rules! assert_token_eq {
    ($lexer:expr, $token_kind:expr, $text:expr, $start_line:expr, $start_col:expr, $end_line:expr, $end_col:expr) => {
        assert_eq!(
            $lexer.next().unwrap(),
            Symbol::new(
                $token_kind,
                Some($text),
                Range {
                    start_line: $start_line,
                    start_col: $start_col,
                    end_line: $end_line,
                    end_col: $end_col
                }
            )
        );
    };
}

#[allow(unused_imports)]
pub(crate) use assert_token_eq;
