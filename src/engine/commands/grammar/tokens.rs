//! Lexical token recognisers for the command grammar.
//!
//! Regexes are compiled once via `std::sync::LazyLock` and re-used across
//! calls. Each recogniser answers a **purely lexical** question: it never
//! consults the game state, only the shape of the input text.
//!
//! # Vocabulary
//!
//! | Kind               | Pattern                  | Examples                   |
//! |--------------------|--------------------------|----------------------------|
//! | Subject            | `^[1-9]$`                | `5`                        |
//! | Parameter-less verb| exact lowercased text    | see [`CommandKind`]        |
//! | FC verb            | `^o[1-9]$`               | `o6`                       |
//! | Fly verb           | `^ff?[1-9]$`             | `f8`, `ff3`                |
//! | Line verb          | `^l[1-9]$`               | `l6`                       |
//! | Infield-fly        | `^iff?[1-9]$`            | `if4`, `iff4`              |
//! | Fielding seq       | `^[1-9]{2,}$` or dashed  | `63`, `862`, `6-3`, `8-6-2`|
//! | Unassisted         | `^[1-9]$` (single digit) | `5` (same shape as subject)|
//! | Zone               | enumerated               | `lf`, `rc`, `gll`          |
//! | Base               | enumerated               | `1b`, `sc`, `home`         |
//!
//! Parameter-less verbs cover every command whose token is a fixed
//! keyword with no numeric payload (hit verbs `h`/`2h`/`3h`/`hr`, pitches
//! `b`/`k`/`s`/`f`/`fl`, steal `st`, engine control `exit`/`playball`,
//! status `regular`/`post`/…). They all classify into
//! [`TokenKind::Verb`] parameterised by the matching [`CommandKind`]
//! variant. The full list lives in [`CommandKind`] itself — see
//! `crate::engine::commands::kind`.
//!
//! Ambiguity note: `^[1-9]$` matches both *subject* and *unassisted-out
//! verb*. Disambiguation is done at the segment level, not here.

use regex::Regex;
use std::sync::LazyLock;

use crate::engine::commands::kind::CommandKind;
use crate::models::field_zone::FieldZone;
use crate::models::runner::RunnerDest;

// ─── Regex patterns (compiled once) ──────────────────────────────────────────
//
// Only patterns whose recognisers carry a parameter (a fielder number or
// a fielding sequence) need a regex here. Parameter-less verbs (hit,
// pitch, steal, control, status) are matched by exact lowercased text
// in `classify`, which is both faster and more readable than carrying
// a regex for every one-letter keyword.

/// Single digit 1–9 — matches both the subject and a lone unassisted fielder.
pub(super) static RE_DIGIT_1_9: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[1-9]$").unwrap());

/// Fielder's-choice verb: `o<fielder>` with fielder in 1–9.
pub(super) static RE_FC_VERB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?i)o([1-9])$").unwrap());

/// Fly-out verb: `f<fielder>` (fair) or `ff<fielder>` (foul).
pub(super) static RE_FLY_VERB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?i)(ff?)([1-9])$").unwrap());

/// Line-out verb: `l<fielder>`.
pub(super) static RE_LINE_VERB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?i)l([1-9])$").unwrap());

/// Infield-fly verb: `if<fielder>` (legacy `iff<fielder>` also accepted).
pub(super) static RE_IFLY_VERB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?i)iff?([1-9])$").unwrap());

/// Compact fielding sequence (no dashes): two or more consecutive fielder digits (1-9).
pub(super) static RE_FIELDING_SEQ_COMPACT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[1-9]{2,}$").unwrap());

/// Hyphenated fielding sequence: fielder digits (1-9) separated by `-`.
pub(super) static RE_FIELDING_SEQ_DASHED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[1-9](-[1-9])+$").unwrap());

// ─── Lexical kinds ───────────────────────────────────────────────────────────

