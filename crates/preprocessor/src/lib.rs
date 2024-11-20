use std::{cmp::max, hash::Hash};

use anyhow::{bail, Context};
use base_db::{RE_CHEVRON, RE_QUOTE};
use conditions::{ConditionOffsetStack, ConditionStack, ConditionState};
use lsp_types::Diagnostic;
use smol_str::SmolStr;
use sourcepawn_lexer::{
    Comment, Literal, Operator, PreprocDir, SourcepawnLexer, Symbol, TextRange, TextSize, TokenKind,
};
use vfs::FileId;

use errors::{ExpansionError, PreprocessorErrors, UnresolvedIncludeError};
use evaluator::IfCondition;
use macros::{expand_identifier, Macro};

mod buffer;
mod conditions;
pub mod db;
mod errors;
pub(crate) mod evaluator;
mod macros;
mod offset;
mod preprocessor_operator;
mod result;
mod symbol;

use buffer::PreprocessorBuffer;
pub use errors::{EvaluationError, PreprocessorError};
pub(crate) use macros::MacroStore;
pub use macros::{HMacrosMap, MacrosMap};
pub use offset::SourceMap;
pub use result::PreprocessingResult;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct SourcepawnPreprocessor<'a, F>
where
    F: FnMut(&mut MacrosMap, String, FileId, bool) -> anyhow::Result<()>,
{
    lexer: SourcepawnLexer<'a>,
    input: &'a str,
    macro_store: MacroStore,
    expansion_stack: Vec<Symbol>,
    errors: PreprocessorErrors,
    file_id: FileId,
    conditions_stack: ConditionStack,
    condition_offsets_stack: ConditionOffsetStack,
    buffer: PreprocessorBuffer,
    include_file: &'a mut F,
}

/// Parse status of `using __intrinsics__.Handle;`.
/// This is used to handle the `using __intrinsics__.Handle;` in handles.inc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IntrinsicsParseStatus {
    Using,
    Dot,
    Intrinsics,
    Handle,
}

