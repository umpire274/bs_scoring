use crate::models::types::{HalfInning, Pitch};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A compact, persisted representation of a completed Plate Appearance (PA).
///
/// This is the "definitive" storage format used to drastically reduce DB rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateAppearance {
    pub inning: u32,
    pub half: HalfInning,
    pub batter_id: i64,
    pub pitcher_id: i64,
    /// Total pitches thrown in this PA.
    pub pitches: u32,
    /// Full pitch sequence faced by the batter in this PA (JSON persisted).
    pub pitches_sequence: Vec<PlateAppearanceStep>,
    pub outcome: PlateAppearanceOutcome,
    pub outs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlateAppearanceStep {
    Pitch(Pitch),
    Single,
    Double,
    Triple,
    HomeRun,
}

impl fmt::Display for PlateAppearanceStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlateAppearanceStep::Pitch(p) => write!(f, "{p}"),
            PlateAppearanceStep::Single => write!(f, "1B"),
            PlateAppearanceStep::Double => write!(f, "2B"),
            PlateAppearanceStep::Triple => write!(f, "3B"),
            PlateAppearanceStep::HomeRun => write!(f, "HR"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PlateAppearanceOutcome {
    Walk,
    Strikeout(crate::models::events::StrikeoutKind),
    Out,
    Single,
    Double,
    Triple,
    HomeRun,
}
