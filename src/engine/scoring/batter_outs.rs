//! Batter-out parsing and defensive-play domain types for scoring plays.
//!
//! This module handles:
//! - batter-only outs such as:
//!   - ground outs (`63`, `6-3`, `862`, `8-6-2`, ...)
//!   - fly outs (`F8`, `FF3`)
//!   - line outs (`L6`)
//!   - infield fly (`IF4`, legacy `IFF4`)
//! - composed defensive plays such as:
//!   - `63`
//!   - `o6`
//!   - `o6 1b`
//!   - `2 64, o6`
//!   - `l6, 2 64, 3 43`
//!   - `3 o5 2b`

use crate::RunnerDest;
use std::error::Error;
use std::fmt;

/// Represents the ordered defensive sequence that produced an out.
///
/// The last fielder is credited with the putout.
/// All previous fielders are credited with assists.
///
/// Examples:
/// - `6-3` => assist: 6, putout: 3
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

/// Represents batter-only out types supported by the legacy batter-out parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatterOutType {
    /// Batter is retired unassisted by a single fielder such as `3` or `5`.
    UnassistedOut { fielder: u8 },

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
            Self::UnassistedOut { .. } => "unassisted_out",
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

/// Identifies who is affected by a defensive play segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefensivePlayTarget {
    Batter,
    Runner(u8),
}

/// Represents the kind of out recorded by the defense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefensiveOutKind {
    UnassistedOut {
        fielder: u8,
    },

    GroundOut {
        sequence: FieldingSequence,
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
}

/// Represents one defensive out recorded on either the batter or a runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefensiveOutRecord {
    pub target: DefensivePlayTarget,
    pub kind: DefensiveOutKind,
}

/// Represents a safe advance by fielder's choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FielderChoiceAdvance {
    pub target: DefensivePlayTarget,
    pub fielder: u8,
    pub reached_base: RunnerDest,
}

/// Represents a full defensive play that may contain multiple outs
/// and optional safe advances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefensivePlayCommand {
    pub outs: Vec<DefensiveOutRecord>,
    pub safe_advances: Vec<FielderChoiceAdvance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DefensivePlaySegment {
    Out(DefensiveOutRecord),
    SafeAdvance(FielderChoiceAdvance),
}

/// Represents a parsed batter-out command.
///
/// Expected legacy input format:
/// `<lineup_slot> <play_token>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBatterOutCommand {
    pub lineup_slot: u8,
    pub out_type: BatterOutType,
}

/// Parsing errors for batter-out commands and defensive-play commands.
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

/// Parses a legacy batter-out command.
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

/// Parses a full defensive play command.
///
/// Supported examples:
/// - `63`
/// - `f9`
/// - `l6`
/// - `if4`
/// - `o6`
/// - `o6 1b`
/// - `2 64, o6`
/// - `l6, 2 64`
/// - `l6, 2 64, 3 43`
/// - `3 o5 2b`
pub fn parse_defensive_play_command(
    input: &str,
) -> Result<DefensivePlayCommand, BatterOutParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(BatterOutParseError::EmptyInput);
    }

    let mut outs = Vec::new();
    let mut safe_advances = Vec::new();

    for raw_segment in trimmed.split(',') {
        let segment = raw_segment.trim();
        if segment.is_empty() {
            return Err(BatterOutParseError::InvalidFormat(
                "empty segment in defensive play".to_string(),
            ));
        }

        let parts: Vec<&str> = segment.split_whitespace().collect();
        match parse_defensive_play_segment(&parts)? {
            DefensivePlaySegment::Out(out) => outs.push(out),
            DefensivePlaySegment::SafeAdvance(fc) => safe_advances.push(fc),
        }
    }

    if outs.is_empty() && safe_advances.is_empty() {
        return Err(BatterOutParseError::InvalidFormat(
            "defensive play contains no actionable segments".to_string(),
        ));
    }

    Ok(DefensivePlayCommand {
        outs,
        safe_advances,
    })
}

