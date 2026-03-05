use crate::models::types::{HalfInning, Pitch};
use serde::{Deserialize, Serialize};

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
    pub pitches_sequence: Vec<Pitch>,
    pub outcome: PlateAppearanceOutcome,
    pub outs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PlateAppearanceOutcome {
    Walk,
    Strikeout(crate::models::events::StrikeoutKind),
    Out,
}

pub fn format_pitch_sequence(seq: &[Pitch]) -> String {
    let inner = seq
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{}]", inner)
}
