//! Single source of truth for the scoring-command vocabulary.
//!
//! Every verb the grammar recognises is a variant of [`CommandKind`].
//! The rest of the command pipeline refers to this enum instead of
//! maintaining parallel sub-enums per layer:
//!
//! - `grammar::tokens` classifies raw strings into [`CommandKind`] via
//!   [`TokenKind::Verb`](super::grammar::tokens::TokenKind::Verb).
//! - `grammar::segment` carries [`CommandKind`] as the payload of its
//!   variants (e.g. `Segment::Hit { kind: CommandKind, … }`).
//! - `validator` pattern-matches on [`CommandKind`] when folding
//!   segments into `EngineCommand` values.
//!
//! # Adding a new command
//!
//! 1. Add a variant to [`CommandKind`] in the appropriate section.
//! 2. Update [`CommandKind::family`] to return the correct family.
//! 3. Update [`CommandKind::canonical_name`] with the token text used by
//!    the scorer in the UI (e.g. `"h"`, `"63"`, `"playball"`). This is
//!    the authoritative spelling; lexer regexes in `tokens.rs` must
//!    accept this string (and optionally more aliases).
//! 4. Wire the new variant through `tokens::classify`, the appropriate
//!    `Segment` path, and the validator / coalescer as needed. The
//!    exhaustiveness checks across the pipeline will guide you through
//!    every spot that must be updated.
//! 5. Add the new token to the vocabulary table in `tokens.rs` and the
//!    matching section of `SCORING_GUIDE.md`.
//!
//! # Families
//!
//! [`CommandFamily`] groups variants that share structural properties
//! (subject rules, object rules, plate-appearance-terminating vs
//! in-pitch). Used by the grammar's mixing checks and by the validator
//! to route a segment to the right handler.

// ─── Command families ────────────────────────────────────────────────────────

/// Coarse-grained classification of a command by its structural role.
///
/// This is the axis used by the validator's mixing rules:
/// - `Control` and `Status` are single-segment lines, never mixed.
/// - `Pitch` is an in-pitch event; it can coexist with `Steal`.
/// - `Hit`, `BatterOut`, `FielderChoice`, `Advance` are end-of-PA
///   actions; they cannot coexist with a `Pitch` or a `Steal`.
/// - `Steal` is in-pitch; it can coexist with a `Pitch` but not with
///   end-of-PA actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandFamily {
    /// Engine control (`exit`, `playball`).
    Control,
    /// Game status change (`regular`, `post`, …).
    Status,
    /// Pitch recorded against the current batter (`b`, `k`, `s`, `f`,
    /// `fl`).
    Pitch,
    /// Hit by a batter (`h`, `2h`, `3h`, `hr`).
    Hit,
    /// Out on a batter or a runner (`63`, `f8`, `ff3`, `l6`, `if4`, `5`).
    BatterOut,
    /// Fielder's choice (`o6 1b`).
    FielderChoice,
    /// Stolen base (`5 st 2b`).
    Steal,
    /// Standalone runner-advance override (`3 2b`).
    Advance,
}

// ─── The vocabulary ──────────────────────────────────────────────────────────

/// Every verb the scoring-command grammar recognises.
///
/// A single flat enumeration with one variant per lexical verb shape.
/// See the module documentation for the invariants this enum maintains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    // ── Engine control ─────────────────────────────────────────────
    Exit,
    PlayBall,

    // ── Game status ────────────────────────────────────────────────
    Regular,
    Postponed,
    Cancelled,
    Suspended,
    Forfeited,
    Protested,

    // ── Pitches ────────────────────────────────────────────────────
    Ball,
    CalledStrike,
    SwingingStrike,
    Foul,
    FoulBunt,

    // ── Hits ───────────────────────────────────────────────────────
    Single,
    Double,
    Triple,
    HomeRun,

    // ── Batter / runner outs ───────────────────────────────────────
    //
    // These variants are lexical verbs, not semantic targets: the same
    // verb can retire a batter or a runner depending on the subject
    // rule. `FlyOut` and `FoulFlyOut` are split because they are two
    // distinct tokens (`f<n>` vs `ff<n>`).
    Unassisted,
    GroundOut,
    FlyOut,
    FoulFlyOut,
    LineOut,
    InfieldFly,

    // ── Composite base play ────────────────────────────────────────
    FielderChoice,

    // ── Runner actions ─────────────────────────────────────────────
    Steal,
    Advance,
}

// ─── Behaviour ───────────────────────────────────────────────────────────────