pub fn parse_batter_out_token(token: &str) -> Result<BatterOutType, BatterOutParseError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(BatterOutParseError::UnsupportedToken(
            "empty token".to_string(),
        ));
    }

    let normalized = token.to_ascii_uppercase();

    // Legacy compatibility: check longer prefix first.
    if let Some(rest) = normalized.strip_prefix("IFF") {
        let fielder = parse_single_fielder(rest)?;
        return Ok(BatterOutType::InfieldFly { fielder });
    }

    // Documented syntax.
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

    // Single numeric fielder => unassisted out.
    if normalized.len() == 1 && normalized.chars().all(|ch| ch.is_ascii_digit()) {
        let fielder = parse_single_fielder(&normalized)?;
        return Ok(BatterOutType::UnassistedOut { fielder });
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

/// Parses an optional defensive play target from the start of a token list.
///
/// Supported forms:
/// - implicit batter target:
///   - `63`
///   - `F8`
///   - `O6`
/// - explicit runner target:
///   - `2 64`
///   - `3 O5 2B`
fn parse_defensive_play_target_and_rest<'a>(
    parts: &'a [&'a str],
) -> Result<(DefensivePlayTarget, &'a [&'a str]), BatterOutParseError> {
    let Some(first) = parts.first() else {
        return Err(BatterOutParseError::InvalidFormat(
            "empty defensive play segment".to_string(),
        ));
    };

    // Treat the first token as an explicit runner order only when:
    // - there is at least one more token after it
    // - it is a single-digit lineup slot in the range 1..=9
    if parts.len() >= 2
        && first.len() == 1
        && let Ok(order) = first.parse::<u8>()
        && (1..=9).contains(&order)
    {
        return Ok((DefensivePlayTarget::Runner(order), &parts[1..]));
    }

    Ok((DefensivePlayTarget::Batter, parts))
}

/// Parses a fielder's choice safe-advance token.
///
/// Supported forms:
/// - `O6 1B`
/// - `O5 2B`
/// - `1 O6 1B`
/// - `3 O5 2B`
///
/// Base is always required.
fn parse_fielder_choice_advance(
    target: DefensivePlayTarget,
    parts: &[&str],
) -> Result<FielderChoiceAdvance, BatterOutParseError> {
    let Some(first) = parts.first() else {
        return Err(BatterOutParseError::InvalidFormat(
            "missing fielder's choice token".to_string(),
        ));
    };

    let normalized = first.trim().to_ascii_uppercase();

    let Some(rest) = normalized.strip_prefix('O') else {
        return Err(BatterOutParseError::UnsupportedToken(first.to_string()));
    };

    let fielder = parse_single_fielder(rest)?;

    let reached_base = match parts.get(1).map(|s| s.trim().to_ascii_uppercase()) {
        Some(base) => parse_runner_dest_token(&base)?,
        None => {
            return Err(BatterOutParseError::InvalidFormat(
                "fielder's choice requires explicit destination base".to_string(),
            ));
        }
    };

    if parts.len() > 2 {
        return Err(BatterOutParseError::InvalidFormat(
            "too many tokens in fielder's choice segment".to_string(),
        ));
    }

    Ok(FielderChoiceAdvance {
        target,
        fielder,
        reached_base,
    })
}

/// Parses a runner destination token used by defensive-play syntax.
fn parse_runner_dest_token(token: &str) -> Result<RunnerDest, BatterOutParseError> {
    match token {
        "1B" => Ok(RunnerDest::First),
        "2B" => Ok(RunnerDest::Second),
        "3B" => Ok(RunnerDest::Third),
        "SC" | "HOME" => Ok(RunnerDest::Score),
        _ => Err(BatterOutParseError::InvalidFormat(format!(
            "invalid destination base '{token}'"
        ))),
    }
}

