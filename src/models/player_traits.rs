use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Throwing hand of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThrowHand {
    L,
    R,
    S,
}

impl ThrowHand {
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

            // Backward-compatible legacy values.
            "LHP" => Some(Self::L),
            "RHP" => Some(Self::R),
            "SHP" => Some(Self::S),

            _ => None,
        }
    }

    /// Iterate all variants.
    pub fn all() -> &'static [Self] {
        &[Self::L, Self::R, Self::S]
    }
}

impl fmt::Display for ThrowHand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ThrowHand {
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

/// Parses a combined BAT/THROW handedness value.
///
/// Expected format:
/// - `R/R`
/// - `R/L`
/// - `L/R`
/// - `S/S`
///
/// The value before `/` is the batting side.
/// The value after `/` is the throwing hand.
pub fn parse_bat_throw(input: &str) -> Option<(BatSide, ThrowHand)> {
    let mut parts = input.trim().split('/');

    let bat = parts.next()?.trim().parse::<BatSide>().ok()?;
    let throw = parts.next()?.trim().parse::<ThrowHand>().ok()?;

    if parts.next().is_some() {
        return None;
    }

    Some((bat, throw))
}
