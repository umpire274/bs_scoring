use crate::HalfInning;
use crate::commands::engine_parser::parse_engine_commands;
use crate::commands::types::EngineCommand;
use crate::core::play_ball::set_game_status;
use crate::core::play_ball_apply::apply_engine_command;
use crate::core::play_ball_reducer::apply_domain_event;
use crate::db::game_events::{append_game_event, list_game_events};
use crate::models::events::{DomainEvent, SideChangeData};
use crate::models::play_ball::GameState;
use crate::ui::Ui;
use crate::ui::events::UiEvent;
use rusqlite::{Connection, params};

pub enum EngineExit {
    ExitToMenu,
}

/// Returns (player_id, first_name, last_name) for the away batter at batting_order=1.
/// Play Ball engine loop.
pub fn run_play_ball_engine(
    conn: &mut Connection,
    ui: &mut dyn Ui,
    game_pk: i64,
    game_id: &str,
    away_team_id: i64,
    home_team_id: i64,
) -> EngineExit {
    // Rebuild state from persisted events (resume-friendly).
    let mut state = GameState::new();

    // Track whether we already have any events (if yes, PLAYBALL is not allowed).
    let mut has_events = false;

    // --------- Replay persisted events (resume) ----------
    match list_game_events(conn, game_pk) {
        Ok(rows) => {
            has_events = !rows.is_empty();

            for r in rows {
                // Push stored description into UI log (if any)
                if let Some(desc) = r.description {
                    ui.emit(UiEvent::Line(desc));
                }

                // Rebuild state from structured event JSON
                if let Some(data) = r.event_data.as_deref()
                    && let Ok(ev) = serde_json::from_str::<DomainEvent>(data)
                {
                    apply_domain_event(&mut state, &ev);
                }
            }

            // ✅ ensure scoreboard has the rebuilt state
            ui.set_state(&state);
        }
        Err(e) => ui.emit(UiEvent::Error(format!("Failed to load game events: {e}"))),
    }

    // ---------------- Engine loop ----------------
    loop {
        // Keep UI scoreboard in sync before prompting
        ui.set_state(&state);

        let Some(line) = ui.read_command_line("> ") else {
            return EngineExit::ExitToMenu;
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let commands = parse_engine_commands(line);

        for cmd in commands {
            // ---------------- Special: PLAYBALL (DB-backed) ----------------
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

                if !persist_event(
                    conn,
                    ui,
                    game_pk,
                    state.inning,
                    state.half,
                    &ev_start,
                    &msg_start,
                ) {
                    continue;
                }

                // Emit log line + update state/UI
                ui.emit(UiEvent::Line(msg_start.clone()));
                apply_domain_event(&mut state, &ev_start);
                ui.set_state(&state);

                // 2) Determine away leadoff batter (batting_order=1)
                let (batter_id, team_abbrv, jersey_no, first, last) =
                    match get_batter_by_order(conn, game_id, away_team_id, 1) {
                        Ok(v) => v,
                        Err(e) => {
                            ui.emit(UiEvent::Error(format!(
                                "Cannot start game: missing AWAY lineup batter #1 ({e})"
                            )));
                            continue;
                        }
                    };

                // 3) Determine starting pitcher from fielding team (HOME when away bats)
                let fielding_team_id = home_team_id;
                let (pitcher_id, pitcher_no, p_first, p_last) =
                    match get_current_pitcher(conn, game_id, fielding_team_id) {
                        Ok(v) => v,
                        Err(e) => {
                            ui.emit(UiEvent::Error(format!(
                            "Cannot start game: missing starting pitcher for fielding team ({e})"
                        )));
                            continue;
                        }
                    };

                let msg_ab = format!("At bat: {} #{} {} {}", team_abbrv, jersey_no, first, last);

                let ev_ab = DomainEvent::AtBatStarted {
                    team_abbrv,
                    batting_team_id: away_team_id,

                    batter_id,
                    batter_jersey_no: jersey_no,
                    batter_first_name: first,
                    batter_last_name: last,

                    pitcher_id,
                    pitcher_jersey_no: pitcher_no,
                    pitcher_first_name: p_first,
                    pitcher_last_name: p_last,
                };

                if !persist_event(conn, ui, game_pk, state.inning, state.half, &ev_ab, &msg_ab) {
                    continue;
                }

                // Emit log line + update state/UI
                ui.emit(UiEvent::Line(msg_ab.clone()));
                apply_domain_event(&mut state, &ev_ab);
                ui.set_state(&state);

                state.away_next_batting_order = bump_order(1);

                has_events = true;
                continue;
            }

            // ---------------- Default path (apply -> emit -> persist -> reduce -> status -> exit) ----------------
            let result = apply_engine_command(&mut state, cmd);

            for ev in result.events {
                ui.emit(ev);
            }

            // Persist replayable events + apply to in-memory state (for scoreboard)
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
                    continue;
                }

                // ✅ Update in-memory state
                apply_domain_event(&mut state, &pe.event);

                has_events = true;
            }

            if result.needs_next_at_bat {
                if !start_next_at_bat(
                    conn,
                    ui,
                    game_pk,
                    game_id,
                    &mut state,
                    away_team_id,
                    home_team_id,
                ) {
                    ui.emit(UiEvent::Error("Failed to start next at-bat.".to_string()));
                }
                ui.set_state(&state);
            }

            // ✅ Push updated state to UI (scoreboard refresh)
            ui.set_state(&state);

            // Status change (DB)
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

fn persist_event(
    conn: &mut Connection,
    ui: &mut dyn Ui,
    game_pk: i64,
    inning: u32,
    half: HalfInning,
    event: &DomainEvent,
    description: &str,
) -> bool {
    match append_game_event(conn, game_pk, inning, half, event, description) {
        Ok(_) => true,
        Err(e) => {
            ui.emit(UiEvent::Error(format!("Failed to append game event: {e}")));
            false
        }
    }
}

fn get_current_pitcher(
    conn: &Connection,
    game_id: &str,
    fielding_team_id: i64,
) -> rusqlite::Result<(i64, i32, String, String)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT p.id, p.number, p.first_name, p.last_name
        FROM game_lineups gl
        JOIN players p ON gl.player_id = p.id
        WHERE gl.game_id = ?1
          AND gl.team_id = ?2
          AND gl.is_starting = 1
          AND gl.defensive_position = '1'
        LIMIT 1
        "#,
    )?;

    stmt.query_row(params![game_id, fielding_team_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })
}

