//! Batter-out parsing and domain types for scoring plays.
//!
//! This module handles batter-only outs such as:
//! - ground outs (`63`, `6-3`, `862`, `8-6-2`, ...)
//! - fly outs (`F8`, `FF3`)
//! - line outs (`L6`)
//! - infield fly (`IF4`)

use std::error::Error;
use std::fmt;

/// Represents the ordered defensive sequence that produced an out.
///
/// The last fielder is credited with the putout.
/// All previous fielders are credited with assists.
///
/// Examples:
/// - `6-3`  => assist: 6, putout: 3
/// - `8-6-2` => assists: 8, 6, putout: 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldingSequence {
    fielders: Vec<u8>,
}

impl FieldingSequence {
    /// Creates a validated fielding sequence.
    ///
    /// Rules:
    /// - at least 2 fielders
    /// - each fielder must be between 1 and 9
    pub fn new(fielders: Vec<u8>) -> Result<Self, BatterOutParseError> {
        if fielders.len() < 2 {
            return Err(BatterOutParseError::InvalidFieldingSequence(
                "a fielding sequence must contain at least two fielders".to_string(),
            ));
        }

        if fielders.iter().any(|&n| !(1..=9).contains(&n)) {
            return Err(BatterOutParseError::InvalidFielder(
                "fielders must be in the range 1..=9".to_string(),
            ));
        }

        Ok(Self { fielders })
    }

    /// Returns the full validated fielding chain.
    pub fn fielders(&self) -> &[u8] {
        &self.fielders
    }

    /// Returns the assisting fielders.
    pub fn assists(&self) -> &[u8] {
        &self.fielders[..self.fielders.len() - 1]
    }

    /// Returns the fielder credited with the putout.
    pub fn putout(&self) -> u8 {
        self.fielders[self.fielders.len() - 1]
    }

    /// Returns the normalized textual representation, e.g. `6-3` or `8-6-2`.
    pub fn as_hyphenated_string(&self) -> String {
        self.fielders
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join("-")
    }
}

/// Represents batter-only out types supported by the scoring parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatterOutType {
    /// Batter is retired on a defensive sequence such as `6-3` or `8-6-2`.
    GroundOut { sequence: FieldingSequence },

    /// Fly out in fair or foul territory.
    FlyOut {
        fielder: u8,
        in_foul_territory: bool,
    },

    /// Line out.
    LineOut { fielder: u8 },

    /// Infield fly.
    InfieldFly { fielder: u8 },
}

impl BatterOutType {
    /// Returns a compact human-readable label for the outcome.
    pub fn label(&self) -> &'static str {
        match self {
            Self::GroundOut { .. } => "ground_out",
            Self::FlyOut {
                in_foul_territory: false,
                ..
            } => "fly_out",
            Self::FlyOut {
                in_foul_territory: true,
                ..
            } => "foul_fly_out",
            Self::LineOut { .. } => "line_out",
            Self::InfieldFly { .. } => "infield_fly",
        }
    }
}

/// Represents a parsed batter-out command.
///
/// Expected input format:
/// `<lineup_slot> <play_token>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBatterOutCommand {
    pub lineup_slot: u8,
    pub out_type: BatterOutType,
}

/// Parsing errors for batter-out commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatterOutParseError {
    EmptyInput,
    InvalidFormat(String),
    InvalidLineupSlot(String),
    InvalidFielder(String),
    InvalidFieldingSequence(String),
    UnsupportedToken(String),
}

impl fmt::Display for BatterOutParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "input is empty"),
            Self::InvalidFormat(msg) => write!(f, "invalid format: {msg}"),
            Self::InvalidLineupSlot(msg) => write!(f, "invalid lineup slot: {msg}"),
            Self::InvalidFielder(msg) => write!(f, "invalid fielder: {msg}"),
            Self::InvalidFieldingSequence(msg) => write!(f, "invalid fielding sequence: {msg}"),
            Self::UnsupportedToken(token) => write!(f, "unsupported token: {token}"),
        }
    }
}

impl Error for BatterOutParseError {}

