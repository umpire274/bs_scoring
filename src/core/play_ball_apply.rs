use crate::commands::types::EngineCommand;
use crate::models::events::{DomainEvent, StatusChangedData};
use crate::models::play_ball::{GameState, HalfInning};
use crate::models::types::GameStatus;
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
}

pub fn apply_engine_command(state: &mut GameState, cmd: EngineCommand) -> ApplyResult {
    match cmd {
        EngineCommand::Exit => ApplyResult {
            events: vec![],
            persisted: vec![],
            exit: true,
            status_change: None,
        },

        // NOTE: PLAYBALL is handled in the engine layer because it requires DB lookups
        // (away lineup #1 batter + player names). We keep this branch for exhaustiveness.
        EngineCommand::PlayBall => ApplyResult {
            events: vec![UiEvent::Error(
                "PLAYBALL must be handled by the engine (DB-backed).".to_string(),
            )],
            persisted: vec![],
            exit: false,
            status_change: None,
        },

        EngineCommand::SetStatus(status) => ApplyResult {
            events: vec![UiEvent::Line(format!(
                "{} Game set to {}.",
                status.icon(),
                status
            ))],
            persisted: vec![PersistedEvent {
                inning: state.inning,
                half: state.half,
                event: DomainEvent::StatusChanged(StatusChangedData { to: status }),
                description: format!("{} Game set to {}.", status.icon(), status),
            }],
            exit: true,
            status_change: Some(status),
        },

        EngineCommand::Pitch => {
            if let Some(pitcher_id) = state.current_pitcher_id {
                let ev = DomainEvent::PitchThrown { pitcher_id };

                ApplyResult {
                    events: vec![UiEvent::Line("Pitch".to_string())],
                    persisted: vec![PersistedEvent {
                        inning: state.inning,
                        half: state.half,
                        event: ev,
                        description: "Pitch".to_string(),
                    }],
                    status_change: None,
                    exit: false,
                }
            } else {
                ApplyResult {
                    events: vec![UiEvent::Error("No active pitcher".to_string())],
                    persisted: vec![],
                    status_change: None,
                    exit: false,
                }
            }
        }

        EngineCommand::Unknown(s) => ApplyResult {
            events: vec![UiEvent::Error(format!("Unknown command: {s}"))],
            persisted: vec![],
            exit: false,
            status_change: None,
        },
    }
}
