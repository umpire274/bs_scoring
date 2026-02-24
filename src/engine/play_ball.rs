use crate::commands::engine_parser::parse_engine_commands;
use crate::commands::types::EngineCommand;
use crate::core::play_ball::set_game_status;
use crate::core::play_ball_apply::apply_engine_command;
use crate::core::play_ball_reducer::apply_domain_event;
use crate::db::game_events::{append_game_event, list_game_events};
use crate::models::events::DomainEvent;
use crate::models::play_ball::GameState;
use crate::ui::Ui;
use crate::ui::events::UiEvent;
use rusqlite::{Connection, params};

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

/// Returns (player_id, first_name, last_name) for the away batter at batting_order=1.
fn get_away_leadoff_batter(
    conn: &Connection,
    game_id: &str,
    away_team_id: i64,
) -> rusqlite::Result<(i64, String, String)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT p.id, p.first_name, p.last_name
        FROM game_lineups gl
        JOIN players p ON gl.player_id = p.id
        WHERE gl.game_id = ?1
          AND gl.team_id = ?2
          AND gl.is_starting = 1
          AND gl.batting_order = 1
        LIMIT 1
        "#,
    )?;

    stmt.query_row(params![game_id, away_team_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })
}

/// Play Ball engine loop.
pub fn run_play_ball_engine(
    conn: &mut Connection,
    ui: &mut dyn Ui,
    game_pk: i64,
    game_id: &str,
    away_team_id: i64, // NEW: needed for playball batter lookup
    away: &str,
    home: &str,
) -> EngineExit {
    // Rebuild state from persisted events (resume-friendly).
    let mut state = GameState::new();

    // Track whether we already have any events (if yes, PLAYBALL is not allowed).
    let mut has_events = false;

    match list_game_events(conn, game_pk) {
        Ok(rows) => {
            has_events = !rows.is_empty();

            for r in rows {
                // Push stored description into UI log (if any)
                if let Some(desc) = r.description {
                    ui.emit(UiEvent::Line(desc));
                }

                // Rebuild prompt state from structured event data if available.
                if let Some(data) = r.event_data.as_deref()
                    && let Ok(ev) = serde_json::from_str::<DomainEvent>(data)
                {
                    apply_domain_event(&mut state, &ev);
                }
            }
        }
        Err(e) => ui.emit(UiEvent::Error(format!("Failed to load game events: {e}"))),
    }

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
            // Special: PLAYBALL must be handled here (DB-backed)
            if let EngineCommand::PlayBall = cmd {
                if has_events {
                    ui.emit(UiEvent::Error(
                        "PLAYBALL is only allowed when there are no previous events for this game."
                            .to_string(),
                    ));
                    continue;
                }

                // 1) Persist GameStarted
                let msg_start = "🏁 Play Ball!".to_string();

                let ev_start = DomainEvent::GameStarted;
                if let Err(e) = append_game_event(
                    conn,
                    game_pk,
                    state.inning,
                    state.half,
                    &ev_start,
                    &msg_start,
                ) {
                    ui.emit(UiEvent::Error(format!("Failed to append game event: {e}")));
                    continue;
                }
                ui.emit(UiEvent::Line(msg_start.clone()));
                apply_domain_event(&mut state, &ev_start);

                // 2) Determine away leadoff batter (batting_order=1)
                let (batter_id, first, last) =
                    match get_away_leadoff_batter(conn, game_id, away_team_id) {
                        Ok(v) => v,
                        Err(e) => {
                            ui.emit(UiEvent::Error(format!(
                                "Cannot start game: missing AWAY lineup batter #1 ({e})"
                            )));
                            continue;
                        }
                    };

                let at_bat_no: u32 = 1;
                let msg_ab = format!("At bat number {:02} {} {}", at_bat_no, first, last);

                let ev_ab = DomainEvent::AtBatStarted {
                    at_bat_no,
                    batting_team_id: away_team_id,
                    batter_id,
                };

                if let Err(e) =
                    append_game_event(conn, game_pk, state.inning, state.half, &ev_ab, &msg_ab)
                {
                    ui.emit(UiEvent::Error(format!("Failed to append game event: {e}")));
                    continue;
                }

                ui.emit(UiEvent::Line(msg_ab.clone()));
                apply_domain_event(&mut state, &ev_ab);

                has_events = true;
                continue;
            }

            // Default path (apply -> emit -> persist -> status update -> exit)
            let result = apply_engine_command(&mut state, cmd);

            for ev in result.events {
                ui.emit(ev);
            }

            // Persist replayable events.
            for pe in &result.persisted {
                if let Err(e) = append_game_event(
                    conn,
                    game_pk,
                    pe.inning,
                    pe.half,
                    &pe.event,
                    &pe.description,
                ) {
                    ui.emit(UiEvent::Error(format!("Failed to append game event: {e}")));
                }
                // keep has_events in sync
                has_events = true;
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
