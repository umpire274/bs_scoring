//! Runner advancement types — explicit overrides entered by the scorer.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::game_state::BatterOrder;

/// Explicit destination for a runner after a hit.
///
/// When the scorer specifies where a runner ends up, this overrides the
/// automatic advancement logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunnerDest {
    /// Runner stays on / goes to first base.
    First,
    /// Runner stays on / goes to second base.
    Second,
    /// Runner advances to third base.
    Third,
    /// Runner scores.
    Score,
}

impl RunnerDest {
    /// Parse from scorer input: `"1b"`, `"2b"`, `"3b"`, `"sc"`, `"score"`, `"home"`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "1b" => Some(Self::First),
            "2b" => Some(Self::Second),
            "3b" => Some(Self::Third),
            "sc" | "score" | "home" => Some(Self::Score),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::First => "1B",
            Self::Second => "2B",
            Self::Third => "3B",
            Self::Score => "SC",
        }
    }
}

impl fmt::Display for RunnerDest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An explicit override for one runner: "batting-order slot N goes to dest D".
///
/// Entered by the scorer as part of a hit command:
/// `"6 h, 5 2b"` → batter #6 singles; runner #5 stays on 2nd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerOverride {
    /// Batting order of the runner being overridden (1–9).
    pub order: BatterOrder,
    /// Where this runner ends up after the play.
    pub dest: RunnerDest,
}
