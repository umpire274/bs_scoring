use crate::commands::types::EngineCommand;
use crate::models::events::{DomainEvent, OutRecordedData, PersistedEvent, StatusChangedData, StrikeoutKind};
use crate::models::game_state::GameState;
use crate::models::types::{GameStatus, Pitch};
use crate::ui::events::UiEvent;

#[derive(Default)]
pub struct ApplyResult {
    pub events: Vec<UiEvent>,
    /// Events to persist in `game_events` (administrative / low-frequency).
    pub persisted: Vec<PersistedEvent>,
    /// Events to apply to the in-memory state (scoreboard) but NOT persist.
    pub applied: Vec<DomainEvent>,
    /// Optional: compact 1-row-per-batter record persisted at end of PA.
    pub plate_appearance: Option<crate::models::plate_appearance::PlateAppearance>,
    /// Runner movements to persist in `runner_movements` table.
    pub runner_movements: Vec<crate::db::runner_movements::RunnerMovementInsert>,
    pub exit: bool,
    pub status_change: Option<GameStatus>,
    pub needs_next_at_bat: bool,
}

fn empty_result() -> ApplyResult {
    ApplyResult::default()
}

pub fn apply_engine_command(state: &mut GameState, cmd: EngineCommand) -> ApplyResult {
    match cmd {
        EngineCommand::Exit => ApplyResult {
            exit: true,
            ..empty_result()
        },

        // NOTE: PLAYBALL is handled in the engine layer because it requires DB lookups
        EngineCommand::PlayBall => ApplyResult {
            events: vec![UiEvent::Error(
                "PLAYBALL must be handled by the engine (DB-backed).".to_string(),
            )],
            ..empty_result()
        },

        EngineCommand::SetStatus(status) => {
            let msg = format!("{} Game set to {}.", status.icon(), status);

            ApplyResult {
                events: vec![UiEvent::Line(msg.clone())],
                persisted: vec![PersistedEvent {
                    inning: state.inning,
                    half: state.half,
                    event: DomainEvent::StatusChanged(StatusChangedData { to: status }),
                    description: msg,
                }],
                applied: vec![],
                plate_appearance: None,
                exit: true,
                status_change: Some(status),
                needs_next_at_bat: false,
                ..empty_result()
            }
        }

        // ✅ NEW: pitch command (0.6.7 baseline)
        EngineCommand::Pitch(pitch) => apply_pitch(state, pitch),

        EngineCommand::Single { zone, runner_overrides } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::Single { zone },
            "H",
            &runner_overrides,
        ),

        EngineCommand::Double { zone, runner_overrides } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::Double { zone },
            "2H",
            &runner_overrides,
        ),

        EngineCommand::Triple { zone, runner_overrides } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::Triple { zone },
            "3H",
            &runner_overrides,
        ),

        EngineCommand::HomeRun { zone, runner_overrides } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { zone },
            "HR",
            &runner_overrides,
        ),

        EngineCommand::Unknown(s) => ApplyResult {
            events: vec![UiEvent::Error(format!("Unknown command: {s}"))],
            ..empty_result()
        },

        EngineCommand::StealBase { order, dest } => apply_steal(state, order, dest),
    }
}

