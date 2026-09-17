use std::fmt;

use crate::error::{ErrorKind, ParseError};
use crate::scanner::Scanner;

/// A recognized byte-size unit. Decimal units count in powers of 1000,
/// binary units in powers of 1024; these are genuinely different
/// quantities, so a normalizer must not collapse the distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Byte,
    Kilobyte,
    Megabyte,
    Gigabyte,
    Terabyte,
    Petabyte,
    Kibibyte,
    Mebibyte,
    Gibibyte,
    Tebibyte,
    Pebibyte,
}

impl Unit {
    fn from_str(text: &str) -> Option<Unit> {
        match text.to_ascii_lowercase().as_str() {
            "b" | "byte" | "bytes" => Some(Unit::Byte),
            "k" | "kb" | "kilobyte" | "kilobytes" => Some(Unit::Kilobyte),
            "m" | "mb" | "megabyte" | "megabytes" => Some(Unit::Megabyte),
            "g" | "gb" | "gigabyte" | "gigabytes" => Some(Unit::Gigabyte),
            "t" | "tb" | "terabyte" | "terabytes" => Some(Unit::Terabyte),
            "p" | "pb" | "petabyte" | "petabytes" => Some(Unit::Petabyte),
            "ki" | "kib" | "kibibyte" | "kibibytes" => Some(Unit::Kibibyte),
            "mi" | "mib" | "mebibyte" | "mebibytes" => Some(Unit::Mebibyte),
            "gi" | "gib" | "gibibyte" | "gibibytes" => Some(Unit::Gibibyte),
            "ti" | "tib" | "tebibyte" | "tebibytes" => Some(Unit::Tebibyte),
            "pi" | "pib" | "pebibyte" | "pebibytes" => Some(Unit::Pebibyte),
            _ => None,
        }
    }

    fn multiplier(self) -> f64 {
        match self {
            Unit::Byte => 1.0,
            Unit::Kilobyte => 1_000.0,
            Unit::Megabyte => 1_000_000.0,
            Unit::Gigabyte => 1_000_000_000.0,
            Unit::Terabyte => 1_000_000_000_000.0,
            Unit::Petabyte => 1_000_000_000_000_000.0,
            Unit::Kibibyte => 1024.0,
            Unit::Mebibyte => 1024.0_f64.powi(2),
            Unit::Gibibyte => 1024.0_f64.powi(3),
            Unit::Tebibyte => 1024.0_f64.powi(4),
            Unit::Pebibyte => 1024.0_f64.powi(5),
        }
    }

    fn is_binary(self) -> bool {
        matches!(
            self,
            Unit::Kibibyte | Unit::Mebibyte | Unit::Gibibyte | Unit::Tebibyte | Unit::Pebibyte
        )
    }
}

/// A parsed byte size, normalized to a byte count. Formatting picks back
/// the largest unit that reads naturally, preserving whether the original
/// text used decimal or binary units since those aren't interchangeable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ByteSize {
    bytes: f64,
    binary: bool,
}

impl ByteSize {
    fn from_value_and_unit(value: f64, unit: Unit) -> Self {
        ByteSize { bytes: value * unit.multiplier(), binary: unit.is_binary() }
    }

    pub fn bytes(&self) -> f64 {
        self.bytes
    }

    pub fn format(&self) -> String {
        const DECIMAL: &[(f64, &str)] = &[
            (1e15, "PB"),
            (1e12, "TB"),
            (1e9, "GB"),
            (1e6, "MB"),
            (1e3, "KB"),
        ];
        const BINARY: &[(f64, &str)] = &[
            (1024.0_f64.powi(5), "PiB"),
            (1024.0_f64.powi(4), "TiB"),
            (1024.0_f64.powi(3), "GiB"),
            (1024.0_f64.powi(2), "MiB"),
            (1024.0, "KiB"),
        ];

        let table = if self.binary { BINARY } else { DECIMAL };
        for &(threshold, label) in table {
            if self.bytes >= threshold {
                return format!("{:.2} {}", self.bytes / threshold, label);
            }
        }
        format!("{:.0} B", self.bytes)
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Parses one "<number> <unit>" entry starting at the scanner's current
/// position and stopping at the next newline or end of input.
pub fn parse_size(scanner: &mut Scanner) -> Result<ByteSize, ParseError> {
    scanner.skip_inline_whitespace();

    let number = scanner.parse_number()?;

    scanner.skip_inline_whitespace();
    let unit_start = scanner.position();
    let unit_text = scanner.parse_alpha_text();

    if unit_text.is_empty() {
        return Err(scanner.error_at(unit_start, ErrorKind::MissingUnit));
    }

    let unit = Unit::from_str(&unit_text)
        .ok_or_else(|| scanner.error_at(unit_start, ErrorKind::UnknownUnit(unit_text.clone())))?;

    scanner.skip_inline_whitespace();
    match scanner.peek() {
        None | Some('\n') => {}
        Some(_) => {
            let rest = scanner.rest_of_line();
            return Err(scanner.error(ErrorKind::TrailingCharacters(rest)));
        }
    }

    Ok(ByteSize::from_value_and_unit(number, unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<ByteSize, ParseError> {
        let mut scanner = Scanner::new(input);
        parse_size(&mut scanner)
    }

    #[test]
    fn parses_decimal_units_case_insensitively() {
        let size = parse("3.5gb").unwrap();
        assert_eq!(size.bytes(), 3_500_000_000.0);
        assert_eq!(size.format(), "3.50 GB");
    }

    #[test]
    fn parses_binary_units_with_spacing() {
        let size = parse("1024 KiB").unwrap();
        assert_eq!(size.bytes(), 1024.0 * 1024.0);
        assert_eq!(size.format(), "1.00 MiB");
    }

    #[test]
    fn reports_column_of_unknown_unit() {
        let err = parse("12 gigs").unwrap_err();
        assert_eq!(err.column, 4);
        assert!(matches!(err.kind, ErrorKind::UnknownUnit(ref s) if s == "gigs"));
    }

    #[test]
    fn reports_missing_unit() {
        let err = parse("42").unwrap_err();
        assert_eq!(err.kind, ErrorKind::MissingUnit);
    }

    #[test]
    fn reports_trailing_characters() {
        let err = parse("5 GB extra").unwrap_err();
        assert!(matches!(err.kind, ErrorKind::TrailingCharacters(ref s) if s == "extra"));
    }
}