/// Returns (batter_id, team_abbrv, jersey_no, first_name, last_name)
fn get_batter_by_order(
    conn: &Connection,
    game_id: &str,
    team_id: i64,
    batting_order: u8,
) -> rusqlite::Result<(i64, String, i32, String, String)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT p.id, t.abbreviation, p.number, p.first_name, p.last_name
        FROM game_lineups gl
        JOIN players p ON gl.player_id = p.id
        JOIN teams t ON gl.team_id = t.id
        WHERE gl.game_id = ?1
          AND gl.team_id = ?2
          AND gl.is_starting = 1
          AND gl.batting_order = ?3
        LIMIT 1
        "#,
    )?;

    stmt.query_row(params![game_id, team_id, batting_order as i64], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    })
}

fn bump_order(x: u8) -> u8 {
    if x >= 9 { 1 } else { x + 1 }
}

fn start_next_at_bat(
    conn: &mut Connection,
    ui: &mut dyn Ui,
    game_pk: i64,
    game_id: &str,
    state: &mut GameState,
    away_team_id: i64,
    home_team_id: i64,
) -> bool {
    // 1) If 3 outs -> change side (and maybe inning)
    if state.outs >= 3 && !handle_three_outs_and_change_side(conn, ui, game_pk, state) {
        return false;
    }

    // 2) Determine batting/fielding teams based on half
    let (batting_team_id, fielding_team_id, next_order) = match state.half {
        HalfInning::Top => (away_team_id, home_team_id, state.away_next_batting_order),
        HalfInning::Bottom => (home_team_id, away_team_id, state.home_next_batting_order),
    };

    // 3) Get next batter (by batting order cursor)
    let (batter_id, team_abbrv, jersey_no, first, last) =
        match get_batter_by_order(conn, game_id, batting_team_id, next_order) {
            Ok(v) => v,
            Err(e) => {
                ui.emit(UiEvent::Error(format!(
                "Failed to load next batter (team_id={batting_team_id}, order={next_order}): {e}"
            )));
                return false;
            }
        };

    // 4) Get current pitcher from fielding team lineup (your helper already does this)
    let (pitcher_id, pitcher_no, p_first, p_last) =
        match get_current_pitcher(conn, game_id, fielding_team_id) {
            Ok(v) => v,
            Err(e) => {
                ui.emit(UiEvent::Error(format!(
                    "Failed to load current pitcher (fielding_team_id={fielding_team_id}): {e}"
                )));
                return false;
            }
        };

    // 5) Persist AtBatStarted
    let msg_ab = format!("At bat: {team_abbrv} #{jersey_no} {first} {last}");

    let ev_ab = DomainEvent::AtBatStarted {
        team_abbrv,
        batting_team_id,

        batter_id,
        batter_jersey_no: jersey_no,
        batter_first_name: first,
        batter_last_name: last,

        pitcher_id,
        pitcher_jersey_no: pitcher_no,
        pitcher_first_name: p_first,
        pitcher_last_name: p_last,
    };

    if !persist_event(conn, ui, game_pk, state.inning, state.half, &ev_ab, &msg_ab) {
        return false;
    }

    ui.emit(UiEvent::Line(msg_ab));
    apply_domain_event(state, &ev_ab);

    // 6) Advance batting order cursor (resume-safe)
    match state.half {
        HalfInning::Top => state.away_next_batting_order = bump_order(next_order),
        HalfInning::Bottom => state.home_next_batting_order = bump_order(next_order),
    }

    true
}