fn apply_pitch(state: &mut GameState, pitch: Pitch) -> ApplyResult {
    let Some(batter_id) = state.current_batter_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter. Use PLAYBALL (or resume the game) first.".to_string(),
            )],
            ..empty_result()
        };
    };

    let Some(batter_order) = state.current_batter_order else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter order in state.".to_string(),
            )],
            ..empty_result()
        };
    };

    let Some(pitcher_id) = state.current_pitcher_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active pitcher in state (cannot record pitch).".to_string(),
            )],
            ..empty_result()
        };
    };

    // Count BEFORE applying this pitch
    let balls_before = state.pitch_count.balls;
    let strikes_before = state.pitch_count.strikes;

    // Compute count AFTER applying this pitch (domain rules)
    let mut balls_after = balls_before;
    let mut strikes_after = strikes_before;

    match pitch {
        Pitch::Ball => {
            balls_after = balls_after.saturating_add(1);
        }
        Pitch::CalledStrike | Pitch::SwingingStrike => {
            strikes_after = strikes_after.saturating_add(1);
        }
        Pitch::Foul => {
            // counts as strike only if strikes < 2
            if strikes_after < 2 {
                strikes_after = strikes_after.saturating_add(1);
            }
        }
        Pitch::FoulBunt => {
            // ALWAYS counts as strike (can be K on strike 3)
            strikes_after = strikes_after.saturating_add(1);
        }
        Pitch::InPlay | Pitch::HittedBy => {
            // reserved for future use (no count changes here)
        }
    }

    let mut events_ui = vec![UiEvent::Line(format!("Pitch: {}", pitch))];
    let mut applied: Vec<DomainEvent> = vec![DomainEvent::PitchRecorded {
        pitcher_id,
        batter_id,
        pitch: pitch.clone(),
    }];

    let mut needs_next_at_bat = false;
    let mut plate_appearance: Option<crate::models::plate_appearance::PlateAppearance> = None;
    let mut walk_movements: Vec<crate::db::runner_movements::RunnerMovementInsert> = vec![];

    // This pitch counts as one more pitch in the PA
    let pitches_in_pa = state.pitch_count.sequence.len() as u32 + 1;

    // Shared final PA sequence = current sequence + this pitch
    let mut final_sequence = build_pa_sequence(state);
    final_sequence.push(crate::models::plate_appearance::PlateAppearanceStep::Pitch(
        pitch.clone(),
    ));

    // Helper closure to finalize a PA without duplicating struct construction
    let finalize_pa = |outcome: crate::models::plate_appearance::PlateAppearanceOutcome,
                       outs: u8|
     -> crate::models::plate_appearance::PlateAppearance {
        crate::models::plate_appearance::PlateAppearance {
            inning: state.inning,
            half: state.half,
            batter_id,
            batter_order,
            pitcher_id,
            pitches: pitches_in_pa,
            pitches_sequence: final_sequence.clone(),
            outcome,
            outs,
            runner_overrides: vec![], // pitch-by-pitch outcomes have no runner overrides
        }
    };

    // Walk: 4 balls and strikes < 3
    if balls_after >= 4 && strikes_after < 3 {
        applied.push(DomainEvent::WalkIssued { batter_id });

        let runner_jersey_no = state.current_batter_jersey_no.unwrap_or(0);
        let runner_first_name = state
            .current_batter_first_name
            .as_deref()
            .unwrap_or("-")
            .to_string();
        let runner_last_name = state
            .current_batter_last_name
            .as_deref()
            .unwrap_or("-")
            .to_string();

        applied.push(DomainEvent::RunnerToFirst {
            runner_id: batter_id,
            runner_jersey_no,
            runner_first_name,
            runner_last_name,
            batter_order,
        });

        applied.push(DomainEvent::CountReset);

        // Build runner_movements rows for all forced advancements.
        // State has NOT been mutated yet so we can read current bases.
        {
            use crate::db::runner_movements::RunnerMovementInsert;
            let half_str = match state.half {
                crate::models::types::HalfInning::Top    => "Top",
                crate::models::types::HalfInning::Bottom => "Bottom",
            };
            let mk = |runner_id: Option<i64>, border: u8, start: &'static str, end: &'static str, scored: bool| {
                RunnerMovementInsert {
                    game_id: 0,          // filled in by engine loop
                    pa_seq: None,        // linked to PA by engine after PA insert
                    game_event_id: None,
                    inning: state.inning,
                    half_inning: half_str.to_string(),
                    runner_id,
                    batter_order: border,
                    start_base: start,
                    end_base: end,
                    advancement_type: "walk",
                    is_out: false,
                    scored,
                    is_earned: true,
                }
            };
            // Bases loaded: runner on 3B scores
            if state.on_1b.is_some() && state.on_2b.is_some() && state.on_3b.is_some() {
                let r3 = state.on_3b.unwrap_or(0);
                walk_movements.push(mk(None, r3, "3B", "HOME", true));
            }
            if state.on_1b.is_some() && state.on_2b.is_some() {
                let r2 = state.on_2b.unwrap_or(0);
                walk_movements.push(mk(None, r2, "2B", "3B", false));
            }
            if state.on_1b.is_some() {
                let r1 = state.on_1b.unwrap_or(0);
                walk_movements.push(mk(None, r1, "1B", "2B", false));
            }
            // Batter to 1B
            walk_movements.push(mk(Some(batter_id), batter_order, "BAT", "1B", false));
        }

        events_ui.push(UiEvent::Line("BB: batter to 1B".to_string()));
        needs_next_at_bat = true;

        plate_appearance = Some(finalize_pa(
            crate::models::plate_appearance::PlateAppearanceOutcome::Walk,
            state.outs,
        ));
    }
    // Strikeout: 3 strikes before 4 balls
    else if strikes_after >= 3 && balls_after < 4 {
        let kind = match pitch {
            Pitch::CalledStrike => StrikeoutKind::Called,
            Pitch::SwingingStrike => StrikeoutKind::Swinging,
            Pitch::FoulBunt => StrikeoutKind::FoulBunt,
            // safety fallback
            _ => StrikeoutKind::Called,
        };

        applied.push(DomainEvent::Strikeout {
            batter_id,
            kind: kind.clone(),
        });

        applied.push(DomainEvent::OutRecorded(OutRecordedData {
            outs_before: state.outs,
            outs_after: state.outs.saturating_add(1),
        }));

        applied.push(DomainEvent::CountReset);

        events_ui.push(UiEvent::Line("K: batter out".to_string()));
        needs_next_at_bat = true;

        plate_appearance = Some(finalize_pa(
            crate::models::plate_appearance::PlateAppearanceOutcome::Strikeout(kind),
            state.outs.saturating_add(1),
        ));
    } else if matches!(pitch, Pitch::InPlay | Pitch::HittedBy) {
        events_ui.push(UiEvent::Line("Note: X/H not implemented yet".to_string()));
    }

    ApplyResult {
        events: events_ui,
        persisted: vec![],
        applied,
        plate_appearance,
        runner_movements: walk_movements,
        exit: false,
        status_change: None,
        needs_next_at_bat,
    }
}

