//! Pure-function command application layer.
//!
//! Takes an `EngineCommand` and a mutable `GameState`, produces an `ApplyResult`
//! describing what happened (UI events, domain events, PA record, runner movements).
//! No DB access — the engine loop handles persistence.

use crate::db::runner_movements::RunnerMovementInsert;
use crate::engine::commands::types::EngineCommand;
use crate::engine::runners::add_runs_to_score;
use crate::engine::scoring::BatterOutType;
use crate::engine::scoring::batter_outs::{
    DefensiveOutKind, DefensivePlayCommand, DefensivePlayTarget,
};
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
use crate::{BatterOrder, HalfInning, RunnerDest};
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

        EngineCommand::BatterOut { order, out_type } => {
            apply_batter_out_command(state, order, out_type)
        }

        EngineCommand::StealBase { order, dest } => apply_steal(state, order, dest),

        EngineCommand::DefensivePlay(play) => apply_defensive_play_command(state, play),
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
                };
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
                };
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
                };
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

fn build_pa_sequence_with_terminal_step(
    state: &GameState,
    final_step: PlateAppearanceStep,
) -> Vec<PlateAppearanceStep> {
    let mut seq = build_pa_sequence(state);
    seq.push(final_step);
    seq
}

fn batter_out_terminal_step(out_type: &BatterOutType) -> PlateAppearanceStep {
    match out_type {
        BatterOutType::UnassistedOut { fielder } => {
            PlateAppearanceStep::UnassistedOut { fielder: *fielder }
        }

        BatterOutType::GroundOut { sequence } => PlateAppearanceStep::GroundOut {
            sequence: sequence.as_hyphenated_string(),
        },
        BatterOutType::FlyOut {
            fielder,
            in_foul_territory,
        } => PlateAppearanceStep::FlyOut {
            fielder: *fielder,
            in_foul_territory: *in_foul_territory,
        },
        BatterOutType::LineOut { fielder } => PlateAppearanceStep::LineOut { fielder: *fielder },
        BatterOutType::InfieldFly { fielder } => {
            PlateAppearanceStep::InfieldFly { fielder: *fielder }
        }
    }
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

    let final_sequence =
        build_pa_sequence_with_terminal_step(state, PlateAppearanceStep::Pitch(pitch.clone()));
    let pitches_in_pa = final_sequence.len() as u32;

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
            };
        }
    };

    let final_sequence = build_pa_sequence_with_terminal_step(state, final_step);

    // Validate runner overrides before touching state
    if let Err(msg) =
        crate::engine::runners::validate_runner_overrides(state, batter_order, runner_overrides)
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

    // Remove runner from the source base.
    match expected {
        1 => state.on_1b = None,
        2 => state.on_2b = None,
        3 => state.on_3b = None,
        _ => {}
    }

    // Apply destination and scoring.
    match dest {
        RunnerDest::Second => state.on_2b = Some(order),
        RunnerDest::Third => state.on_3b = Some(order),
        RunnerDest::Score => add_runs_to_score(state, 1),
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

fn apply_batter_out_command(
    state: &mut GameState,
    order: BatterOrder,
    out_type: BatterOutType,
) -> ApplyResult {
    let (batter_id, batter_order, pitcher_id) = require_batter!(state);

    if order != batter_order {
        return ApplyResult {
            events: vec![UiEvent::Error(format!(
                "Batter mismatch: expected #{}, found #{}.",
                batter_order, order
            ))],
            ..Default::default()
        };
    }

    if let BatterOutType::InfieldFly { .. } = out_type {
        let valid_infield_fly = state.outs < 2 && state.on_1b.is_some() && state.on_2b.is_some();

        if !valid_infield_fly {
            return ApplyResult {
                events: vec![UiEvent::Error("Invalid infield fly situation.".to_string())],
                ..Default::default()
            };
        }
    }

    let mut events_ui: Vec<UiEvent> = Vec::new();
    let mut applied: Vec<DomainEvent> = Vec::new();

    let final_step = batter_out_terminal_step(&out_type);
    let final_sequence = build_pa_sequence_with_terminal_step(state, final_step);
    let pitches_in_pa = final_sequence.len() as u32;

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

    let (description, pa_outcome) = match &out_type {
        BatterOutType::UnassistedOut { fielder } => (
            format!("Batter #{} out unassisted by {}.", batter_order, fielder),
            PlateAppearanceOutcome::UnassistedOut { fielder: *fielder },
        ),

        BatterOutType::GroundOut { sequence } => (
            format!(
                "Batter #{} grounded out {}.",
                batter_order,
                sequence.as_hyphenated_string()
            ),
            PlateAppearanceOutcome::GroundOut {
                sequence: sequence.as_hyphenated_string(),
            },
        ),

        BatterOutType::FlyOut {
            fielder,
            in_foul_territory: false,
        } => (
            format!("Batter #{} flied out to F{}.", batter_order, fielder),
            PlateAppearanceOutcome::FlyOut {
                fielder: *fielder,
                in_foul_territory: false,
            },
        ),

        BatterOutType::FlyOut {
            fielder,
            in_foul_territory: true,
        } => (
            format!("Batter #{} fouled out to F{}.", batter_order, fielder),
            PlateAppearanceOutcome::FlyOut {
                fielder: *fielder,
                in_foul_territory: true,
            },
        ),

        BatterOutType::LineOut { fielder } => (
            format!("Batter #{} lined out to L{}.", batter_order, fielder),
            PlateAppearanceOutcome::LineOut { fielder: *fielder },
        ),

        BatterOutType::InfieldFly { fielder } => (
            format!(
                "Batter #{} out on infield fly to IF{}.",
                batter_order, fielder
            ),
            PlateAppearanceOutcome::InfieldFly { fielder: *fielder },
        ),
    };

    applied.push(DomainEvent::OutRecorded(OutRecordedData {
        outs_before: state.outs,
        outs_after: state.outs.saturating_add(1),
    }));

    applied.push(DomainEvent::CountReset);

    events_ui.push(UiEvent::Line(description));

    ApplyResult {
        events: events_ui,
        persisted: vec![],
        applied,
        plate_appearance: Some(finalize_pa(pa_outcome, state.outs.saturating_add(1))),
        runner_movements: vec![],
        exit: false,
        status_change: None,
        needs_next_at_bat: true,
    }
}

fn apply_defensive_play_command(state: &mut GameState, play: DefensivePlayCommand) -> ApplyResult {
    let (batter_id, batter_order, pitcher_id) = require_batter!(state);

    let normalize_target = |target: &DefensivePlayTarget| -> DefensivePlayTarget {
        match target {
            DefensivePlayTarget::Runner(order) if *order == batter_order => {
                DefensivePlayTarget::Batter
            }
            other => other.clone(),
        }
    };

    let normalized_outs: Vec<(DefensivePlayTarget, DefensiveOutKind)> = play
        .outs
        .iter()
        .map(|out| (normalize_target(&out.target), out.kind.clone()))
        .collect();

    let normalized_fc: Vec<(DefensivePlayTarget, u8, RunnerDest)> = play
        .safe_advances
        .iter()
        .map(|fc| (normalize_target(&fc.target), fc.fielder, fc.reached_base))
        .collect();

    let batter_out_count = normalized_outs
        .iter()
        .filter(|(target, _)| matches!(target, DefensivePlayTarget::Batter))
        .count();

    let batter_fc_count = normalized_fc
        .iter()
        .filter(|(target, _, _)| matches!(target, DefensivePlayTarget::Batter))
        .count();

    if batter_out_count > 1 {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "Invalid defensive play: multiple batter outs found.".to_string(),
            )],
            ..Default::default()
        };
    }

    if batter_fc_count > 1 {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "Invalid defensive play: multiple batter fielder's choices found.".to_string(),
            )],
            ..Default::default()
        };
    }

    if batter_out_count > 0 && batter_fc_count > 0 {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "Invalid defensive play: batter cannot be both out and safe.".to_string(),
            )],
            ..Default::default()
        };
    }

    if batter_out_count == 0 && batter_fc_count == 0 {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "Invalid defensive play: batter result is missing.".to_string(),
            )],
            ..Default::default()
        };
    }

    // Infield Fly validation must happen here too, because commands like `if4`
    // are parsed as DefensivePlayCommand, not legacy BatterOut.
    for (target, kind) in &normalized_outs {
        if matches!(target, DefensivePlayTarget::Batter)
            && matches!(kind, DefensiveOutKind::InfieldFly { .. })
        {
            let valid_infield_fly =
                state.outs < 2 && state.on_1b.is_some() && state.on_2b.is_some();

            if !valid_infield_fly {
                return ApplyResult {
                    events: vec![UiEvent::Error("Invalid infield fly situation.".to_string())],
                    ..Default::default()
                };
            }
        }
    }

    let outs_before = state.outs;
    let outs_gained = normalized_outs.len() as u8;
    let outs_after = state.outs.saturating_add(outs_gained);

    let batter_out = normalized_outs
        .iter()
        .find(|(target, _)| matches!(target, DefensivePlayTarget::Batter));

    let batter_fc = normalized_fc
        .iter()
        .find(|(target, _, _)| matches!(target, DefensivePlayTarget::Batter));

    let final_step = if let Some((_, out_kind)) = batter_out {
        match out_kind {
            DefensiveOutKind::UnassistedOut { fielder } => {
                PlateAppearanceStep::UnassistedOut { fielder: *fielder }
            }
            DefensiveOutKind::GroundOut { sequence } => PlateAppearanceStep::GroundOut {
                sequence: sequence.as_hyphenated_string(),
            },
            DefensiveOutKind::FlyOut {
                fielder,
                in_foul_territory,
            } => PlateAppearanceStep::FlyOut {
                fielder: *fielder,
                in_foul_territory: *in_foul_territory,
            },
            DefensiveOutKind::LineOut { fielder } => {
                PlateAppearanceStep::LineOut { fielder: *fielder }
            }
            DefensiveOutKind::InfieldFly { fielder } => {
                PlateAppearanceStep::InfieldFly { fielder: *fielder }
            }
        }
    } else {
        let (_, fielder, reached_base) = batter_fc.expect("batter_fc already validated");
        PlateAppearanceStep::FieldersChoice {
            fielder: *fielder,
            reached_base: *reached_base,
        }
    };

    let final_sequence = build_pa_sequence_with_terminal_step(state, final_step);
    let pitches_in_pa = final_sequence.len() as u32;

    let pa_outcome = if let Some((_, out_kind)) = batter_out {
        match out_kind {
            DefensiveOutKind::UnassistedOut { fielder } => {
                PlateAppearanceOutcome::UnassistedOut { fielder: *fielder }
            }
            DefensiveOutKind::GroundOut { sequence } => PlateAppearanceOutcome::GroundOut {
                sequence: sequence.as_hyphenated_string(),
            },
            DefensiveOutKind::FlyOut {
                fielder,
                in_foul_territory,
            } => PlateAppearanceOutcome::FlyOut {
                fielder: *fielder,
                in_foul_territory: *in_foul_territory,
            },
            DefensiveOutKind::LineOut { fielder } => {
                PlateAppearanceOutcome::LineOut { fielder: *fielder }
            }
            DefensiveOutKind::InfieldFly { fielder } => {
                PlateAppearanceOutcome::InfieldFly { fielder: *fielder }
            }
        }
    } else {
        let (_, fielder, reached_base) = batter_fc.expect("batter_fc already validated");
        PlateAppearanceOutcome::FieldersChoice {
            fielder: *fielder,
            reached_base: *reached_base,
        }
    };

    let half_str = match state.half {
        HalfInning::Top => "Top",
        HalfInning::Bottom => "Bottom",
    };

    let mut runner_movements: Vec<RunnerMovementInsert> = Vec::new();

    for (target, out_kind) in &normalized_outs {
        let (runner_id, order, start_base) = match target {
            DefensivePlayTarget::Batter => (Some(batter_id), batter_order, "BAT"),
            DefensivePlayTarget::Runner(order) => {
                (None, *order, runner_start_base_label(state, *order))
            }
        };

        let advancement_type = match out_kind {
            DefensiveOutKind::UnassistedOut { .. } => "unassisted_out",
            DefensiveOutKind::GroundOut { .. } => "ground_out",
            DefensiveOutKind::FlyOut { .. } => "fly_out",
            DefensiveOutKind::LineOut { .. } => "line_out",
            DefensiveOutKind::InfieldFly { .. } => "infield_fly",
        };

        runner_movements.push(RunnerMovementInsert {
            game_id: 0,
            pa_seq: None,
            game_event_id: None,
            inning: state.inning,
            half_inning: half_str.to_string(),
            runner_id,
            batter_order: order,
            start_base,
            end_base: "OUT",
            advancement_type,
            is_out: true,
            scored: false,
            is_earned: true,
        });
    }

    for (target, _fielder, reached_base) in &normalized_fc {
        let (runner_id, order, start_base) = match target {
            DefensivePlayTarget::Batter => (Some(batter_id), batter_order, "BAT"),
            DefensivePlayTarget::Runner(order) => {
                (None, *order, runner_start_base_label(state, *order))
            }
        };

        runner_movements.push(RunnerMovementInsert {
            game_id: 0,
            pa_seq: None,
            game_event_id: None,
            inning: state.inning,
            half_inning: half_str.to_string(),
            runner_id,
            batter_order: order,
            start_base,
            end_base: runner_dest_to_base_label(*reached_base),
            advancement_type: "fielders_choice",
            is_out: false,
            scored: matches!(reached_base, RunnerDest::Score),
            is_earned: true,
        });
    }

    let mut events_ui: Vec<UiEvent> = Vec::new();

    for (target, out_kind) in &normalized_outs {
        let line = match (target, out_kind) {
            (DefensivePlayTarget::Batter, DefensiveOutKind::UnassistedOut { fielder }) => {
                format!("Batter out unassisted by {}.", fielder)
            }
            (DefensivePlayTarget::Batter, DefensiveOutKind::GroundOut { sequence }) => {
                format!("Batter grounded out {}.", sequence.as_hyphenated_string())
            }
            (
                DefensivePlayTarget::Batter,
                DefensiveOutKind::FlyOut {
                    fielder,
                    in_foul_territory: false,
                },
            ) => format!("Batter flied out to F{}.", fielder),
            (
                DefensivePlayTarget::Batter,
                DefensiveOutKind::FlyOut {
                    fielder,
                    in_foul_territory: true,
                },
            ) => format!("Batter fouled out to FF{}.", fielder),
            (DefensivePlayTarget::Batter, DefensiveOutKind::LineOut { fielder }) => {
                format!("Batter lined out to L{}.", fielder)
            }
            (DefensivePlayTarget::Batter, DefensiveOutKind::InfieldFly { fielder }) => {
                format!("Batter out on infield fly to IF{}.", fielder)
            }

            (DefensivePlayTarget::Runner(order), DefensiveOutKind::UnassistedOut { fielder }) => {
                format!("Runner #{} out unassisted by {}.", order, fielder)
            }
            (DefensivePlayTarget::Runner(order), DefensiveOutKind::GroundOut { sequence }) => {
                format!(
                    "Runner #{} out on {}.",
                    order,
                    sequence.as_hyphenated_string()
                )
            }
            (
                DefensivePlayTarget::Runner(order),
                DefensiveOutKind::FlyOut {
                    fielder,
                    in_foul_territory: false,
                },
            ) => format!("Runner #{} out on F{}.", order, fielder),
            (
                DefensivePlayTarget::Runner(order),
                DefensiveOutKind::FlyOut {
                    fielder,
                    in_foul_territory: true,
                },
            ) => format!("Runner #{} out on FF{}.", order, fielder),
            (DefensivePlayTarget::Runner(order), DefensiveOutKind::LineOut { fielder }) => {
                format!("Runner #{} out on L{}.", order, fielder)
            }
            (DefensivePlayTarget::Runner(order), DefensiveOutKind::InfieldFly { fielder }) => {
                format!("Runner #{} out on IF{}.", order, fielder)
            }
        };

        events_ui.push(UiEvent::Line(line));
    }

    for (target, fielder, reached_base) in &normalized_fc {
        let line = match target {
            DefensivePlayTarget::Batter => format!(
                "Batter safe on fielder's choice by {} to {}.",
                fielder,
                runner_dest_to_base_label(*reached_base)
            ),
            DefensivePlayTarget::Runner(order) => format!(
                "Runner #{} safe on fielder's choice by {} to {}.",
                order,
                fielder,
                runner_dest_to_base_label(*reached_base)
            ),
        };

        events_ui.push(UiEvent::Line(line));
    }

    let mut applied: Vec<DomainEvent> = Vec::new();

    if outs_gained > 0 {
        applied.push(DomainEvent::OutRecorded(OutRecordedData {
            outs_before,
            outs_after,
        }));
    }

    applied.push(DomainEvent::CountReset);

    let plate_appearance = PlateAppearance {
        inning: state.inning,
        half: state.half,
        batter_id,
        batter_order,
        pitcher_id,
        pitches: pitches_in_pa,
        pitches_sequence: final_sequence,
        outcome: pa_outcome,
        outs: outs_after,
        runner_overrides: vec![],
    };

    ApplyResult {
        events: events_ui,
        persisted: vec![],
        applied,
        plate_appearance: Some(plate_appearance),
        runner_movements,
        exit: false,
        status_change: None,
        needs_next_at_bat: true,
    }
}

