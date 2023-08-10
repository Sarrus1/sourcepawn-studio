use lsp_types::Position;
use sourcepawn_lexer::{SourcepawnLexer, Symbol, TokenKind};
use store::document::Document;

use crate::utils::range_to_position_average;

#[derive(Debug, Default)]
pub(crate) struct SignatureAttributes {
    // Position of the function identifier.
    pub(crate) position: Position,

    // Number of parameters in the function call.
    pub(crate) parameter_count: u32,
}

impl SignatureAttributes {
    /// Build a [SignatureAttributes] object from a [Position] in a [Document].
    ///
    /// # Arguments
    ///
    /// * `document` - [Document] the request was made from.
    /// * `position` - Original [Position] of the trigger character.
    pub(crate) fn get_signature_attributes(
        document: &Document,
        position: Position,
    ) -> Option<SignatureAttributes> {
        let lexer = SourcepawnLexer::new(&document.text);
        let mut function_identifier_stack = vec![];
        let mut last_token: Option<Symbol> = None;
        let mut in_array_literal = false;
        for token in lexer {
            if token.range.start.line > position.line
                || (token.range.start.line == position.line
                    && token.range.start.character >= position.character)
            {
                break;
            }
            match token.token_kind {
                TokenKind::LParen => {
                    if let Some(last_token) = last_token {
                        if last_token.token_kind == TokenKind::Identifier {
                            function_identifier_stack.push((last_token, 0));
                        }
                    }
                }
                TokenKind::LBracket => {
                    in_array_literal = true;
                }
                TokenKind::RBracket => {
                    in_array_literal = false;
                }
                TokenKind::Comma => {
                    if in_array_literal {
                        continue;
                    }
                    if let Some((last_token, count)) = function_identifier_stack.last_mut() {
                        if last_token.token_kind == TokenKind::Identifier {
                            *count += 1;
                        }
                    }
                }
                TokenKind::RParen => {
                    function_identifier_stack.pop();
                }
                _ => (),
            }
            last_token = Some(token);
        }

        if let Some((last_token, parameter_count)) = function_identifier_stack.last() {
            if last_token.token_kind == TokenKind::Identifier {
                return Some(SignatureAttributes {
                    position: range_to_position_average(&last_token.range),
                    parameter_count: *parameter_count,
                });
            }
        }

        None
    }
}