fn apply_hit_command(
    state: &mut GameState,
    outcome: crate::models::plate_appearance::PlateAppearanceOutcome,
    label: &str,
    runner_overrides: &[crate::models::runner::RunnerOverride],
) -> ApplyResult {
    let Some(batter_id) = state.current_batter_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter. Use PLAYBALL (or resume the game) first.".to_string(),
            )],
            ..empty_result()
        };
    };

    let Some(batter_order) = state.current_batter_order else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter order in state.".to_string(),
            )],
            ..empty_result()
        };
    };

    let Some(pitcher_id) = state.current_pitcher_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active pitcher in state (cannot record hit).".to_string(),
            )],
            ..empty_result()
        };
    };

    let final_step = match &outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::Single
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::Double
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::Triple
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::HomeRun
        }
        _ => {
            return ApplyResult {
                events: vec![UiEvent::Error(
                    "Invalid hit outcome passed to apply_hit_command.".to_string(),
                )],
                ..empty_result()
            };
        }
    };

    let mut final_sequence = build_pa_sequence(state);
    final_sequence.push(final_step);

    // Validate runner overrides before touching state:
    // 1. No two overrides may target the same destination base.
    // 2. No override may target a base already occupied by a runner who is
    //    NOT being moved (i.e. not present in the overrides list).
    if let Err(msg) = validate_runner_overrides(state, batter_order, runner_overrides) {
        return ApplyResult {
            events: vec![UiEvent::Error(msg)],
            ..empty_result()
        };
    }

    let pitches_in_pa = final_sequence.len() as u32;

    let plate_appearance = crate::models::plate_appearance::PlateAppearance {
        inning: state.inning,
        half: state.half,
        batter_id,
        batter_order,
        pitcher_id,
        pitches: pitches_in_pa,
        pitches_sequence: final_sequence,
        outcome: outcome.clone(),
        outs: state.outs,
        runner_overrides: runner_overrides.to_vec(),
    };

    let zone = match &outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { zone }
        | crate::models::plate_appearance::PlateAppearanceOutcome::Double { zone }
        | crate::models::plate_appearance::PlateAppearanceOutcome::Triple { zone }
        | crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { zone } => *zone,
        _ => None,
    };

    let human_label = match label {
        "H" => "Single",
        "2H" => "Double",
        "3H" => "Triple",
        "HR" => "Home run",
        _ => label,
    };

    let message = if let Some(z) = zone {
        format!("{human_label} to {}", z.as_str())
    } else {
        human_label.to_string()
    };

    ApplyResult {
        events: vec![UiEvent::Line(message)],
        persisted: vec![],
        applied: vec![DomainEvent::CountReset],
        plate_appearance: Some(plate_appearance),
        exit: false,
        status_change: None,
        needs_next_at_bat: true,
        ..empty_result()
    }
}

