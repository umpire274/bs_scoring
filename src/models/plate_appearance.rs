use crate::models::field_zone::FieldZone;
use crate::models::game_state::BatterOrder;
use crate::models::runner::RunnerOverride;
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
    pub batter_order: BatterOrder,
    pub pitcher_id: i64,
    /// Total pitches thrown in this PA.
    pub pitches: u32,
    /// Full pitch sequence faced by the batter in this PA (JSON persisted).
    pub pitches_sequence: Vec<PlateAppearanceStep>,
    pub outcome: PlateAppearanceOutcome,
    pub outs: u8,
    /// Explicit runner destinations entered by the scorer (0.8.0+).
    /// Empty vec means full automatic advancement.
    #[serde(default)]
    pub runner_overrides: Vec<RunnerOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlateAppearanceStep {
    Pitch(Pitch),
    Single,
    Double,
    Triple,
    HomeRun,
    Walk,
    Strikeout,
    Out,
}

impl fmt::Display for PlateAppearanceStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlateAppearanceStep::Pitch(p) => write!(f, "{p}"),
            PlateAppearanceStep::Single => write!(f, "H"),
            PlateAppearanceStep::Double => write!(f, "2H"),
            PlateAppearanceStep::Triple => write!(f, "3H"),
            PlateAppearanceStep::HomeRun => write!(f, "HR"),
            PlateAppearanceStep::Walk => write!(f, "BB"),
            PlateAppearanceStep::Strikeout => write!(f, "K"),
            PlateAppearanceStep::Out => write!(f, "OUT"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitOutcomeData {
    pub zone: Option<FieldZone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PlateAppearanceOutcome {
    Walk,
    Strikeout(crate::models::events::StrikeoutKind),
    Out,
    Single { zone: Option<FieldZone> },
    Double { zone: Option<FieldZone> },
    Triple { zone: Option<FieldZone> },
    HomeRun { zone: Option<FieldZone> },
}
