#[allow(unused_macros)]
macro_rules! assert_token_eq {
    ($lexer:expr, $token_kind:expr, $text:expr, $start_line:expr, $start_col:expr, $end_line:expr, $end_col:expr, $delta_line:expr, $delta_col:expr) => {
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
                },
                Delta {
                    line: $delta_line,
                    col: $delta_col
                }
            )
        );
    };
}

#[allow(unused_imports)]
pub(crate) use assert_token_eq;
