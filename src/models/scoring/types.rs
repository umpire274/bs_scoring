//! Full scoring domain types — used by the notation parser (`core/parser.rs`)
//! and reserved for future play-by-play engine extensions.
//!
//! These are **not** used by the live play-ball engine, which uses the compact
//! `PlateAppearanceOutcome` model in `models/plate_appearance.rs`.

use crate::models::types::Position;
use serde::{Deserialize, Serialize};

// ─── Base ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Base {
    First,
    Second,
    Third,
    Home,
}

// ─── Hit ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HitType {
    Single,
    Double,
    Triple,
    HomeRun,
    GroundRule,
    InsideThePark,
}

// ─── Out ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutType {
    Strikeout { swinging: bool, looking: bool },
    Flyout { positions: Vec<Position> },
    Groundout { positions: Vec<Position> },
    Lineout { positions: Vec<Position> },
    Popup { positions: Vec<Position> },
    Foulout { positions: Vec<Position> },
    Bunt { positions: Vec<Position> },
    DoublePlay { positions: Vec<Position> },
    TriplePlay { positions: Vec<Position> },
    Forceout { positions: Vec<Position> },
    TagOut { position: Position, base: Base },
    CaughtStealing { catcher_to: Position, base: Base },
    PickedOff { positions: Vec<Position>, base: Base },
    IntentionalWalk,
}

// ─── Walk / HBP ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Walk {
    BaseOnBalls,
    Intentional,
    HitByPitch,
}

// ─── Error ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoringError {
    pub position: Position,
    pub description: String,
}

// ─── Advanced plays ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdvancedPlay {
    StolenBase { from: Base, to: Base },
    Balk,
    WildPitch,
    PassedBall,
    Interference { by: String },
    Obstruction,
    SacrificeHit,
    SacrificeFly { positions: Vec<Position> },
}

// ─── Plate appearance result ──────────────────────────────────────────────────

/// Complete plate appearance result — covers all official scoring outcomes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlateAppearanceResult {
    Hit {
        hit_type: HitType,
        location: Option<String>,
        rbis: u8,
    },
    Out {
        out_type: OutType,
        rbi: bool,
    },
    Walk(Walk),
    Error {
        reached_base: Base,
    },
    FieldersChoice {
        positions: Vec<Position>,
        out_at: Option<Base>,
    },
    DroppedThirdStrike,
    AdvancedPlay(AdvancedPlay),
}
