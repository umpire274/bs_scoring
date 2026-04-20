//! Syntactic segment parser.
//!
//! A **segment** is one comma-separated chunk of the input line. This module
//! turns a raw segment string into a fully-typed [`Segment`] value — or a
//! [`ParseError`] if the shape is invalid.
//!
//! Parsing is **stateless**: it does not know about the game state. Batter
//! slot coherence, runner presence, infield-fly preconditions, etc. are
//! checked later by the validator.
//!
//! # Grammar (informal)
//!
//! ```text
//! Segment   := Control | Status | Pitch | Action
//! Control   := 'exit' | 'quit' | 'playball'
//! Status    := 'regular' | 'post' | 'cancel' | 'susp' | 'forf' | 'protest'
//! Pitch     := 'b' | 'k' | 's' | 'f' | 'fl'
//! Action    := Subject? ActionVerb
//! Subject   := [1-9]
//! ActionVerb:= HitVerb Zone?
//!            | OutVerb                         -- batter-only or runner with subject
//!            | FcVerb Base
//!            | StealVerb Base
//!            | Base                             -- runner advance
//! ```
//!
//! # Subject-optional rule
//!
//! The subject is mandatory **except** when the verb is lexically
//! distinguishable from a lone digit AND represents an action the batter
//! owns:
//!
//! - Hit verbs (`h`, `2h`, `3h`, `hr`)
//! - Batter out verbs with a multi-char shape: fielding sequence (`63`),
//!   fly (`F8`), foul fly (`FF3`), line out (`L6`), infield fly (`IF4`)
//! - Fielder's choice (`o6 1b`)
//!
//! An unassisted-out verb is a single digit (`5`) and therefore collides with
//! a lone subject. In that case the subject is required: the user writes
//! `5 5` to mean "batter #5 unassisted by fielder #5".
//!
//! Steal (`st`), runner-advance (`2b`) and runner-targeted outs all REQUIRE
//! a subject — there is no implicit-runner shortcut.
//!
//! Pitch and control verbs FORBID a subject.

use super::tokens::{HitVerbKind, KeywordKind, PitchVerbKind, TokenKind, classify};
use crate::engine::commands::errors::ParseError;
use crate::models::field_zone::FieldZone;
use crate::models::runner::RunnerDest;

/// A parsed segment — the shape of one comma-separated chunk of the line.
///
/// The batter subject (1–9) is optional only on the variants that allow an
/// implicit-batter shortcut; see module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Engine-control keyword.
    Control(ControlKind),

    /// Game-status change.
    Status(StatusKind),

    /// Pitch recorded against the current batter.
    Pitch(PitchKind),

    /// Hit by a batter. `subject` is optional (implicit = current batter).
    Hit {
        subject: Option<u8>,
        kind: HitKind,
        zone: Option<FieldZone>,
    },

    /// Batter retired on a batter-only out. `subject` is optional for all
    /// multi-char out shapes; required for the single-digit unassisted
    /// variant.
    BatterOut {
        subject: Option<u8>,
        out: BatterOutKind,
    },

    /// Runner retired on a runner-targeted out. `subject` is ALWAYS required.
    RunnerOut { subject: u8, out: BatterOutKind },

    /// Fielder's choice. `subject` is optional (implicit = current batter).
    FielderChoice {
        subject: Option<u8>,
        fielder: u8,
        base: RunnerDest,
    },

    /// Stolen base. `subject` is ALWAYS required (runner only).
    Steal { subject: u8, dest: RunnerDest },

    /// Runner advance (`<n> <base>`). `subject` is ALWAYS required.
    Advance { subject: u8, dest: RunnerDest },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlKind {
    Exit,
    PlayBall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Regular,
    Postponed,
    Cancelled,
    Suspended,
    Forfeited,
    Protested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchKind {
    Ball,
    CalledStrike,
    SwingingStrike,
    Foul,
    FoulBunt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Single,
    Double,
    Triple,
    HomeRun,
}

/// Batter/runner out shape. Both BatterOut and RunnerOut reuse this type
/// because the fielding patterns are identical; only the target differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatterOutKind {
    /// Unassisted out by a single fielder (e.g. `3`, `5`).
    Unassisted { fielder: u8 },
    /// Ground-out sequence (e.g. `6-3`, `8-6-2`).
    GroundOut { fielders: Vec<u8> },
    /// Fly out (fair or foul).
    FlyOut { fielder: u8, foul: bool },
    /// Line out.
    LineOut { fielder: u8 },
    /// Infield fly.
    InfieldFly { fielder: u8 },
}

