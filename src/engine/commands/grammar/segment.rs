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

use super::tokens::{TokenKind, classify};
use crate::engine::commands::errors::ParseError;
use crate::engine::commands::kind::CommandKind;
use crate::models::field_zone::FieldZone;
use crate::models::runner::RunnerDest;

/// A parsed segment. Every `Segment` variant is the result of
/// syntactically recognising one comma-separated chunk of an input line;
/// no state-dependent checks (`is runner on base?`, `is this the current
/// batter?`) are performed here — those belong to the validator.
///
/// Subjects: whether the subject is mandatory, forbidden, or optional
/// depends on the variant. See the [`kind::CommandKind`] module docs for
/// the subject-rule per family. When the subject is optional the
/// implicit default is the current batter; the *validator* substitutes
/// that default, segment-parsing stays game-state-agnostic and uses an
/// `Option<u8>` with `None` meaning "implicit current batter". See
/// module docs for the full shape grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Engine-control keyword. Carries the precise `CommandKind`
    /// (`Exit` or `PlayBall`); the validator does a simple match.
    Control(CommandKind),

    /// Game-status change. `CommandKind` is one of
    /// `Regular`/`Postponed`/`Cancelled`/`Suspended`/`Forfeited`/`Protested`.
    Status(CommandKind),

    /// Pitch recorded against the current batter. `CommandKind` is one
    /// of `Ball`/`CalledStrike`/`SwingingStrike`/`Foul`/`FoulBunt`.
    Pitch(CommandKind),

    /// Hit by a batter. `subject` is optional (implicit = current batter).
    /// `kind` is one of `Single`/`Double`/`Triple`/`HomeRun`.
    Hit {
        subject: Option<u8>,
        kind: CommandKind,
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

/// Batter/runner out shape. Both `Segment::BatterOut` and
/// `Segment::RunnerOut` reuse this type because the fielding patterns
/// are identical; only the target differs.
///
/// Unlike `CommandKind` (which is a pure tag enum), `BatterOutKind`
/// carries the fielder identifiers that are part of the out's lexical
/// shape. Use [`BatterOutKind::command_kind`] to get the corresponding
/// `CommandKind` variant.
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

impl BatterOutKind {
    /// The `CommandKind` corresponding to this batter-out shape. Note
    /// that `FlyOut { foul: true }` maps to `CommandKind::FoulFlyOut`
    /// (a distinct lexical verb in the grammar).
    pub fn command_kind(&self) -> CommandKind {
        match self {
            Self::Unassisted { .. } => CommandKind::Unassisted,
            Self::GroundOut { .. } => CommandKind::GroundOut,
            Self::FlyOut { foul: false, .. } => CommandKind::FlyOut,
            Self::FlyOut { foul: true, .. } => CommandKind::FoulFlyOut,
            Self::LineOut { .. } => CommandKind::LineOut,
            Self::InfieldFly { .. } => CommandKind::InfieldFly,
        }
    }
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

    use crate::engine::commands::kind::CommandFamily;

    // Dispatch on the FIRST token. Most paths are deterministic from here.
    match (&kinds[0], tokens[0]) {
        // ── Verb tokens that carry a precise CommandKind ───────────────────
        //
        // Every parameter-less verb (control / status / pitch / hit / steal)
        // shows up as TokenKind::Verb(CommandKind::X). Dispatch on the
        // family, not on the individual variant, so we don't fan out across
        // 13 arms here.
        (TokenKind::Verb(ck), _) => match ck.family() {
            CommandFamily::Control | CommandFamily::Status => parse_keyword_segment(*ck, &tokens),
            CommandFamily::Pitch => parse_pitch_segment(*ck, &tokens),
            CommandFamily::Hit => parse_hit(None, *ck, &tokens, &kinds),
            CommandFamily::Steal => Err(ParseError::MissingSubject {
                verb: "st".to_string(),
            }),
            // A parameter-less verb in families that require a subject or
            // follow a different path should never land here: the grammar
            // has no parameter-less BatterOut/FielderChoice/Advance.
            _ => unreachable!(
                "parameter-less verb {:?} in unexpected family {:?}",
                ck,
                ck.family()
            ),
        },

        // ── Subject-first path: `<n> <verb> [<obj>]` ──────────────────────────
        (TokenKind::Digit(subject), _) => parse_with_subject(*subject, &tokens, &kinds),

        // ── Implicit-batter paths for verbs with a numeric parameter ─────────
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

        // ── Base in first position: standalone advance without subject ───────
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

fn parse_keyword_segment(ck: CommandKind, tokens: &[&str]) -> Result<Segment, ParseError> {
    use crate::engine::commands::kind::CommandFamily;

    if tokens.len() > 1 {
        return Err(ParseError::ExtraTokens {
            verb: ck.canonical_name().to_string(),
            extra: tokens[1..].join(" "),
        });
    }

    Ok(match ck.family() {
        CommandFamily::Control => Segment::Control(ck),
        CommandFamily::Status => Segment::Status(ck),
        _ => unreachable!("parse_keyword_segment called with {:?}", ck),
    })
}

fn parse_pitch_segment(ck: CommandKind, tokens: &[&str]) -> Result<Segment, ParseError> {
    debug_assert_eq!(
        ck.family(),
        crate::engine::commands::kind::CommandFamily::Pitch,
        "parse_pitch_segment called with non-pitch {:?}",
        ck
    );
    if tokens.len() > 1 {
        return Err(ParseError::ExtraTokens {
            verb: tokens[0].to_string(),
            extra: tokens[1..].join(" "),
        });
    }
    Ok(Segment::Pitch(ck))
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
    if let TokenKind::Verb(ck) = &kinds[1] {
        use crate::engine::commands::kind::CommandFamily;
        match ck.family() {
            CommandFamily::Pitch | CommandFamily::Control | CommandFamily::Status => {
                return Err(ParseError::SubjectNotAllowed {
                    verb: tokens[1].to_string(),
                });
            }
            _ => {}
        }
    }

    let rest_tokens = &tokens[1..];
    let rest_kinds = &kinds[1..];

    match &rest_kinds[0] {
        TokenKind::Verb(ck) => {
            use crate::engine::commands::kind::CommandFamily;
            match ck.family() {
                CommandFamily::Hit => parse_hit(Some(subject), *ck, rest_tokens, rest_kinds),
                CommandFamily::Steal => parse_steal(subject, rest_tokens, rest_kinds),
                // Pitch/Control/Status already rejected above.
                _ => unreachable!("unexpected verb family {:?} after subject", ck.family()),
            }
        }
        TokenKind::FcVerb { fielder } => parse_fc(Some(subject), *fielder, rest_tokens, rest_kinds),

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
    }
}

// ─── Hit ─────────────────────────────────────────────────────────────────────

/// `hit_tokens` starts at the hit verb (index 0 of the slice).
/// Accepts optional zone as the next token.
///
/// `hit_kind` must be a variant of [`CommandKind`] in the `Hit` family
/// (Single / Double / Triple / HomeRun). Other variants are rejected by
/// the upstream dispatch.
fn parse_hit(
    subject: Option<u8>,
    hit_kind: CommandKind,
    hit_tokens: &[&str],
    hit_kinds: &[TokenKind],
) -> Result<Segment, ParseError> {
    debug_assert_eq!(
        hit_kind.family(),
        crate::engine::commands::kind::CommandFamily::Hit,
        "parse_hit called with non-hit {:?}",
        hit_kind
    );

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
        kind: hit_kind,
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
                kind: CommandKind::Single,
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
                kind: CommandKind::Single,
                zone: Some(FieldZone::LF),
            }
        );
    }
    #[test]
    fn double_triple_hr() {
        assert!(matches!(
            seg("2h"),
            Segment::Hit {
                kind: CommandKind::Double,
                ..
            }
        ));
        assert!(matches!(
            seg("3 3h cf"),
            Segment::Hit {
                subject: Some(3),
                kind: CommandKind::Triple,
                ..
            }
        ));
        assert!(matches!(
            seg("hr"),
            Segment::Hit {
                kind: CommandKind::HomeRun,
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
        assert_eq!(seg("b"), Segment::Pitch(CommandKind::Ball));
        assert_eq!(seg("k"), Segment::Pitch(CommandKind::CalledStrike));
        assert_eq!(seg("s"), Segment::Pitch(CommandKind::SwingingStrike));
        assert_eq!(seg("f"), Segment::Pitch(CommandKind::Foul));
        assert_eq!(seg("fl"), Segment::Pitch(CommandKind::FoulBunt));
    }
    #[test]
    fn pitch_with_subject_rejected() {
        assert!(matches!(err("5 b"), ParseError::SubjectNotAllowed { .. }));
        assert!(matches!(err("5 k"), ParseError::SubjectNotAllowed { .. }));
    }

    // ── Control / status ──
    #[test]
    fn control_keywords() {
        assert_eq!(seg("exit"), Segment::Control(CommandKind::Exit));
        assert_eq!(seg("quit"), Segment::Control(CommandKind::Exit));
        assert_eq!(seg("playball"), Segment::Control(CommandKind::PlayBall));
    }
    #[test]
    fn status_keywords() {
        assert_eq!(seg("regular"), Segment::Status(CommandKind::Regular));
        assert_eq!(seg("post"), Segment::Status(CommandKind::Postponed));
        assert_eq!(seg("cancel"), Segment::Status(CommandKind::Cancelled));
        assert_eq!(seg("susp"), Segment::Status(CommandKind::Suspended));
        assert_eq!(seg("forf"), Segment::Status(CommandKind::Forfeited));
        assert_eq!(seg("protest"), Segment::Status(CommandKind::Protested));
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