fn build_pa_sequence(
    state: &GameState,
) -> Vec<crate::models::plate_appearance::PlateAppearanceStep> {
    state
        .pitch_count
        .sequence
        .iter()
        .cloned()
        .map(crate::models::plate_appearance::PlateAppearanceStep::Pitch)
        .collect()
}

// ─── Steal ────────────────────────────────────────────────────────────────────

fn apply_steal(
    state: &mut GameState,
    order: u8,
    dest: crate::models::runner::RunnerDest,
) -> ApplyResult {
    use crate::db::runner_movements::RunnerMovementInsert;
    use crate::models::runner::RunnerDest;

    // Validate: the runner must currently be on the expected source base.
    let expected_source: Option<u8> = match dest {
        RunnerDest::Second => Some(1),
        RunnerDest::Third  => Some(2),
        RunnerDest::Score  => Some(3),
        RunnerDest::First  => None, // stealing first is not a valid play
    };

    let Some(expected) = expected_source else {
        return ApplyResult {
            events: vec![UiEvent::Error(format!(
                "Steal to 1B is not valid (order {order})"
            ))],
            ..empty_result()
        };
    };

    let on_expected = match expected {
        1 => state.on_1b == Some(order),
        2 => state.on_2b == Some(order),
        3 => state.on_3b == Some(order),
        _ => false,
    };

    if !on_expected {
        return ApplyResult {
            events: vec![UiEvent::Error(format!(
                "Runner {order} is not on {}B — cannot steal {}",
                expected, dest,
            ))],
            ..empty_result()
        };
    }

    let (runner_id, first_name, last_name) = resolve_runner_identity(state, order);

    let start_base: &'static str = match expected {
        1 => "1B",
        2 => "2B",
        _ => "3B",
    };
    let end_base: &'static str = match dest {
        RunnerDest::Second => "2B",
        RunnerDest::Third  => "3B",
        RunnerDest::Score  => "HOME",
        RunnerDest::First  => "1B",
    };
    let scored = dest == RunnerDest::Score;

    let log_msg = format!(
        "[{order}] {first_name} {last_name} ruba la {}",
        end_base,
    );

    // Move the runner in state.
    match expected {
        1 => state.on_1b = None,
        2 => state.on_2b = None,
        3 => state.on_3b = None,
        _ => {}
    }
    match dest {
        RunnerDest::Second => state.on_2b = Some(order),
        RunnerDest::Third  => state.on_3b = Some(order),
        RunnerDest::Score  => {
            match state.half {
                crate::models::types::HalfInning::Top    => state.score.away += 1,
                crate::models::types::HalfInning::Bottom => state.score.home += 1,
            }
        }
        RunnerDest::First => {}
    }

    // Steal is persisted as a runner_movement (not a game_event).
    // The engine loop will use game_event_id = None + pa_seq = None to signal
    // this is a standalone movement; the DB row id is assigned at insert time.
    let rm = RunnerMovementInsert {
        game_id: 0,           // filled in by engine loop
        pa_seq: None,         // not part of a PA
        game_event_id: None,  // standalone runner movement
        inning: state.inning,
        half_inning: match state.half {
            crate::models::types::HalfInning::Top    => "Top".to_string(),
            crate::models::types::HalfInning::Bottom => "Bottom".to_string(),
        },
        runner_id: if runner_id != 0 { Some(runner_id) } else { None },
        batter_order: order,
        start_base,
        end_base,
        advancement_type: "steal",
        is_out: false,
        scored,
        is_earned: true,
    };

    ApplyResult {
        events: vec![UiEvent::Line(log_msg)],
        runner_movements: vec![rm],
        ..empty_result()
    }
}