fn runner_dest_to_base_label(dest: RunnerDest) -> &'static str {
    match dest {
        RunnerDest::First => "1B",
        RunnerDest::Second => "2B",
        RunnerDest::Third => "3B",
        RunnerDest::Score => "HOME",
    }
}

pub fn serialize_runner_dest(dest: RunnerDest) -> &'static str {
    match dest {
        RunnerDest::First => "1B",
        RunnerDest::Second => "2B",
        RunnerDest::Third => "3B",
        RunnerDest::Score => "HOME",
    }
}

pub fn apply_batter_fielders_choice(
    state: &mut GameState,
    batter_order: u8,
    reached_base: RunnerDest,
) {
    match reached_base {
        RunnerDest::First => {
            // Force existing runners only as needed to free 1B.
            if state.on_1b.is_some() {
                if state.on_2b.is_some() {
                    if state.on_3b.is_some() {
                        // Runner on 3B is forced home.
                        state.on_3b = None;
                    }
                    // Runner on 2B is forced to 3B.
                    state.on_3b = state.on_2b;
                }
                // Runner on 1B is forced to 2B.
                state.on_2b = state.on_1b;
            }

            state.on_1b = Some(batter_order);
        }

        RunnerDest::Second => {
            // Clear destination and place batter there.
            // No automatic force-chain here beyond the target base:
            // caller is making an explicit scoring decision.
            if state.on_2b.is_some() {
                if state.on_3b.is_some() {
                    state.on_3b = None;
                }
                state.on_3b = state.on_2b;
            }

            state.on_2b = Some(batter_order);
        }

        RunnerDest::Third => {
            if state.on_3b.is_some() {
                state.on_3b = None;
            }

            state.on_3b = Some(batter_order);
        }

        RunnerDest::Score => {
            add_runs_to_score(state, 1);
            // Batter reaches home; base occupancy unchanged.
        }
    }
}