/// Parses one defensive out segment.
///
/// Supported forms:
/// - `63`
/// - `6-3`
/// - `F8`
/// - `FF3`
/// - `L6`
/// - `IF4`
/// - `2 64`
/// - `3 43`
fn parse_defensive_out_segment(parts: &[&str]) -> Result<DefensiveOutRecord, BatterOutParseError> {
    let (target, rest) = parse_defensive_play_target_and_rest(parts)?;

    if rest.is_empty() {
        return Err(BatterOutParseError::InvalidFormat(
            "missing defensive out token".to_string(),
        ));
    }

    if rest.len() != 1 {
        return Err(BatterOutParseError::InvalidFormat(
            "too many tokens in defensive out segment".to_string(),
        ));
    }

    let token = rest[0].trim();
    let normalized = token.to_ascii_uppercase();

    if let Some(rest) = normalized.strip_prefix("IFF") {
        let fielder = parse_single_fielder(rest)?;
        return Ok(DefensiveOutRecord {
            target,
            kind: DefensiveOutKind::InfieldFly { fielder },
        });
    }

    if let Some(rest) = normalized.strip_prefix("IF") {
        let fielder = parse_single_fielder(rest)?;
        return Ok(DefensiveOutRecord {
            target,
            kind: DefensiveOutKind::InfieldFly { fielder },
        });
    }

    if let Some(rest) = normalized.strip_prefix("FF") {
        let fielder = parse_single_fielder(rest)?;
        return Ok(DefensiveOutRecord {
            target,
            kind: DefensiveOutKind::FlyOut {
                fielder,
                in_foul_territory: true,
            },
        });
    }

    if let Some(rest) = normalized.strip_prefix('F') {
        let fielder = parse_single_fielder(rest)?;
        return Ok(DefensiveOutRecord {
            target,
            kind: DefensiveOutKind::FlyOut {
                fielder,
                in_foul_territory: false,
            },
        });
    }

    if let Some(rest) = normalized.strip_prefix('L') {
        let fielder = parse_single_fielder(rest)?;
        return Ok(DefensiveOutRecord {
            target,
            kind: DefensiveOutKind::LineOut { fielder },
        });
    }

    // Single numeric fielder => unassisted out.
    if normalized.len() == 1 && normalized.chars().all(|ch| ch.is_ascii_digit()) {
        let fielder = parse_single_fielder(&normalized)?;
        return Ok(DefensiveOutRecord {
            target,
            kind: DefensiveOutKind::UnassistedOut { fielder },
        });
    }

    if normalized.chars().all(|ch| ch.is_ascii_digit()) || normalized.contains('-') {
        let sequence = parse_fielding_sequence(&normalized)?;
        return Ok(DefensiveOutRecord {
            target,
            kind: DefensiveOutKind::GroundOut { sequence },
        });
    }

    Err(BatterOutParseError::UnsupportedToken(token.to_string()))
}