/// What kind of token a single whitespace-separated chunk of a segment
/// matches. This classification is purely lexical — no state, no semantics.
///
/// `Digit` is intentionally distinct from the unassisted-out verb (which is
/// also a single digit): both have the same shape (`^[1-9]$`), but only the
/// segment-level parser knows which role the token plays at each position.
///
/// All verb tokens that do not carry a numeric parameter in their shape
/// (hit verbs, pitches, steal, control / status keywords) collapse into
/// a single [`TokenKind::Verb`] variant parameterised by
/// [`CommandKind`]. Verbs that do carry a parameter (`f<n>`, `l<n>`,
/// `if<n>`, `o<n>`) keep their own variants because the parameter is
/// part of the token's lexical shape, not a separate field downstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind {
    /// A digit 1–9 (may be a subject or a lone unassisted-fielder verb).
    Digit(u8),
    /// A parameter-less verb keyword: hit, pitch, steal, control, status.
    Verb(CommandKind),
    /// Fielder's choice `o<n>`.
    FcVerb { fielder: u8 },
    /// Fly out `f<n>` or foul-fly `ff<n>`.
    FlyVerb { fielder: u8, foul: bool },
    /// Line out `l<n>`.
    LineVerb { fielder: u8 },
    /// Infield fly `if<n>` / `iff<n>`.
    InfieldFlyVerb { fielder: u8 },
    /// Multi-fielder sequence (`63`, `6-3`, `862`, `8-6-2`).
    FieldingSeq(Vec<u8>),
    /// Zone tag (only valid as object of a hit verb).
    Zone(FieldZone),
    /// Base tag (only valid as object of FC / steal / advance).
    Base(RunnerDest),
    /// Anything that did not match a known pattern. Carries the raw token.
    Unknown(String),
}

// ─── Classifier ──────────────────────────────────────────────────────────────

