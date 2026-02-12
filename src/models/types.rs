use serde::{Deserialize, Serialize};
use std::fmt;

/// Base positions (1B, 2B, 3B, etc.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Base {
    First,
    Second,
    Third,
    Home,
}

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
}

#[allow(dead_code)]
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
        };
        write!(f, "{}", name)
    }
}

/// Game status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Pregame = 1,
    InProgress = 2,
    Finished = 3,
}

impl GameStatus {
    pub fn from_i64(n: i64) -> Option<Self> {
        match n {
            1 => Some(GameStatus::Pregame),
            2 => Some(GameStatus::InProgress),
            3 => Some(GameStatus::Finished),
            _ => None,
        }
    }

    pub fn to_i64(self) -> i64 {
        self as i64
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GameStatus::Pregame => "pregame",
            GameStatus::InProgress => "in_progress",
            GameStatus::Finished => "finished",
        }
    }
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            GameStatus::Pregame => "Pre-Game",
            GameStatus::InProgress => "In Progress",
            GameStatus::Finished => "Finished",
        };
        write!(f, "{}", name)
    }
}

/// Type of hit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HitType {
    Single,        // 1B
    Double,        // 2B
    Triple,        // 3B
    HomeRun,       // HR
    GroundRule,    // GRD (Ground Rule Double)
    InsideThePark, // ITP (Inside the Park Home Run)
}

/// Type of out
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutType {
    Strikeout {
        swinging: bool,
        looking: bool,
    }, // K (swinging), ꓘ or K-L (looking)
    Flyout {
        positions: Vec<Position>,
    }, // F7, F8, etc.
    Groundout {
        positions: Vec<Position>,
    }, // 6-3, 4-3, etc.
    Lineout {
        positions: Vec<Position>,
    }, // L6, L9, etc.
    Popup {
        positions: Vec<Position>,
    }, // P4, P5, etc.
    Foulout {
        positions: Vec<Position>,
    }, // FF (foul fly)
    Bunt {
        positions: Vec<Position>,
    }, // SAC (sacrifice bunt)
    DoublePlay {
        positions: Vec<Position>,
    }, // 6-4-3, 4-6-3, etc.
    TriplePlay {
        positions: Vec<Position>,
    },
    Forceout {
        positions: Vec<Position>,
    }, // FC (fielder's choice)
    TagOut {
        position: Position,
        base: Base,
    },
    CaughtStealing {
        catcher_to: Position,
        base: Base,
    }, // CS (caught stealing)
    PickedOff {
        positions: Vec<Position>,
        base: Base,
    }, // PO (picked off)
    IntentionalWalk, // IBB
}

/// Walks and hit by pitch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Walk {
    BaseOnBalls, // BB
    Intentional, // IBB
    HitByPitch,  // HBP
}

/// Errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Error {
    pub position: Position,
    pub description: String, // E6, E4, etc.
}

/// Advanced plays
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdvancedPlay {
    StolenBase { from: Base, to: Base },       // SB
    Balk,                                      // BK
    WildPitch,                                 // WP
    PassedBall,                                // PB
    Interference { by: String },               // INT (catcher interference, etc.)
    Obstruction,                               // OBS
    SacrificeHit,                              // SH (sacrifice bunt)
    SacrificeFly { positions: Vec<Position> }, // SF
}

/// Result of a plate appearance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlateAppearanceResult {
    Hit {
        hit_type: HitType,
        location: Option<String>, // "7-8" (between LF and CF), etc.
        rbis: u8,
    },
    Out {
        out_type: OutType,
        rbi: bool, // Can score on out (sac fly, groundout)
    },
    Walk(Walk),
    Error {
        error: Error,
        reached_base: Base,
    },
    FieldersChoice {
        positions: Vec<Position>,
        out_at: Option<Base>,
    }, // FC
    DroppedThirdStrike, // K+E2, K+WP, K+PB
    AdvancedPlay(AdvancedPlay),
}

/// Runners on base and their advancement
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseRunner {
    pub number: u8, // Jersey number or batting order position
    pub starting_base: Base,
    pub ending_base: Option<Base>, // None if out
    pub scored: bool,
    pub how_advanced: RunnerAdvancement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunnerAdvancement {
    OnHit,
    StolenBase,
    WildPitch,
    PassedBall,
    Balk,
    Error(Position),
    FieldersChoice,
    Advance, // Advanced on play without specific reason
    Out(OutType),
}

/// Complete plate appearance
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateAppearance {
    pub inning: u8,
    pub half_inning: HalfInning,
    pub batter_number: u8,
    pub batter_name: String,
    pub pitcher_name: String,
    pub result: PlateAppearanceResult,
    pub pitch_count: Option<PitchCount>,
    pub runners: Vec<BaseRunner>,
    pub outs_before: u8,
    pub outs_after: u8,
    pub runs_scored: u8,
    pub notes: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HalfInning {
    Top,    // Visiting team batting
    Bottom, // Home team batting
}

/// Pitch count details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchCount {
    pub balls: u8,
    pub strikes: u8,
    pub sequence: Vec<Pitch>, // Detailed pitch sequence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pitch {
    Ball,           // B
    CalledStrike,   // C
    SwingingStrike, // S
    Foul,           // F
    FoulBunt,       // L
    InPlay,         // X
    HittedBy,       // H
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Pitch::Ball => "B",
            Pitch::CalledStrike => "C",
            Pitch::SwingingStrike => "S",
            Pitch::Foul => "F",
            Pitch::FoulBunt => "L",
            Pitch::InPlay => "X",
            Pitch::HittedBy => "H",
        };
        write!(f, "{}", symbol)
    }
}

/// Complete game scoresheet
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub game_id: String,
    pub date: String,
    pub home_team: GameTeam,
    pub away_team: GameTeam,
    pub venue: String,
    pub plate_appearances: Vec<PlateAppearance>,
    pub current_inning: u8,
    pub current_half: HalfInning,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTeam {
    pub name: String,
    pub lineup: Vec<GamePlayer>,
    pub runs: u8,
    pub hits: u8,
    pub errors: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayer {
    pub number: u8,
    pub name: String,
    pub position: Position,
    pub batting_order: u8,
}

#[allow(dead_code)]
impl Game {
    pub fn new(
        game_id: String,
        date: String,
        home_team: GameTeam,
        away_team: GameTeam,
        venue: String,
    ) -> Self {
        Game {
            game_id,
            date,
            home_team,
            away_team,
            venue,
            plate_appearances: Vec::new(),
            current_inning: 1,
            current_half: HalfInning::Top,
        }
    }

    pub fn add_plate_appearance(&mut self, pa: PlateAppearance) {
        self.plate_appearances.push(pa);
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
