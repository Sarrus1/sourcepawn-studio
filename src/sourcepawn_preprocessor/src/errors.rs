use lsp_types::Range;
use std::{error, fmt};

#[derive(Debug)]
pub(super) enum ExpansionError {
    MacroNotFound(MacroNotFoundError),
    Parse(ParseIntError),
}

impl fmt::Display for ExpansionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExpansionError::MacroNotFound(err) => err.fmt(f),
            ExpansionError::Parse(err) => err.fmt(f),
        }
    }
}

impl From<MacroNotFoundError> for ExpansionError {
    fn from(err: MacroNotFoundError) -> ExpansionError {
        ExpansionError::MacroNotFound(err)
    }
}

impl From<ParseIntError> for ExpansionError {
    fn from(err: ParseIntError) -> ExpansionError {
        ExpansionError::Parse(err)
    }
}

impl error::Error for ExpansionError {}

#[derive(Debug, Clone)]
pub(super) struct MacroNotFoundError {
    pub(super) macro_name: String,
    pub(super) range: Range,
}

impl MacroNotFoundError {
    pub(super) fn new(macro_name: String, range: Range) -> MacroNotFoundError {
        MacroNotFoundError { macro_name, range }
    }
}

impl fmt::Display for MacroNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Macro {} not found at {:?}", self.macro_name, self.range)
    }
}

impl error::Error for MacroNotFoundError {}

#[derive(Debug, Clone)]
pub(super) struct ParseIntError {
    pub(super) text: String,
    pub(super) range: Range,
}

impl ParseIntError {
    pub(super) fn new(text: String, range: Range) -> ParseIntError {
        ParseIntError { text, range }
    }
}

impl fmt::Display for ParseIntError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} could not be parsed as an integer at {:?}",
            self.text, self.range
        )
    }
}

impl error::Error for ParseIntError {}

#[derive(Debug, Clone)]
pub(super) struct EvaluationError {
    pub(super) text: String,
    pub(super) range: Range,
}

impl EvaluationError {
    pub(super) fn new(text: String, range: Range) -> EvaluationError {
        EvaluationError { text, range }
    }
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid condition, assumed false ({}) {:?}",
            self.text, self.range
        )
    }
}

impl error::Error for EvaluationError {}
