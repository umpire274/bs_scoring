use crate::commands::types::EngineCommand;
use crate::models::events::{DomainEvent, StatusChangedData};
use crate::models::play_ball::{GameState, HalfInning};
use crate::models::types::{GameStatus, Pitch};
use crate::ui::events::UiEvent;

#[derive(Debug, Clone)]
pub struct PersistedEvent {
    pub inning: u32,
    pub half: HalfInning,
    pub event: DomainEvent,
    pub description: String,
}

pub struct ApplyResult {
    pub events: Vec<UiEvent>,
    pub persisted: Vec<PersistedEvent>,
    pub exit: bool,
    pub status_change: Option<GameStatus>,
    pub needs_next_at_bat: bool,
}

fn empty_result() -> ApplyResult {
    ApplyResult {
        events: vec![],
        persisted: vec![],
        exit: false,
        status_change: None,
        needs_next_at_bat: false,
    }
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
                exit: true,
                status_change: Some(status),
                needs_next_at_bat: false,
            }
        }

        // ✅ NEW: pitch command (0.6.7 baseline)
        EngineCommand::Pitch(pitch) => apply_pitch(state, pitch),

        EngineCommand::Unknown(s) => ApplyResult {
            events: vec![UiEvent::Error(format!("Unknown command: {s}"))],
            ..empty_result()
        },
    }
}

fn apply_pitch(state: &mut GameState, pitch: Pitch) -> ApplyResult {
    // Must have an active PA
    let Some(batter_id) = state.current_batter_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter. Use PLAYBALL (or start/resume the game) first.".to_string(),
            )],
            ..empty_result()
        };
    };

    let Some(_pitcher_id) = state.current_pitcher_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active pitcher in state (cannot record pitch).".to_string(),
            )],
            ..empty_result()
        };
    };

    // We compute the "post-pitch" balls/strikes here (do NOT rely on reducer timing).
    let mut balls = state.pitch_count.balls;
    let strikes = state.pitch_count.strikes;

    if matches!(pitch, Pitch::Ball) {
        balls = balls.saturating_add(1);
    }

    let mut ui_events = Vec::new();
    let mut persisted: Vec<PersistedEvent> = Vec::new();

    // 1) Always record the pitch
    let msg_pitch = format!("Pitch: {}", pitch);
    ui_events.push(UiEvent::Line(msg_pitch.clone()));

    persisted.push(PersistedEvent {
        inning: state.inning,
        half: state.half,
        event: DomainEvent::PitchRecorded {
            pitcher_id: state.current_pitcher_id.unwrap(), // safe: checked above
            batter_id,
            pitch: pitch.clone(),
        },
        description: msg_pitch,
    });

    // 2) Walk logic (4 balls before 3 strikes)
    if balls >= 4 && strikes < 3 {
        // Need batter identity for RunnerToFirst (your variant requires these fields)
        let Some(runner_jersey_no) = state.current_batter_jersey_no else {
            return ApplyResult {
                events: vec![UiEvent::Error(
                    "Walk detected but batter jersey number is missing in state.".to_string(),
                )],
                ..empty_result()
            };
        };
        let Some(runner_first_name) = state.current_batter_first_name.clone() else {
            return ApplyResult {
                events: vec![UiEvent::Error(
                    "Walk detected but batter first name is missing in state.".to_string(),
                )],
                ..empty_result()
            };
        };
        let Some(runner_last_name) = state.current_batter_last_name.clone() else {
            return ApplyResult {
                events: vec![UiEvent::Error(
                    "Walk detected but batter last name is missing in state.".to_string(),
                )],
                ..empty_result()
            };
        };

        let msg_bb = "Walk (BB) — batter awarded 1B".to_string();
        ui_events.push(UiEvent::Line(msg_bb.clone()));

        // WalkIssued: adjust to your actual fields (NO pitcher_id)
        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::WalkIssued { batter_id },
            description: msg_bb,
        });

        // RunnerToFirst requires runner details in your enum
        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::RunnerToFirst {
                runner_id: batter_id,
                runner_jersey_no,
                runner_first_name,
                runner_last_name,
            },
            description: "Runner to 1B".to_string(),
        });

        // Reset count for next batter
        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::CountReset,
            description: "Count reset".to_string(),
        });

        return ApplyResult {
            events: ui_events,
            persisted,
            exit: false,
            status_change: None,
            needs_next_at_bat: true,
        };
    }

    ApplyResult {
        events: ui_events,
        persisted,
        exit: false,
        status_change: None,
        needs_next_at_bat: false,
    }
}
