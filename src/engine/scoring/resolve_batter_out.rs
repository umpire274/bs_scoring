//! Resolution logic for batter-only outs.

use std::error::Error;
use std::fmt;

use super::batter_outs::{BatterOutType, ParsedBatterOutCommand};

/// Represents the result of resolving a batter-only out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatterOutResolution {
    pub lineup_slot: u8,
    pub out_type: BatterOutType,
    pub outs_before: u8,
    pub outs_after: u8,
    pub inning_ended: bool,
}

/// Errors returned while resolving a batter-out command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveBatterOutError {
    InvalidCurrentOuts(u8),
    LineupMismatch { expected: u8, found: u8 },
}

impl fmt::Display for ResolveBatterOutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCurrentOuts(value) => {
                write!(f, "invalid current outs value: {value}")
            }
            Self::LineupMismatch { expected, found } => {
                write!(
                    f,
                    "lineup mismatch: expected batter slot {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for ResolveBatterOutError {}

/// Resolves a batter-only out against the current inning state.
///
/// This function:
/// - validates the current batter slot
/// - records one out
/// - determines whether the half-inning has ended
///
/// It does not move runners and does not handle multi-out plays yet.
pub fn resolve_batter_out(
    current_batter_slot: u8,
    current_outs: u8,
    command: ParsedBatterOutCommand,
) -> Result<BatterOutResolution, ResolveBatterOutError> {
    if current_outs > 2 {
        return Err(ResolveBatterOutError::InvalidCurrentOuts(current_outs));
    }

    if command.lineup_slot != current_batter_slot {
        return Err(ResolveBatterOutError::LineupMismatch {
            expected: current_batter_slot,
            found: command.lineup_slot,
        });
    }

    let outs_after = current_outs + 1;
    let inning_ended = outs_after >= 3;

    Ok(BatterOutResolution {
        lineup_slot: command.lineup_slot,
        out_type: command.out_type,
        outs_before: current_outs,
        outs_after,
        inning_ended,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scoring::batter_outs::{BatterOutType, parse_batter_out_command};

    #[test]
    fn resolve_regular_batter_out() {
        let cmd = parse_batter_out_command("7 F8").unwrap();
        let resolution = resolve_batter_out(7, 1, cmd).unwrap();

        assert_eq!(resolution.outs_before, 1);
        assert_eq!(resolution.outs_after, 2);
        assert!(!resolution.inning_ended);

        match resolution.out_type {
            BatterOutType::FlyOut {
                fielder,
                in_foul_territory,
            } => {
                assert_eq!(fielder, 8);
                assert!(!in_foul_territory);
            }
            other => panic!("expected FlyOut, found {other:?}"),
        }
    }

    #[test]
    fn resolve_third_out_ends_inning() {
        let cmd = parse_batter_out_command("2 6-3").unwrap();
        let resolution = resolve_batter_out(2, 2, cmd).unwrap();

        assert_eq!(resolution.outs_before, 2);
        assert_eq!(resolution.outs_after, 3);
        assert!(resolution.inning_ended);
    }

    #[test]
    fn reject_lineup_mismatch() {
        let cmd = parse_batter_out_command("5 L6").unwrap();
        let err = resolve_batter_out(4, 1, cmd).unwrap_err();

        assert!(matches!(
            err,
            ResolveBatterOutError::LineupMismatch {
                expected: 4,
                found: 5
            }
        ));
    }

    #[test]
    fn reject_invalid_current_outs() {
        let cmd = parse_batter_out_command("5 L6").unwrap();
        let err = resolve_batter_out(5, 3, cmd).unwrap_err();

        assert!(matches!(err, ResolveBatterOutError::InvalidCurrentOuts(3)));
    }
}
