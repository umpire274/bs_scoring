use crate::commands::types::EngineCommand;
use crate::models::events::{DomainEvent, OutRecordedData, SideChangeData, StatusChangedData};
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

        EngineCommand::Out => {
            let before_outs = state.outs;
            let before_inning = state.inning;
            let before_half = state.half;

            let mut persisted: Vec<PersistedEvent> = Vec::new();

            if before_outs < 2 {
                state.outs = before_outs + 1;

                let msg = format!("Out recorded ({} OUTS).", state.outs);
                persisted.push(PersistedEvent {
                    inning: before_inning,
                    half: before_half,
                    event: DomainEvent::OutRecorded(OutRecordedData {
                        outs_before: before_outs,
                        outs_after: state.outs,
                    }),
                    description: msg.clone(),
                });

                ApplyResult {
                    events: vec![UiEvent::Line(msg)],
                    persisted,
                    exit: false,
                    status_change: None,
                }
            } else {
                // Third out: record out (outs_after=3) and then side change.
                let out_msg = "Out recorded (3 OUTS).".to_string();
                persisted.push(PersistedEvent {
                    inning: before_inning,
                    half: before_half,
                    event: DomainEvent::OutRecorded(OutRecordedData {
                        outs_before: before_outs,
                        outs_after: 3,
                    }),
                    description: out_msg.clone(),
                });

                // Advance half/inning in state
                match state.half {
                    HalfInning::Top => state.half = HalfInning::Bottom,
                    HalfInning::Bottom => {
                        state.half = HalfInning::Top;
                        state.inning += 1;
                    }
                }
                state.outs = 0;

                let side_msg = format!(
                    "Side retired. {} {} (0 OUTS).",
                    state.half_symbol(),
                    state.inning
                );
                persisted.push(PersistedEvent {
                    inning: state.inning,
                    half: state.half,
                    event: DomainEvent::SideChange(SideChangeData {
                        inning: state.inning,
                        half: state.half,
                    }),
                    description: side_msg.clone(),
                });

                ApplyResult {
                    events: vec![UiEvent::Line(out_msg), UiEvent::Line(side_msg)],
                    persisted,
                    exit: false,
                    status_change: None,
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
