//! Play Ball session context — game metadata and pre-game gate checks.
//!
//! These types describe the game *before* it starts (context) and the
//! conditions required to start it (gate). They are distinct from the
//! live `GameState` which tracks what happens during play.

use crate::models::types::GameStatus;

/// Static metadata about the game being scored — team names, venue, IDs.
/// Loaded once at session start; not mutated during play.
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

/// Which side of the lineup is being referenced.
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

/// Result of the pre-game lineup gate check.
///
/// A game can only start (`Ready`) if both teams have a valid number of
/// starting batters in their lineup.
#[derive(Debug)]
pub enum PlayBallGate {
    Ready,
    InvalidLineup {
        side: LineupSide,
        required: i64,
        found: i64,
    },
}
