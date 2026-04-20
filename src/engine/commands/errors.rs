//! Error types for command parsing and validation.
//!
//! The command pipeline has two stages:
//!
//! 1. **Parsing** (syntactic): `ParseError` — the text of a single segment is
//!    lexically invalid.
//! 2. **Validation** (semantic): `ValidationError` — a segment is well-formed
//!    but does not make sense in the current `GameState` (wrong batter slot,
//!    runner not on base, infield-fly preconditions not met, …).
//!
//! `CommandError` wraps a single error together with the 1-based index of the
//! segment where it was produced. The top-level facade returns
//! `Vec<CommandError>` so the user sees **every** problem in a single pass,
//! not just the first one.

use std::error::Error;
use std::fmt;

/// A syntactic problem with a single segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Segment text was empty (e.g. trailing comma).
    EmptySegment,

    /// The segment requires an explicit batting-order subject (1–9) but none
    /// was provided.
    MissingSubject { verb: String },

    /// A subject was provided on a verb that does not accept one
    /// (pitch, control, status).
    SubjectNotAllowed { verb: String },

    /// The batting-order subject was not a digit in the 1–9 range.
    InvalidSubject { token: String },

    /// The verb token is not a recognised keyword nor a valid fielding /
    /// hit / out / FC pattern.
    UnknownVerb { token: String },

    /// The verb requires an object (zone for hits, base for FC / steal /
    /// advance) but none was given.
    MissingObject { verb: String, expected: &'static str },

    /// A zone token (e.g. `lf`) was not one of the documented codes.
    InvalidZone { token: String },

    /// A base token (e.g. `2b`) was not one of the documented codes.
    InvalidBase { token: String },

    /// Too many whitespace-separated tokens in the segment for the verb type.
    ExtraTokens { verb: String, extra: String },

    /// A fielding-sequence token (`6-3`, `862`, …) is malformed.
    InvalidFieldingSequence { token: String, reason: String },

    /// A fielder number is outside the 1–9 range.
    InvalidFielder { token: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySegment => write!(f, "empty segment"),
            Self::MissingSubject { verb } => {
                write!(f, "verb '{verb}' requires a batting-order subject (1–9)")
            }
            Self::SubjectNotAllowed { verb } => {
                write!(f, "verb '{verb}' does not accept a batting-order subject")
            }
            Self::InvalidSubject { token } => {
                write!(f, "invalid batting-order subject '{token}' (expected 1–9)")
            }
            Self::UnknownVerb { token } => write!(f, "unknown verb '{token}'"),
            Self::MissingObject { verb, expected } => {
                write!(f, "verb '{verb}' requires a {expected}")
            }
            Self::InvalidZone { token } => write!(f, "invalid field zone '{token}'"),
            Self::InvalidBase { token } => write!(f, "invalid base '{token}'"),
            Self::ExtraTokens { verb, extra } => {
                write!(f, "verb '{verb}' does not accept extra tokens: '{extra}'")
            }
            Self::InvalidFieldingSequence { token, reason } => {
                write!(f, "invalid fielding sequence '{token}': {reason}")
            }
            Self::InvalidFielder { token } => {
                write!(f, "invalid fielder '{token}' (expected 1–9)")
            }
        }
    }
}

impl Error for ParseError {}

/// A semantic problem: the segment is syntactically valid but does not fit
/// the current game state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Batter-only verb (hit, batter out, batter FC) carries an explicit
    /// subject that does not match the current batter slot.
    BatterSlotMismatch { given: u8, current: Option<u8> },

    /// A runner-only verb (steal, runner-advance override, runner-out) was
    /// issued for a batting slot that is not currently on base.
    RunnerNotOnBase { order: u8 },

    /// A runner-advance override (`<n> <base>`) appeared in a line with no
    /// triggering play (no hit, no FC).
    AdvanceWithoutTrigger { order: u8 },

    /// The same batting slot appears as both the batter on a hit and as a
    /// runner override in the same line.
    DuplicateSubject { order: u8 },

    /// Infield-fly rule pre-conditions are not met: requires < 2 outs and
    /// runners on both 1B and 2B.
    InfieldFlyConditionsNotMet,

    /// More than 3 outs would be recorded on a single play.
    TooManyOuts { count: usize },

    /// Control/status/pitch command appeared mixed with action segments.
    /// Control and pitch segments cannot be combined with actions on the
    /// same line.
    ControlMixedWithActions { verb: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BatterSlotMismatch { given, current } => match current {
                Some(c) => write!(
                    f,
                    "batter slot #{given} does not match current batter #{c}"
                ),
                None => write!(f, "batter slot #{given} provided but no batter is active"),
            },
            Self::RunnerNotOnBase { order } => {
                write!(f, "runner #{order} is not on base")
            }
            Self::AdvanceWithoutTrigger { order } => write!(
                f,
                "runner advance #{order} has no triggering play (hit or FC) in this line"
            ),
            Self::DuplicateSubject { order } => write!(
                f,
                "batting slot #{order} appears both as batter and as a runner override"
            ),
            Self::InfieldFlyConditionsNotMet => write!(
                f,
                "infield-fly rule requires < 2 outs and runners on 1B and 2B"
            ),
            Self::TooManyOuts { count } => {
                write!(f, "play would record {count} outs (maximum is 3)")
            }
            Self::ControlMixedWithActions { verb } => write!(
                f,
                "'{verb}' is a control command and cannot be combined with action segments"
            ),
        }
    }
}

impl Error for ValidationError {}

/// A single error attached to the 1-based index of the segment that produced
/// it. Segment `0` is reserved for line-level errors not tied to a single
/// segment (e.g. an entirely empty line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandError {
    pub segment_index: usize,
    pub segment_text: String,
    pub kind: CommandErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandErrorKind {
    Parse(ParseError),
    Validation(ValidationError),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            CommandErrorKind::Parse(e) => write!(
                f,
                "error at segment {}: '{}': {e}",
                self.segment_index, self.segment_text
            ),
            CommandErrorKind::Validation(e) => write!(
                f,
                "error at segment {}: '{}': {e}",
                self.segment_index, self.segment_text
            ),
        }
    }
}

impl Error for CommandError {}

impl From<(usize, &str, ParseError)> for CommandError {
    fn from((idx, text, e): (usize, &str, ParseError)) -> Self {
        Self {
            segment_index: idx,
            segment_text: text.to_string(),
            kind: CommandErrorKind::Parse(e),
        }
    }
}

impl From<(usize, &str, ValidationError)> for CommandError {
    fn from((idx, text, e): (usize, &str, ValidationError)) -> Self {
        Self {
            segment_index: idx,
            segment_text: text.to_string(),
            kind: CommandErrorKind::Validation(e),
        }
    }
}
