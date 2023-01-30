use lsp_types::{Position, SignatureHelp, SignatureHelpParams, SignatureInformation};

use crate::{document::Document, spitem::get_items_from_position};

use super::FeatureRequest;

#[derive(Debug, Default)]
struct SignatureAttributes {
    position: Position,
    parameter_count: u32,
}

pub fn provide_signature_help(
    request: FeatureRequest<SignatureHelpParams>,
) -> Option<SignatureHelp> {
    let uri = request
        .params
        .text_document_position_params
        .text_document
        .uri;
    let document = request.store.get(&uri)?;
    let signature_attributes = get_signature_attributes(
        document,
        request.params.text_document_position_params.position,
    )?;
    eprintln!("{:?}", signature_attributes);

    let items = get_items_from_position(&request.store, signature_attributes.position, uri);
    let mut signatures: Vec<SignatureInformation> = Vec::new();
    for item in items {
        let signature_help = item
            .lock()
            .unwrap()
            .to_signature_help(signature_attributes.parameter_count);
        if let Some(signature_help) = signature_help {
            signatures.push(signature_help);
        }
    }
    eprintln!("{:?}", signatures);

    Some(SignatureHelp {
        signatures,
        active_parameter: None,
        active_signature: None,
    })
}

fn get_signature_attributes(document: Document, position: Position) -> Option<SignatureAttributes> {
    let mut line_nb = position.line as usize + 1;
    let lines = document
        .text
        .lines()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    // if line[(position.character - 1) as usize] == ")" {
    //   // We've finished this call
    //   return None;
    // }

    let mut parameter_count: u32 = 0;
    let mut parenthesis_count: i32 = 0;
    let mut first_loop = true;
    let mut character: usize = 0;
    while parenthesis_count < 1 {
        if line_nb == 0 {
            return None;
        }
        line_nb -= 1;
        let line = &lines[line_nb];
        let chars: Vec<char> = line.chars().collect();
        for (i, char) in chars.iter().enumerate().rev() {
            if first_loop && i >= position.character as usize {
                continue;
            }
            match char.to_string().as_str() {
                "(" => parenthesis_count += 1,
                ")" => parenthesis_count -= 1,
                "," => {
                    if !is_in_a_string_or_array(line, i) {
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
    if character > 0 {
        character -= 1;
    }
    let position = Position {
        line: line_nb as u32,
        character: character as u32,
    };

    Some(SignatureAttributes {
        position,
        parameter_count: parameter_count as u32,
    })
}

fn is_in_a_string_or_array(line: &str, i: usize) -> bool {
    let mut double_quote_count = 0;
    let mut found_double_quote = false;
    let mut single_quote_count = 0;
    let mut found_single_quote = false;
    let mut bracket_count = 0;

    let chars: Vec<char> = line.chars().collect();
    for (idx, char) in chars.iter().enumerate().rev() {
        if idx > i {
            continue;
        }
        match char.to_string().as_str() {
            "\"" => {
                found_double_quote = true;
                double_quote_count += 1;
            }
            "'" => {
                found_single_quote = true;
                single_quote_count += 1;
            }
            "{" => {
                bracket_count += 1;
            }
            "}" => {
                bracket_count -= 1;
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