// ─── Parser entry point ──────────────────────────────────────────────────────

/// Parse a single segment (one comma-separated chunk of the input line).
pub fn parse_segment(raw: &str) -> Result<Segment, ParseError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ParseError::EmptySegment);
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let kinds: Vec<TokenKind> = tokens.iter().map(|t| classify(t)).collect();

    // Dispatch on the FIRST token. Most paths are deterministic from here.
    match (&kinds[0], tokens[0]) {
        // ── Control / status / pitch (subject forbidden) ──────────────────────
        (TokenKind::Keyword(kw), _) => parse_keyword_segment(*kw, &tokens),
        (TokenKind::PitchVerb(pv), _) => parse_pitch_segment(*pv, &tokens),

        // ── Subject-first path: `<n> <verb> [<obj>]` ──────────────────────────
        (TokenKind::Digit(subject), _) => parse_with_subject(*subject, &tokens, &kinds),

        // ── Implicit-batter paths ─────────────────────────────────────────────
        (TokenKind::HitVerb(hv), _) => parse_hit(None, *hv, &tokens, &kinds),
        (TokenKind::FcVerb { fielder }, _) => parse_fc(None, *fielder, &tokens, &kinds),
        (TokenKind::FlyVerb { fielder, foul }, _) => parse_batter_out_implicit(
            BatterOutKind::FlyOut {
                fielder: *fielder,
                foul: *foul,
            },
            &tokens,
        ),
        (TokenKind::LineVerb { fielder }, _) => {
            parse_batter_out_implicit(BatterOutKind::LineOut { fielder: *fielder }, &tokens)
        }
        (TokenKind::InfieldFlyVerb { fielder }, _) => {
            parse_batter_out_implicit(BatterOutKind::InfieldFly { fielder: *fielder }, &tokens)
        }
        (TokenKind::FieldingSeq(fielders), _) => parse_batter_out_implicit(
            BatterOutKind::GroundOut {
                fielders: fielders.clone(),
            },
            &tokens,
        ),

        // ── No-subject verbs that REQUIRE a subject — these are the ones that
        //    can never be implicit-batter: steal and runner-advance. ───────────
        (TokenKind::StealVerb, _) => Err(ParseError::MissingSubject {
            verb: "st".to_string(),
        }),
        (TokenKind::Base(_), first) => Err(ParseError::MissingSubject {
            verb: first.to_string(),
        }),

        // ── Zone in first position is never valid alone ──────────────────────
        (TokenKind::Zone(_), first) => Err(ParseError::UnknownVerb {
            token: first.to_string(),
        }),

        // ── Everything else ───────────────────────────────────────────────────
        (TokenKind::Unknown(_), first) => Err(ParseError::UnknownVerb {
            token: first.to_string(),
        }),
    }
}

// ─── Keyword / control / status / pitch ──────────────────────────────────────

fn parse_keyword_segment(kw: KeywordKind, tokens: &[&str]) -> Result<Segment, ParseError> {
    if tokens.len() > 1 {
        return Err(ParseError::ExtraTokens {
            verb: kw.as_str().to_string(),
            extra: tokens[1..].join(" "),
        });
    }

    Ok(match kw {
        KeywordKind::Exit => Segment::Control(ControlKind::Exit),
        KeywordKind::PlayBall => Segment::Control(ControlKind::PlayBall),
        KeywordKind::Regular => Segment::Status(StatusKind::Regular),
        KeywordKind::Postponed => Segment::Status(StatusKind::Postponed),
        KeywordKind::Cancelled => Segment::Status(StatusKind::Cancelled),
        KeywordKind::Suspended => Segment::Status(StatusKind::Suspended),
        KeywordKind::Forfeited => Segment::Status(StatusKind::Forfeited),
        KeywordKind::Protested => Segment::Status(StatusKind::Protested),
    })
}

