//! Public facade of the command pipeline.
//!
//! This module composes the two stages into a single entry point consumed by
//! the game loop:
//!
//! ```text
//!                  raw line ──► parse_engine_commands(&state) ──► Vec<EngineCommand>
//!                                     │
//!                  grammar::parse_line (syntactic, no state)
//!                                     │
//!                  validator::validate (semantic, needs state)
//! ```
//!
//! The facade returns `Result<Vec<EngineCommand>, Vec<CommandError>>` so that
//! **every** syntactic or semantic problem in a line surfaces in a single
//! pass — no "first-error wins".

use crate::engine::commands::errors::CommandError;
use crate::engine::commands::grammar::parse_line;
use crate::engine::commands::types::EngineCommand;
use crate::engine::commands::validator::{IndexedSegment, validate};
use crate::models::game_state::GameState;

/// Parse and validate a single input line against the current game state.
///
/// A completely empty/blank line returns `Ok(vec![])`; the caller is expected
/// to simply re-prompt.
pub fn parse_engine_commands(
    line: &str,
    state: &GameState,
) -> Result<Vec<EngineCommand>, Vec<CommandError>> {
    // Step 1 — syntactic parsing (stateless).
    let segments = parse_line(line)?;

    // Step 2 — re-attach segment text + 1-based index for validator
    // error reporting. Done by re-splitting the line rather than
    // widening grammar's public API.
    let texts: Vec<&str> = line.trim().split(',').map(|s| s.trim()).collect();
    let indexed: Vec<IndexedSegment> = segments
        .into_iter()
        .zip(texts.iter())
        .enumerate()
        .map(|(i, (seg, &text))| IndexedSegment {
            index: i + 1,
            text: text.to_string(),
            segment: seg,
        })
        .collect();

    // Step 3 — semantic validation against the game state.
    validate(indexed, state)
}

// ─── Tests ────────────────────────────────────────────────────────────────────
//
// The grammar and validator modules each carry their own exhaustive unit
// tests. This file keeps a smaller integration suite focused on the facade
// itself — a sanity check that the two stages are wired correctly.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::commands::errors::CommandErrorKind;
    use crate::models::field_zone::FieldZone;
    use crate::models::runner::RunnerDest;
    use crate::models::types::HalfInning;

    fn state(
        current_batter: u8,
        on_1b: Option<u8>,
        on_2b: Option<u8>,
        on_3b: Option<u8>,
    ) -> GameState {
        let mut s = GameState::new();
        s.half = HalfInning::Top;
        s.current_batter_order = Some(current_batter);
        s.on_1b = on_1b;
        s.on_2b = on_2b;
        s.on_3b = on_3b;
        s
    }

    #[test]
    fn empty_line_returns_empty_vec() {
        let s = state(1, None, None, None);
        let cmds = parse_engine_commands("", &s).expect("empty line parses");
        assert!(cmds.is_empty());
        let cmds = parse_engine_commands("   ", &s).expect("whitespace-only line parses");
        assert!(cmds.is_empty());
    }

    #[test]
    fn bare_hit_without_subject() {
        let s = state(1, None, None, None);
        let cmds = parse_engine_commands("h", &s).unwrap();
        assert!(matches!(cmds[0], EngineCommand::Single { .. }));
    }

    #[test]
    fn hit_with_zone_and_override() {
        let s = state(6, Some(5), None, None);
        let cmds = parse_engine_commands("6 h lf, 5 sc", &s).unwrap();
        match &cmds[0] {
            EngineCommand::Single {
                zone: Some(FieldZone::LF),
                runner_overrides,
            } => {
                assert_eq!(runner_overrides.len(), 1);
                assert_eq!(runner_overrides[0].order, 5);
                assert_eq!(runner_overrides[0].dest, RunnerDest::Score);
            }
            _ => panic!("expected Single LF with one override"),
        }
    }

    #[test]
    fn parse_error_surfaces_with_segment_index() {
        let s = state(5, None, None, None);
        let errs = parse_engine_commands("5 h, xyz", &s).unwrap_err();
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].segment_index, 2);
        assert!(matches!(errs[0].kind, CommandErrorKind::Parse(_)));
    }

    #[test]
    fn validation_error_surfaces_with_segment_index() {
        let s = state(5, None, None, None);
        let errs = parse_engine_commands("8 h", &s).unwrap_err();
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].segment_index, 1);
        assert!(matches!(errs[0].kind, CommandErrorKind::Validation(_)));
    }

    #[test]
    fn triple_play_from_facade() {
        let s = state(5, Some(4), Some(3), None);
        let cmds = parse_engine_commands("5 l6, 3 64, 4 43", &s).unwrap();
        let EngineCommand::DefensivePlay(dp) = &cmds[0] else {
            panic!("expected defensive play")
        };
        assert_eq!(dp.outs.len(), 3);
    }

    #[test]
    fn double_steal_after_pitch() {
        let s = state(6, Some(5), Some(4), None);
        let cmds = parse_engine_commands("b, 5 st 2b, 4 st 3b", &s).unwrap();
        assert_eq!(cmds.len(), 3);
        assert!(matches!(cmds[0], EngineCommand::Pitch(_)));
        assert!(matches!(cmds[1], EngineCommand::StealBase { .. }));
        assert!(matches!(cmds[2], EngineCommand::StealBase { .. }));
    }
}
