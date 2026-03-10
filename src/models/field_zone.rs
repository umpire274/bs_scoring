use serde::{Deserialize, Serialize};

/// Field zones used to tag hits according to scorer spray chart notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldZone {
    LL,
    LF,
    LC,
    CF,
    RC,
    RF,
    RL,
    GLL,
    LS,
    MI,
    RS,
    GRL,
}

impl FieldZone {
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_uppercase().as_str() {
            "LL" => Some(Self::LL),
            "LF" => Some(Self::LF),
            "LC" => Some(Self::LC),
            "CF" => Some(Self::CF),
            "RC" => Some(Self::RC),
            "RF" => Some(Self::RF),
            "RL" => Some(Self::RL),
            "GLL" => Some(Self::GLL),
            "LS" => Some(Self::LS),
            "MI" => Some(Self::MI),
            "RS" => Some(Self::RS),
            "GRL" => Some(Self::GRL),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LL => "LL",
            Self::LF => "LF",
            Self::LC => "LC",
            Self::CF => "CF",
            Self::RC => "RC",
            Self::RF => "RF",
            Self::RL => "RL",
            Self::GLL => "GLL",
            Self::LS => "LS",
            Self::MI => "MI",
            Self::RS => "RS",
            Self::GRL => "GRL",
        }
    }
}