/// Parses a batter-out command.
///
/// Supported examples:
/// - `7 63`
/// - `7 6-3`
/// - `7 862`
/// - `7 8-6-2`
/// - `7 F8`
/// - `7 FF3`
/// - `7 L6`
/// - `7 IF4`
pub fn parse_batter_out_command(
    input: &str,
) -> Result<ParsedBatterOutCommand, BatterOutParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(BatterOutParseError::EmptyInput);
    }

    let mut parts = trimmed.split_whitespace();
    let lineup_slot_raw = parts
        .next()
        .ok_or_else(|| BatterOutParseError::InvalidFormat("missing lineup slot".to_string()))?;
    let token = parts
        .next()
        .ok_or_else(|| BatterOutParseError::InvalidFormat("missing play token".to_string()))?;

    if parts.next().is_some() {
        return Err(BatterOutParseError::InvalidFormat(
            "too many whitespace-separated tokens".to_string(),
        ));
    }

    let lineup_slot = parse_lineup_slot(lineup_slot_raw)?;
    let out_type = parse_batter_out_token(token)?;

    Ok(ParsedBatterOutCommand {
        lineup_slot,
        out_type,
    })
}

/// Parses only the play token of a batter-out command.
pub fn parse_batter_out_token(token: &str) -> Result<BatterOutType, BatterOutParseError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(BatterOutParseError::UnsupportedToken(
            "empty token".to_string(),
        ));
    }

    let normalized = token.to_ascii_uppercase();

    // Legacy compatibility: check the longer prefix first.
    if let Some(rest) = normalized.strip_prefix("IFF") {
        let fielder = parse_single_fielder(rest)?;
        return Ok(BatterOutType::InfieldFly { fielder });
    }

    // Documented syntax
    if let Some(rest) = normalized.strip_prefix("IF") {
        let fielder = parse_single_fielder(rest)?;
        return Ok(BatterOutType::InfieldFly { fielder });
    }

    if let Some(rest) = normalized.strip_prefix("FF") {
        let fielder = parse_single_fielder(rest)?;
        return Ok(BatterOutType::FlyOut {
            fielder,
            in_foul_territory: true,
        });
    }

    if let Some(rest) = normalized.strip_prefix('F') {
        let fielder = parse_single_fielder(rest)?;
        return Ok(BatterOutType::FlyOut {
            fielder,
            in_foul_territory: false,
        });
    }

    if let Some(rest) = normalized.strip_prefix('L') {
        let fielder = parse_single_fielder(rest)?;
        return Ok(BatterOutType::LineOut { fielder });
    }

    if normalized.chars().all(|ch| ch.is_ascii_digit()) || normalized.contains('-') {
        let sequence = parse_fielding_sequence(&normalized)?;
        return Ok(BatterOutType::GroundOut { sequence });
    }

    Err(BatterOutParseError::UnsupportedToken(token.to_string()))
}

/// Parses a lineup slot.
///
/// Valid range: `1..=9`.
fn parse_lineup_slot(raw: &str) -> Result<u8, BatterOutParseError> {
    let value = raw.parse::<u8>().map_err(|_| {
        BatterOutParseError::InvalidLineupSlot(format!("'{raw}' is not a valid number"))
    })?;

    if !(1..=9).contains(&value) {
        return Err(BatterOutParseError::InvalidLineupSlot(format!(
            "'{raw}' must be in the range 1..=9"
        )));
    }

    Ok(value)
}

/// Parses a single fielder number.
///
/// Valid range: `1..=9`.
fn parse_single_fielder(raw: &str) -> Result<u8, BatterOutParseError> {
    if raw.is_empty() {
        return Err(BatterOutParseError::InvalidFielder(
            "missing fielder number".to_string(),
        ));
    }

    let value = raw.parse::<u8>().map_err(|_| {
        BatterOutParseError::InvalidFielder(format!("'{raw}' is not a valid fielder"))
    })?;

    if !(1..=9).contains(&value) {
        return Err(BatterOutParseError::InvalidFielder(format!(
            "'{raw}' must be in the range 1..=9"
        )));
    }

    Ok(value)
}

