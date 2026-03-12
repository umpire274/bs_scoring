use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Throwing hand of a pitcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PitchHand {
    Lhp,
    Rhp,
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

    /// Iterate all variants (replaces strum EnumIter).
    pub fn all() -> &'static [Self] {
        &[Self::Lhp, Self::Rhp, Self::Shp]
    }
}

impl fmt::Display for PitchHand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PitchHand {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

/// Batting side of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatSide {
    L,
    R,
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

    /// Iterate all variants (replaces strum EnumIter).
    pub fn all() -> &'static [Self] {
        &[Self::L, Self::R, Self::S]
    }
}

impl fmt::Display for BatSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BatSide {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}
