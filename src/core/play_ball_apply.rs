use crate::commands::types::EngineCommand;
use crate::models::play_ball::{GameState, HalfInning};
use crate::models::types::GameStatus;
use crate::ui::events::UiEvent;

pub struct ApplyResult {
    pub events: Vec<UiEvent>,
    pub exit: bool,
    pub status_change: Option<GameStatus>,
}

pub fn apply_engine_command(state: &mut GameState, cmd: EngineCommand) -> ApplyResult {
    match cmd {
        EngineCommand::Exit => ApplyResult {
            events: vec![],
            exit: true,
            status_change: None,
        },

        EngineCommand::SetStatus(status) => ApplyResult {
            events: vec![UiEvent::Line(format!(
                "{} Game set to {}.",
                status.icon(),
                status
            ))],
            exit: true,
            status_change: Some(status),
        },

        EngineCommand::Out => {
            if state.outs < 2 {
                state.outs += 1;
            } else {
                state.outs = 0;
                match state.half {
                    HalfInning::Top => state.half = HalfInning::Bottom,
                    HalfInning::Bottom => {
                        state.half = HalfInning::Top;
                        state.inning += 1;
                    }
                }
            }

            ApplyResult {
                events: vec![],
                exit: false,
                status_change: None,
            }
        }

        EngineCommand::Unknown(s) => ApplyResult {
            events: vec![UiEvent::Error(format!("Unknown command: {s}"))],
            exit: false,
            status_change: None,
        },
    }
}