/// Parses a defensive sequence.
///
/// Supported formats:
/// - compact: `63`, `862`, `643`
/// - hyphenated: `6-3`, `8-6-2`, `6-4-3`
pub fn parse_fielding_sequence(token: &str) -> Result<FieldingSequence, BatterOutParseError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(BatterOutParseError::InvalidFieldingSequence(
            "empty fielding sequence".to_string(),
        ));
    }

    let fielders = if token.contains('-') {
        parse_hyphenated_fielding_sequence(token)?
    } else {
        parse_compact_fielding_sequence(token)?
    };

    FieldingSequence::new(fielders)
}

/// Parses a compact fielding sequence where every digit is a fielder.
///
/// Example:
/// - `63` => [6, 3]
/// - `862` => [8, 6, 2]
fn parse_compact_fielding_sequence(token: &str) -> Result<Vec<u8>, BatterOutParseError> {
    let mut fielders = Vec::with_capacity(token.len());

    for ch in token.chars() {
        let value = ch.to_digit(10).ok_or_else(|| {
            BatterOutParseError::InvalidFieldingSequence(format!(
                "invalid character '{ch}' in compact sequence"
            ))
        })?;

        let value = u8::try_from(value).map_err(|_| {
            BatterOutParseError::InvalidFieldingSequence(format!(
                "invalid numeric value '{ch}' in compact sequence"
            ))
        })?;

        if !(1..=9).contains(&value) {
            return Err(BatterOutParseError::InvalidFielder(format!(
                "'{value}' must be in the range 1..=9"
            )));
        }

        fielders.push(value);
    }

    Ok(fielders)
}

