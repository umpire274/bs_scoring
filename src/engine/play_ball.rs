use crate::commands::engine_parser::parse_engine_commands;
use crate::core::play_ball::set_game_status;
use crate::core::play_ball_apply::apply_engine_command;
use crate::models::play_ball::GameState;
use crate::ui::Ui;
use crate::ui::events::UiEvent;
use rusqlite::Connection;

pub enum EngineExit {
    ExitToMenu,
}

fn format_prompt(state: &GameState, away: &str, home: &str) -> String {
    format!(
        "{}{} ({} OUTS) {} {} - {} {} > ",
        state.inning,
        state.half_symbol(),
        state.outs,
        away,
        state.score.away,
        state.score.home,
        home
    )
}

/// Play Ball engine loop.
///
/// Minimal structural refactor: the core no longer reads stdin / prints output directly.
/// UI is abstracted via the `Ui` trait.
pub fn run_play_ball_engine(
    conn: &mut Connection,
    ui: &mut dyn Ui,
    game_id: &str,
    away: &str,
    home: &str,
) -> EngineExit {
    let mut state = GameState::new();

    loop {
        let prompt = format_prompt(&state, away, home);
        let Some(line) = ui.read_command_line(&prompt) else {
            return EngineExit::ExitToMenu;
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let commands = parse_engine_commands(line);
        for cmd in commands {
            let result = apply_engine_command(&mut state, cmd);

            for ev in result.events {
                ui.emit(ev);
            }

            if let Some(status) = result.status_change {
                match set_game_status(conn, game_id, status) {
                    Ok(true) => {}
                    Ok(false) => ui.emit(UiEvent::Error(
                        "Game status was not updated (game not found?)".to_string(),
                    )),
                    Err(e) => ui.emit(UiEvent::Error(format!("Failed to update status: {e}"))),
                }
            }

            if result.exit {
                return EngineExit::ExitToMenu;
            }
        }
    }
}
