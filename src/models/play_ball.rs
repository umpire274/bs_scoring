use crate::models::types::{GameStatus, HalfInning, Score};
use crate::{PitchCount, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct PlayBallGameContext {
    pub id: i64,
    pub game_id: String,
    pub game_date: String,
    pub venue: String,

    pub away_team_id: i64,
    pub away_team_name: String,
    pub away_team_abbr: Option<String>,

    pub home_team_id: i64,
    pub home_team_name: String,
    pub home_team_abbr: Option<String>,

    pub status: GameStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum LineupSide {
    Away,
    Home,
}

impl LineupSide {
    pub fn label(self) -> &'static str {
        match self {
            LineupSide::Away => "Away",
            LineupSide::Home => "Home",
        }
    }
}

#[derive(Debug)]
pub enum PlayBallGate {
    Ready,
    InvalidLineup {
        side: LineupSide,
        required: i64,
        found: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PitchStats {
    pub balls: u32,
    pub strikes: u32,
}

pub type BatterOrder = u8;

// ─── Runner advancement overrides ────────────────────────────────────────────

/// Explicit destination for a runner after a hit.
///
/// When the scorer specifies where a runner ends up, this overrides the
/// automatic advancement logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunnerDest {
    /// Runner stays on first base (dest = 1B)
    First,
    /// Runner advances to / stays on second base (dest = 2B)
    Second,
    /// Runner advances to / stays on third base (dest = 3B)
    Third,
    /// Runner scores (dest = home)
    Score,
}

impl RunnerDest {
    /// Parse from a token like "1b", "2b", "3b", "sc", "score".
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerOverride {
    /// Batting order of the runner being overridden (1-9).
    pub order: BatterOrder,
    /// Where this runner ends up after the play.
    pub dest: RunnerDest,
}

// ─── Game state ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GameState {
    pub inning: u32,
    pub half: HalfInning,
    pub outs: u8,
    pub score: Score,
    pub started: bool,

    pub current_batter_id: Option<i64>,
    pub current_batter_jersey_no: Option<i32>,
    pub current_batter_first_name: Option<String>,
    pub current_batter_last_name: Option<String>,
    pub current_batter_order: Option<BatterOrder>,
    pub current_batter_position: Option<Position>,

    pub current_pitcher_id: Option<i64>,
    pub current_pitcher_jersey_no: Option<i32>,
    pub current_pitcher_first_name: Option<String>,
    pub current_pitcher_last_name: Option<String>,

    pub pitch_count: PitchCount,
    pub pitcher_stats: HashMap<i64, PitchStats>,

    /// Cursor for next batter — resume-safe
    pub away_next_batting_order: u8,
    pub home_next_batting_order: u8,

    /// Who is on each base, identified by batting order (None = base empty).
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

    /// Returns which base (1/2/3) this batting-order slot currently occupies,
    /// or `None` if the runner is not on base.
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

pub enum OutcomeSymbol {
    Walk,
    Strikeout,
    InPlay,
    Out,
    Single,
    Double,
    Triple,
    HomeRun,
}

impl fmt::Display for OutcomeSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            OutcomeSymbol::Walk => "BB",
            OutcomeSymbol::Strikeout => "K",
            OutcomeSymbol::InPlay => "In Play",
            OutcomeSymbol::Out => "Out",
            OutcomeSymbol::Single => "H",
            OutcomeSymbol::Double => "2H",
            OutcomeSymbol::Triple => "3H",
            OutcomeSymbol::HomeRun => "HR",
        };
        write!(f, "{}", symbol)
    }
}
