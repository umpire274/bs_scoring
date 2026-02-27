pub(crate) use crate::HalfInning;
use crate::PitchCount;
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
    pub started: bool,

    pub current_batter_id: Option<i64>,
    pub current_batter_jersey_no: Option<i32>,
    pub current_batter_first_name: Option<String>,
    pub current_batter_last_name: Option<String>,

    pub current_pitcher_id: Option<i64>,
    pub current_pitcher_jersey_no: Option<i32>,
    pub current_pitcher_first_name: Option<String>,
    pub current_pitcher_last_name: Option<String>,

    pub current_pitch_count: u32, // già ce l’hai per pitcher pitches
    pub pitch_count: PitchCount,  // NEW: balls/strikes + sequence (per PA)

    // NEW: cursore per prossimo battitore (resume-safe)
    pub away_next_batting_order: u8, // 1..=9
    pub home_next_batting_order: u8, // 1..=9

    // (opzionale per 0.6.7: basi)
    pub on_1b: bool,
    pub on_2b: bool,
    pub on_3b: bool,
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

            current_pitcher_id: None,
            current_pitcher_jersey_no: None,
            current_pitcher_first_name: None,
            current_pitcher_last_name: None,

            current_pitch_count: 0,
            pitch_count: PitchCount {
                balls: 0,
                strikes: 0,
                sequence: vec![],
            },
            away_next_batting_order: 1,
            home_next_batting_order: 1,
            on_1b: false,
            on_2b: false,
            on_3b: false,
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
