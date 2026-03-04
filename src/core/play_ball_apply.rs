use crate::commands::types::EngineCommand;
use crate::models::events::{DomainEvent, OutRecordedData, StatusChangedData, StrikeoutKind};
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
    let Some(batter_id) = state.current_batter_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter. Use PLAYBALL (or resume the game) first.".to_string(),
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
            // reserved for v0.6.9+ (no count changes here)
        }
    }

    let mut events_ui = vec![UiEvent::Line(format!("Pitch: {}", pitch))];
    let mut persisted: Vec<PersistedEvent> = vec![];

    // 1) Always persist PitchRecorded (the reducer will update pitch_count from this)
    persisted.push(PersistedEvent {
        inning: state.inning,
        half: state.half,
        event: DomainEvent::PitchRecorded {
            pitcher_id,
            batter_id,
            pitch: pitch.clone(),
        },
        description: format!("Pitch: {}", pitch),
    });

    // 2) Terminal outcomes
    let mut needs_next_at_bat = false;

    // Walk: 4 balls and strikes < 3
    if balls_after >= 4 && strikes_after < 3 {
        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::WalkIssued { batter_id },
            description: "Walk".to_string(),
        });

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

        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::RunnerToFirst {
                runner_id: batter_id,
                runner_jersey_no,
                runner_first_name: runner_first_name.clone(),
                runner_last_name: runner_last_name.clone(),
            },
            description: format!(
                "BB: #{runner_jersey_no} {runner_first_name} {runner_last_name} to 1B"
            ),
        });

        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::CountReset,
            description: "Count reset".to_string(),
        });

        events_ui.push(UiEvent::Line("BB: batter to 1B".to_string()));
        needs_next_at_bat = true;
    }
    // Strikeout: 3 strikes before 4 balls
    else if strikes_after >= 3 && balls_after < 4 {
        let kind = match pitch {
            Pitch::CalledStrike => StrikeoutKind::Called,
            Pitch::SwingingStrike => StrikeoutKind::Swinging,
            Pitch::FoulBunt => StrikeoutKind::FoulBunt,
            // non dovrebbe mai succedere qui, ma fallback safety:
            _ => StrikeoutKind::Called,
        };

        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::Strikeout {
                batter_id,
                kind: kind.clone(),
            },
            description: match kind {
                StrikeoutKind::Called => "Strikeout (called)".to_string(),
                StrikeoutKind::Swinging => "Strikeout (swinging)".to_string(),
                StrikeoutKind::FoulBunt => "Strikeout (foul bunt)".to_string(),
            },
        });

        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::OutRecorded(OutRecordedData {
                outs_before: state.outs,
                outs_after: state.outs.saturating_add(1),
            }),
            description: "Out recorded".to_string(),
        });

        persisted.push(PersistedEvent {
            inning: state.inning,
            half: state.half,
            event: DomainEvent::CountReset,
            description: "Count reset".to_string(),
        });

        events_ui.push(UiEvent::Line("K: batter out".to_string()));
        needs_next_at_bat = true;
    } else {
        // Optional: if pitch is reserved, make it explicit in the log
        if matches!(pitch, Pitch::InPlay | Pitch::HittedBy) {
            events_ui.push(UiEvent::Line("Note: X/H not implemented yet".to_string()));
        }
    }

    ApplyResult {
        events: events_ui,
        persisted,
        exit: false,
        status_change: None,
        needs_next_at_bat,
    }
}
