use logos::Lexer;

use crate::lexer::Token;

pub fn lex_pragma_arguments(lex: &mut Lexer<Token>) -> Option<()> {
    let mut in_block_comment = false;
    let mut looking_for_newline = false;
    let mut ignore_newline = 0;
    let mut offset = 0;
    let mut iter = lex.remainder().chars().peekable();
    while let Some(ch) = iter.next() {
        let mut next_ch = '\0';
        if let Some(ch) = iter.peek() {
            next_ch = *ch;
        }
        if in_block_comment {
            match ch {
                '*' => {
                    if next_ch == '/' {
                        // Exit block comment.
                        in_block_comment = false;
                        looking_for_newline = true;
                    }
                }
                '\\' => match next_ch {
                    '\n' => ignore_newline += 1,
                    '\r' => ignore_newline += 2,
                    _ => {}
                },
                '\n' | '\r' => {
                    if ignore_newline > 0 {
                        // Line continuation.
                        ignore_newline -= 1;
                    } else {
                        // Newline in block comment breaks the pragma.
                        return Some(());
                    }
                }
                _ => {}
            }
            offset += 1;
        } else if looking_for_newline {
            // Lookahead for a newline without any non-whitespace characters.
            if next_ch == '\n' || next_ch == '\r' {
                // Found a newline, the block comment is not part of the pragma.
                return Some(());
            }
            if next_ch.is_whitespace() {
                offset += 1;
            } else {
                // Non-whitespace character found, bump the lexer and continue.
                lex.bump(offset + 2);
                looking_for_newline = false;
            }
        } else {
            match ch {
                '/' => {
                    match next_ch {
                        '/' => {
                            // Line comments break the pragma.
                            return Some(());
                        }
                        '*' => {
                            // Enter block comment.
                            in_block_comment = true;
                            continue;
                        }
                        _ => {}
                    }
                }
                '\n' | '\r' => {
                    if ignore_newline == 0 {
                        // Reached the end of the pragma.
                        return Some(());
                    }
                    // Line continuation.
                    ignore_newline -= 1;
                }
                '\\' => match next_ch {
                    '\n' => ignore_newline += 1,
                    '\r' => ignore_newline += 2,
                    _ => {}
                },
                _ => {}
            }
            lex.bump(1);
        }
    }

    Some(())
}
