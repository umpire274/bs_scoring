//! Pure-function command application layer.
//!
//! Takes an `EngineCommand` and a mutable `GameState`, produces an `ApplyResult`
//! describing what happened (UI events, domain events, PA record, runner movements).
//! No DB access — the engine loop handles persistence.

use crate::commands::types::EngineCommand;
use crate::core::runner_logic;
use crate::db::runner_movements::RunnerMovementInsert;
use crate::models::events::{
    DomainEvent, OutRecordedData, PersistedEvent, StatusChangedData, StrikeoutKind,
};
use crate::models::game_state::GameState;
use crate::models::plate_appearance::{
    PlateAppearance, PlateAppearanceOutcome, PlateAppearanceStep,
};
use crate::models::runner::RunnerOverride;
use crate::models::types::{GameStatus, Pitch};
use crate::ui::events::UiEvent;

// ─── Result type ──────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct ApplyResult {
    pub events: Vec<UiEvent>,
    /// Events to persist in `game_events` (administrative / low-frequency).
    pub persisted: Vec<PersistedEvent>,
    /// Events to apply to the in-memory state (scoreboard) but NOT persist.
    pub applied: Vec<DomainEvent>,
    /// Optional: compact 1-row-per-batter record persisted at end of PA.
    pub plate_appearance: Option<PlateAppearance>,
    /// Runner movements to persist in `runner_movements` table.
    pub runner_movements: Vec<RunnerMovementInsert>,
    pub exit: bool,
    pub status_change: Option<GameStatus>,
    pub needs_next_at_bat: bool,
}

// ─── Main dispatch ────────────────────────────────────────────────────────────