fn parse_pitch_segment(pv: PitchVerbKind, tokens: &[&str]) -> Result<Segment, ParseError> {
    if tokens.len() > 1 {
        return Err(ParseError::ExtraTokens {
            verb: tokens[0].to_string(),
            extra: tokens[1..].join(" "),
        });
    }
    Ok(Segment::Pitch(match pv {
        PitchVerbKind::Ball => PitchKind::Ball,
        PitchVerbKind::CalledStrike => PitchKind::CalledStrike,
        PitchVerbKind::SwingingStrike => PitchKind::SwingingStrike,
        PitchVerbKind::Foul => PitchKind::Foul,
        PitchVerbKind::FoulBunt => PitchKind::FoulBunt,
    }))
}

// ─── Subject-first path ──────────────────────────────────────────────────────

/// Handles every segment that starts with a digit subject.
fn parse_with_subject(
    subject: u8,
    tokens: &[&str],
    kinds: &[TokenKind],
) -> Result<Segment, ParseError> {
    // A lone digit is not a valid segment.
    if tokens.len() == 1 {
        return Err(ParseError::UnknownVerb {
            token: tokens[0].to_string(),
        });
    }

    // Reject subject-on-pitch / subject-on-control right away.
    match &kinds[1] {
        TokenKind::PitchVerb(_) => {
            return Err(ParseError::SubjectNotAllowed {
                verb: tokens[1].to_string(),
            });
        }
        TokenKind::Keyword(_) => {
            return Err(ParseError::SubjectNotAllowed {
                verb: tokens[1].to_string(),
            });
        }
        _ => {}
    }

    let rest_tokens = &tokens[1..];
    let rest_kinds = &kinds[1..];

    match &rest_kinds[0] {
        TokenKind::HitVerb(hv) => parse_hit(Some(subject), *hv, rest_tokens, rest_kinds),
        TokenKind::FcVerb { fielder } => parse_fc(Some(subject), *fielder, rest_tokens, rest_kinds),
        TokenKind::StealVerb => parse_steal(subject, rest_tokens, rest_kinds),

        // Runner-targeted OUTs.
        TokenKind::FlyVerb { fielder, foul } => parse_targeted_out(
            subject,
            BatterOutKind::FlyOut {
                fielder: *fielder,
                foul: *foul,
            },
            rest_tokens,
        ),
        TokenKind::LineVerb { fielder } => parse_targeted_out(
            subject,
            BatterOutKind::LineOut { fielder: *fielder },
            rest_tokens,
        ),
        TokenKind::InfieldFlyVerb { fielder } => parse_targeted_out(
            subject,
            BatterOutKind::InfieldFly { fielder: *fielder },
            rest_tokens,
        ),
        TokenKind::FieldingSeq(fielders) => parse_targeted_out(
            subject,
            BatterOutKind::GroundOut {
                fielders: fielders.clone(),
            },
            rest_tokens,
        ),

        // Single-digit second token: unassisted out by that fielder.
        TokenKind::Digit(fielder) => parse_targeted_out(
            subject,
            BatterOutKind::Unassisted { fielder: *fielder },
            rest_tokens,
        ),

        // Runner advance: `<n> <base>`.
        TokenKind::Base(base) => {
            if rest_tokens.len() > 1 {
                return Err(ParseError::ExtraTokens {
                    verb: tokens[0].to_string(),
                    extra: rest_tokens[1..].join(" "),
                });
            }
            Ok(Segment::Advance {
                subject,
                dest: *base,
            })
        }

        TokenKind::Zone(_) => Err(ParseError::UnknownVerb {
            token: rest_tokens[0].to_string(),
        }),
        TokenKind::Unknown(_) => Err(ParseError::UnknownVerb {
            token: rest_tokens[0].to_string(),
        }),

        // Pitch / Keyword already handled above, cannot arrive here.
        TokenKind::PitchVerb(_) | TokenKind::Keyword(_) => unreachable!(),
    }
}

// ─── Hit ─────────────────────────────────────────────────────────────────────

