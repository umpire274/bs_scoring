//! Lexical token recognisers for the command grammar.
//!
//! Regexes are compiled once via `std::sync::LazyLock` and re-used across
//! calls. Each recogniser answers a **purely lexical** question: it never
//! consults the game state, only the shape of the input text.
//!
//! # Vocabulary
//!
//! | Kind            | Pattern                  | Examples                   |
//! |-----------------|--------------------------|----------------------------|
//! | Subject         | `^[1-9]$`                | `5`                        |
//! | Hit verb        | `^(h|2h|3h|hr)$`         | `h`, `2h`, `hr`            |
//! | Pitch verb      | `^(b|k|s|f|fl)$`         | `b`, `k`, `fl`             |
//! | Steal verb      | `^st$`                   | `st`                       |
//! | FC verb         | `^o[1-9]$`               | `o6`                       |
//! | Fly verb        | `^ff?[1-9]$`             | `f8`, `ff3`                |
//! | Line verb       | `^l[1-9]$`               | `l6`                       |
//! | Infield-fly     | `^iff?[1-9]$`            | `if4`, `iff4`              |
//! | Fielding seq    | `^\d{2,}$` or dashed     | `63`, `862`, `6-3`, `8-6-2`|
//! | Unassisted      | `^[1-9]$` (single digit) | `5` (same shape as subject)|
//! | Zone            | enumerated               | `lf`, `rc`, `gll`          |
//! | Base            | enumerated               | `1b`, `sc`, `home`         |
//! | Control / Status| enumerated keywords      | `exit`, `playball`, …      |
//!
//! Ambiguity note: `^[1-9]$` matches both *subject* and *unassisted-out
//! verb*. Disambiguation is done at the segment level, not here.

use regex::Regex;
use std::sync::LazyLock;

use crate::models::field_zone::FieldZone;
use crate::models::runner::RunnerDest;

// ─── Regex patterns (compiled once) ──────────────────────────────────────────

/// Single digit 1–9 — matches both the subject and a lone unassisted fielder.
pub(super) static RE_DIGIT_1_9: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[1-9]$").unwrap());

/// Hit verbs: `h`, `2h`, `3h`, `hr`.
pub(super) static RE_HIT_VERB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?i)(h|2h|3h|hr)$").unwrap());

/// Pitch verbs: `b`, `k`, `s`, `f`, `fl`.
pub(super) static RE_PITCH_VERB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?i)(b|k|s|f|fl)$").unwrap());

/// Steal verb keyword.
pub(super) static RE_STEAL_VERB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?i)st$").unwrap());

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

/// Compact fielding sequence (no dashes): two or more consecutive digits.
pub(super) static RE_FIELDING_SEQ_COMPACT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{2,}$").unwrap());

/// Hyphenated fielding sequence: digits separated by `-`.
pub(super) static RE_FIELDING_SEQ_DASHED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d(-\d)+$").unwrap());

// ─── Lexical kinds ───────────────────────────────────────────────────────────