/// Parses one defensive play segment, either an out or a safe advance.
fn parse_defensive_play_segment(
    parts: &[&str],
) -> Result<DefensivePlaySegment, BatterOutParseError> {
    let (target, rest) = parse_defensive_play_target_and_rest(parts)?;

    let Some(first) = rest.first() else {
        return Err(BatterOutParseError::InvalidFormat(
            "missing defensive play token".to_string(),
        ));
    };

    let normalized = first.trim().to_ascii_uppercase();

    if normalized.starts_with('O') {
        return parse_fielder_choice_advance(target, rest).map(DefensivePlaySegment::SafeAdvance);
    }

    parse_defensive_out_segment(parts).map(DefensivePlaySegment::Out)
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

    #[test]
    fn parse_defensive_play_batter_ground_out_implicit() {
        let cmd = parse_defensive_play_command("63").unwrap();
        assert_eq!(cmd.safe_advances.len(), 0);
        assert_eq!(cmd.outs.len(), 1);

        match &cmd.outs[0] {
            DefensiveOutRecord {
                target: DefensivePlayTarget::Batter,
                kind: DefensiveOutKind::GroundOut { sequence },
            } => {
                assert_eq!(sequence.fielders(), &[6, 3]);
            }
            other => panic!("unexpected defensive out record: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_batter_fly_out_implicit() {
        let cmd = parse_defensive_play_command("f9").unwrap();
        assert_eq!(cmd.outs.len(), 1);

        match &cmd.outs[0] {
            DefensiveOutRecord {
                target: DefensivePlayTarget::Batter,
                kind:
                    DefensiveOutKind::FlyOut {
                        fielder,
                        in_foul_territory,
                    },
            } => {
                assert_eq!(*fielder, 9);
                assert!(!*in_foul_territory);
            }
            other => panic!("unexpected defensive out record: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_batter_fielder_choice_explicit_first() {
        let cmd = parse_defensive_play_command("o6 1b").unwrap();
        assert!(cmd.outs.is_empty());
        assert_eq!(cmd.safe_advances.len(), 1);

        match &cmd.safe_advances[0] {
            FielderChoiceAdvance {
                target: DefensivePlayTarget::Batter,
                fielder,
                reached_base,
            } => {
                assert_eq!(*fielder, 6);
                assert_eq!(*reached_base, RunnerDest::First);
            }
            other => panic!("unexpected FC advance: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_runner_out_plus_batter_fc() {
        let cmd = parse_defensive_play_command("2 64, o6 1b").unwrap();
        assert_eq!(cmd.outs.len(), 1);
        assert_eq!(cmd.safe_advances.len(), 1);

        match &cmd.outs[0] {
            DefensiveOutRecord {
                target: DefensivePlayTarget::Runner(2),
                kind: DefensiveOutKind::GroundOut { sequence },
            } => {
                assert_eq!(sequence.fielders(), &[6, 4]);
            }
            other => panic!("unexpected defensive out record: {other:?}"),
        }

        match &cmd.safe_advances[0] {
            FielderChoiceAdvance {
                target: DefensivePlayTarget::Batter,
                fielder,
                reached_base,
            } => {
                assert_eq!(*fielder, 6);
                assert_eq!(*reached_base, RunnerDest::First);
            }
            other => panic!("unexpected FC advance: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_runner_fc_requires_explicit_base() {
        let err = parse_defensive_play_command("3 o5").unwrap_err();
        assert!(matches!(err, BatterOutParseError::InvalidFormat(_)));
    }

    #[test]
    fn parse_defensive_play_runner_fc_with_explicit_base() {
        let cmd = parse_defensive_play_command("3 o5 2b").unwrap();
        assert!(cmd.outs.is_empty());
        assert_eq!(cmd.safe_advances.len(), 1);

        match &cmd.safe_advances[0] {
            FielderChoiceAdvance {
                target: DefensivePlayTarget::Runner(3),
                fielder,
                reached_base,
            } => {
                assert_eq!(*fielder, 5);
                assert_eq!(*reached_base, RunnerDest::Second);
            }
            other => panic!("unexpected FC advance: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_triple_play_style() {
        let cmd = parse_defensive_play_command("l6, 2 64, 3 43").unwrap();
        assert_eq!(cmd.outs.len(), 3);
        assert!(cmd.safe_advances.is_empty());

        match &cmd.outs[0] {
            DefensiveOutRecord {
                target: DefensivePlayTarget::Batter,
                kind: DefensiveOutKind::LineOut { fielder },
            } => assert_eq!(*fielder, 6),
            other => panic!("unexpected first out: {other:?}"),
        }

        match &cmd.outs[1] {
            DefensiveOutRecord {
                target: DefensivePlayTarget::Runner(2),
                kind: DefensiveOutKind::GroundOut { sequence },
            } => assert_eq!(sequence.fielders(), &[6, 4]),
            other => panic!("unexpected second out: {other:?}"),
        }

        match &cmd.outs[2] {
            DefensiveOutRecord {
                target: DefensivePlayTarget::Runner(3),
                kind: DefensiveOutKind::GroundOut { sequence },
            } => assert_eq!(sequence.fielders(), &[4, 3]),
            other => panic!("unexpected third out: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_explicit_runner_fc_with_first_base() {
        let cmd = parse_defensive_play_command("1 o6 1b").unwrap();
        assert!(cmd.outs.is_empty());
        assert_eq!(cmd.safe_advances.len(), 1);

        match &cmd.safe_advances[0] {
            FielderChoiceAdvance {
                target: DefensivePlayTarget::Runner(1),
                fielder,
                reached_base,
            } => {
                assert_eq!(*fielder, 6);
                assert_eq!(*reached_base, RunnerDest::First);
            }
            other => panic!("unexpected FC advance: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_batter_fielder_choice_requires_base() {
        let err = parse_defensive_play_command("o6").unwrap_err();
        assert!(matches!(err, BatterOutParseError::InvalidFormat(_)));
    }

    #[test]
    fn parse_unassisted_out_legacy() {
        let cmd = parse_batter_out_command("8 5").unwrap();

        match cmd.out_type {
            BatterOutType::UnassistedOut { fielder } => assert_eq!(fielder, 5),
            other => panic!("expected UnassistedOut, found {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_unassisted_batter_implicit() {
        let cmd = parse_defensive_play_command("5").unwrap();
        assert_eq!(cmd.outs.len(), 1);

        match &cmd.outs[0] {
            DefensiveOutRecord {
                target: DefensivePlayTarget::Batter,
                kind: DefensiveOutKind::UnassistedOut { fielder },
            } => assert_eq!(*fielder, 5),
            other => panic!("unexpected defensive out record: {other:?}"),
        }
    }

    #[test]
    fn parse_defensive_play_unassisted_multi_segment() {
        let cmd = parse_defensive_play_command("8 5, 9 54, 1 o5 1b").unwrap();
        assert_eq!(cmd.outs.len(), 2);
        assert_eq!(cmd.safe_advances.len(), 1);
    }
}