/// Resolve (runner_id, first_name, last_name) from batting order slot.
/// Checks current batter first, then falls back to placeholder.
fn resolve_runner_identity(state: &GameState, order: u8) -> (i64, String, String) {
    // If the current batter happens to be this runner (unusual but possible
    // in edge cases), use their data.
    if state.current_batter_order == Some(order) {
        return (
            state.current_batter_id.unwrap_or(0),
            state.current_batter_first_name.clone().unwrap_or_default(),
            state.current_batter_last_name.clone().unwrap_or_default(),
        );
    }
    // In future we'll look up the lineup; for now a lightweight placeholder.
    (0, format!("#{order}"), String::new())
}

// ─── Override validation ──────────────────────────────────────────────────────

/// Validate runner overrides before applying a hit, returning an error message
/// if the overrides would produce an inconsistent state.
///
/// Checks performed:
/// 1. No two overrides (including the batter's implicit destination) target the
///    same base — this would silently drop one runner.
/// 2. No override sends a runner to a base already occupied by a runner who is
///    *not* being moved in this play — that runner would be silently evicted.
fn validate_runner_overrides(
    state: &GameState,
    _batter_order: u8,
    overrides: &[crate::models::runner::RunnerOverride],
) -> Result<(), String> {
    use crate::models::runner::RunnerDest;
    use std::collections::HashSet;

    // Collect all destination bases claimed by overrides (excluding Score).
    let mut claimed: HashSet<u8> = HashSet::new();

    for ro in overrides {
        let dest_base: Option<u8> = match ro.dest {
            RunnerDest::First  => Some(1),
            RunnerDest::Second => Some(2),
            RunnerDest::Third  => Some(3),
            RunnerDest::Score  => None, // multiple runners can score
        };
        if let Some(b) = dest_base
            && !claimed.insert(b) {
                return Err(format!(
                    "Two runners cannot end up on the same base ({}B)",
                    b
                ));
        }
    }

    // Check that no override destination is occupied by a runner who is NOT
    // in the overrides list (i.e. not being moved by this play).
    let moved_orders: HashSet<u8> = overrides.iter().map(|r| r.order).collect();

    let check_base = |base_occupant: Option<u8>, base_no: u8| -> Result<(), String> {
        if let Some(occupant) = base_occupant
            && claimed.contains(&base_no) && !moved_orders.contains(&occupant) {
                return Err(format!(
                    "Runner {} on {}B would be overwritten — add an explicit override for them",
                    occupant, base_no
                ));
        }
        Ok(())
    };

    check_base(state.on_1b, 1)?;
    check_base(state.on_2b, 2)?;
    check_base(state.on_3b, 3)?;

    Ok(())
}
