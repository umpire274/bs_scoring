//! Grammar layer for the command line.
//!
//! This layer is **purely syntactic**: it turns a raw input string into a
//! list of strongly-typed [`Segment`] values, with no knowledge of the game
//! state. All state-dependent checks (batter-slot coherence, runner presence,
//! infield-fly preconditions, advance-without-trigger, etc.) live in
//! [`crate::engine::commands::validator`].
//!
//! # Entry points
//!
//! - [`parse_segment`] — parse one comma-separated chunk.
//! - [`parse_line`] — split the line on commas and parse every chunk,
//!   returning either the full list of segments or **all** the parse errors
//!   collected in a single pass.

mod tokens;

pub mod segment;

pub use segment::{
    BatterOutKind, ControlKind, HitKind, PitchKind, Segment, StatusKind, parse_segment,
};

use crate::engine::commands::errors::{CommandError, CommandErrorKind};

/// Parse an entire input line (`,`-separated segments) into a list of
/// segments. Collects every syntactic error instead of stopping at the first.
///
/// A completely empty/blank line returns `Ok(vec![])` — higher layers decide
/// how to react (typically: ignore and re-prompt).
pub fn parse_line(line: &str) -> Result<Vec<Segment>, Vec<CommandError>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut segments = Vec::new();
    let mut errors = Vec::new();

    for (idx, raw) in trimmed.split(',').enumerate() {
        let segment_index = idx + 1; // 1-based for humans
        let segment_text = raw.trim().to_string();

        match parse_segment(raw) {
            Ok(seg) => segments.push(seg),
            Err(e) => errors.push(CommandError {
                segment_index,
                segment_text,
                kind: CommandErrorKind::Parse(e),
            }),
        }
    }

    if errors.is_empty() {
        Ok(segments)
    } else {
        Err(errors)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::commands::errors::CommandErrorKind;

    #[test]
    fn empty_line_is_empty_vec() {
        assert_eq!(parse_line(""), Ok(vec![]));
        assert_eq!(parse_line("   "), Ok(vec![]));
    }

    #[test]
    fn single_segment_line() {
        let segs = parse_line("5 h lf").unwrap();
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn multi_segment_line() {
        let segs = parse_line("5 h, 3 sc").unwrap();
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn triple_play_any_order() {
        // These three lines must produce the same set of segments (order
        // may differ — the validator will later normalise/accept either).
        let a = parse_line("5 l6, 3 64, 4 43").unwrap();
        let b = parse_line("3 64, 5 l6, 4 43").unwrap();
        let c = parse_line("4 43, 5 l6, 3 64").unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 3);
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn accumulates_all_errors() {
        let result = parse_line("5 h, 3 x9, 4 43");
        let errs = result.expect_err("mid segment is invalid");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].segment_index, 2);
        assert_eq!(errs[0].segment_text, "3 x9");
        assert!(matches!(errs[0].kind, CommandErrorKind::Parse(_)));
    }

    #[test]
    fn accumulates_multiple_errors() {
        let result = parse_line("xyz, 3 x9, 4 43");
        let errs = result.expect_err("two segments are invalid");
        assert_eq!(errs.len(), 2);
        assert_eq!(errs[0].segment_index, 1);
        assert_eq!(errs[1].segment_index, 2);
    }

    #[test]
    fn empty_segment_between_commas() {
        let errs = parse_line("5 h, , 3 sc").expect_err("middle segment is empty");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].segment_index, 2);
    }
}