/// `hit_tokens` starts at the hit verb (index 0 of the slice).
/// Accepts optional zone as the next token.
fn parse_hit(
    subject: Option<u8>,
    hv: HitVerbKind,
    hit_tokens: &[&str],
    hit_kinds: &[TokenKind],
) -> Result<Segment, ParseError> {
    let kind = match hv {
        HitVerbKind::Single => HitKind::Single,
        HitVerbKind::Double => HitKind::Double,
        HitVerbKind::Triple => HitKind::Triple,
        HitVerbKind::HomeRun => HitKind::HomeRun,
    };

    let zone = match hit_tokens.get(1) {
        None => None,
        Some(_) => match &hit_kinds[1] {
            TokenKind::Zone(z) => Some(*z),
            _ => {
                return Err(ParseError::InvalidZone {
                    token: hit_tokens[1].to_string(),
                });
            }
        },
    };

    if hit_tokens.len() > 2 {
        return Err(ParseError::ExtraTokens {
            verb: hit_tokens[0].to_string(),
            extra: hit_tokens[2..].join(" "),
        });
    }

    Ok(Segment::Hit {
        subject,
        kind,
        zone,
    })
}

// ─── Fielder's choice ────────────────────────────────────────────────────────

/// `fc_tokens` starts at the FC verb (`o<n>`). Base is MANDATORY.
fn parse_fc(
    subject: Option<u8>,
    fielder: u8,
    fc_tokens: &[&str],
    fc_kinds: &[TokenKind],
) -> Result<Segment, ParseError> {
    let base_tok = fc_tokens.get(1).ok_or_else(|| ParseError::MissingObject {
        verb: fc_tokens[0].to_string(),
        expected: "destination base (1B / 2B / 3B / SC)",
    })?;

    let base = match &fc_kinds[1] {
        TokenKind::Base(b) => *b,
        _ => {
            return Err(ParseError::InvalidBase {
                token: base_tok.to_string(),
            });
        }
    };

    if fc_tokens.len() > 2 {
        return Err(ParseError::ExtraTokens {
            verb: fc_tokens[0].to_string(),
            extra: fc_tokens[2..].join(" "),
        });
    }

    Ok(Segment::FielderChoice {
        subject,
        fielder,
        base,
    })
}

// ─── Steal ───────────────────────────────────────────────────────────────────

/// Steal path, called after the subject has been consumed.
/// `steal_tokens[0]` is `st`, followed by a mandatory base.
fn parse_steal(
    subject: u8,
    steal_tokens: &[&str],
    steal_kinds: &[TokenKind],
) -> Result<Segment, ParseError> {
    let base_tok = steal_tokens
        .get(1)
        .ok_or_else(|| ParseError::MissingObject {
            verb: "st".to_string(),
            expected: "destination base (1B / 2B / 3B / SC)",
        })?;

    let dest = match &steal_kinds[1] {
        TokenKind::Base(b) => *b,
        _ => {
            return Err(ParseError::InvalidBase {
                token: base_tok.to_string(),
            });
        }
    };

    if steal_tokens.len() > 2 {
        return Err(ParseError::ExtraTokens {
            verb: "st".to_string(),
            extra: steal_tokens[2..].join(" "),
        });
    }

    Ok(Segment::Steal { subject, dest })
}

// ─── Batter-out / runner-out helpers ─────────────────────────────────────────

/// Batter-out with implicit subject. The verb is the only token.
fn parse_batter_out_implicit(out: BatterOutKind, tokens: &[&str]) -> Result<Segment, ParseError> {
    if tokens.len() > 1 {
        return Err(ParseError::ExtraTokens {
            verb: tokens[0].to_string(),
            extra: tokens[1..].join(" "),
        });
    }
    Ok(Segment::BatterOut { subject: None, out })
}

