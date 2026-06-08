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

/// Field positions that can be assigned to a player roster profile.
///
/// These positions describe the player's roster capabilities and are
/// independent from lineup defensive positions, which remain numeric
/// values from 1 to 9 plus DH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerFieldPosition {
    P,
    C,
    FirstBase,
    SecondBase,
    ThirdBase,
    Shortstop,
    LeftField,
    CenterField,
    RightField,
    Infield,
    Outfield,
    DH,
    UTL,
}

impl PlayerFieldPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::P => "P",
            Self::C => "C",
            Self::FirstBase => "1B",
            Self::SecondBase => "2B",
            Self::ThirdBase => "3B",
            Self::Shortstop => "SS",
            Self::LeftField => "LF",
            Self::CenterField => "CF",
            Self::RightField => "RF",
            Self::Infield => "IF",
            Self::Outfield => "OF",
            Self::DH => "DH",
            Self::UTL => "UTL",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_uppercase().as_str() {
            "P" => Some(Self::P),
            "C" => Some(Self::C),
            "1B" => Some(Self::FirstBase),
            "2B" => Some(Self::SecondBase),
            "3B" => Some(Self::ThirdBase),
            "SS" => Some(Self::Shortstop),
            "LF" => Some(Self::LeftField),
            "CF" => Some(Self::CenterField),
            "RF" => Some(Self::RightField),
            "IF" => Some(Self::Infield),
            "OF" => Some(Self::Outfield),
            "DH" => Some(Self::DH),
            "UTL" => Some(Self::UTL),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::P,
            Self::C,
            Self::FirstBase,
            Self::SecondBase,
            Self::ThirdBase,
            Self::Shortstop,
            Self::LeftField,
            Self::CenterField,
            Self::RightField,
            Self::Infield,
            Self::Outfield,
            Self::DH,
            Self::UTL,
        ]
    }
}

impl fmt::Display for PlayerFieldPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PlayerFieldPosition {
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

/// Parses and normalizes a comma-separated list of roster field positions.
///
/// Accepted values:
/// `P`, `C`, `1B`, `2B`, `3B`, `SS`, `LF`, `CF`, `RF`, `IF`, `OF`, `DH`. `UTL`
///
/// Example:
/// `P, C, IF` becomes `P,C,IF`.
pub fn parse_player_positions(input: &str) -> Option<String> {
    let positions = input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PlayerFieldPosition::parse)
        .collect::<Option<Vec<_>>>()?;

    if positions.is_empty() {
        return None;
    }

    Some(
        positions
            .iter()
            .map(PlayerFieldPosition::as_str)
            .collect::<Vec<_>>()
            .join(","),
    )
}
