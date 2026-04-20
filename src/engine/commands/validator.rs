//! Semantic validation layer.
//!
//! Takes the syntactically-parsed segments produced by
//! [`crate::engine::commands::grammar`] and turns them into a
//! `Vec<EngineCommand>` ready for the reducer, applying every rule that
//! requires access to the current [`GameState`]:
//!
//! - a batter-only verb carrying an explicit subject must match the
//!   current batter slot;
//! - a runner-targeted segment must name a runner that is actually on
//!   base; if the subject equals the current batter, the segment is
//!   reinterpreted as a batter-out;
//! - a lone runner-advance override (`<n> <base>`) needs a triggering
//!   play (hit or FC) on the same line;
//! - the infield-fly rule requires fewer than two outs and runners on
//!   both 1B and 2B simultaneously;
//! - no single action may record more than three outs;
//! - control / status / pitch segments cannot be mixed with action
//!   segments on the same line.
//!
//! Errors are **accumulated**: when per-segment semantic checks spot a
//! problem, the offending segment is dropped but validation continues for
//! the remaining ones. Line-level checks (duplicate-subject,
//! infield-fly-preconditions, too-many-outs, illegal coalescing) are then
//! evaluated. The caller receives either the full `Vec<EngineCommand>` or
//! the full list of `CommandError` values produced along the way.

use crate::engine::commands::errors::{CommandError, CommandErrorKind, ValidationError};
use crate::engine::commands::grammar::{
    BatterOutKind, ControlKind, HitKind, PitchKind, Segment, StatusKind,
};
use crate::engine::commands::types::EngineCommand;
use crate::engine::scoring::batter_outs::{
    DefensiveOutKind, DefensiveOutRecord, DefensivePlayCommand, DefensivePlayTarget,
    FielderChoiceAdvance, FieldingSequence,
};
use crate::models::field_zone::FieldZone;
use crate::models::game_state::GameState;
use crate::models::runner::{RunnerDest, RunnerOverride};
use crate::models::types::{GameStatus, Pitch};

/// A parsed [`Segment`] paired with its original position in the line.
///
/// The facade produces one `IndexedSegment` per non-empty comma-separated
/// chunk and hands them to [`validate`]. The text and 1-based index travel
/// alongside the typed segment so the validator can build diagnostics that
/// point at the offending segment.
#[derive(Debug, Clone)]
pub struct IndexedSegment {
    /// 1-based position of this segment in the input line.
    pub index: usize,
    /// The trimmed original text of the segment. Used only for diagnostics.
    pub text: String,
    /// The typed segment produced by the grammar layer.
    pub segment: Segment,
}

