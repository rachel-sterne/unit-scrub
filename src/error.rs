use std::fmt;

/// Where and why parsing a size failed. Line and column are 1-based.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    InvalidNumber(String),
    MissingUnit,
    UnknownUnit(String),
    TrailingCharacters(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}: {}", self.line, self.column, self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::InvalidNumber(s) if s.is_empty() => write!(f, "expected a number"),
            ErrorKind::InvalidNumber(s) => write!(f, "'{}' is not a valid number", s),
            ErrorKind::MissingUnit => {
                write!(f, "expected a unit (e.g. KB, MiB, GB) after the number")
            }
            ErrorKind::UnknownUnit(s) => write!(f, "'{}' is not a recognized unit", s),
            ErrorKind::TrailingCharacters(s) => {
                write!(f, "unexpected trailing text '{}'", s.trim_end())
            }
        }
    }
}

impl std::error::Error for ParseError {}
