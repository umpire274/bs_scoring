//! Live game state — the single source of truth for the in-memory scoreboard
//! during a Play Ball session.

use crate::models::types::{HalfInning, PitchCount, Position, Score};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Batting order slot (1–9).
pub type BatterOrder = u8;

/// Per-pitcher pitch count accumulated during the current game session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PitchStats {
    pub balls: u32,
    pub strikes: u32,
}

/// The complete in-memory state of an active Play Ball session.
///
/// Rebuilt from the DB on resume; mutated by the reducer on every play.
#[derive(Debug, Clone)]
pub struct GameState {
    pub inning: u32,
    pub half: HalfInning,
    pub outs: u8,
    pub score: Score,
    pub started: bool,

    // ── Current batter ───────────────────────────────────────────────────────
    pub current_batter_id: Option<i64>,
    pub current_batter_jersey_no: Option<i32>,
    pub current_batter_first_name: Option<String>,
    pub current_batter_last_name: Option<String>,
    pub current_batter_order: Option<BatterOrder>,
    pub current_batter_position: Option<Position>,

    // ── Current pitcher ──────────────────────────────────────────────────────
    pub current_pitcher_id: Option<i64>,
    pub current_pitcher_jersey_no: Option<i32>,
    pub current_pitcher_first_name: Option<String>,
    pub current_pitcher_last_name: Option<String>,

    // ── Count / pitch tracking ───────────────────────────────────────────────
    pub pitch_count: PitchCount,
    pub pitcher_stats: HashMap<i64, PitchStats>,

    // ── Batting order cursors (resume-safe) ───────────────────────────────────
    pub away_next_batting_order: u8,
    pub home_next_batting_order: u8,

    // ── Base occupancy — None = empty, Some(order) = runner identity ─────────
    pub on_1b: Option<BatterOrder>,
    pub on_2b: Option<BatterOrder>,
    pub on_3b: Option<BatterOrder>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            inning: 1,
            half: HalfInning::Top,
            outs: 0,
            score: Score::new(),
            started: false,

            current_batter_id: None,
            current_batter_jersey_no: None,
            current_batter_first_name: None,
            current_batter_last_name: None,
            current_batter_order: None,
            current_batter_position: None,

            current_pitcher_id: None,
            current_pitcher_jersey_no: None,
            current_pitcher_first_name: None,
            current_pitcher_last_name: None,

            pitch_count: PitchCount {
                balls: 0,
                strikes: 0,
                sequence: vec![],
            },
            pitcher_stats: HashMap::new(),

            away_next_batting_order: 1,
            home_next_batting_order: 1,

            on_1b: None,
            on_2b: None,
            on_3b: None,
        }
    }

    pub fn half_symbol(&self) -> &'static str {
        match self.half {
            HalfInning::Top => "↑",
            HalfInning::Bottom => "↓",
        }
    }

    /// Returns true if the given batting-order slot is currently on any base.
    pub fn is_on_base(&self, order: BatterOrder) -> bool {
        self.on_1b == Some(order) || self.on_2b == Some(order) || self.on_3b == Some(order)
    }

    /// Returns which base (1/2/3) this batting-order slot occupies, or `None`.
    pub fn base_of(&self, order: BatterOrder) -> Option<u8> {
        if self.on_1b == Some(order) {
            Some(1)
        } else if self.on_2b == Some(order) {
            Some(2)
        } else if self.on_3b == Some(order) {
            Some(3)
        } else {
            None
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