/// Classify a single token. Case-insensitive.
///
/// Precedence matters when two patterns could match the same text. In
/// practice the patterns above are disjoint **except** `Digit(n)` vs
/// `FcVerb`/`Fly`/`Line`/`InfieldFly` (which all begin with a letter), so no
/// ambiguity exists at this level. The single-digit/unassisted overlap is
/// resolved at parse time based on position.
pub(super) fn classify(raw: &str) -> TokenKind {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return TokenKind::Unknown(String::new());
    }

    // Keyword-style tokens first: control, status, pitch, steal, hit.
    // These are parameter-less verbs that collapse into TokenKind::Verb.
    // `quit` is accepted as an alias for `exit`; canonical_name on
    // CommandKind::Exit returns "exit".
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "exit" | "quit" => return TokenKind::Verb(CommandKind::Exit),
        "playball" => return TokenKind::Verb(CommandKind::PlayBall),

        "regular" => return TokenKind::Verb(CommandKind::Regular),
        "post" => return TokenKind::Verb(CommandKind::Postponed),
        "cancel" => return TokenKind::Verb(CommandKind::Cancelled),
        "susp" => return TokenKind::Verb(CommandKind::Suspended),
        "forf" => return TokenKind::Verb(CommandKind::Forfeited),
        "protest" => return TokenKind::Verb(CommandKind::Protested),

        "b" => return TokenKind::Verb(CommandKind::Ball),
        "k" => return TokenKind::Verb(CommandKind::CalledStrike),
        "s" => return TokenKind::Verb(CommandKind::SwingingStrike),
        "f" => return TokenKind::Verb(CommandKind::Foul),
        "fl" => return TokenKind::Verb(CommandKind::FoulBunt),

        "h" => return TokenKind::Verb(CommandKind::Single),
        "2h" => return TokenKind::Verb(CommandKind::Double),
        "3h" => return TokenKind::Verb(CommandKind::Triple),
        "hr" => return TokenKind::Verb(CommandKind::HomeRun),

        "st" => return TokenKind::Verb(CommandKind::Steal),

        _ => {}
    }

    // Single digit: subject or lone unassisted fielder (caller decides).
    if RE_DIGIT_1_9.is_match(trimmed) {
        let n = trimmed.parse::<u8>().unwrap();
        return TokenKind::Digit(n);
    }

    // Order matters below: `ff3` must match infield/fly patterns before the
    // lone-letter fallbacks. IFly is tested before FlyVerb to catch `if4` /
    // `iff4` before the letter-f rule.
    if let Some(caps) = RE_IFLY_VERB.captures(trimmed) {
        let fielder = caps[1].parse::<u8>().unwrap();
        return TokenKind::InfieldFlyVerb { fielder };
    }

    if let Some(caps) = RE_FC_VERB.captures(trimmed) {
        let fielder = caps[1].parse::<u8>().unwrap();
        return TokenKind::FcVerb { fielder };
    }

    if let Some(caps) = RE_FLY_VERB.captures(trimmed) {
        let prefix = &caps[1];
        let fielder = caps[2].parse::<u8>().unwrap();
        let foul = prefix.len() == 2; // "ff"
        return TokenKind::FlyVerb { fielder, foul };
    }

    if let Some(caps) = RE_LINE_VERB.captures(trimmed) {
        let fielder = caps[1].parse::<u8>().unwrap();
        return TokenKind::LineVerb { fielder };
    }

    // Fielding sequence: dashed first (wouldn't match compact anyway).
    if RE_FIELDING_SEQ_DASHED.is_match(trimmed) {
        let fielders: Vec<u8> = trimmed
            .split('-')
            .map(|s| s.parse::<u8>().unwrap())
            .collect();
        return TokenKind::FieldingSeq(fielders);
    }
    if RE_FIELDING_SEQ_COMPACT.is_match(trimmed) {
        let fielders: Vec<u8> = trimmed
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();
        return TokenKind::FieldingSeq(fielders);
    }

    // Zone / base tags — these match by exact keyword.
    if let Some(zone) = FieldZone::parse(trimmed) {
        return TokenKind::Zone(zone);
    }
    if let Some(base) = RunnerDest::parse(trimmed) {
        return TokenKind::Base(base);
    }

    TokenKind::Unknown(trimmed.to_string())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_is_classified() {
        assert_eq!(classify("5"), TokenKind::Digit(5));
        assert_eq!(classify("1"), TokenKind::Digit(1));
        assert_eq!(classify("9"), TokenKind::Digit(9));
    }

    #[test]
    fn digit_out_of_range_is_unknown() {
        assert!(matches!(classify("0"), TokenKind::Unknown(_)));
        assert!(!matches!(classify("10"), TokenKind::Digit(_)));
    }

    #[test]
    fn hit_verbs_are_case_insensitive() {
        assert_eq!(classify("h"), TokenKind::Verb(CommandKind::Single));
        assert_eq!(classify("H"), TokenKind::Verb(CommandKind::Single));
        assert_eq!(classify("2h"), TokenKind::Verb(CommandKind::Double));
        assert_eq!(classify("3H"), TokenKind::Verb(CommandKind::Triple));
        assert_eq!(classify("HR"), TokenKind::Verb(CommandKind::HomeRun));
    }

    #[test]
    fn pitch_verbs_all_recognised() {
        assert_eq!(classify("b"), TokenKind::Verb(CommandKind::Ball));
        assert_eq!(classify("k"), TokenKind::Verb(CommandKind::CalledStrike));
        assert_eq!(classify("s"), TokenKind::Verb(CommandKind::SwingingStrike));
        assert_eq!(classify("f"), TokenKind::Verb(CommandKind::Foul));
        assert_eq!(classify("fl"), TokenKind::Verb(CommandKind::FoulBunt));
    }

    #[test]
    fn fc_verb_extracts_fielder() {
        assert_eq!(classify("o6"), TokenKind::FcVerb { fielder: 6 });
        assert_eq!(classify("O5"), TokenKind::FcVerb { fielder: 5 });
    }

    #[test]
    fn fly_vs_foul_fly() {
        assert_eq!(
            classify("f8"),
            TokenKind::FlyVerb {
                fielder: 8,
                foul: false,
            }
        );
        assert_eq!(
            classify("ff3"),
            TokenKind::FlyVerb {
                fielder: 3,
                foul: true,
            }
        );
    }

    #[test]
    fn line_verb() {
        assert_eq!(classify("l6"), TokenKind::LineVerb { fielder: 6 });
        assert_eq!(classify("L9"), TokenKind::LineVerb { fielder: 9 });
    }

    #[test]
    fn infield_fly_verb() {
        assert_eq!(classify("if4"), TokenKind::InfieldFlyVerb { fielder: 4 });
        assert_eq!(classify("iff4"), TokenKind::InfieldFlyVerb { fielder: 4 });
    }

    #[test]
    fn fielding_sequence_compact_and_dashed() {
        assert_eq!(classify("63"), TokenKind::FieldingSeq(vec![6, 3]));
        assert_eq!(classify("6-3"), TokenKind::FieldingSeq(vec![6, 3]));
        assert_eq!(classify("862"), TokenKind::FieldingSeq(vec![8, 6, 2]));
        assert_eq!(classify("8-6-2"), TokenKind::FieldingSeq(vec![8, 6, 2]));
    }

    #[test]
    fn fielding_sequence_with_zero_is_unknown() {
        // Fielder 0 is illegal — sequences containing it must be rejected at
        // lex time so that the segment-specific parse error fires instead of a
        // generic build-command failure (issue #60).
        assert!(matches!(classify("60"), TokenKind::Unknown(_)));
        assert!(matches!(classify("06"), TokenKind::Unknown(_)));
        assert!(matches!(classify("6-0"), TokenKind::Unknown(_)));
        assert!(matches!(classify("0-6"), TokenKind::Unknown(_)));
        assert!(matches!(classify("630"), TokenKind::Unknown(_)));
    }

    #[test]
    fn zone_and_base_disambiguate() {
        assert!(matches!(classify("lf"), TokenKind::Zone(FieldZone::LF)));
        assert!(matches!(
            classify("2b"),
            TokenKind::Base(RunnerDest::Second)
        ));
        assert!(matches!(classify("sc"), TokenKind::Base(RunnerDest::Score)));
        assert!(matches!(
            classify("home"),
            TokenKind::Base(RunnerDest::Score)
        ));
    }

    #[test]
    fn keywords() {
        assert_eq!(classify("exit"), TokenKind::Verb(CommandKind::Exit));
        assert_eq!(classify("quit"), TokenKind::Verb(CommandKind::Exit));
        assert_eq!(classify("playball"), TokenKind::Verb(CommandKind::PlayBall));
        assert_eq!(classify("regular"), TokenKind::Verb(CommandKind::Regular));
        assert_eq!(classify("post"), TokenKind::Verb(CommandKind::Postponed));
        assert_eq!(classify("cancel"), TokenKind::Verb(CommandKind::Cancelled));
        assert_eq!(classify("susp"), TokenKind::Verb(CommandKind::Suspended));
        assert_eq!(classify("forf"), TokenKind::Verb(CommandKind::Forfeited));
        assert_eq!(classify("protest"), TokenKind::Verb(CommandKind::Protested));
        assert_eq!(classify("st"), TokenKind::Verb(CommandKind::Steal));
    }

    #[test]
    fn line_verb_not_confused_with_zone() {
        // `l6` is a line out, not a zone. `ll` is a zone, not a line verb.
        assert_eq!(classify("l6"), TokenKind::LineVerb { fielder: 6 });
        assert!(matches!(classify("ll"), TokenKind::Zone(FieldZone::LL)));
        assert!(matches!(classify("lf"), TokenKind::Zone(FieldZone::LF)));
    }

    #[test]
    fn gibberish_is_unknown() {
        assert!(matches!(classify("xyz"), TokenKind::Unknown(_)));
        assert!(matches!(classify("h99"), TokenKind::Unknown(_)));
    }

    #[test]
    fn single_digit_overlap_with_unassisted() {
        // `5` is classified as Digit; the parser decides subject vs unassisted
        // based on position within the segment.
        assert_eq!(classify("5"), TokenKind::Digit(5));
    }
}