pub fn apply_engine_command(state: &mut GameState, cmd: EngineCommand) -> ApplyResult {
    match cmd {
        EngineCommand::Exit => ApplyResult {
            exit: true,
            ..Default::default()
        },

        EngineCommand::PlayBall => ApplyResult {
            events: vec![UiEvent::Error(
                "PLAYBALL must be handled by the engine (DB-backed).".to_string(),
            )],
            ..Default::default()
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
                exit: true,
                status_change: Some(status),
                ..Default::default()
            }
        }

        EngineCommand::Pitch(pitch) => apply_pitch(state, pitch),

        EngineCommand::Single {
            zone,
            runner_overrides,
        } => apply_hit_command(
            state,
            PlateAppearanceOutcome::Single { zone },
            &runner_overrides,
        ),

        EngineCommand::Double {
            zone,
            runner_overrides,
        } => apply_hit_command(
            state,
            PlateAppearanceOutcome::Double { zone },
            &runner_overrides,
        ),

        EngineCommand::Triple {
            zone,
            runner_overrides,
        } => apply_hit_command(
            state,
            PlateAppearanceOutcome::Triple { zone },
            &runner_overrides,
        ),

        EngineCommand::HomeRun {
            zone,
            runner_overrides,
        } => apply_hit_command(
            state,
            PlateAppearanceOutcome::HomeRun { zone },
            &runner_overrides,
        ),

        EngineCommand::StealBase { order, dest } => apply_steal(state, order, dest),

        EngineCommand::Unknown(s) => ApplyResult {
            events: vec![UiEvent::Error(format!("Unknown command: {s}"))],
            ..Default::default()
        },
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extract the active batter context, or return an error result.
macro_rules! require_batter {
    ($state:expr) => {{
        let batter_id = match $state.current_batter_id {
            Some(id) => id,
            None => {
                return ApplyResult {
                    events: vec![UiEvent::Error(
                        "No active batter. Use PLAYBALL (or resume the game) first.".to_string(),
                    )],
                    ..Default::default()
                }
            }
        };
        let batter_order = match $state.current_batter_order {
            Some(o) => o,
            None => {
                return ApplyResult {
                    events: vec![UiEvent::Error(
                        "No active batter order in state.".to_string(),
                    )],
                    ..Default::default()
                }
            }
        };
        let pitcher_id = match $state.current_pitcher_id {
            Some(id) => id,
            None => {
                return ApplyResult {
                    events: vec![UiEvent::Error(
                        "No active pitcher in state (cannot record pitch).".to_string(),
                    )],
                    ..Default::default()
                }
            }
        };
        (batter_id, batter_order, pitcher_id)
    }};
}

fn build_pa_sequence(state: &GameState) -> Vec<PlateAppearanceStep> {
    state
        .pitch_count
        .sequence
        .iter()
        .cloned()
        .map(PlateAppearanceStep::Pitch)
        .collect()
}

// ─── Pitch ────────────────────────────────────────────────────────────────────

fn apply_pitch(state: &mut GameState, pitch: Pitch) -> ApplyResult {
    let (batter_id, batter_order, pitcher_id) = require_batter!(state);

    // Count AFTER applying this pitch
    let mut balls_after = state.pitch_count.balls;
    let mut strikes_after = state.pitch_count.strikes;

    match pitch {
        Pitch::Ball => balls_after = balls_after.saturating_add(1),
        Pitch::CalledStrike | Pitch::SwingingStrike => {
            strikes_after = strikes_after.saturating_add(1)
        }
        Pitch::Foul => {
            if strikes_after < 2 {
                strikes_after = strikes_after.saturating_add(1);
            }
        }
        Pitch::FoulBunt => strikes_after = strikes_after.saturating_add(1),
        Pitch::InPlay | Pitch::HittedBy => {}
    }

    let mut events_ui = vec![UiEvent::Line(format!("Pitch: {}", pitch))];
    let mut applied: Vec<DomainEvent> = vec![DomainEvent::PitchRecorded {
        pitcher_id,
        batter_id,
        pitch: pitch.clone(),
    }];

    let mut needs_next_at_bat = false;
    let mut plate_appearance: Option<PlateAppearance> = None;
    let mut walk_movements: Vec<RunnerMovementInsert> = vec![];

    let pitches_in_pa = state.pitch_count.sequence.len() as u32 + 1;
    let mut final_sequence = build_pa_sequence(state);
    final_sequence.push(PlateAppearanceStep::Pitch(pitch.clone()));

    // Helper to finalize a PA
    let finalize_pa = |outcome: PlateAppearanceOutcome, outs: u8| -> PlateAppearance {
        PlateAppearance {
            inning: state.inning,
            half: state.half,
            batter_id,
            batter_order,
            pitcher_id,
            pitches: pitches_in_pa,
            pitches_sequence: final_sequence.clone(),
            outcome,
            outs,
            runner_overrides: vec![],
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

        // Build walk movements using unified runner logic
        // We read the state BEFORE mutation (state hasn't been changed yet by applied events)
        {
            let half_str = state.half.as_str();
            let mk = |runner_id: Option<i64>,
                       border: u8,
                       start: &'static str,
                       end: &'static str,
                       scored: bool| {
                RunnerMovementInsert {
                    game_id: 0,
                    pa_seq: None,
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

        plate_appearance = Some(finalize_pa(PlateAppearanceOutcome::Walk, state.outs));
    }
    // Strikeout: 3 strikes before 4 balls
    else if strikes_after >= 3 && balls_after < 4 {
        let kind = match pitch {
            Pitch::CalledStrike => StrikeoutKind::Called,
            Pitch::SwingingStrike => StrikeoutKind::Swinging,
            Pitch::FoulBunt => StrikeoutKind::FoulBunt,
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
            PlateAppearanceOutcome::Strikeout(kind),
            state.outs.saturating_add(1),
        ));
    } else if matches!(pitch, Pitch::InPlay | Pitch::HittedBy) {
        events_ui.push(UiEvent::Line(
            "Note: X/H not implemented yet".to_string(),
        ));
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

// ─── Hit commands ─────────────────────────────────────────────────────────────

fn apply_hit_command(
    state: &mut GameState,
    outcome: PlateAppearanceOutcome,
    runner_overrides: &[RunnerOverride],
) -> ApplyResult {
    let (batter_id, batter_order, pitcher_id) = require_batter!(state);

    // Map outcome to terminal PlateAppearanceStep
    let final_step = match &outcome {
        PlateAppearanceOutcome::Single { .. } => PlateAppearanceStep::Single,
        PlateAppearanceOutcome::Double { .. } => PlateAppearanceStep::Double,
        PlateAppearanceOutcome::Triple { .. } => PlateAppearanceStep::Triple,
        PlateAppearanceOutcome::HomeRun { .. } => PlateAppearanceStep::HomeRun,
        _ => {
            return ApplyResult {
                events: vec![UiEvent::Error(
                    "Invalid hit outcome passed to apply_hit_command.".to_string(),
                )],
                ..Default::default()
            }
        }
    };

    let mut final_sequence = build_pa_sequence(state);
    final_sequence.push(final_step);

    // Validate runner overrides before touching state
    if let Err(msg) = runner_logic::validate_runner_overrides(state, batter_order, runner_overrides)
    {
        return ApplyResult {
            events: vec![UiEvent::Error(msg)],
            ..Default::default()
        };
    }

    let pitches_in_pa = final_sequence.len() as u32;

    let plate_appearance = PlateAppearance {
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

    let message = {
        let label = outcome.display_label();
        if let Some(z) = outcome.zone() {
            format!("{label} to {}", z.as_str())
        } else {
            label.to_string()
        }
    };

    ApplyResult {
        events: vec![UiEvent::Line(message)],
        persisted: vec![],
        applied: vec![DomainEvent::CountReset],
        plate_appearance: Some(plate_appearance),
        exit: false,
        status_change: None,
        needs_next_at_bat: true,
        ..Default::default()
    }
}

// ─── Steal ────────────────────────────────────────────────────────────────────

fn apply_steal(
    state: &mut GameState,
    order: u8,
    dest: crate::models::runner::RunnerDest,
) -> ApplyResult {
    use crate::models::runner::RunnerDest;

    // Validate: the runner must currently be on the expected source base.
    let expected_source: Option<u8> = match dest {
        RunnerDest::Second => Some(1),
        RunnerDest::Third => Some(2),
        RunnerDest::Score => Some(3),
        RunnerDest::First => None,
    };

    let Some(expected) = expected_source else {
        return ApplyResult {
            events: vec![UiEvent::Error(format!(
                "Steal to 1B is not valid (order {order})"
            ))],
            ..Default::default()
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
            ..Default::default()
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
        RunnerDest::Third => "3B",
        RunnerDest::Score => "HOME",
        RunnerDest::First => "1B",
    };
    let scored = dest == RunnerDest::Score;

    let log_msg = format!("[{order}] {first_name} {last_name} ruba la {end_base}");

    // Move the runner in state.
    match expected {
        1 => state.on_1b = None,
        2 => state.on_2b = None,
        3 => state.on_3b = None,
        _ => {}
    }
    match dest {
        RunnerDest::Second => state.on_2b = Some(order),
        RunnerDest::Third => state.on_3b = Some(order),
        RunnerDest::Score => match state.half {
            crate::models::types::HalfInning::Top => state.score.away += 1,
            crate::models::types::HalfInning::Bottom => state.score.home += 1,
        },
        RunnerDest::First => {}
    }

    let rm = RunnerMovementInsert {
        game_id: 0,
        pa_seq: None,
        game_event_id: None,
        inning: state.inning,
        half_inning: state.half.as_str().to_string(),
        runner_id: if runner_id != 0 {
            Some(runner_id)
        } else {
            None
        },
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
        ..Default::default()
    }
}

/// Resolve (runner_id, first_name, last_name) from batting order slot.
fn resolve_runner_identity(state: &GameState, order: u8) -> (i64, String, String) {
    if state.current_batter_order == Some(order) {
        return (
            state.current_batter_id.unwrap_or(0),
            state.current_batter_first_name.clone().unwrap_or_default(),
            state.current_batter_last_name.clone().unwrap_or_default(),
        );
    }
    (0, format!("#{order}"), String::new())
}
