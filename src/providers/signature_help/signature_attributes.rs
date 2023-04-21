use lsp_types::Position;

use crate::document::Document;

#[derive(Debug, Default)]
pub(crate) struct SignatureAttributes {
    // Position of the
    pub(crate) position: Position,
    pub(crate) parameter_count: u32,
}

impl SignatureAttributes {
    /// Build a [SignatureAttributes] object from a [Position] in a [Document].
    ///
    /// Go character by character from the trigger [Position] and count the number of open parenthesis and commas.
    /// If we reach a point where we have more than one unmatched opened parenthesis, we have reached the start
    /// of the method call.
    ///
    /// # Arguments
    ///
    /// * `document` - [Document] the request was made from.
    /// * `position` - Original [Position] of the trigger character.
    pub(crate) fn get_signature_attributes(
        document: Document,
        position: Position,
    ) -> Option<SignatureAttributes> {
        // Provide an initial one offset to counter the initial -1 in the while loop.
        let mut line_nb = position.line as usize + 1;
        let lines = document
            .preprocessed_text
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let mut parameter_count: u32 = 0;
        let mut parenthesis_count: i32 = 0;
        let mut first_loop = true;
        let mut character: usize = 0;
        while parenthesis_count < 1 {
            if line_nb == 0 {
                // We have reached the beginning of the document.
                return None;
            }
            line_nb -= 1;
            let line = &lines[line_nb];
            // Collect the chars of the string to be able to iterate backwards on them
            // by knowing the total length of the vector.
            let chars: Vec<char> = line.chars().collect();
            for (i, char) in chars.iter().enumerate().rev() {
                if first_loop && i >= position.character as usize {
                    // We are before the trigger position.
                    continue;
                }
                match char.to_string().as_str() {
                    "(" => parenthesis_count += 1,
                    ")" => parenthesis_count -= 1,
                    "," => {
                        if !is_in_a_string_or_array(&chars, i) {
                            parameter_count += 1;
                        }
                    }
                    _ => continue,
                }
                character = i;
                if parenthesis_count >= 1 {
                    break;
                }
            }
            first_loop = false;
        }
        // Shift by one character to get the position of the method name token.
        // FIXME: This only works if there is no character between the last ( and the name of the method.
        if character > 0 {
            character -= 1;
        }

        Some(SignatureAttributes {
            position: Position {
                line: line_nb as u32,
                character: character as u32,
            },
            parameter_count: parameter_count as u32,
        })
    }
}

/// Check if the current character is in a string or an array literal.
///
/// The heuristic is that we count the number of opened, unmatched { and " before the
/// given character position. If some are still opened, the character is not a parameter separator.
///
/// # Arguments
///
/// * `chars` - Vector of [char] which represent the line to analyze.
/// * `i` - Index of the  [Position] of the trigger character in the vector of chars.
fn is_in_a_string_or_array(chars: &[char], i: usize) -> bool {
    let mut double_quote_count = 0;
    let mut found_double_quote = false;
    let mut single_quote_count = 0;
    let mut found_single_quote = false;
    let mut bracket_count = 0;

    for char in chars.iter().rev().skip(i) {
        match char.to_string().as_str() {
            "{" => bracket_count += 1,
            "}" => bracket_count -= 1,
            "\"" => {
                found_double_quote = true;
                double_quote_count += 1;
            }
            "'" => {
                found_single_quote = true;
                single_quote_count += 1;
            }
            "\\" => {
                if found_double_quote {
                    found_double_quote = false;
                    double_quote_count -= 1;
                } else if found_single_quote {
                    found_single_quote = false;
                    single_quote_count -= 1;
                }
            }
            _ => continue,
        }
    }

    single_quote_count % 2 == 1 || double_quote_count % 2 == 1 || bracket_count != 0
}
