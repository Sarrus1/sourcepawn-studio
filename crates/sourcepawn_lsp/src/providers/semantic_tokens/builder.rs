use anyhow::anyhow;
use fxhash::FxHashMap;
use lsp_types::{
    Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend,
};

pub(super) mod build_define;
pub(super) mod build_enum;
pub(super) mod build_enum_member;
pub(super) mod build_enum_struct;
pub(super) mod build_es_field;
pub(super) mod build_function;
pub(super) mod build_global_variable;
pub(super) mod build_local_variable;
pub(super) mod build_method;
pub(super) mod build_methodmap;
pub(super) mod build_property;

#[derive(Debug, Default)]
pub(super) struct SemanticTokensBuilder {
    prev_line: u32,
    prev_char: u32,
    data_is_sorted_and_delta_encoded: bool,
    data: Vec<SemanticToken>,
    data_len: usize,
    token_type_str_to_int: FxHashMap<SemanticTokenType, u32>,
    token_modifier_str_to_int: FxHashMap<SemanticTokenModifier, u32>,
    has_legend: bool,
}

impl SemanticTokensBuilder {
    pub(super) fn new(legend: Option<SemanticTokensLegend>) -> Self {
        let mut builder = Self {
            data_is_sorted_and_delta_encoded: true,
            ..Default::default()
        };
        if let Some(legend) = legend {
            builder.has_legend = true;
            for i in 0..legend.token_types.len() {
                builder
                    .token_type_str_to_int
                    .insert(legend.token_types[i].clone(), i as u32);
            }
            for i in 0..legend.token_modifiers.len() {
                builder
                    .token_modifier_str_to_int
                    .insert(legend.token_modifiers[i].clone(), i as u32);
            }
        }

        builder
    }

    pub(super) fn push(
        &mut self,
        range: Range,
        token_type: SemanticTokenType,
        token_modifiers: Option<Vec<SemanticTokenModifier>>,
    ) -> anyhow::Result<()> {
        if !self.has_legend {
            return Err(anyhow!("Legend must be provided in constructor"));
        }
        if range.start.line != range.end.line {
            return Err(anyhow!("{:?} cannot span multiple lines", range));
        }
        if !self.token_type_str_to_int.contains_key(&token_type) {
            return Err(anyhow!("{:?} is not in the provided legend", token_type));
        }

        let line = range.start.line;
        let char = range.start.character;
        let length = range.end.character - range.start.character;
        let n_token_type = self.token_type_str_to_int.get(&token_type).unwrap();
        let mut n_token_modifiers = 0;
        if let Some(token_modifiers) = token_modifiers {
            for token_modifier in token_modifiers {
                let n_token_modifier = self.token_modifier_str_to_int.get(&token_modifier);
                if let Some(n_token_modifier) = n_token_modifier {
                    let c_as_u32: u32 = {
                        let c: i32 = 1 << n_token_modifier;
                        let bytes = c.to_be_bytes();
                        u32::from_be_bytes(bytes)
                    };
                    n_token_modifiers |= (1 << n_token_modifier) >> c_as_u32;
                } else {
                    return Err(anyhow!(
                        "{:?} is not in the provided legend",
                        token_modifier
                    ));
                }
            }
        }
        self.push_encoded(line, char, length, *n_token_type, n_token_modifiers);
        Ok(())
    }

    fn push_encoded(
        &mut self,
        line: u32,
        char: u32,
        length: u32,
        token_type: u32,
        token_modifiers: u32,
    ) {
        if self.data_is_sorted_and_delta_encoded
            && (line < self.prev_line || (line == self.prev_line && char < self.prev_char))
        {
            // push calls were ordered and are no longer ordered
            self.data_is_sorted_and_delta_encoded = false;

            // Remove delta encoding from data
            let token_count = self.data.len();
            let mut prev_line = 0;
            let mut prev_char = 0;
            for i in 0..token_count {
                let mut line = self.data[i].delta_line;
                let mut char = self.data[i].delta_start;

                if line == 0 {
                    // on the same line as previous token
                    line = prev_line;
                    char += prev_char;
                } else {
                    // on a different line than previous token
                    line += prev_line;
                }

                self.data[i].delta_line = line;
                self.data[i].delta_start = char;

                prev_line = line;
                prev_char = char;
            }
        }

        let mut push_line = line;
        let mut push_char = char;
        if self.data_is_sorted_and_delta_encoded && self.data_len > 0 {
            push_line -= self.prev_line;
            if push_line == 0 {
                push_char -= self.prev_char;
            }
        }

        self.data_len += 1;
        self.data.push(SemanticToken {
            delta_line: push_line,
            delta_start: push_char,
            length,
            token_type,
            token_modifiers_bitset: token_modifiers,
        });

        self.prev_line = line;
        self.prev_char = char;
    }

    fn sort_and_delta_encode(data: &Vec<SemanticToken>) -> Vec<SemanticToken> {
        let mut pos = vec![];
        for i in 0..data.len() {
            pos.push(i);
        }
        pos.sort_by(|a, b| {
            let a_line = data[*a].delta_line;
            let b_line = data[*b].delta_line;
            if a_line == b_line {
                let a_char = data[*a].delta_start;
                let b_char = data[*b].delta_start;
                return a_char.cmp(&b_char);
            }

            a_line.cmp(&b_line)
        });

        let mut result = vec![
            SemanticToken {
                delta_line: 0,
                delta_start: 0,
                length: 0,
                token_type: 0,
                token_modifiers_bitset: 0
            };
            data.len()
        ];
        let mut prev_line = 0;
        let mut prev_char = 0;
        for (i, src_offset) in pos.iter().enumerate() {
            let line = data[*src_offset].delta_line;
            let char = data[*src_offset].delta_start;
            let length = data[*src_offset].length;
            let token_type = data[*src_offset].token_type;
            let token_modifiers = data[*src_offset].token_modifiers_bitset;

            let push_line = line - prev_line;
            let push_char = {
                if push_line == 0 {
                    char - prev_char
                } else {
                    char
                }
            };
            result[i] = SemanticToken {
                delta_line: push_line,
                delta_start: push_char,
                length,
                token_type,
                token_modifiers_bitset: token_modifiers,
            };

            prev_line = line;
            prev_char = char;
        }

        result
    }

    pub(super) fn build(&self, result_id: Option<String>) -> SemanticTokens {
        if !self.data_is_sorted_and_delta_encoded {
            return SemanticTokens {
                result_id,
                data: SemanticTokensBuilder::sort_and_delta_encode(&self.data),
            };
        }
        SemanticTokens {
            result_id,
            data: self.data.clone(),
        }
    }
}
