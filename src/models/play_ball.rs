pub(crate) use crate::HalfInning;
use crate::models::types::{GameStatus, Score};

#[derive(Debug, Clone)]
pub struct PlayBallGameContext {
    pub id: i64,         // games.id (pk interno)
    pub game_id: String, // games.game_id (string id che usi ovunque)
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

#[derive(Debug, Clone)]
pub struct GameState {
    pub inning: u32,
    pub half: HalfInning,
    pub outs: u8,
    pub score: Score,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            inning: 1,
            half: HalfInning::Top,
            outs: 0,
            score: Score::new(),
        }
    }

    pub fn half_symbol(&self) -> &'static str {
        match self.half {
            HalfInning::Top => "↑",
            HalfInning::Bottom => "↓",
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
