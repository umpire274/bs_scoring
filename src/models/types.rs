use serde::{Deserialize, Serialize};
use std::fmt;

/// Defensive positions with official scoring numbers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Position {
    Pitcher = 1,
    Catcher = 2,
    FirstBase = 3,
    SecondBase = 4,
    ThirdBase = 5,
    Shortstop = 6,
    LeftField = 7,
    CenterField = 8,
    RightField = 9,
    DesignatedHitter = 10,
}

impl Position {
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Position::Pitcher),
            2 => Some(Position::Catcher),
            3 => Some(Position::FirstBase),
            4 => Some(Position::SecondBase),
            5 => Some(Position::ThirdBase),
            6 => Some(Position::Shortstop),
            7 => Some(Position::LeftField),
            8 => Some(Position::CenterField),
            9 => Some(Position::RightField),
            10 => Some(Position::DesignatedHitter),
            _ => None,
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "1" | "P" => Some(Position::Pitcher),
            "2" | "C" => Some(Position::Catcher),
            "3" | "1B" => Some(Position::FirstBase),
            "4" | "2B" => Some(Position::SecondBase),
            "5" | "3B" => Some(Position::ThirdBase),
            "6" | "SS" => Some(Position::Shortstop),
            "7" | "LF" => Some(Position::LeftField),
            "8" | "CF" => Some(Position::CenterField),
            "9" | "RF" => Some(Position::RightField),
            "10" | "DH" => Some(Position::DesignatedHitter),
            _ => None,
        }
    }

    pub fn to_number(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Position::Pitcher => "P",
            Position::Catcher => "C",
            Position::FirstBase => "1B",
            Position::SecondBase => "2B",
            Position::ThirdBase => "3B",
            Position::Shortstop => "SS",
            Position::LeftField => "LF",
            Position::CenterField => "CF",
            Position::RightField => "RF",
            Position::DesignatedHitter => "DH",
        };
        write!(f, "{}", name)
    }
}

/// Game status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Pregame = 1,
    InProgress = 2,
    Regulation = 3,
    Postponed = 4,
    Cancelled = 5,
    Suspended = 6,
    Forfeited = 7,
    Protested = 8,
}

impl GameStatus {
    /// Parse from DB integer. Prefer `TryFrom<i64>` for new code.
    pub fn from_i64(n: i64) -> Option<Self> {
        Self::try_from(n).ok()
    }

    /// Convert to DB integer. Prefer `i64::from(status)` for new code.
    pub fn to_i64(self) -> i64 {
        i64::from(self)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GameStatus::Pregame => "pregame",
            GameStatus::InProgress => "in_progress",
            GameStatus::Regulation => "regulation_game",
            GameStatus::Postponed => "postponed",
            GameStatus::Cancelled => "cancelled",
            GameStatus::Suspended => "suspended",
            GameStatus::Forfeited => "forfeited",
            GameStatus::Protested => "protested",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            GameStatus::Pregame => "🆕",
            GameStatus::InProgress => "▶️",
            GameStatus::Regulation => "✅",
            GameStatus::Postponed => "⏳",
            GameStatus::Cancelled => "❌",
            GameStatus::Suspended => "⏸️",
            GameStatus::Forfeited => "⚠️",
            GameStatus::Protested => "🚩",
        }
    }
}

impl TryFrom<i64> for GameStatus {
    type Error = i64;

    fn try_from(n: i64) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(GameStatus::Pregame),
            2 => Ok(GameStatus::InProgress),
            3 => Ok(GameStatus::Regulation),
            4 => Ok(GameStatus::Postponed),
            5 => Ok(GameStatus::Cancelled),
            6 => Ok(GameStatus::Suspended),
            7 => Ok(GameStatus::Forfeited),
            8 => Ok(GameStatus::Protested),
            other => Err(other),
        }
    }
}

impl From<GameStatus> for i64 {
    fn from(s: GameStatus) -> i64 {
        s as i64
    }
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.icon(),
            match self {
                GameStatus::Pregame => "Pre-Game",
                GameStatus::InProgress => "In Progress",
                GameStatus::Regulation => "Regulation Game",
                GameStatus::Postponed => "Postponed Game",
                GameStatus::Cancelled => "Cancelled Game",
                GameStatus::Suspended => "Suspended Game",
                GameStatus::Forfeited => "Forfeited Game",
                GameStatus::Protested => "Protested Game",
            }
        )
    }
}

/// Pitch count details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchCount {
    pub balls: u8,
    pub strikes: u8,
    pub sequence: Vec<Pitch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pitch {
    Ball,           // B
    CalledStrike,   // K
    SwingingStrike, // S
    Foul,           // F
    FoulBunt,       // FL
    InPlay,         // X
    HittedBy,       // H
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Pitch::Ball => "B",
            Pitch::CalledStrike => "K",
            Pitch::SwingingStrike => "S",
            Pitch::Foul => "F",
            Pitch::FoulBunt => "FL",
            Pitch::InPlay => "X",
            Pitch::HittedBy => "H",
        };
        write!(f, "{}", symbol)
    }
}

/// Half inning (Top = visiting team bats, Bottom = home team bats)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HalfInning {
    Top,    // Visiting team batting
    Bottom, // Home team batting
}

/// Score tracking for a game
#[derive(Debug, Clone, Default)]
pub struct Score {
    pub away: u16,
    pub home: u16,
    pub away_innings: Vec<u16>,
    pub home_innings: Vec<u16>,
    pub away_hits: u16,
    pub home_hits: u16,
    pub away_errors: u16,
    pub home_errors: u16,
}

impl Score {
    pub fn new() -> Self {
        Self::default()
    }
}

// ─── Legacy types used only by CommandParser ────────────────────────────────
