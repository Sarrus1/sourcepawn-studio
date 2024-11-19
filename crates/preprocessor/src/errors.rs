use sourcepawn_lexer::TextRange;
use std::{error, fmt};

pub trait PreprocessorError {
    fn text(&self) -> &str;

    fn range(&self) -> &TextRange;
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroNotFoundError {
    pub(super) macro_name: String,
    pub(super) range: TextRange,
}

impl MacroNotFoundError {
    pub(super) fn new(macro_name: String, range: TextRange) -> MacroNotFoundError {
        MacroNotFoundError { macro_name, range }
    }
}

impl fmt::Display for MacroNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Macro {} not found at {:?}", self.macro_name, self.range)
    }
}

impl error::Error for MacroNotFoundError {}

impl PreprocessorError for MacroNotFoundError {
    fn text(&self) -> &str {
        &self.macro_name
    }

    fn range(&self) -> &TextRange {
        &self.range
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedIncludeError {
    pub(super) include_text: String,
    pub(super) range: TextRange,
}

impl PreprocessorError for UnresolvedIncludeError {
    fn text(&self) -> &str {
        &self.include_text
    }

    fn range(&self) -> &TextRange {
        &self.range
    }
}

impl UnresolvedIncludeError {
    pub(super) fn new(include_text: String, range: TextRange) -> UnresolvedIncludeError {
        UnresolvedIncludeError {
            include_text,
            range,
        }
    }
}

impl fmt::Display for UnresolvedIncludeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Unresolved include {} at {:?}",
            self.include_text, self.range
        )
    }
}

impl error::Error for UnresolvedIncludeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParseIntError {
    pub(super) text: String,
    pub(super) range: TextRange,
}

impl PreprocessorError for ParseIntError {
    fn text(&self) -> &str {
        &self.text
    }

    fn range(&self) -> &TextRange {
        &self.range
    }
}

impl ParseIntError {
    pub(super) fn new(text: String, range: TextRange) -> ParseIntError {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationError {
    pub(super) text: String,
    pub(super) range: TextRange,
}

impl PreprocessorError for EvaluationError {
    fn text(&self) -> &str {
        &self.text
    }

    fn range(&self) -> &TextRange {
        &self.range
    }
}

impl EvaluationError {
    pub(super) fn new(text: String, range: TextRange) -> EvaluationError {
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreprocessorErrors {
    pub macro_not_found_errors: Vec<MacroNotFoundError>,
    pub evaluation_errors: Vec<EvaluationError>,
    pub unresolved_include_errors: Vec<UnresolvedIncludeError>,
}

impl PreprocessorErrors {
    pub fn shrink_to_fit(&mut self) {
        self.macro_not_found_errors.shrink_to_fit();
        self.evaluation_errors.shrink_to_fit();
        self.unresolved_include_errors.shrink_to_fit();
    }
}