/// Parses a hyphenated fielding sequence.
///
/// Example:
/// - `6-3` => [6, 3]
/// - `8-6-2` => [8, 6, 2]
fn parse_hyphenated_fielding_sequence(token: &str) -> Result<Vec<u8>, BatterOutParseError> {
    let mut fielders = Vec::new();

    for part in token.split('-') {
        let part = part.trim();

        if part.is_empty() {
            return Err(BatterOutParseError::InvalidFieldingSequence(format!(
                "invalid hyphenated sequence '{token}'"
            )));
        }

        let value = part.parse::<u8>().map_err(|_| {
            BatterOutParseError::InvalidFieldingSequence(format!(
                "invalid fielder '{part}' in sequence '{token}'"
            ))
        })?;

        if !(1..=9).contains(&value) {
            return Err(BatterOutParseError::InvalidFielder(format!(
                "'{value}' must be in the range 1..=9"
            )));
        }

        fielders.push(value);
    }

    Ok(fielders)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ground_out_compact_two_fielders() {
        let cmd = parse_batter_out_command("7 63").unwrap();
        assert_eq!(cmd.lineup_slot, 7);

        match cmd.out_type {
            BatterOutType::GroundOut { sequence } => {
                assert_eq!(sequence.fielders(), &[6, 3]);
                assert_eq!(sequence.assists(), &[6]);
                assert_eq!(sequence.putout(), 3);
                assert_eq!(sequence.as_hyphenated_string(), "6-3");
            }
            other => panic!("expected GroundOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_ground_out_hyphenated_two_fielders() {
        let cmd = parse_batter_out_command("7 6-3").unwrap();

        match cmd.out_type {
            BatterOutType::GroundOut { sequence } => {
                assert_eq!(sequence.fielders(), &[6, 3]);
            }
            other => panic!("expected GroundOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_ground_out_compact_three_fielders() {
        let cmd = parse_batter_out_command("4 862").unwrap();

        match cmd.out_type {
            BatterOutType::GroundOut { sequence } => {
                assert_eq!(sequence.fielders(), &[8, 6, 2]);
                assert_eq!(sequence.assists(), &[8, 6]);
                assert_eq!(sequence.putout(), 2);
            }
            other => panic!("expected GroundOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_ground_out_hyphenated_three_fielders() {
        let cmd = parse_batter_out_command("4 8-6-2").unwrap();

        match cmd.out_type {
            BatterOutType::GroundOut { sequence } => {
                assert_eq!(sequence.fielders(), &[8, 6, 2]);
            }
            other => panic!("expected GroundOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_fly_out_fair() {
        let cmd = parse_batter_out_command("3 F8").unwrap();

        match cmd.out_type {
            BatterOutType::FlyOut {
                fielder,
                in_foul_territory,
            } => {
                assert_eq!(fielder, 8);
                assert!(!in_foul_territory);
            }
            other => panic!("expected FlyOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_fly_out_foul() {
        let cmd = parse_batter_out_command("3 FF3").unwrap();

        match cmd.out_type {
            BatterOutType::FlyOut {
                fielder,
                in_foul_territory,
            } => {
                assert_eq!(fielder, 3);
                assert!(in_foul_territory);
            }
            other => panic!("expected FlyOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_line_out() {
        let cmd = parse_batter_out_command("5 L6").unwrap();

        match cmd.out_type {
            BatterOutType::LineOut { fielder } => {
                assert_eq!(fielder, 6);
            }
            other => panic!("expected LineOut, found {other:?}"),
        }
    }

    #[test]
    fn reject_invalid_lineup_slot_zero() {
        let err = parse_batter_out_command("0 F8").unwrap_err();
        assert!(matches!(err, BatterOutParseError::InvalidLineupSlot(_)));
    }

    #[test]
    fn reject_invalid_fielder_ten() {
        let err = parse_batter_out_command("7 F10").unwrap_err();
        assert!(matches!(err, BatterOutParseError::InvalidFielder(_)));
    }

    #[test]
    fn reject_invalid_compact_sequence_with_zero() {
        let err = parse_batter_out_command("7 60").unwrap_err();
        assert!(matches!(err, BatterOutParseError::InvalidFielder(_)));
    }

    #[test]
    fn reject_invalid_hyphenated_sequence_double_hyphen() {
        let err = parse_batter_out_command("7 6--3").unwrap_err();
        assert!(matches!(
            err,
            BatterOutParseError::InvalidFieldingSequence(_)
        ));
    }

    #[test]
    fn reject_missing_token() {
        let err = parse_batter_out_command("7").unwrap_err();
        assert!(matches!(err, BatterOutParseError::InvalidFormat(_)));
    }

    #[test]
    fn reject_unknown_token() {
        let err = parse_batter_out_command("7 X9").unwrap_err();
        assert!(matches!(err, BatterOutParseError::UnsupportedToken(_)));
    }

    #[test]
    fn parse_fly_out_lowercase() {
        let cmd = parse_batter_out_command("9 f9").unwrap();

        assert_eq!(cmd.lineup_slot, 9);
        match cmd.out_type {
            BatterOutType::FlyOut {
                fielder,
                in_foul_territory,
            } => {
                assert_eq!(fielder, 9);
                assert!(!in_foul_territory);
            }
            other => panic!("expected FlyOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_foul_fly_out_lowercase() {
        let cmd = parse_batter_out_command("9 ff3").unwrap();

        match cmd.out_type {
            BatterOutType::FlyOut {
                fielder,
                in_foul_territory,
            } => {
                assert_eq!(fielder, 3);
                assert!(in_foul_territory);
            }
            other => panic!("expected FlyOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_line_out_lowercase() {
        let cmd = parse_batter_out_command("9 l6").unwrap();

        match cmd.out_type {
            BatterOutType::LineOut { fielder } => {
                assert_eq!(fielder, 6);
            }
            other => panic!("expected LineOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_infield_fly_if() {
        let cmd = parse_batter_out_command("7 IF4").unwrap();
        match cmd.out_type {
            BatterOutType::InfieldFly { fielder } => assert_eq!(fielder, 4),
            _ => panic!("expected InfieldFly"),
        }
    }

    #[test]
    fn parse_infield_fly_if_lowercase() {
        let cmd = parse_batter_out_command("7 if4").unwrap();
        match cmd.out_type {
            BatterOutType::InfieldFly { fielder } => assert_eq!(fielder, 4),
            _ => panic!("expected InfieldFly"),
        }
    }

    #[test]
    fn parse_infield_fly_legacy_iff() {
        let cmd = parse_batter_out_command("7 IFF4").unwrap();
        match cmd.out_type {
            BatterOutType::InfieldFly { fielder } => assert_eq!(fielder, 4),
            _ => panic!("expected InfieldFly"),
        }
    }
}
