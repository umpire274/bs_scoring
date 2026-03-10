use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, EnumString, Display,
)]
pub enum PitchHand {
    #[strum(serialize = "LHP")]
    Lhp,
    #[strum(serialize = "RHP")]
    Rhp,
    #[strum(serialize = "SHP")]
    Shp,
}

impl PitchHand {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lhp => "LHP",
            Self::Rhp => "RHP",
            Self::Shp => "SHP",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_uppercase().as_str() {
            "LHP" => Some(Self::Lhp),
            "RHP" => Some(Self::Rhp),
            "SHP" => Some(Self::Shp),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, EnumString, Display,
)]
pub enum BatSide {
    #[strum(serialize = "L")]
    L,
    #[strum(serialize = "R")]
    R,
    #[strum(serialize = "S")]
    S,
}

impl BatSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::L => "L",
            Self::R => "R",
            Self::S => "S",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_uppercase().as_str() {
            "L" => Some(Self::L),
            "R" => Some(Self::R),
            "S" => Some(Self::S),
            _ => None,
        }
    }
}