impl<'a, F> SourcepawnPreprocessor<'a, F>
where
    F: FnMut(&mut MacrosMap, String, FileId, bool) -> anyhow::Result<()>,
{
    pub fn new(file_id: FileId, input: &'a str, include_file: &'a mut F) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            input,
            file_id,
            include_file,
            errors: Default::default(),
            conditions_stack: Default::default(),
            condition_offsets_stack: Default::default(),
            buffer: Default::default(),
            macro_store: Default::default(),
            expansion_stack: Default::default(),
        }
    }

    pub fn set_macros(&mut self, map: MacrosMap) {
        self.macro_store.extend(map);
    }

    pub fn result(mut self) -> PreprocessingResult {
        let inactive_ranges = self.get_inactive_ranges();
        let mut res = PreprocessingResult::new(
            self.buffer.contents().into(),
            self.macro_store.to_macros_map(),
            self.buffer.to_source_map(),
            self.errors,
            inactive_ranges,
        );
        res.shrink_to_fit();
        res
    }

    pub fn error_result(mut self) -> PreprocessingResult {
        let inactive_ranges = self.get_inactive_ranges();
        let mut res = PreprocessingResult::new(
            self.input.to_owned().into(),
            self.macro_store.to_macros_map(),
            self.buffer.to_source_map(),
            self.errors,
            inactive_ranges,
        );
        res.shrink_to_fit();
        res
    }

    pub fn add_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        self.get_macro_not_found_diagnostics(diagnostics);
        self.get_evaluation_error_diagnostics(diagnostics);
        self.get_include_not_found_diagnostics(diagnostics);
    }

    fn get_inactive_ranges(&mut self) -> Vec<TextRange> {
        if self.condition_offsets_stack.skipped_ranges().is_empty() {
            return Vec::new();
        }
        self.condition_offsets_stack.sort_skipped_ranges();
        let mut ranges = vec![*self
            .condition_offsets_stack
            .skipped_ranges()
            .first()
            .unwrap()];
        for range in self.condition_offsets_stack.skipped_ranges() {
            let last_range = ranges.pop().unwrap();
            if last_range.end() >= range.start() {
                ranges.push(TextRange::new(
                    last_range.start(),
                    max(last_range.end(), range.end()),
                ));
            } else {
                ranges.push(last_range);
                ranges.push(*range);
            }
        }

        ranges
    }

    fn get_macro_not_found_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(
            self.errors
                .macro_not_found_errors
                .iter()
                .map(|err| Diagnostic {
                    range: Default::default(), // FIXME: Default range
                    message: format!("Macro {} not found.", err.macro_name),
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    ..Default::default()
                }),
        );
    }

    fn get_include_not_found_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(
            self.errors
                .unresolved_include_errors
                .iter()
                .map(|err| Diagnostic {
                    range: Default::default(), // FIXME: Default range
                    message: format!("Include \"{}\" not found.", err.include_text),
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    ..Default::default()
                }),
        );
    }

    fn get_evaluation_error_diagnostics(&self, diagnostics: &mut Vec<Diagnostic>) {
        diagnostics.extend(self.errors.evaluation_errors.iter().map(|err| Diagnostic {
            range: Default::default(), // FIXME: Default range
            message: format!("Preprocessor condition is invalid: {}", err.text),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            ..Default::default()
        }));
    }

    fn include_sourcemod(&mut self) {
        let _ = (self.include_file)(
            self.macro_store.map_mut(),
            "sourcemod".to_string(),
            self.file_id,
            false,
        );
    }

    pub fn preprocess_input(mut self) -> PreprocessingResult {
        self.include_sourcemod();
        let mut intrinsics_parse_status = None;
        // let mut col_offset: Option<i32> = None;
        // let mut expanded_symbol: Option<(Symbol, u32, FileId)> = None;
        // let mut args_diff = 0u32;
        while let Some(symbol) = if !self.expansion_stack.is_empty() {
            self.expansion_stack.pop()
        } else {
            // if let Some((expanded_symbol, idx, file_id)) = expanded_symbol.take() {
            //     if let Some(symbol) = symbol.clone() {
            //         self.offsets
            //             .entry(symbol.range.start.line)
            //             .or_default()
            //             .push(Offset {
            //                 idx,
            //                 file_id,
            //                 range: expanded_symbol.range,
            //                 diff: (col_offset.take().unwrap_or(0)
            //                     - (expanded_symbol.range.end.character
            //                         - expanded_symbol.range.start.character)
            //                         as i32),
            //                 args_diff,
            //             });
            //         args_diff = 0;
            //     }
            // }

            self.lexer.next()
        } {
            // if let Some(original_range) = token.original_range() {
            //     let new_range = self.current_range(&token);
            //     self.args_maps
            //         .entry(original_range.start.line)
            //         .or_default()
            //         .push((original_range, new_range));
            // }
            if self.conditions_stack.top_is_activated_or_not_activated() {
                if self.process_negative_condition(&symbol).is_err() {
                    return self.error_result();
                }
                continue;
            }
            match &symbol.token_kind {
                TokenKind::Unknown => return self.error_result(),
                TokenKind::PreprocDir(dir) => {
                    if self.process_directive(dir, &symbol).is_err() {
                        return self.error_result();
                    }
                }
                TokenKind::Identifier => {
                    // This is a hack to handle `using __intrinsics__.Handle;` in handles.inc, which
                    // is not a part of sourcemod as of 060c832f89709e6a6222cf039071061dcc0a36da.
                    // see: https://github.com/alliedmodders/sourcemod/commit/060c832f89709e6a6222cf039071061dcc0a36da
                    if intrinsics_parse_status == Some(IntrinsicsParseStatus::Dot) {
                        if symbol.text() == "Handle" {
                            self.buffer.push_str("methodmap Handle __nullable__ {public native ~Handle();public native void Close();};")
                        }
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Handle);
                        continue;
                    }
                    match self.macro_store.get(&symbol.text()).cloned() {
                        // TODO: Evaluate the performance dropoff of supporting macro expansion when overriding reserved keywords.
                        // This might only be a problem for a very small subset of users.
                        Some(macro_) => {
                            // Skip the macro if it is disabled and reenable it.
                            if self.macro_store.is_macro_disabled(&macro_) {
                                self.macro_store.enable_macro(&macro_);
                                self.buffer.push_symbol(&symbol);
                                continue;
                            }
                            let idx = macro_.idx;
                            let file_id: FileId = macro_.file_id;
                            match expand_identifier(
                                &mut self.lexer,
                                &mut self.macro_store,
                                &symbol,
                                &mut self.expansion_stack,
                                true,
                            ) {
                                Ok(()) => {
                                    // expanded_symbol =
                                    //     Some((token.symbol().to_owned(), idx, file_id));
                                    // args_diff = args_diff_;
                                    continue;
                                }
                                Err(ExpansionError::MacroNotFound(err)) => {
                                    self.errors.macro_not_found_errors.push(err.clone());
                                    return self.error_result();
                                }
                                Err(ExpansionError::Parse(_)) => {
                                    return self.error_result();
                                }
                            }
                        }
                        None => {
                            self.buffer.push_symbol(&symbol);
                        }
                    }
                }
                TokenKind::Using => {
                    if intrinsics_parse_status.take().is_none() {
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Using);
                    }
                }
                TokenKind::Intrinsics => {
                    if intrinsics_parse_status.take() == Some(IntrinsicsParseStatus::Using) {
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Intrinsics);
                    }
                }
                TokenKind::Semicolon => {
                    if intrinsics_parse_status.take() != Some(IntrinsicsParseStatus::Handle) {
                        self.buffer.push_symbol(&symbol);
                    }
                }
                TokenKind::Dot => match intrinsics_parse_status {
                    Some(IntrinsicsParseStatus::Intrinsics) => {
                        intrinsics_parse_status = Some(IntrinsicsParseStatus::Dot);
                    }
                    _ => self.buffer.push_symbol(&symbol),
                },
                TokenKind::Eof => {
                    self.buffer.push_ws(&symbol);
                    break;
                }
                TokenKind::Newline => {
                    self.buffer.push_new_line();
                }
                _ => self.buffer.push_symbol(&symbol),
            }
        }

        self.result()
    }

    fn process_if_directive(&mut self, symbol: &Symbol) {
        self.condition_offsets_stack.push(symbol.range.start());
        let mut if_condition =
            IfCondition::new(&mut self.macro_store, self.buffer.source_map_mut());
        while self.lexer.in_preprocessor() {
            if let Some(symbol) = self.lexer.next() {
                if_condition.symbols.push(symbol);
            } else {
                break;
            }
        }
        let if_condition_eval = match if_condition.evaluate() {
            Ok(res) => res,
            Err(err) => {
                self.errors.evaluation_errors.push(err);
                // Default to false when we fail to evaluate a condition.
                false
            }
        };

        if if_condition_eval {
            self.conditions_stack.push(ConditionState::Active);
        } else {
            self.conditions_stack.push(ConditionState::NotActivated);
        }
        let line_continuation_count = if_condition.line_continuation_count();
        self.errors
            .macro_not_found_errors
            .extend(if_condition.macro_not_found_errors.clone());
        drop(if_condition);
        self.buffer.push_new_lines(line_continuation_count);
    }

    fn process_elseif_directive(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        let top = self
            .conditions_stack
            .pop()
            .context("Expect if before elseif clause.")?;
        match top {
            ConditionState::NotActivated => {
                self.condition_offsets_stack
                    .pop_and_push_skipped_range(symbol.range.end());
                self.process_if_directive(symbol);
            }
            ConditionState::Active => {
                let _ = self.condition_offsets_stack.pop();
                self.condition_offsets_stack.push(symbol.range.start());
                self.conditions_stack.push(ConditionState::Activated);
            }
            ConditionState::Activated => {
                self.condition_offsets_stack
                    .pop_and_push_skipped_range(symbol.range.end());
                self.condition_offsets_stack.push(symbol.range.start());
                self.conditions_stack.push(ConditionState::Activated);
            }
        }

        Ok(())
    }

    fn process_else_directive(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        let top = self
            .conditions_stack
            .pop()
            .context("Expect if before else clause.")?;
        match top {
            ConditionState::NotActivated => {
                self.condition_offsets_stack
                    .pop_and_push_skipped_range(symbol.range.end());
                self.conditions_stack.push(ConditionState::Active);
            }
            ConditionState::Active => {
                let _ = self.condition_offsets_stack.pop();
                self.condition_offsets_stack.push(symbol.range.start());
                self.conditions_stack.push(ConditionState::Activated);
            }
            ConditionState::Activated => {
                self.condition_offsets_stack
                    .pop_and_push_skipped_range(symbol.range.end());
                self.condition_offsets_stack.push(symbol.range.start());
                self.conditions_stack.push(ConditionState::Activated);
            }
        }

        Ok(())
    }

    fn process_endif_directive(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        // self.conditions_stack
        //     .pop()
        //     .context("Expect if before endif clause")?;
        // // Skip the endif if it is in a nested condition.
        if let Some(top) = self.conditions_stack.pop() {
            if top != ConditionState::Active {
                self.condition_offsets_stack
                    .pop_and_push_skipped_range(symbol.range.end());
            }
        }

        Ok(())
    }

    fn process_directive(&mut self, dir: &PreprocDir, symbol: &Symbol) -> anyhow::Result<()> {
        match dir {
            PreprocDir::MDefine => {
                self.buffer.push_symbol(symbol);
                let mut macro_name = SmolStr::default();
                let mut macro_ = Macro::default(self.file_id);
                enum State {
                    Start,
                    Params,
                    Body,
                }
                let mut args = vec![-1; 10];
                let mut found_params = false;
                let mut state = State::Start;
                let mut args_idx = 0;
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        if symbol.token_kind == TokenKind::Eof {
                            break;
                        }
                        // self.buffer.push_ws(&symbol);
                        if !matches!(symbol.token_kind, TokenKind::Newline | TokenKind::Eof) {
                            self.buffer.push_symbol(&symbol);
                        }
                        match state {
                            State::Start => {
                                if macro_name.is_empty()
                                    && TokenKind::Identifier == symbol.token_kind
                                {
                                    macro_name = symbol.text();
                                } else if symbol.delta == 0
                                    && symbol.token_kind == TokenKind::LParen
                                {
                                    state = State::Params;
                                } else {
                                    macro_.body.push(symbol.into());
                                    state = State::Body;
                                }
                            }
                            State::Params => {
                                if symbol.delta > 0 {
                                    macro_.body.push(symbol.into());
                                    state = State::Body;
                                    continue;
                                }
                                match &symbol.token_kind {
                                    TokenKind::RParen => {
                                        state = State::Body;
                                    }
                                    TokenKind::Literal(Literal::IntegerLiteral) => {
                                        found_params = true;
                                        let idx = symbol.to_int().context(format!(
                                            "Could not convert {:?} to an int value.",
                                            symbol.text()
                                        ))?
                                            as usize;
                                        if idx >= args.len() {
                                            bail!(
                                                "Argument index out of bounds for macro {}",
                                                symbol.text()
                                            );
                                        }
                                        args[idx] = args_idx;
                                    }
                                    TokenKind::Comma => {
                                        args_idx += 1;
                                    }
                                    TokenKind::Operator(Operator::Percent) => (),
                                    _ => {
                                        bail!("Unexpected symbol {} in macro args", symbol.text())
                                    }
                                }
                            }
                            State::Body => {
                                macro_.body.push(symbol.into());
                            }
                        }
                    }
                }
                if found_params {
                    macro_.nb_params = args.iter().filter(|&n| *n != -1).count() as i8;
                    macro_.params = Some(args);
                }
                self.buffer.push_new_line();
                self.macro_store.insert_macro(macro_name, macro_);
            }
            PreprocDir::MUndef => {
                self.buffer.push_symbol(symbol);
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        self.buffer.push_ws(&symbol);
                        if !matches!(symbol.token_kind, TokenKind::Newline | TokenKind::Eof) {
                            self.buffer.push_symbol_no_delta(&symbol);
                        }
                        if symbol.token_kind == TokenKind::Identifier {
                            self.macro_store.remove_macro(&symbol.text());
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            PreprocDir::MIf => self.process_if_directive(symbol),
            PreprocDir::MElseif => self.process_elseif_directive(symbol)?,
            PreprocDir::MElse => self.process_else_directive(symbol)?,
            PreprocDir::MEndif => self.process_endif_directive(symbol)?,
            PreprocDir::MInclude => self.process_include_directive(symbol, false),
            PreprocDir::MTryinclude => self.process_include_directive(symbol, true),
            _ => self.buffer.push_symbol(symbol),
        }

        Ok(())
    }

    fn process_include_directive(&mut self, symbol: &Symbol, is_try: bool) {
        let text = symbol.inline_text().trim().to_string();
        let delta: u32 = symbol.range.len().into();
        let symbol = Symbol::new(
            symbol.token_kind,
            Some(&text),
            TextRange::at(symbol.range.start(), TextSize::new(text.len() as u32)),
            symbol.delta,
        );

        if let Some(path) = RE_CHEVRON.captures(&text).and_then(|c| c.get(1)) {
            match (self.include_file)(
                self.macro_store.map_mut(),
                path.as_str().to_string(),
                self.file_id,
                false,
            ) {
                Ok(_) => (),
                Err(_) => {
                    if !is_try {
                        // TODO: Emit a warning here for #tryinclude?
                        let start: usize = symbol.range.start().into();
                        let range = TextRange::new(
                            TextSize::new((start + path.start()) as u32),
                            TextSize::new((start + path.end()) as u32),
                        );
                        self.errors
                            .unresolved_include_errors
                            .push(UnresolvedIncludeError::new(
                                path.as_str().to_string(),
                                range,
                            ))
                    }
                }
            }
        };
        if let Some(path) = RE_QUOTE.captures(&text).and_then(|c| c.get(1)) {
            match (self.include_file)(
                self.macro_store.map_mut(),
                path.as_str().to_string(),
                self.file_id,
                true,
            ) {
                Ok(_) => (),
                Err(_) => {
                    if !is_try {
                        // TODO: Emit a warning here for #tryinclude?
                        let start: usize = symbol.range.start().into();
                        let range = TextRange::new(
                            TextSize::new((start + path.start()) as u32),
                            TextSize::new((start + path.end()) as u32),
                        );
                        self.errors
                            .unresolved_include_errors
                            .push(UnresolvedIncludeError::new(
                                path.as_str().to_string(),
                                range,
                            ))
                    }
                }
            }
        };

        self.buffer.push_symbol(&symbol);
        self.buffer.push_new_lines(delta);
    }

    fn process_negative_condition(&mut self, symbol: &Symbol) -> anyhow::Result<()> {
        match &symbol.token_kind {
            TokenKind::PreprocDir(dir) => match dir {
                PreprocDir::MIf => {
                    // Keep track of any nested if statements to ensure we properly pop when reaching an endif.
                    self.conditions_stack.push(ConditionState::Activated);
                }
                PreprocDir::MEndif => self.process_endif_directive(symbol)?,
                PreprocDir::MElse => self.process_else_directive(symbol)?,
                PreprocDir::MElseif => self.process_elseif_directive(symbol)?,
                _ => (),
            },
            TokenKind::Newline => {
                // Keep the newline to keep the line numbers in sync.
                let start_offset = self.buffer.offset();
                self.buffer.push_new_line();
                self.condition_offsets_stack
                    .push_skipped_range(TextRange::new(
                        start_offset.into(),
                        self.buffer.offset().into(),
                    ));
            }
            TokenKind::Comment(Comment::BlockComment) => {
                let start_offset = self.buffer.offset();
                let line_delta = linebreak_count(symbol.text().as_str());
                self.buffer.push_new_lines(line_delta as u32);
                self.condition_offsets_stack
                    .push_skipped_range(TextRange::new(
                        start_offset.into(),
                        self.buffer.offset().into(),
                    ));
            }
            // Skip any token that is not a directive or a newline.
            _ => (),
        }

        Ok(())
    }
}

pub fn linebreak_count(text: &str) -> usize {
    let substring = "\n".as_bytes();
    text.as_bytes()
        .windows(substring.len())
        .filter(|&w| w == substring)
        .count()
}
