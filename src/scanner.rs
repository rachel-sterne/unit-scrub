use crate::error::{ErrorKind, ParseError};

/// A 1-based line/column position in the source text, the same convention
/// most editors and compilers use so the numbers are directly useful.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Walks a string one character at a time while keeping track of line and
/// column, so every error built from it can point at an exact spot.
pub struct Scanner<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Scanner { input, pos: 0, line: 1, column: 1 }
    }

    pub fn position(&self) -> Position {
        Position { line: self.line, column: self.column }
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    pub fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    /// Skips spaces and tabs, but not newlines: a newline ends an entry.
    pub fn skip_inline_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skips any whitespace, including newlines, used between entries so
    /// blank lines don't need special-casing.
    pub fn skip_whitespace_and_newlines(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Used for error recovery: discards the rest of a broken line so the
    /// next entry can still be parsed.
    pub fn skip_to_line_end(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    pub fn rest_of_line(&self) -> String {
        match self.input[self.pos..].find('\n') {
            Some(idx) => self.input[self.pos..self.pos + idx].to_string(),
            None => self.input[self.pos..].to_string(),
        }
    }

    pub fn error(&self, kind: ErrorKind) -> ParseError {
        self.error_at(self.position(), kind)
    }

    pub fn error_at(&self, pos: Position, kind: ErrorKind) -> ParseError {
        ParseError { line: pos.line, column: pos.column, kind }
    }

    /// Reads a plain decimal number (digits, optionally a '.' and more
    /// digits). Shared by every kind of entry that starts with a magnitude.
    pub fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.position();
        let mut text = String::new();
        let mut saw_digit = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                saw_digit = true;
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') {
            text.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    saw_digit = true;
                    text.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if !saw_digit {
            let found = self.peek().map(|c| c.to_string()).unwrap_or_default();
            return Err(self.error_at(start, ErrorKind::InvalidNumber(found)));
        }

        text.parse::<f64>()
            .map_err(|_| self.error_at(start, ErrorKind::InvalidNumber(text.clone())))
    }

    /// Reads a run of ASCII letters, used for the unit that follows a
    /// number. Returns an empty string if there's nothing to read, leaving
    /// the caller to decide whether that's an error.
    pub fn parse_alpha_text(&mut self) -> String {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        text
    }
}