fn handle_three_outs_and_change_side(
    conn: &mut Connection,
    ui: &mut dyn Ui,
    game_pk: i64,
    state: &mut GameState,
) -> bool {
    // Compute next half + inning
    let (next_inning, next_half) = match state.half {
        HalfInning::Top => (state.inning, HalfInning::Bottom),
        HalfInning::Bottom => (state.inning.saturating_add(1), HalfInning::Top),
    };

    // ✅ SideChange must be persisted (resume-friendly)
    let ev_side = DomainEvent::SideChange(SideChangeData {
        inning: next_inning,
        half: next_half,
    });

    let desc = format!(
        "Side change: {} {}",
        match next_half {
            HalfInning::Top => "Top",
            HalfInning::Bottom => "Bottom",
        },
        next_inning
    );

    if !persist_event(conn, ui, game_pk, state.inning, state.half, &ev_side, &desc) {
        return false;
    }

    ui.emit(UiEvent::Line(desc));
    apply_domain_event(state, &ev_side);

    // ✅ In un half inning nuovo, count deve essere 0-0.
    // Se tu già persist CountReset in BB/K, qui comunque è corretto farlo sempre
    // per sicurezza (e per evitare carry-over se l'ultimo evento non lo ha emesso).
    let ev_reset = DomainEvent::CountReset;
    if !persist_event(
        conn,
        ui,
        game_pk,
        state.inning,
        state.half,
        &ev_reset,
        "Count reset",
    ) {
        return false;
    }
    apply_domain_event(state, &ev_reset);

    // ⚠️ IMPORTANT: bases should clear on side change.
    // Se NON hai un DomainEvent dedicato, almeno azzera lo stato runtime:
    // (idealmente: aggiungi clearing dentro reducer su SideChange)
    state.on_1b = false;
    state.on_2b = false;
    state.on_3b = false;

    true
}