/// Validate a list of parsed segments against the current game state and
/// coalesce them into the `EngineCommand` values expected by the reducer.
///
/// Accumulates **every** error found in the line instead of stopping at the
/// first. On success the returned vector keeps the declared intent of the
/// input — hit segments are merged with their runner overrides, defensive
/// outs are merged into a single `DefensivePlay`, steals and pitches stay
/// as independent commands.
pub fn validate(
    indexed: Vec<IndexedSegment>,
    state: &GameState,
) -> Result<Vec<EngineCommand>, Vec<CommandError>> {
    if indexed.is_empty() {
        return Ok(Vec::new());
    }

    let mut errors: Vec<CommandError> = Vec::new();

    // ── Line-level mixing rules (structural): emit and bail early ──────────
    if let Err(mut e) = check_mixing(&indexed) {
        errors.append(&mut e);
        return Err(errors);
    }

    // ── Pure control / status line (exactly one segment) ───────────────────
    if let Some(cmd) = try_build_control_line(&indexed) {
        return Ok(vec![cmd]);
    }

    // ── Per-segment classification and validation ─────────────────────────
    let mut resolved: Vec<Option<Resolved>> = Vec::with_capacity(indexed.len());
    let current_batter: Option<u8> = state.current_batter_order;

    let has_hit = indexed
        .iter()
        .any(|s| matches!(s.segment, Segment::Hit { .. }));
    let has_fc = indexed
        .iter()
        .any(|s| matches!(s.segment, Segment::FielderChoice { .. }));
    let has_trigger_for_advance = has_hit || has_fc;

    for seg in &indexed {
        match classify_segment(&seg.segment, current_batter, state, has_trigger_for_advance) {
            Ok(r) => resolved.push(Some(r)),
            Err(e) => {
                errors.push(to_err(seg, e));
                resolved.push(None);
            }
        }
    }

    // ── Line-level semantic checks on the resolved segments ────────────────
    run_line_level_checks(&resolved, &indexed, state, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    // ── Coalesce valid segments into EngineCommand values ──────────────────
    build_commands(&resolved)
}

// ─── Internal intermediate form ──────────────────────────────────────────────

/// The per-segment result after semantic classification. Every variant
/// carries the actual batting-order slot to use (no more `Option<u8>`).
#[derive(Debug, Clone)]
enum Resolved {
    Pitch(PitchKind),
    Hit {
        #[allow(dead_code)] // retained for future reducer needs
        batter: u8,
        kind: HitKind,
        zone: Option<FieldZone>,
    },
    BatterOut {
        #[allow(dead_code)]
        batter: u8,
        out: BatterOutKind,
    },
    RunnerOut {
        subject: u8,
        out: BatterOutKind,
    },
    Fc {
        #[allow(dead_code)]
        batter: u8,
        fielder: u8,
        base: RunnerDest,
    },
    Steal {
        subject: u8,
        dest: RunnerDest,
    },
    Advance {
        subject: u8,
        dest: RunnerDest,
    },
}

// ─── Mixing checks ───────────────────────────────────────────────────────────

fn check_mixing(indexed: &[IndexedSegment]) -> Result<(), Vec<CommandError>> {
    let mut errors = Vec::new();

    let has_control_or_status = indexed
        .iter()
        .any(|s| matches!(s.segment, Segment::Control(_) | Segment::Status(_)));

    if has_control_or_status && indexed.len() > 1 {
        for seg in indexed {
            if matches!(seg.segment, Segment::Control(_) | Segment::Status(_)) {
                errors.push(mixing_err(seg, seg.text.clone()));
            }
        }
    }

    // Pitches cannot be combined with hits / outs / FC. They CAN be
    // combined with steals (e.g. `b, 5 st 2b`) and with other pitches.
    let has_non_steal_action = indexed.iter().any(|s| {
        matches!(
            s.segment,
            Segment::Hit { .. }
                | Segment::BatterOut { .. }
                | Segment::RunnerOut { .. }
                | Segment::FielderChoice { .. }
                | Segment::Advance { .. }
        )
    });
    let has_pitch = indexed
        .iter()
        .any(|s| matches!(s.segment, Segment::Pitch(_)));
    if has_pitch && has_non_steal_action {
        for seg in indexed {
            if matches!(seg.segment, Segment::Pitch(_)) {
                errors.push(mixing_err(seg, seg.text.clone()));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn mixing_err(seg: &IndexedSegment, verb: String) -> CommandError {
    CommandError {
        segment_index: seg.index,
        segment_text: seg.text.clone(),
        kind: CommandErrorKind::Validation(ValidationError::ControlMixedWithActions { verb }),
    }
}

// ─── Fast path: pure control/status line ─────────────────────────────────────

fn try_build_control_line(indexed: &[IndexedSegment]) -> Option<EngineCommand> {
    if indexed.len() != 1 {
        return None;
    }
    match &indexed[0].segment {
        Segment::Control(ControlKind::Exit) => Some(EngineCommand::Exit),
        Segment::Control(ControlKind::PlayBall) => Some(EngineCommand::PlayBall),
        Segment::Status(sk) => Some(EngineCommand::SetStatus(status_to_game(*sk))),
        _ => None,
    }
}

// ─── Per-segment classification ──────────────────────────────────────────────

fn classify_segment(
    seg: &Segment,
    current_batter: Option<u8>,
    state: &GameState,
    has_trigger_for_advance: bool,
) -> Result<Resolved, ValidationError> {
    match seg {
        Segment::Pitch(pk) => Ok(Resolved::Pitch(*pk)),

        Segment::Control(_) | Segment::Status(_) => {
            // Already handled before this pass.
            Err(ValidationError::ControlMixedWithActions {
                verb: "control/status".to_string(),
            })
        }

        Segment::Hit {
            subject,
            kind,
            zone,
        } => {
            let batter = resolve_batter_subject(*subject, current_batter)?;
            Ok(Resolved::Hit {
                batter,
                kind: *kind,
                zone: *zone,
            })
        }

        Segment::BatterOut { subject, out } => {
            let batter = resolve_batter_subject(*subject, current_batter)?;
            Ok(Resolved::BatterOut {
                batter,
                out: out.clone(),
            })
        }

        Segment::RunnerOut { subject, out } => {
            if Some(*subject) == current_batter {
                Ok(Resolved::BatterOut {
                    batter: *subject,
                    out: out.clone(),
                })
            } else if state.is_on_base(*subject) {
                Ok(Resolved::RunnerOut {
                    subject: *subject,
                    out: out.clone(),
                })
            } else {
                Err(ValidationError::RunnerNotOnBase { order: *subject })
            }
        }

        Segment::FielderChoice {
            subject,
            fielder,
            base,
        } => {
            let batter = resolve_batter_subject(*subject, current_batter)?;
            Ok(Resolved::Fc {
                batter,
                fielder: *fielder,
                base: *base,
            })
        }

        Segment::Steal { subject, dest } => {
            if !state.is_on_base(*subject) {
                return Err(ValidationError::RunnerNotOnBase { order: *subject });
            }
            Ok(Resolved::Steal {
                subject: *subject,
                dest: *dest,
            })
        }

        Segment::Advance { subject, dest } => {
            if !has_trigger_for_advance {
                return Err(ValidationError::AdvanceWithoutTrigger { order: *subject });
            }
            if !state.is_on_base(*subject) {
                return Err(ValidationError::RunnerNotOnBase { order: *subject });
            }
            Ok(Resolved::Advance {
                subject: *subject,
                dest: *dest,
            })
        }
    }
}

fn resolve_batter_subject(subject: Option<u8>, current: Option<u8>) -> Result<u8, ValidationError> {
    match (subject, current) {
        (Some(given), Some(cur)) if given == cur => Ok(cur),
        (Some(given), cur) => Err(ValidationError::BatterSlotMismatch {
            given,
            current: cur,
        }),
        (None, Some(cur)) => Ok(cur),
        (None, None) => Err(ValidationError::BatterSlotMismatch {
            given: 0,
            current: None,
        }),
    }
}

// ─── Line-level checks after per-segment classification ─────────────────────

fn run_line_level_checks(
    resolved: &[Option<Resolved>],
    indexed: &[IndexedSegment],
    state: &GameState,
    errors: &mut Vec<CommandError>,
) {
    // 1) Duplicate subject: batter slot cannot appear as both hitter/FC
    //    and as a runner advance override on the same line.
    let batter_slot: Option<u8> = resolved.iter().find_map(|r| match r {
        Some(Resolved::Hit { batter, .. }) => Some(*batter),
        Some(Resolved::Fc { batter, .. }) => Some(*batter),
        _ => None,
    });
    if let Some(b) = batter_slot {
        for (i, r) in resolved.iter().enumerate() {
            if let Some(Resolved::Advance { subject, .. }) = r
                && *subject == b
            {
                errors.push(CommandError {
                    segment_index: indexed[i].index,
                    segment_text: indexed[i].text.clone(),
                    kind: CommandErrorKind::Validation(ValidationError::DuplicateSubject {
                        order: b,
                    }),
                });
            }
        }
    }

    // 2) Infield-fly preconditions.
    for (i, r) in resolved.iter().enumerate() {
        if let Some(Resolved::BatterOut {
            out: BatterOutKind::InfieldFly { .. },
            ..
        }) = r
            && !infield_fly_allowed(state)
        {
            errors.push(CommandError {
                segment_index: indexed[i].index,
                segment_text: indexed[i].text.clone(),
                kind: CommandErrorKind::Validation(ValidationError::InfieldFlyConditionsNotMet),
            });
        }
    }

    // 3) Too-many-outs cap.
    let out_count = resolved
        .iter()
        .filter(|r| {
            matches!(
                r,
                Some(Resolved::BatterOut { .. }) | Some(Resolved::RunnerOut { .. })
            )
        })
        .count();
    if out_count > 3 {
        errors.push(CommandError {
            segment_index: indexed[0].index,
            segment_text: indexed[0].text.clone(),
            kind: CommandErrorKind::Validation(ValidationError::TooManyOuts { count: out_count }),
        });
    }

    // 4) Mutual-exclusion rules that the grammar alone cannot enforce.
    let has_hit = resolved
        .iter()
        .any(|r| matches!(r, Some(Resolved::Hit { .. })));
    let has_fc = resolved
        .iter()
        .any(|r| matches!(r, Some(Resolved::Fc { .. })));
    let hit_count = resolved
        .iter()
        .filter(|r| matches!(r, Some(Resolved::Hit { .. })))
        .count();
    let fc_count = resolved
        .iter()
        .filter(|r| matches!(r, Some(Resolved::Fc { .. })))
        .count();

    // Multiple hits on one line → two at-bats, illegal.
    if hit_count > 1 {
        let mut seen = false;
        for (i, r) in resolved.iter().enumerate() {
            if let Some(Resolved::Hit { .. }) = r {
                if seen {
                    errors.push(CommandError {
                        segment_index: indexed[i].index,
                        segment_text: indexed[i].text.clone(),
                        kind: CommandErrorKind::Validation(
                            ValidationError::ControlMixedWithActions {
                                verb: "multiple hits on the same line".to_string(),
                            },
                        ),
                    });
                }
                seen = true;
            }
        }
    }

    if fc_count > 1 {
        let mut seen = false;
        for (i, r) in resolved.iter().enumerate() {
            if let Some(Resolved::Fc { .. }) = r {
                if seen {
                    errors.push(CommandError {
                        segment_index: indexed[i].index,
                        segment_text: indexed[i].text.clone(),
                        kind: CommandErrorKind::Validation(
                            ValidationError::ControlMixedWithActions {
                                verb: "multiple FCs on the same line".to_string(),
                            },
                        ),
                    });
                }
                seen = true;
            }
        }
    }

    if has_hit && has_fc {
        let i = resolved
            .iter()
            .position(|r| matches!(r, Some(Resolved::Fc { .. })))
            .unwrap();
        errors.push(CommandError {
            segment_index: indexed[i].index,
            segment_text: indexed[i].text.clone(),
            kind: CommandErrorKind::Validation(ValidationError::ControlMixedWithActions {
                verb: "FC cannot be combined with a hit".to_string(),
            }),
        });
    }

    // Hit + any out: contradictory (the batter is either safe or out).
    if has_hit
        && resolved.iter().any(|r| {
            matches!(
                r,
                Some(Resolved::BatterOut { .. }) | Some(Resolved::RunnerOut { .. })
            )
        })
    {
        let i = resolved
            .iter()
            .position(|r| {
                matches!(
                    r,
                    Some(Resolved::BatterOut { .. }) | Some(Resolved::RunnerOut { .. })
                )
            })
            .unwrap();
        errors.push(CommandError {
            segment_index: indexed[i].index,
            segment_text: indexed[i].text.clone(),
            kind: CommandErrorKind::Validation(ValidationError::ControlMixedWithActions {
                verb: "an out cannot be combined with a hit".to_string(),
            }),
        });
    }

    // FC + BatterOut: contradictory (the batter is either safe on FC or
    // out).
    if has_fc
        && resolved
            .iter()
            .any(|r| matches!(r, Some(Resolved::BatterOut { .. })))
    {
        let i = resolved
            .iter()
            .position(|r| matches!(r, Some(Resolved::BatterOut { .. })))
            .unwrap();
        errors.push(CommandError {
            segment_index: indexed[i].index,
            segment_text: indexed[i].text.clone(),
            kind: CommandErrorKind::Validation(ValidationError::ControlMixedWithActions {
                verb: "batter cannot be both out and safe on FC".to_string(),
            }),
        });
    }

    // FC + Advance: out of scope for this alpha; emit an explicit error so
    // callers are not surprised when it "silently" does nothing.
    if has_fc
        && resolved
            .iter()
            .any(|r| matches!(r, Some(Resolved::Advance { .. })))
    {
        let i = resolved
            .iter()
            .position(|r| matches!(r, Some(Resolved::Advance { .. })))
            .unwrap();
        errors.push(CommandError {
            segment_index: indexed[i].index,
            segment_text: indexed[i].text.clone(),
            kind: CommandErrorKind::Validation(ValidationError::ControlMixedWithActions {
                verb: "runner advance alongside FC is not supported in this version".to_string(),
            }),
        });
    }
}

fn infield_fly_allowed(state: &GameState) -> bool {
    state.outs < 2 && state.on_1b.is_some() && state.on_2b.is_some()
}

fn to_err(seg: &IndexedSegment, e: ValidationError) -> CommandError {
    CommandError {
        segment_index: seg.index,
        segment_text: seg.text.clone(),
        kind: CommandErrorKind::Validation(e),
    }
}

// ─── Coalescing resolved segments into EngineCommand values ─────────────────

fn build_commands(resolved: &[Option<Resolved>]) -> Result<Vec<EngineCommand>, Vec<CommandError>> {
    let items: Vec<&Resolved> = resolved.iter().flatten().collect();
    let mut out: Vec<EngineCommand> = Vec::new();

    let overrides: Vec<RunnerOverride> = items
        .iter()
        .filter_map(|r| match r {
            Resolved::Advance { subject, dest } => Some(RunnerOverride {
                order: *subject,
                dest: *dest,
            }),
            _ => None,
        })
        .collect();

    let mut def_outs: Vec<DefensiveOutRecord> = Vec::new();
    for r in &items {
        match r {
            Resolved::BatterOut { out: k, .. } => {
                def_outs.push(DefensiveOutRecord {
                    target: DefensivePlayTarget::Batter,
                    kind: batter_out_kind_to_defensive(k)?,
                });
            }
            Resolved::RunnerOut { subject, out: k } => {
                def_outs.push(DefensiveOutRecord {
                    target: DefensivePlayTarget::Runner(*subject),
                    kind: batter_out_kind_to_defensive(k)?,
                });
            }
            _ => {}
        }
    }

    let hit = items.iter().find_map(|r| match r {
        Resolved::Hit { kind, zone, .. } => Some((*kind, *zone)),
        _ => None,
    });
    let fc = items.iter().find_map(|r| match r {
        Resolved::Fc { fielder, base, .. } => Some((*fielder, *base)),
        _ => None,
    });

    let steals: Vec<EngineCommand> = items
        .iter()
        .filter_map(|r| match r {
            Resolved::Steal { subject, dest } => Some(EngineCommand::StealBase {
                order: *subject,
                dest: *dest,
            }),
            _ => None,
        })
        .collect();

    // Route 1: line contains a HIT.
    if let Some((kind, zone)) = hit {
        let hit_cmd = match kind {
            HitKind::Single => EngineCommand::Single {
                zone,
                runner_overrides: overrides,
            },
            HitKind::Double => EngineCommand::Double {
                zone,
                runner_overrides: overrides,
            },
            HitKind::Triple => EngineCommand::Triple {
                zone,
                runner_overrides: overrides,
            },
            HitKind::HomeRun => EngineCommand::HomeRun {
                zone,
                runner_overrides: overrides,
            },
        };
        out.extend(steals);
        out.push(hit_cmd);
        return Ok(out);
    }

    // Route 2: line contains a FIELDER'S CHOICE.
    if let Some((fielder, base)) = fc {
        let fc_adv = FielderChoiceAdvance {
            target: DefensivePlayTarget::Batter,
            fielder,
            reached_base: base,
        };
        out.push(EngineCommand::DefensivePlay(DefensivePlayCommand {
            outs: def_outs,
            safe_advances: vec![fc_adv],
        }));
        out.extend(steals);
        return Ok(out);
    }

    // Route 3: no hit, no FC — outs / steals / pitches.
    //
    // Defensive outs coalesce into a single DefensivePlay at the head of
    // the command stream (it has to be applied before the live
    // pitches/steals can touch the resulting state). After that, emit
    // pitches and steals in the original segment order so the at-bat log
    // preserves what the scorer actually typed.
    if !def_outs.is_empty() {
        out.push(EngineCommand::DefensivePlay(DefensivePlayCommand {
            outs: def_outs,
            safe_advances: vec![],
        }));
    }
    for r in &items {
        match r {
            Resolved::Pitch(pk) => out.push(EngineCommand::Pitch(pitch_to_engine(*pk))),
            Resolved::Steal { subject, dest } => out.push(EngineCommand::StealBase {
                order: *subject,
                dest: *dest,
            }),
            _ => {}
        }
    }

    Ok(out)
}

// ─── Conversion helpers ──────────────────────────────────────────────────────

fn status_to_game(sk: StatusKind) -> GameStatus {
    match sk {
        StatusKind::Regular => GameStatus::Regulation,
        StatusKind::Postponed => GameStatus::Postponed,
        StatusKind::Cancelled => GameStatus::Cancelled,
        StatusKind::Suspended => GameStatus::Suspended,
        StatusKind::Forfeited => GameStatus::Forfeited,
        StatusKind::Protested => GameStatus::Protested,
    }
}

fn pitch_to_engine(pk: PitchKind) -> Pitch {
    match pk {
        PitchKind::Ball => Pitch::Ball,
        PitchKind::CalledStrike => Pitch::CalledStrike,
        PitchKind::SwingingStrike => Pitch::SwingingStrike,
        PitchKind::Foul => Pitch::Foul,
        PitchKind::FoulBunt => Pitch::FoulBunt,
    }
}

fn batter_out_kind_to_defensive(
    kind: &BatterOutKind,
) -> Result<DefensiveOutKind, Vec<CommandError>> {
    match kind {
        BatterOutKind::Unassisted { fielder } => {
            Ok(DefensiveOutKind::UnassistedOut { fielder: *fielder })
        }
        BatterOutKind::GroundOut { fielders } => {
            let seq = FieldingSequence::new(fielders.clone()).map_err(|e| {
                // Token regexes already exclude < 2 fielders and values
                // outside 1..=9, so this branch should be unreachable; we
                // still surface a diagnostic rather than panicking.
                vec![CommandError {
                    segment_index: 0,
                    segment_text: format!("{fielders:?}"),
                    kind: CommandErrorKind::Validation(ValidationError::ControlMixedWithActions {
                        verb: format!("invalid fielding sequence: {e}"),
                    }),
                }]
            })?;
            Ok(DefensiveOutKind::GroundOut { sequence: seq })
        }
        BatterOutKind::FlyOut { fielder, foul } => Ok(DefensiveOutKind::FlyOut {
            fielder: *fielder,
            in_foul_territory: *foul,
        }),
        BatterOutKind::LineOut { fielder } => Ok(DefensiveOutKind::LineOut { fielder: *fielder }),
        BatterOutKind::InfieldFly { fielder } => {
            Ok(DefensiveOutKind::InfieldFly { fielder: *fielder })
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::commands::grammar::parse_line;
    use crate::models::game_state::BatterOrder;

    fn make_state(current: Option<BatterOrder>) -> GameState {
        let mut s = GameState::new();
        s.current_batter_order = current;
        s
    }

    /// Build `IndexedSegment`s the same way the facade does, so tests
    /// exercise the exact shape the validator will receive in production.
    fn indexed_for(line: &str) -> Vec<IndexedSegment> {
        let segments = parse_line(line).expect("parse");
        let texts: Vec<&str> = line.trim().split(',').map(|s| s.trim()).collect();
        segments
            .into_iter()
            .zip(texts.iter())
            .enumerate()
            .map(|(i, (seg, &text))| IndexedSegment {
                index: i + 1,
                text: text.to_string(),
                segment: seg,
            })
            .collect()
    }

    fn run(line: &str, state: &GameState) -> Result<Vec<EngineCommand>, Vec<CommandError>> {
        validate(indexed_for(line), state)
    }

    // ── Happy paths ──
    #[test]
    fn bare_pitch() {
        let st = make_state(Some(5));
        let cmds = run("b", &st).unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EngineCommand::Pitch(Pitch::Ball)));
    }

    #[test]
    fn hit_with_matching_subject() {
        let st = make_state(Some(5));
        let cmds = run("5 h lf", &st).unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            &cmds[0],
            EngineCommand::Single {
                zone: Some(FieldZone::LF),
                runner_overrides,
            } if runner_overrides.is_empty()
        ));
    }

    #[test]
    fn hit_implicit_subject() {
        let st = make_state(Some(5));
        let cmds = run("h", &st).unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            &cmds[0],
            EngineCommand::Single {
                zone: None,
                runner_overrides,
            } if runner_overrides.is_empty()
        ));
    }

    #[test]
    fn hit_with_mismatched_subject() {
        let st = make_state(Some(5));
        let errs = run("8 h", &st).expect_err("mismatch");
        assert_eq!(errs.len(), 1);
        assert!(matches!(
            errs[0].kind,
            CommandErrorKind::Validation(ValidationError::BatterSlotMismatch {
                given: 8,
                current: Some(5),
            })
        ));
    }

    #[test]
    fn hit_with_advances_any_order() {
        let mut st = make_state(Some(6));
        st.on_2b = Some(5);
        st.on_3b = Some(3);

        let a = run("6 h, 5 3b, 3 sc", &st).unwrap();
        let b = run("3 sc, 5 3b, 6 h", &st).unwrap();

        for cmds in [&a, &b] {
            assert_eq!(cmds.len(), 1);
            match &cmds[0] {
                EngineCommand::Single {
                    runner_overrides, ..
                } => {
                    assert_eq!(runner_overrides.len(), 2);
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn advance_without_trigger_is_error() {
        let mut st = make_state(Some(6));
        st.on_1b = Some(5);
        let errs = run("5 2b", &st).expect_err("no trigger");
        assert_eq!(errs.len(), 1);
        assert!(matches!(
            errs[0].kind,
            CommandErrorKind::Validation(ValidationError::AdvanceWithoutTrigger { order: 5 })
        ));
    }

    #[test]
    fn advance_of_absent_runner_is_error() {
        let st = make_state(Some(6));
        let errs = run("h, 5 2b", &st).expect_err("runner not on base");
        assert_eq!(errs.len(), 1);
        assert!(matches!(
            errs[0].kind,
            CommandErrorKind::Validation(ValidationError::RunnerNotOnBase { order: 5 })
        ));
    }

    // ── Triple-play invariance ──
    #[test]
    fn triple_play_invariant_under_reordering() {
        let mut st = make_state(Some(5));
        st.on_1b = Some(3);
        st.on_2b = Some(4);

        let a = run("5 l6, 3 64, 4 43", &st).unwrap();
        let b = run("3 64, 5 l6, 4 43", &st).unwrap();
        let c = run("4 43, 5 l6, 3 64", &st).unwrap();

        for cmds in [&a, &b, &c] {
            assert_eq!(cmds.len(), 1);
            match &cmds[0] {
                EngineCommand::DefensivePlay(p) => {
                    assert_eq!(p.outs.len(), 3);
                    assert!(p.safe_advances.is_empty());
                }
                _ => panic!("expected DefensivePlay"),
            }
        }
    }

    #[test]
    fn too_many_outs_rejected() {
        let mut st = make_state(Some(5));
        st.on_1b = Some(3);
        st.on_2b = Some(4);
        st.on_3b = Some(2);
        let errs = run("5 l6, 3 64, 4 43, 2 32", &st).expect_err("4 outs");
        assert!(errs.iter().any(|e| matches!(
            e.kind,
            CommandErrorKind::Validation(ValidationError::TooManyOuts { count: 4 })
        )));
    }

    // ── FC ──
    #[test]
    fn fc_with_runner_out() {
        let mut st = make_state(Some(5));
        st.on_1b = Some(4);
        let cmds = run("4 46, 5 o4 1b", &st).unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            EngineCommand::DefensivePlay(p) => {
                assert_eq!(p.outs.len(), 1);
                assert!(matches!(p.outs[0].target, DefensivePlayTarget::Runner(4)));
                assert_eq!(p.safe_advances.len(), 1);
                assert!(matches!(
                    p.safe_advances[0].target,
                    DefensivePlayTarget::Batter
                ));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn fc_and_hit_rejected() {
        let st = make_state(Some(5));
        let errs = run("5 h, 5 o4 1b", &st).expect_err("fc + hit");
        assert!(
            errs.iter()
                .any(|e| matches!(e.kind, CommandErrorKind::Validation(_)))
        );
    }

    // ── Pitch + steal ──
    #[test]
    fn pitch_and_steal_combined() {
        let mut st = make_state(Some(6));
        st.on_1b = Some(5);
        let cmds = run("b, 5 st 2b", &st).unwrap();
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn pitch_and_hit_rejected() {
        let st = make_state(Some(5));
        let errs = run("b, 5 h", &st).expect_err("pitch + hit");
        assert_eq!(errs.len(), 1);
    }

    // ── Control ──
    #[test]
    fn exit() {
        let st = make_state(None);
        let cmds = run("exit", &st).unwrap();
        assert!(matches!(cmds[0], EngineCommand::Exit));
    }

    #[test]
    fn status_regular() {
        let st = make_state(None);
        let cmds = run("regular", &st).unwrap();
        assert!(matches!(
            cmds[0],
            EngineCommand::SetStatus(GameStatus::Regulation)
        ));
    }

    #[test]
    fn control_mixed_with_action_rejected() {
        let st = make_state(Some(5));
        let errs = run("5 h, exit", &st).expect_err("mix");
        assert!(errs.iter().any(|e| matches!(
            e.kind,
            CommandErrorKind::Validation(ValidationError::ControlMixedWithActions { .. })
        )));
    }

    // ── Infield fly rule ──
    #[test]
    fn infield_fly_requires_runners_and_outs_lt_2() {
        let mut st = make_state(Some(5));
        st.outs = 0;
        st.on_1b = Some(3);
        st.on_2b = Some(4);
        assert!(run("5 if4", &st).is_ok());

        st.outs = 2;
        let errs = run("5 if4", &st).expect_err("ifly cond outs");
        assert!(errs.iter().any(|e| matches!(
            e.kind,
            CommandErrorKind::Validation(ValidationError::InfieldFlyConditionsNotMet)
        )));

        st.outs = 0;
        st.on_1b = None;
        let errs = run("5 if4", &st).expect_err("ifly cond no 1b");
        assert!(errs.iter().any(|e| matches!(
            e.kind,
            CommandErrorKind::Validation(ValidationError::InfieldFlyConditionsNotMet)
        )));
    }

    // ── Double steal ──
    #[test]
    fn double_steal() {
        let mut st = make_state(Some(6));
        st.on_1b = Some(5);
        st.on_2b = Some(3);
        let cmds = run("b, 5 st 2b, 3 st 3b", &st).unwrap();
        assert_eq!(cmds.len(), 3);
    }

    #[test]
    fn steal_of_absent_runner_rejected() {
        let st = make_state(Some(6));
        let errs = run("5 st 2b", &st).expect_err("no runner");
        assert_eq!(errs.len(), 1);
        assert!(matches!(
            errs[0].kind,
            CommandErrorKind::Validation(ValidationError::RunnerNotOnBase { order: 5 })
        ));
    }

    // ── Duplicate subject ──
    #[test]
    fn duplicate_subject_in_hit_and_advance() {
        let mut st = make_state(Some(5));
        st.on_1b = Some(5);
        let errs = run("5 h, 5 2b", &st).expect_err("dup subject");
        assert!(errs.iter().any(|e| matches!(
            e.kind,
            CommandErrorKind::Validation(ValidationError::DuplicateSubject { order: 5 })
        )));
    }

    // ── Multiple validation errors accumulated ──
    #[test]
    fn multiple_validation_errors_in_one_line() {
        let st = make_state(Some(5));
        let errs = run("8 h, 4 2b", &st).expect_err("two issues");
        assert!(errs.len() >= 2);
    }
}
