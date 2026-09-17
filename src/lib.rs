//! Parses byte-size text that humans actually write ("3.5gb", "1024 KiB",
//! "200bytes") into a normalized form, reporting the exact line and column
//! of anything it can't make sense of.

mod duration;
mod error;
mod scanner;
mod size;

pub use duration::Duration;
pub use error::{ErrorKind, ParseError};
pub use size::ByteSize;

use scanner::Scanner;

/// Parses a document where each line holds one size expression. Blank
/// lines are ignored. A broken line doesn't stop the rest of the document
/// from being parsed; its slot in the result simply holds the error.
pub fn parse_document(input: &str) -> Vec<Result<ByteSize, ParseError>> {
    let mut scanner = Scanner::new(input);
    let mut results = Vec::new();

    loop {
        scanner.skip_whitespace_and_newlines();
        if scanner.at_end() {
            break;
        }

        let result = size::parse_size(&mut scanner);
        if result.is_err() {
            scanner.skip_to_line_end();
        }
        results.push(result);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_line_numbers_across_a_document() {
        let input = "128 MB\n\n12 gigs\n4kib\n";
        let results = parse_document(input);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());

        let err = results[1].as_ref().unwrap_err();
        assert_eq!((err.line, err.column), (3, 4));

        assert!(results[2].is_ok());
    }
}
