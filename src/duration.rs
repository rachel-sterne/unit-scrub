use std::fmt;

use crate::error::{ErrorKind, ParseError};
use crate::scanner::Scanner;

/// A recognized duration unit, expressed as a multiplier in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
}

impl Unit {
    fn from_str(text: &str) -> Option<Unit> {
        match text.to_ascii_lowercase().as_str() {
            "ms" | "msec" | "msecs" | "millisecond" | "milliseconds" => Some(Unit::Millisecond),
            "s" | "sec" | "secs" | "second" | "seconds" => Some(Unit::Second),
            "m" | "min" | "mins" | "minute" | "minutes" => Some(Unit::Minute),
            "h" | "hr" | "hrs" | "hour" | "hours" => Some(Unit::Hour),
            "d" | "day" | "days" => Some(Unit::Day),
            "w" | "wk" | "wks" | "week" | "weeks" => Some(Unit::Week),
            _ => None,
        }
    }

    fn seconds(self) -> f64 {
        match self {
            Unit::Millisecond => 0.001,
            Unit::Second => 1.0,
            Unit::Minute => 60.0,
            Unit::Hour => 3_600.0,
            Unit::Day => 86_400.0,
            Unit::Week => 604_800.0,
        }
    }
}

/// A parsed duration, normalized to a count of seconds. Formatting breaks
/// it back down into the largest units that divide it evenly, so a value
/// built from "1h30m" prints as "1h 30m" instead of a raw second count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Duration {
    seconds: f64,
}

impl Duration {
    pub fn seconds(&self) -> f64 {
        self.seconds
    }

    pub fn format(&self) -> String {
        if self.seconds == 0.0 {
            return "0s".to_string();
        }

        const UNITS: &[(f64, &str)] =
            &[(604_800.0, "w"), (86_400.0, "d"), (3_600.0, "h"), (60.0, "m")];

        let mut remaining = self.seconds;
        let mut parts = Vec::new();
        for &(size, label) in UNITS {
            if remaining >= size {
                let count = (remaining / size).floor();
                remaining -= count * size;
                parts.push(format!("{:.0}{}", count, label));
            }
        }

        if remaining > 0.0 || parts.is_empty() {
            parts.push(format_seconds(remaining));
        }

        parts.join(" ")
    }
}

fn format_seconds(secs: f64) -> String {
    if secs.fract() == 0.0 {
        format!("{:.0}s", secs)
    } else {
        let text = format!("{:.3}", secs);
        let trimmed = text.trim_end_matches('0').trim_end_matches('.');
        format!("{}s", trimmed)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl std::str::FromStr for Duration {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut scanner = Scanner::new(s);
        parse_duration(&mut scanner)
    }
}

/// Parses one duration expression starting at the scanner's current
/// position. A duration is one or more "<number><unit>" components run
/// together or separated by spaces ("90s", "1h30m", "2 days"), and stops
/// at the next newline or end of input.
pub fn parse_duration(scanner: &mut Scanner) -> Result<Duration, ParseError> {
    scanner.skip_inline_whitespace();

    let mut total_seconds = 0.0;
    let mut saw_component = false;

    loop {
        scanner.skip_inline_whitespace();
        match scanner.peek() {
            Some(c) if c.is_ascii_digit() || c == '.' => {}
            _ => break,
        }

        let number = scanner.parse_number()?;

        scanner.skip_inline_whitespace();
        let unit_start = scanner.position();
        let unit_text = scanner.parse_alpha_text();
        if unit_text.is_empty() {
            return Err(scanner.error_at(unit_start, ErrorKind::MissingUnit));
        }

        let unit = Unit::from_str(&unit_text).ok_or_else(|| {
            scanner.error_at(unit_start, ErrorKind::UnknownUnit(unit_text.clone()))
        })?;

        total_seconds += number * unit.seconds();
        saw_component = true;
    }

    if !saw_component {
        return Err(scanner.error(ErrorKind::InvalidNumber(String::new())));
    }

    scanner.skip_inline_whitespace();
    match scanner.peek() {
        None | Some('\n') => {}
        Some(_) => {
            let rest = scanner.rest_of_line();
            return Err(scanner.error(ErrorKind::TrailingCharacters(rest)));
        }
    }

    Ok(Duration { seconds: total_seconds })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<Duration, ParseError> {
        let mut scanner = Scanner::new(input);
        parse_duration(&mut scanner)
    }

    #[test]
    fn parses_via_from_str() {
        let d: Duration = "1h30m".parse().unwrap();
        assert_eq!(d.seconds(), 5_400.0);
    }

    #[test]
    fn parses_a_single_component() {
        let d = parse("90s").unwrap();
        assert_eq!(d.seconds(), 90.0);
        assert_eq!(d.format(), "1m 30s");
    }

    #[test]
    fn parses_components_run_together() {
        let d = parse("1h30m").unwrap();
        assert_eq!(d.seconds(), 5_400.0);
        assert_eq!(d.format(), "1h 30m");
    }

    #[test]
    fn parses_a_spaced_component_case_insensitively() {
        let d = parse("2 Days").unwrap();
        assert_eq!(d.seconds(), 172_800.0);
        assert_eq!(d.format(), "2d");
    }

    #[test]
    fn parses_fractional_components() {
        let d = parse("1.5h").unwrap();
        assert_eq!(d.seconds(), 5_400.0);
        assert_eq!(d.format(), "1h 30m");
    }

    #[test]
    fn formats_a_remainder_with_fractional_seconds() {
        let d = parse("1500ms").unwrap();
        assert_eq!(d.format(), "1.5s");
    }

    #[test]
    fn reports_missing_unit() {
        let err = parse("90").unwrap_err();
        assert_eq!(err.kind, ErrorKind::MissingUnit);
    }

    #[test]
    fn reports_unknown_unit() {
        let err = parse("3 fortnights").unwrap_err();
        assert_eq!(err.column, 3);
        assert!(matches!(err.kind, ErrorKind::UnknownUnit(ref s) if s == "fortnights"));
    }

    #[test]
    fn reports_trailing_characters() {
        let err = parse("5m stray").unwrap_err();
        assert!(matches!(err.kind, ErrorKind::TrailingCharacters(ref s) if s == "stray"));
    }

    #[test]
    fn reports_when_nothing_looks_like_a_number() {
        let err = parse("soon").unwrap_err();
        assert!(matches!(err.kind, ErrorKind::InvalidNumber(ref s) if s.is_empty()));
    }
}