fn runner_start_base_label(state: &GameState, order: u8) -> &'static str {
    if state.on_1b == Some(order) {
        "1B"
    } else if state.on_2b == Some(order) {
        "2B"
    } else if state.on_3b == Some(order) {
        "3B"
    } else {
        "UNK"
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game_state::GameState;
    use crate::models::runner::RunnerDest;
    use crate::models::types::HalfInning;

    /// Regression test for the FC-to-home run-credit bug.
    ///
    /// Before v0.11.0-alpha2 the `RunnerDest::Score` branch of
    /// `apply_batter_fielders_choice` was a no-op: it left the base
    /// occupancy unchanged (correctly) but forgot to add the run, so
    /// commands like `o6 sc` recorded a completed plate appearance with
    /// `scored=true` in `runner_movements` while `GameState.score`
    /// remained unchanged. The scoreboard lagged by one run and the
    /// deterministic resume reproduced the same error.
    #[test]
    fn fc_to_home_adds_run_to_away_when_top_half() {
        let mut state = GameState::new();
        state.half = HalfInning::Top;
        state.inning = 1;
        assert_eq!(state.score.away, 0);

        apply_batter_fielders_choice(&mut state, 5, RunnerDest::Score);

        assert_eq!(state.score.away, 1, "away total score must increase");
        assert_eq!(
            state.score.away_innings[0], 1,
            "away inning-by-inning line score must increase for inning 1"
        );
        assert_eq!(state.score.home, 0, "home score must not change");
    }

    #[test]
    fn fc_to_home_adds_run_to_home_when_bottom_half() {
        let mut state = GameState::new();
        state.half = HalfInning::Bottom;
        state.inning = 3;

        apply_batter_fielders_choice(&mut state, 4, RunnerDest::Score);

        assert_eq!(state.score.home, 1, "home total score must increase");
        assert_eq!(
            state.score.home_innings[2], 1,
            "home inning-by-inning line score must increase for inning 3"
        );
        assert_eq!(state.score.away, 0, "away score must not change");
    }

    #[test]
    fn fc_to_home_leaves_base_occupancy_unchanged() {
        // Bases loaded scenario: batter's FC to home scores *the batter*
        // directly (unusual but grammatically legal). Runners on 1B/2B/3B
        // are undisturbed.
        let mut state = GameState::new();
        state.half = HalfInning::Top;
        state.on_1b = Some(7);
        state.on_2b = Some(8);
        state.on_3b = Some(9);

        apply_batter_fielders_choice(&mut state, 5, RunnerDest::Score);

        assert_eq!(state.on_1b, Some(7));
        assert_eq!(state.on_2b, Some(8));
        assert_eq!(state.on_3b, Some(9));
        assert_eq!(state.score.away, 1);
    }

    // ── Non-regression for the other three destinations: they must
    //    never touch the score. ────────────────────────────────────────
    #[test]
    fn fc_to_first_does_not_change_score() {
        let mut state = GameState::new();
        state.half = HalfInning::Top;
        apply_batter_fielders_choice(&mut state, 5, RunnerDest::First);
        assert_eq!(state.score.away, 0);
        assert_eq!(state.score.home, 0);
        assert_eq!(state.on_1b, Some(5));
    }

    #[test]
    fn fc_to_second_does_not_change_score() {
        let mut state = GameState::new();
        state.half = HalfInning::Bottom;
        apply_batter_fielders_choice(&mut state, 5, RunnerDest::Second);
        assert_eq!(state.score.home, 0);
        assert_eq!(state.on_2b, Some(5));
    }

    #[test]
    fn fc_to_third_does_not_change_score() {
        let mut state = GameState::new();
        state.half = HalfInning::Top;
        apply_batter_fielders_choice(&mut state, 5, RunnerDest::Third);
        assert_eq!(state.score.away, 0);
        assert_eq!(state.on_3b, Some(5));
    }
}
