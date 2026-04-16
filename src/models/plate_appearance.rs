use crate::RunnerDest;
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
    UnassistedOut {
        fielder: u8,
    },
    GroundOut {
        sequence: String,
    },
    FlyOut {
        fielder: u8,
        in_foul_territory: bool,
    },
    LineOut {
        fielder: u8,
    },
    InfieldFly {
        fielder: u8,
    },
    FieldersChoice {
        fielder: u8,
        reached_base: RunnerDest,
    },
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

            PlateAppearanceStep::UnassistedOut { .. } => write!(f, "UO"),
            PlateAppearanceStep::GroundOut { .. } => write!(f, "GO"),
            PlateAppearanceStep::FlyOut {
                in_foul_territory: false,
                ..
            } => write!(f, "FO"),
            PlateAppearanceStep::FlyOut {
                in_foul_territory: true,
                ..
            } => write!(f, "FFO"),
            PlateAppearanceStep::LineOut { .. } => write!(f, "LO"),
            PlateAppearanceStep::InfieldFly { .. } => write!(f, "IF"),
            PlateAppearanceStep::FieldersChoice { .. } => write!(f, "FC"),
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
    Single {
        zone: Option<FieldZone>,
    },
    Double {
        zone: Option<FieldZone>,
    },
    Triple {
        zone: Option<FieldZone>,
    },
    HomeRun {
        zone: Option<FieldZone>,
    },

    UnassistedOut {
        fielder: u8,
    },
    GroundOut {
        sequence: String,
    },
    FlyOut {
        fielder: u8,
        in_foul_territory: bool,
    },
    LineOut {
        fielder: u8,
    },
    InfieldFly {
        fielder: u8,
    },
    FieldersChoice {
        fielder: u8,
        reached_base: RunnerDest,
    },
}

impl PlateAppearanceOutcome {
    /// Number of bases the batter reaches on a hit (1-4). Returns 0 for non-hit outcomes.
    pub fn bases(&self) -> u8 {
        match self {
            Self::Single { .. } => 1,
            Self::Double { .. } => 2,
            Self::Triple { .. } => 3,
            Self::HomeRun { .. } => 4,
            _ => 0,
        }
    }

    /// Returns true if this outcome is a hit (Single, Double, Triple, HomeRun).
    pub fn is_hit(&self) -> bool {
        self.bases() > 0
    }

    /// Extract the field zone from hit outcomes.
    pub fn zone(&self) -> Option<FieldZone> {
        match self {
            Self::Single { zone }
            | Self::Double { zone }
            | Self::Triple { zone }
            | Self::HomeRun { zone } => *zone,
            _ => None,
        }
    }

    /// Short label for display (e.g. "H", "2H", "3H", "HR", "BB", "K", "OUT").
    pub fn label(&self) -> &'static str {
        match self {
            Self::Single { .. } => "H",
            Self::Double { .. } => "2H",
            Self::Triple { .. } => "3H",
            Self::HomeRun { .. } => "HR",
            Self::Walk => "BB",
            Self::Strikeout(_) => "K",
            Self::Out => "OUT",
            Self::UnassistedOut { .. } => "UO",
            Self::GroundOut { .. } => "GO",
            Self::FlyOut {
                in_foul_territory: false,
                ..
            } => "FO",
            Self::FlyOut {
                in_foul_territory: true,
                ..
            } => "FFO",
            Self::LineOut { .. } => "LO",
            Self::InfieldFly { .. } => "IF",
            Self::FieldersChoice { .. } => "FC",
        }
    }

    /// Human-readable label for display.
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Single { .. } => "Single",
            Self::Double { .. } => "Double",
            Self::Triple { .. } => "Triple",
            Self::HomeRun { .. } => "Home run",
            Self::Walk => "BB",
            Self::Strikeout(_) => "K",
            Self::Out => "OUT",
            Self::UnassistedOut { .. } => "Unassisted out",
            Self::GroundOut { .. } => "Ground out",
            Self::FlyOut {
                in_foul_territory: false,
                ..
            } => "Fly out",
            Self::FlyOut {
                in_foul_territory: true,
                ..
            } => "Foul fly out",
            Self::LineOut { .. } => "Line out",
            Self::InfieldFly { .. } => "Infield fly",
            Self::FieldersChoice { .. } => "Fielder's choice",
        }
    }
}