/// Runner-targeted out (or explicit batter out): the subject has already been
/// consumed; `rest_tokens[0]` is the verb token we already classified.
///
/// Note: at this stage we cannot tell if the subject is the current batter
/// (→ BatterOut with explicit subject) or a runner (→ RunnerOut). We emit
/// `RunnerOut` and let the validator re-classify once it has the game state.
fn parse_targeted_out(
    subject: u8,
    out: BatterOutKind,
    rest_tokens: &[&str],
) -> Result<Segment, ParseError> {
    if rest_tokens.len() > 1 {
        return Err(ParseError::ExtraTokens {
            verb: rest_tokens[0].to_string(),
            extra: rest_tokens[1..].join(" "),
        });
    }
    Ok(Segment::RunnerOut { subject, out })
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helpers to reduce test boilerplate.
    fn seg(s: &str) -> Segment {
        parse_segment(s).expect("should parse")
    }
    fn err(s: &str) -> ParseError {
        parse_segment(s).expect_err("should fail")
    }

    // ── Hits ──
    #[test]
    fn hit_without_subject() {
        assert_eq!(
            seg("h"),
            Segment::Hit {
                subject: None,
                kind: HitKind::Single,
                zone: None,
            }
        );
    }
    #[test]
    fn hit_with_subject_and_zone() {
        assert_eq!(
            seg("5 h lf"),
            Segment::Hit {
                subject: Some(5),
                kind: HitKind::Single,
                zone: Some(FieldZone::LF),
            }
        );
    }
    #[test]
    fn double_triple_hr() {
        assert!(matches!(
            seg("2h"),
            Segment::Hit {
                kind: HitKind::Double,
                ..
            }
        ));
        assert!(matches!(
            seg("3 3h cf"),
            Segment::Hit {
                subject: Some(3),
                kind: HitKind::Triple,
                ..
            }
        ));
        assert!(matches!(
            seg("hr"),
            Segment::Hit {
                kind: HitKind::HomeRun,
                ..
            }
        ));
    }
    #[test]
    fn hit_invalid_zone() {
        assert!(matches!(err("h xyz"), ParseError::InvalidZone { .. }));
    }

    // ── Batter outs (implicit subject) ──
    #[test]
    fn fly_out_implicit() {
        assert_eq!(
            seg("f8"),
            Segment::BatterOut {
                subject: None,
                out: BatterOutKind::FlyOut {
                    fielder: 8,
                    foul: false,
                },
            }
        );
    }
    #[test]
    fn foul_fly_out_implicit() {
        assert_eq!(
            seg("ff3"),
            Segment::BatterOut {
                subject: None,
                out: BatterOutKind::FlyOut {
                    fielder: 3,
                    foul: true,
                },
            }
        );
    }
    #[test]
    fn line_out_implicit() {
        assert_eq!(
            seg("l6"),
            Segment::BatterOut {
                subject: None,
                out: BatterOutKind::LineOut { fielder: 6 },
            }
        );
    }
    #[test]
    fn infield_fly_implicit() {
        assert_eq!(
            seg("if4"),
            Segment::BatterOut {
                subject: None,
                out: BatterOutKind::InfieldFly { fielder: 4 },
            }
        );
        assert_eq!(
            seg("iff4"),
            Segment::BatterOut {
                subject: None,
                out: BatterOutKind::InfieldFly { fielder: 4 },
            }
        );
    }
    #[test]
    fn ground_out_implicit() {
        assert_eq!(
            seg("63"),
            Segment::BatterOut {
                subject: None,
                out: BatterOutKind::GroundOut {
                    fielders: vec![6, 3],
                },
            }
        );
        assert_eq!(
            seg("8-6-2"),
            Segment::BatterOut {
                subject: None,
                out: BatterOutKind::GroundOut {
                    fielders: vec![8, 6, 2],
                },
            }
        );
    }

    // ── Unassisted requires explicit subject ──
    #[test]
    fn unassisted_alone_is_rejected() {
        // A lone digit cannot be disambiguated from a dangling subject.
        assert!(matches!(err("5"), ParseError::UnknownVerb { .. }));
    }
    #[test]
    fn unassisted_with_subject() {
        assert_eq!(
            seg("5 5"),
            Segment::RunnerOut {
                subject: 5,
                out: BatterOutKind::Unassisted { fielder: 5 },
            }
        );
        // Note: validator will demote "RunnerOut subject=current_batter"
        //       into a BatterOut.
    }

    // ── Runner-targeted out (explicit subject) ──
    #[test]
    fn runner_ground_out() {
        assert_eq!(
            seg("3 64"),
            Segment::RunnerOut {
                subject: 3,
                out: BatterOutKind::GroundOut {
                    fielders: vec![6, 4],
                },
            }
        );
    }

    // ── Fielder's choice ──
    #[test]
    fn fc_implicit() {
        assert_eq!(
            seg("o6 1b"),
            Segment::FielderChoice {
                subject: None,
                fielder: 6,
                base: RunnerDest::First,
            }
        );
    }
    #[test]
    fn fc_with_subject() {
        assert_eq!(
            seg("5 o4 1b"),
            Segment::FielderChoice {
                subject: Some(5),
                fielder: 4,
                base: RunnerDest::First,
            }
        );
    }
    #[test]
    fn fc_without_base_fails() {
        assert!(matches!(err("o6"), ParseError::MissingObject { .. }));
        assert!(matches!(err("5 o6"), ParseError::MissingObject { .. }));
    }

    // ── Steal ──
    #[test]
    fn steal_requires_subject() {
        assert!(matches!(err("st 2b"), ParseError::MissingSubject { .. }));
    }
    #[test]
    fn steal_home() {
        assert_eq!(
            seg("5 st sc"),
            Segment::Steal {
                subject: 5,
                dest: RunnerDest::Score,
            }
        );
    }
    #[test]
    fn steal_requires_base() {
        assert!(matches!(err("5 st"), ParseError::MissingObject { .. }));
    }

    // ── Advance ──
    #[test]
    fn advance_standalone() {
        assert_eq!(
            seg("5 2b"),
            Segment::Advance {
                subject: 5,
                dest: RunnerDest::Second,
            }
        );
    }
    #[test]
    fn advance_without_subject_rejected() {
        assert!(matches!(err("2b"), ParseError::MissingSubject { .. }));
    }

    // ── Pitch ──
    #[test]
    fn pitch_verbs_alone() {
        assert_eq!(seg("b"), Segment::Pitch(PitchKind::Ball));
        assert_eq!(seg("k"), Segment::Pitch(PitchKind::CalledStrike));
        assert_eq!(seg("s"), Segment::Pitch(PitchKind::SwingingStrike));
        assert_eq!(seg("f"), Segment::Pitch(PitchKind::Foul));
        assert_eq!(seg("fl"), Segment::Pitch(PitchKind::FoulBunt));
    }
    #[test]
    fn pitch_with_subject_rejected() {
        assert!(matches!(err("5 b"), ParseError::SubjectNotAllowed { .. }));
        assert!(matches!(err("5 k"), ParseError::SubjectNotAllowed { .. }));
    }

    // ── Control / status ──
    #[test]
    fn control_keywords() {
        assert_eq!(seg("exit"), Segment::Control(ControlKind::Exit));
        assert_eq!(seg("quit"), Segment::Control(ControlKind::Exit));
        assert_eq!(seg("playball"), Segment::Control(ControlKind::PlayBall));
    }
    #[test]
    fn status_keywords() {
        assert_eq!(seg("regular"), Segment::Status(StatusKind::Regular));
        assert_eq!(seg("post"), Segment::Status(StatusKind::Postponed));
        assert_eq!(seg("cancel"), Segment::Status(StatusKind::Cancelled));
        assert_eq!(seg("susp"), Segment::Status(StatusKind::Suspended));
        assert_eq!(seg("forf"), Segment::Status(StatusKind::Forfeited));
        assert_eq!(seg("protest"), Segment::Status(StatusKind::Protested));
    }
    #[test]
    fn keyword_with_subject_rejected() {
        assert!(matches!(
            err("5 playball"),
            ParseError::SubjectNotAllowed { .. }
        ));
        assert!(matches!(
            err("5 exit"),
            ParseError::SubjectNotAllowed { .. }
        ));
    }
    #[test]
    fn keyword_with_extra_token() {
        assert!(matches!(err("exit now"), ParseError::ExtraTokens { .. }));
    }

    // ── Error cases ──
    #[test]
    fn empty_segment() {
        assert_eq!(parse_segment(""), Err(ParseError::EmptySegment));
        assert_eq!(parse_segment("   "), Err(ParseError::EmptySegment));
    }
    #[test]
    fn gibberish() {
        assert!(matches!(err("xyz"), ParseError::UnknownVerb { .. }));
    }
    #[test]
    fn extra_tokens_on_hit() {
        assert!(matches!(
            err("5 h lf extra"),
            ParseError::ExtraTokens { .. }
        ));
    }
    #[test]
    fn case_insensitive() {
        // Every canonical token should still parse when the user types
        // uppercase or mixed case.
        assert!(parse_segment("5 H LF").is_ok());
        assert!(parse_segment("5 HR CF").is_ok());
        assert!(parse_segment("5 O6 1B").is_ok());
        assert!(parse_segment("5 ST 2B").is_ok());
        assert!(parse_segment("PLAYBALL").is_ok());
    }
}