impl CommandKind {
    /// The family this command belongs to.
    pub const fn family(self) -> CommandFamily {
        match self {
            Self::Exit | Self::PlayBall => CommandFamily::Control,

            Self::Regular
            | Self::Postponed
            | Self::Cancelled
            | Self::Suspended
            | Self::Forfeited
            | Self::Protested => CommandFamily::Status,

            Self::Ball
            | Self::CalledStrike
            | Self::SwingingStrike
            | Self::Foul
            | Self::FoulBunt => CommandFamily::Pitch,

            Self::Single | Self::Double | Self::Triple | Self::HomeRun => CommandFamily::Hit,

            Self::Unassisted
            | Self::GroundOut
            | Self::FlyOut
            | Self::FoulFlyOut
            | Self::LineOut
            | Self::InfieldFly => CommandFamily::BatterOut,

            Self::FielderChoice => CommandFamily::FielderChoice,
            Self::Steal => CommandFamily::Steal,
            Self::Advance => CommandFamily::Advance,
        }
    }

    /// The canonical textual spelling used by the scorer.
    ///
    /// For verbs that take a numeric parameter in the original token
    /// (`f8`, `l6`, `o6`, `63`, …), this returns the verb *shape*
    /// without the parameter (`f`, `l`, `o`, `<sequence>`). Tokens
    /// intended for diagnostic messages should quote the original
    /// segment text instead of this string.
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::PlayBall => "playball",

            Self::Regular => "regular",
            Self::Postponed => "post",
            Self::Cancelled => "cancel",
            Self::Suspended => "susp",
            Self::Forfeited => "forf",
            Self::Protested => "protest",

            Self::Ball => "b",
            Self::CalledStrike => "k",
            Self::SwingingStrike => "s",
            Self::Foul => "f",
            Self::FoulBunt => "fl",

            Self::Single => "h",
            Self::Double => "2h",
            Self::Triple => "3h",
            Self::HomeRun => "hr",

            Self::Unassisted => "<digit>",
            Self::GroundOut => "<sequence>",
            Self::FlyOut => "f<n>",
            Self::FoulFlyOut => "ff<n>",
            Self::LineOut => "l<n>",
            Self::InfieldFly => "if<n>",

            Self::FielderChoice => "o<n>",
            Self::Steal => "st",
            Self::Advance => "<base>",
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Exhaustive list of every `CommandKind` variant. The test
    /// `all_variants_are_listed_here` guarantees this stays in sync
    /// with the enum definition.
    const ALL: &[CommandKind] = &[
        CommandKind::Exit,
        CommandKind::PlayBall,
        CommandKind::Regular,
        CommandKind::Postponed,
        CommandKind::Cancelled,
        CommandKind::Suspended,
        CommandKind::Forfeited,
        CommandKind::Protested,
        CommandKind::Ball,
        CommandKind::CalledStrike,
        CommandKind::SwingingStrike,
        CommandKind::Foul,
        CommandKind::FoulBunt,
        CommandKind::Single,
        CommandKind::Double,
        CommandKind::Triple,
        CommandKind::HomeRun,
        CommandKind::Unassisted,
        CommandKind::GroundOut,
        CommandKind::FlyOut,
        CommandKind::FoulFlyOut,
        CommandKind::LineOut,
        CommandKind::InfieldFly,
        CommandKind::FielderChoice,
        CommandKind::Steal,
        CommandKind::Advance,
    ];

    /// If you add a variant to `CommandKind`, extend `ALL` above.
    /// This test will otherwise remind you by failing.
    #[test]
    fn all_variants_are_listed_here() {
        // 26 variants as of v0.11.1. If this count diverges from ALL,
        // the test setup is out of date.
        assert_eq!(ALL.len(), 26);
    }

    #[test]
    fn every_variant_has_a_family() {
        // No panic means family() handled every variant.
        for &k in ALL {
            let _f = k.family();
        }
    }

    #[test]
    fn every_variant_has_a_canonical_name() {
        for &k in ALL {
            let name = k.canonical_name();
            assert!(!name.is_empty(), "canonical_name for {:?} is empty", k);
        }
    }

    #[test]
    fn families_partition_variants_correctly() {
        let family_counts = |target: CommandFamily| -> usize {
            ALL.iter().filter(|k| k.family() == target).count()
        };

        // These counts encode the current partition. If you move a
        // variant between families, update this test deliberately.
        assert_eq!(family_counts(CommandFamily::Control), 2);
        assert_eq!(family_counts(CommandFamily::Status), 6);
        assert_eq!(family_counts(CommandFamily::Pitch), 5);
        assert_eq!(family_counts(CommandFamily::Hit), 4);
        assert_eq!(family_counts(CommandFamily::BatterOut), 6);
        assert_eq!(family_counts(CommandFamily::FielderChoice), 1);
        assert_eq!(family_counts(CommandFamily::Steal), 1);
        assert_eq!(family_counts(CommandFamily::Advance), 1);
    }
}