/// What kind of token a single whitespace-separated chunk of a segment
/// matches. This classification is purely lexical — no state, no semantics.
///
/// `Digit` is intentionally distinct from `Unassisted`: both have the same
/// shape (`^[1-9]$`), but only the segment-level parser knows which role the
/// token plays at each position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind {
    /// A digit 1–9 (may be a subject or a lone unassisted-fielder verb).
    Digit(u8),
    /// Hit verb normalised to lowercase.
    HitVerb(HitVerbKind),
    /// Pitch verb.
    PitchVerb(PitchVerbKind),
    /// Steal keyword `st`.
    StealVerb,
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
    /// Control / status / pitch-less keywords.
    Keyword(KeywordKind),
    /// Anything that did not match a known pattern. Carries the raw token.
    Unknown(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HitVerbKind {
    Single,
    Double,
    Triple,
    HomeRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PitchVerbKind {
    Ball,
    CalledStrike,
    SwingingStrike,
    Foul,
    FoulBunt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeywordKind {
    // Engine control
    Exit,
    PlayBall,
    // Game status
    Regular,
    Postponed,
    Cancelled,
    Suspended,
    Forfeited,
    Protested,
}

impl KeywordKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::PlayBall => "playball",
            Self::Regular => "regular",
            Self::Postponed => "post",
            Self::Cancelled => "cancel",
            Self::Suspended => "susp",
            Self::Forfeited => "forf",
            Self::Protested => "protest",
        }
    }
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

    // Keywords first — these are short strings that would otherwise collide
    // with nothing, but matching them early keeps the branching readable.
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "exit" | "quit" => return TokenKind::Keyword(KeywordKind::Exit),
        "playball" => return TokenKind::Keyword(KeywordKind::PlayBall),
        "regular" => return TokenKind::Keyword(KeywordKind::Regular),
        "post" => return TokenKind::Keyword(KeywordKind::Postponed),
        "cancel" => return TokenKind::Keyword(KeywordKind::Cancelled),
        "susp" => return TokenKind::Keyword(KeywordKind::Suspended),
        "forf" => return TokenKind::Keyword(KeywordKind::Forfeited),
        "protest" => return TokenKind::Keyword(KeywordKind::Protested),
        _ => {}
    }

    // Single digit: subject or lone unassisted fielder (caller decides).
    if RE_DIGIT_1_9.is_match(trimmed) {
        let n = trimmed.parse::<u8>().unwrap();
        return TokenKind::Digit(n);
    }

    // Pitch verbs.
    if RE_PITCH_VERB.is_match(trimmed) {
        let kind = match lower.as_str() {
            "b" => PitchVerbKind::Ball,
            "k" => PitchVerbKind::CalledStrike,
            "s" => PitchVerbKind::SwingingStrike,
            "f" => PitchVerbKind::Foul,
            "fl" => PitchVerbKind::FoulBunt,
            _ => unreachable!(),
        };
        return TokenKind::PitchVerb(kind);
    }

    // Hit verbs.
    if RE_HIT_VERB.is_match(trimmed) {
        let kind = match lower.as_str() {
            "h" => HitVerbKind::Single,
            "2h" => HitVerbKind::Double,
            "3h" => HitVerbKind::Triple,
            "hr" => HitVerbKind::HomeRun,
            _ => unreachable!(),
        };
        return TokenKind::HitVerb(kind);
    }

    // Steal.
    if RE_STEAL_VERB.is_match(trimmed) {
        return TokenKind::StealVerb;
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
        assert_eq!(classify("h"), TokenKind::HitVerb(HitVerbKind::Single));
        assert_eq!(classify("H"), TokenKind::HitVerb(HitVerbKind::Single));
        assert_eq!(classify("2h"), TokenKind::HitVerb(HitVerbKind::Double));
        assert_eq!(classify("3H"), TokenKind::HitVerb(HitVerbKind::Triple));
        assert_eq!(classify("HR"), TokenKind::HitVerb(HitVerbKind::HomeRun));
    }

    #[test]
    fn pitch_verbs_all_recognised() {
        assert_eq!(classify("b"), TokenKind::PitchVerb(PitchVerbKind::Ball));
        assert_eq!(
            classify("k"),
            TokenKind::PitchVerb(PitchVerbKind::CalledStrike)
        );
        assert_eq!(
            classify("s"),
            TokenKind::PitchVerb(PitchVerbKind::SwingingStrike)
        );
        assert_eq!(classify("f"), TokenKind::PitchVerb(PitchVerbKind::Foul));
        assert_eq!(classify("fl"), TokenKind::PitchVerb(PitchVerbKind::FoulBunt));
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
    fn zone_and_base_disambiguate() {
        assert!(matches!(classify("lf"), TokenKind::Zone(FieldZone::LF)));
        assert!(matches!(classify("2b"), TokenKind::Base(RunnerDest::Second)));
        assert!(matches!(classify("sc"), TokenKind::Base(RunnerDest::Score)));
        assert!(matches!(classify("home"), TokenKind::Base(RunnerDest::Score)));
    }

    #[test]
    fn keywords() {
        assert_eq!(classify("exit"), TokenKind::Keyword(KeywordKind::Exit));
        assert_eq!(classify("quit"), TokenKind::Keyword(KeywordKind::Exit));
        assert_eq!(
            classify("playball"),
            TokenKind::Keyword(KeywordKind::PlayBall)
        );
        assert_eq!(
            classify("regular"),
            TokenKind::Keyword(KeywordKind::Regular)
        );
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
