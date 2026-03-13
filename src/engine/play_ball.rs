use crate::commands::engine_parser::parse_engine_commands;
use crate::commands::types::EngineCommand;
use crate::core::play_ball_apply::apply_engine_command;
use crate::core::play_ball_reducer::{
    apply_domain_event, apply_live_plate_appearance, apply_plate_appearance_row,
};
use crate::db::at_bat_draft::{
    AtBatDraftRow, clear_at_bat_draft, load_at_bat_draft, upsert_at_bat_draft,
};
use crate::db::game_events::{GameEventRow, append_game_event, list_game_events};
use crate::db::game_queries::set_game_status;
use crate::db::plate_appearances::{
    PlateAppearanceRow, append_plate_appearance, list_plate_appearances,
};
use crate::models::events::{DomainEvent, SideChangeData};
use crate::models::game_state::{BatterOrder, GameState};
use crate::models::plate_appearance::PlateAppearanceStep;
use crate::ui::Ui;
use crate::ui::events::UiEvent;
use crate::{HalfInning, Pitch, Position};
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

    // --------- Replay persisted events + deterministic rebuild (resume) ----------
    // --------- Resume ----------
    match list_game_events(conn, game_pk) {
        Ok(rows) => {
            has_events = !rows.is_empty();
            replay_admin_logs(ui, &rows);

            // 1) deterministic rebuild from plate appearances
            match list_plate_appearances(conn, game_pk) {
                Ok(pas) => {
                    if !pas.is_empty() {
                        has_events = true;
                    }
                    replay_plate_appearances_and_log(ui, &mut state, &pas);
                }
                Err(e) => ui.emit(UiEvent::Error(format!(
                    "Failed to load plate appearances: {e}"
                ))),
            }

            // 3) restore in-progress at-bat (draft)
            let draft_opt = load_and_apply_draft(conn, ui, game_pk, &mut state);

            // 4) if draft exists, ensure cursor is not behind (avoid repeating batter)
            if let Some(draft) = &draft_opt
                && let Some(batter_id) = draft.batter_id
            {
                let batting_team_id = match state.half {
                    HalfInning::Top => away_team_id,
                    HalfInning::Bottom => home_team_id,
                };

                if let Some(order) =
                    find_order_for_batter(conn, game_id, batting_team_id, batter_id)
                {
                    match state.half {
                        HalfInning::Top => state.away_next_batting_order = bump_order(order),
                        HalfInning::Bottom => state.home_next_batting_order = bump_order(order),
                    }
                }
            }

            // 5) hydrate display fields
            if let Err(e) =
                hydrate_current_matchup(conn, game_id, &mut state, away_team_id, home_team_id)
            {
                ui.emit(UiEvent::Error(format!("Failed to hydrate matchup: {e}")));
            }

            state.started = has_events;
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
                let (batter_id, team_abbrv, jersey_no, first, last, batter_order, batter_position) =
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

                let msg_ab = format_live_at_bat(&FormatLiveAtBatInput {
                    inning: state.inning,
                    half: state.half,
                    outs: state.outs,
                    order: batter_order,
                    first: first.clone(),
                    last: last.clone(),
                    jersey: jersey_no,
                    pos: batter_position,
                });

                let ev_ab = DomainEvent::AtBatStarted {
                    team_abbrv,
                    batting_team_id: away_team_id,

                    batter_id,
                    batter_jersey_no: jersey_no,
                    batter_first_name: first,
                    batter_last_name: last,
                    batter_order,
                    batter_position,

                    pitcher_id,
                    pitcher_jersey_no: pitcher_no,
                    pitcher_first_name: p_first,
                    pitcher_last_name: p_last,
                };

                // Emit log line + update state/UI (AtBatStarted is NOT persisted anymore)
                ui.emit(UiEvent::Line(msg_ab.clone()));
                apply_domain_event(&mut state, &ev_ab);
                ui.set_state(&state);

                // Create draft row so resume keeps batter/pitcher even before first pitch
                let _ = upsert_at_bat_draft(
                    conn,
                    game_pk,
                    state.inning,
                    state.half,
                    state.current_batter_id,
                    state.current_pitcher_id,
                    &state.pitch_count,
                );

                state.away_next_batting_order = bump_order(1);

                has_events = true;
                continue;
            }

            // ---------------- Default path (apply -> emit -> persist -> reduce -> status -> exit) ----------------
            let result = apply_engine_command(&mut state, cmd);

            for ev in result.events {
                ui.emit(ev);
            }

            // Persist low-frequency events (admin) + apply to in-memory state
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

                apply_domain_event(&mut state, &pe.event);
                has_events = true;
            }

            // Apply high-frequency events (NOT persisted)
            for ev in &result.applied {
                apply_domain_event(&mut state, ev);

                // Maintain a single-row draft for resume (no pitch-by-pitch persistence).
                if matches!(
                    ev,
                    DomainEvent::PitchRecorded { .. } | DomainEvent::AtBatStarted { .. }
                ) {
                    let _ = upsert_at_bat_draft(
                        conn,
                        game_pk,
                        state.inning,
                        state.half,
                        state.current_batter_id,
                        state.current_pitcher_id,
                        &state.pitch_count,
                    );
                }
            }

            // Track whether we have already applied a compact PA to live state.
            // If yes, the batting-order cursor has already been advanced by
            // apply_live_plate_appearance(), so we must NOT call start_next_at_bat(),
            // otherwise we would skip one hitter.
            let mut pa_applied_live = false;

            // If a PA was completed, persist a single compact record
            if let Some(pa) = &result.plate_appearance {
                if let Err(e) = append_plate_appearance(conn, game_pk, pa) {
                    ui.emit(UiEvent::Error(format!(
                        "Failed to append plate appearance: {e}"
                    )));
                } else {
                    // Apply the compact PA immediately to live state.
                    // IMPORTANT: this already advances the batting-order cursor.
                    apply_live_plate_appearance(&mut state, pa);

                    // PA is over, clear the draft now.
                    let _ = clear_at_bat_draft(conn, game_pk);

                    has_events = true;
                    pa_applied_live = true;
                }
            }

            let should_start_next_at_bat = result.needs_next_at_bat || pa_applied_live;

            if should_start_next_at_bat {
                if pa_applied_live {
                    let inning_ended = state.outs >= 3;

                    if inning_ended
                        && !handle_three_outs_and_change_side(conn, ui, game_pk, &mut state)
                    {
                        ui.emit(UiEvent::Error(
                            "Failed to change side after 3 outs.".to_string(),
                        ));
                        ui.set_state(&state);
                        continue;
                    }

                    let (batting_team_id, fielding_team_id, next_order) = match state.half {
                        HalfInning::Top => {
                            (away_team_id, home_team_id, state.away_next_batting_order)
                        }
                        HalfInning::Bottom => {
                            (home_team_id, away_team_id, state.home_next_batting_order)
                        }
                    };

                    let next_order = if inning_ended {
                        next_order
                    } else {
                        let pa = match &result.plate_appearance {
                            Some(pa) => pa,
                            None => {
                                ui.emit(UiEvent::Error(
                                    "Internal error: live PA was applied but no plate appearance is available."
                                        .to_string(),
                                ));
                                ui.set_state(&state);
                                continue;
                            }
                        };

                        let completed_batter_id = pa.batter_id;

                        let completed_order = match find_order_for_batter(
                            conn,
                            game_id,
                            batting_team_id,
                            completed_batter_id,
                        ) {
                            Some(order) => order,
                            None => {
                                ui.emit(UiEvent::Error(format!(
                                    "Failed to start next at-bat: cannot resolve batting order for batter_id={completed_batter_id}."
                                )));
                                ui.set_state(&state);
                                continue;
                            }
                        };

                        bump_order(completed_order)
                    };

                    let (
                        batter_id,
                        team_abbrv,
                        jersey_no,
                        first,
                        last,
                        batter_order,
                        batter_position,
                    ) = match get_batter_by_order(conn, game_id, batting_team_id, next_order) {
                        Ok(v) => v,
                        Err(e) => {
                            ui.emit(UiEvent::Error(format!(
                                "Failed to start next at-bat: missing lineup batter #{next_order} ({e})"
                            )));
                            ui.set_state(&state);
                            continue;
                        }
                    };

                    let (pitcher_id, pitcher_no, p_first, p_last) = match get_current_pitcher(
                        conn,
                        game_id,
                        fielding_team_id,
                    ) {
                        Ok(v) => v,
                        Err(e) => {
                            ui.emit(UiEvent::Error(format!(
                                "Failed to start next at-bat: missing current pitcher for fielding team ({e})"
                            )));
                            ui.set_state(&state);
                            continue;
                        }
                    };

                    let msg_ab = format_live_at_bat(&FormatLiveAtBatInput {
                        inning: state.inning,
                        half: state.half,
                        outs: state.outs,
                        order: batter_order,
                        first: first.clone(),
                        last: last.clone(),
                        jersey: jersey_no,
                        pos: batter_position,
                    });

                    let ev_ab = DomainEvent::AtBatStarted {
                        team_abbrv,
                        batting_team_id,

                        batter_id,
                        batter_jersey_no: jersey_no,
                        batter_first_name: first,
                        batter_last_name: last,
                        batter_order,
                        batter_position,

                        pitcher_id,
                        pitcher_jersey_no: pitcher_no,
                        pitcher_first_name: p_first,
                        pitcher_last_name: p_last,
                    };

                    ui.emit(UiEvent::Line(msg_ab));
                    apply_domain_event(&mut state, &ev_ab);

                    let _ = upsert_at_bat_draft(
                        conn,
                        game_pk,
                        state.inning,
                        state.half,
                        state.current_batter_id,
                        state.current_pitcher_id,
                        &state.pitch_count,
                    );

                    match state.half {
                        HalfInning::Top => state.away_next_batting_order = bump_order(next_order),
                        HalfInning::Bottom => {
                            state.home_next_batting_order = bump_order(next_order)
                        }
                    }
                } else if result.plate_appearance.is_some() {
                    ui.emit(UiEvent::Error(
                        "Next at-bat was not started because the completed plate appearance could not be persisted."
                            .to_string(),
                    ));
                } else if !start_next_at_bat(
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

fn get_player_basic(conn: &Connection, player_id: i64) -> rusqlite::Result<(i32, String, String)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT number, first_name, last_name
        FROM players
        WHERE id = ?1
        LIMIT 1
        "#,
    )?;
    stmt.query_row(params![player_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })
}

fn get_batter_order_and_position(
    conn: &Connection,
    game_id: &str,
    team_id: i64,
    batter_id: i64,
) -> rusqlite::Result<(BatterOrder, Position)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            gl.batting_order,
            gl.defensive_position
        FROM game_lineups gl
        WHERE gl.game_id = ?1
          AND gl.team_id = ?2
          AND gl.player_id = ?3
          AND gl.is_starting = 1
        LIMIT 1
        "#,
    )?;

    stmt.query_row(params![game_id, team_id, batter_id], |row| {
        let order: BatterOrder = row.get::<_, i64>(0)? as u8;

        let position_raw: String = row.get(1)?;

        let position = Position::from_db_value(&position_raw).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                format!("Invalid defensive_position value: {}", position_raw).into(),
            )
        })?;

        Ok((order, position))
    })
}

fn hydrate_current_matchup(
    conn: &Connection,
    game_id: &str,
    state: &mut GameState,
    away_team_id: i64,
    home_team_id: i64,
) -> rusqlite::Result<()> {
    let (batting_team_id, fielding_team_id, next_order) = match state.half {
        HalfInning::Top => (away_team_id, home_team_id, state.away_next_batting_order),
        HalfInning::Bottom => (home_team_id, away_team_id, state.home_next_batting_order),
    };

    if let Some(bid) = state.current_batter_id {
        if let Ok((num, first, last)) = get_player_basic(conn, bid) {
            state.current_batter_jersey_no = Some(num);
            state.current_batter_first_name = Some(first);
            state.current_batter_last_name = Some(last);
        }

        if let Ok((order, position)) =
            get_batter_order_and_position(conn, game_id, batting_team_id, bid)
        {
            state.current_batter_order = Some(order);
            state.current_batter_position = Some(position);
        }
    } else if (1..=9).contains(&next_order)
        && let Ok((batter_id, _abbr, jersey_no, first, last, batter_order, batter_position)) =
            get_batter_by_order(conn, game_id, batting_team_id, next_order)
    {
        state.current_batter_id = Some(batter_id);
        state.current_batter_jersey_no = Some(jersey_no);
        state.current_batter_first_name = Some(first);
        state.current_batter_last_name = Some(last);
        state.current_batter_order = Some(batter_order);
        state.current_batter_position = Some(batter_position);
    }

    if let Some(pid) = state.current_pitcher_id {
        if let Ok((num, first, last)) = get_player_basic(conn, pid) {
            state.current_pitcher_jersey_no = Some(num);
            state.current_pitcher_first_name = Some(first);
            state.current_pitcher_last_name = Some(last);
        }
    } else if let Ok((pid, num, first, last)) = get_current_pitcher(conn, game_id, fielding_team_id)
    {
        state.current_pitcher_id = Some(pid);
        state.current_pitcher_jersey_no = Some(num);
        state.current_pitcher_first_name = Some(first);
        state.current_pitcher_last_name = Some(last);
    }

    Ok(())
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
          AND gl.defensive_position IN ('1', 'P')
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
) -> rusqlite::Result<(i64, String, i32, String, String, BatterOrder, Position)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            p.id,
            t.abbreviation,
            p.number,
            p.first_name,
            p.last_name,
            gl.batting_order,
            gl.defensive_position
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
        let batter_order: BatterOrder = row.get::<_, i64>(5)? as u8;

        let position_raw: String = row.get(6)?;
        let position = Position::from_db_value(&position_raw).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                format!("Invalid defensive_position value: {}", position_raw).into(),
            )
        })?;

        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            batter_order,
            position,
        ))
    })
}

/// Returns the batting order position for a given batter in a game lineup.
/// Uses a direct SQL lookup instead of iterating 1..=9.
fn find_order_for_batter(
    conn: &mut Connection,
    game_id: &str,
    batting_team_id: i64,
    batter_id: i64,
) -> Option<u8> {
    conn.query_row(
        r#"
        SELECT batting_order
        FROM game_lineups
        WHERE game_id = ?1
          AND team_id = ?2
          AND player_id = ?3
          AND is_starting = 1
        LIMIT 1
        "#,
        params![game_id, batting_team_id, batter_id],
        |row| row.get::<_, i64>(0),
    )
    .ok()
    .map(|n| n as u8)
}

pub fn bump_order(x: u8) -> u8 {
    if x >= 9 { 1 } else { x + 1 }
}

pub fn bump_order_str(order: &str) -> u8 {
    match order.parse::<u8>() {
        Ok(n) if (1..=9).contains(&n) => bump_order(n),
        _ => 1,
    }
}

#[derive(Debug, Clone)]
struct FormatLiveAtBatInput {
    inning: u32,
    half: HalfInning,
    outs: u8,
    order: BatterOrder,
    first: String,
    last: String,
    jersey: i32,
    pos: Position,
}

fn live_half_label(inning: u32, half: HalfInning) -> String {
    let sym = match half {
        HalfInning::Top => '↑',
        HalfInning::Bottom => '↓',
    };

    format!("{inning}{sym}")
}

fn format_live_at_bat(flabi: &FormatLiveAtBatInput) -> String {
    let outs_label = if flabi.outs == 1 {
        "1 out".to_string()
    } else {
        format!("{} outs", flabi.outs)
    };

    format!(
        "{:<3} {:<6} At bat: {}. {} {} (#{:>2} {})",
        live_half_label(flabi.inning, flabi.half),
        outs_label,
        flabi.order,
        flabi.first,
        flabi.last,
        flabi.jersey,
        flabi.pos
    )
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
    let (batter_id, team_abbrv, jersey_no, first, last, batter_order, batter_position) =
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

    // 5) Start next at-bat (NOT persisted anymore)
    let msg_ab = format_live_at_bat(&FormatLiveAtBatInput {
        inning: state.inning,
        half: state.half,
        outs: state.outs,
        order: batter_order,
        first: first.clone(),
        last: last.clone(),
        jersey: jersey_no,
        pos: batter_position,
    });

    let ev_ab = DomainEvent::AtBatStarted {
        team_abbrv,
        batting_team_id,

        batter_id,
        batter_jersey_no: jersey_no,
        batter_first_name: first,
        batter_last_name: last,
        batter_order,
        batter_position,

        pitcher_id,
        pitcher_jersey_no: pitcher_no,
        pitcher_first_name: p_first,
        pitcher_last_name: p_last,
    };

    ui.emit(UiEvent::Line(msg_ab));
    apply_domain_event(state, &ev_ab);

    // Create/refresh draft row for resume (even before any pitch is recorded)
    let _ = upsert_at_bat_draft(
        conn,
        game_pk,
        state.inning,
        state.half,
        state.current_batter_id,
        state.current_pitcher_id,
        &state.pitch_count,
    );

    // 6) Advance batting order cursor (resume-safe)
    match state.half {
        HalfInning::Top => state.away_next_batting_order = bump_order(next_order),
        HalfInning::Bottom => state.home_next_batting_order = bump_order(next_order),
    }

    true
}

fn handle_three_outs_and_change_side(
    _conn: &mut Connection,
    ui: &mut dyn Ui,
    _game_pk: i64,
    state: &mut GameState,
) -> bool {
    // Compute next half + inning
    let (next_inning, next_half) = match state.half {
        HalfInning::Top => (state.inning, HalfInning::Bottom),
        HalfInning::Bottom => (state.inning.saturating_add(1), HalfInning::Top),
    };

    // ✅ SideChange is NOT persisted anymore.
    // Resume will be reconstructed from compact PA rows + the at-bat draft.
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

    ui.emit(UiEvent::Line(desc));
    apply_domain_event(state, &ev_side);

    // ✅ In un half inning nuovo, count deve essere 0-0.
    // Se tu già persist CountReset in BB/K, qui comunque è corretto farlo sempre
    // per sicurezza (e per evitare carry-over se l'ultimo evento non lo ha emesso).
    // Ensure count is 0-0 on new half inning.
    apply_domain_event(state, &DomainEvent::CountReset);

    // ⚠️ IMPORTANT: bases should clear on side change.
    // Se NON hai un DomainEvent dedicato, almeno azzera lo stato runtime:
    // (idealmente: aggiungi clearing dentro reducer su SideChange)
    state.on_1b = None;
    state.on_2b = None;
    state.on_3b = None;

    true
}

fn replay_batter_label(pa: &PlateAppearanceRow) -> String {
    pa.batter_order.to_string()
}

fn format_replay_prefix(
    inning: i64,
    half_inning: &str,
    outs: i64,
    batter_label: &str,
    show_half: bool,
    show_outs: bool,
) -> String {
    let half_part = if show_half {
        format!("{}{}", inning, half_symbol(half_inning))
    } else {
        "  ".to_string()
    };

    let outs_part = if show_outs {
        outs_label(outs)
    } else {
        String::new()
    };

    // colonne fisse:
    // - half_part: 3
    // - outs_part: 6
    // - batter_label: 3
    //format!("{:<3} {:<6} {:>3}", half_part, outs_part, batter_label)
    //format!("{:<3} {:<6} {:>2}", half_part, outs_part, batter_label)
    format!("{:<4} {:<7} {:>2}", half_part, outs_part, batter_label)
}

fn half_symbol(half: &str) -> char {
    match half {
        "Top" | "top" => '↑',
        "Bottom" | "bottom" => '↓',
        _ => '?',
    }
}

/// Formatta la sequenza come: [B, K, S, F]
fn format_pitch_sequence(seq: &[PlateAppearanceStep]) -> String {
    let inner = seq
        .iter()
        .map(|step| step.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{}]", inner)
}

fn outcome_symbol_from_row(pa: &PlateAppearanceRow) -> String {
    let base = match pa.outcome_type.as_str() {
        "walk" | "bb" => "BB".to_string(),
        "strikeout" | "k" => "K".to_string(),
        "in_play" | "inplay" => "IP".to_string(),
        "out" => "OUT".to_string(),
        "single" | "h" => "H".to_string(),
        "double" | "2h" => "2H".to_string(),
        "triple" | "3h" => "3H".to_string(),
        "home_run" | "hr" => "HR".to_string(),
        _ => "OUT".to_string(),
    };

    match pa.outcome_type.as_str() {
        "single" | "double" | "triple" | "home_run" => {
            if let Some(raw) = pa.outcome_data.as_deref()
                && let Ok(data) =
                    serde_json::from_str::<crate::models::plate_appearance::HitOutcomeData>(raw)
                && let Some(zone) = data.zone
            {
                return format!("{} {}", base, zone.as_str());
            }
            base
        }
        _ => base,
    }
}

fn replay_admin_logs(ui: &mut dyn Ui, rows: &[GameEventRow]) {
    for r in rows {
        if let Some(desc) = &r.description {
            ui.emit(UiEvent::Line(desc.clone()));
        }
    }
}
pub(crate) fn parse_pa_sequence(json: &str) -> Vec<PlateAppearanceStep> {
    if let Ok(seq) = serde_json::from_str::<Vec<PlateAppearanceStep>>(json) {
        return seq;
    }

    if let Ok(old_seq) = serde_json::from_str::<Vec<Pitch>>(json) {
        return old_seq
            .into_iter()
            .map(PlateAppearanceStep::Pitch)
            .collect();
    }

    Vec::new()
}

fn outs_label(outs: i64) -> String {
    if outs == 1 {
        "1 out".to_string()
    } else {
        format!("{outs} outs")
    }
}

fn runs_scored_from_pa(state_before: &GameState, pa: &PlateAppearanceRow) -> u32 {
    let mut runs = 0;

    if pa.outcome_type.as_str() == "home_run" {
        runs += 1; // batter

        if state_before.on_1b.is_some() {
            runs += 1;
        }
        if state_before.on_2b.is_some() {
            runs += 1;
        }
        if state_before.on_3b.is_some() {
            runs += 1;
        }
    }

    runs
}

fn replay_plate_appearances_and_log(
    ui: &mut dyn Ui,
    state: &mut GameState,
    pas: &[PlateAppearanceRow],
) {
    let mut last_inning: Option<i64> = None;
    let mut last_half: Option<String> = None;
    let mut last_outs: Option<i64> = None;

    for pa in pas {
        // snapshot stato prima della PA
        let state_before = state.clone();

        // applica PA (ricostruisce lo stato)
        apply_plate_appearance_row(state, pa);

        // ricostruzione sequenza
        let seq = parse_pa_sequence(&pa.pitches_sequence);
        let seq_text = format_pitch_sequence(&seq);
        let outcome_sym = outcome_symbol_from_row(pa);

        // calcolo punti segnati
        let runs = runs_scored_from_pa(&state_before, pa);
        let run_text = if runs > 0 {
            format!(" (+{})", runs)
        } else {
            String::new()
        };

        // label battitore (temporanea: poi la sostituiremo col vero batting_order string)
        let batter_label = replay_batter_label(pa);

        // mostro inning/half solo al primo PA del half inning
        let show_half = match (&last_inning, &last_half) {
            (Some(prev_inning), Some(prev_half)) => {
                *prev_inning != pa.inning || prev_half != &pa.half_inning
            }
            _ => true,
        };

        // mostro outs solo quando cambiano, oppure quando cambia half inning
        let show_outs = if show_half {
            true
        } else {
            match last_outs {
                Some(prev_outs) => prev_outs != pa.outs,
                None => true,
            }
        };

        let prefix = format_replay_prefix(
            pa.inning,
            &pa.half_inning,
            pa.outs,
            &batter_label,
            show_half,
            show_outs,
        );

        ui.emit(UiEvent::Line(format!(
            "{} -> {} -> {}{}",
            prefix, seq_text, outcome_sym, run_text
        )));

        last_inning = Some(pa.inning);
        last_half = Some(pa.half_inning.clone());
        last_outs = Some(pa.outs);
    }
}

fn load_and_apply_draft(
    conn: &mut Connection,
    ui: &mut dyn Ui,
    game_pk: i64,
    state: &mut GameState,
) -> Option<AtBatDraftRow> {
    let draft_opt: Option<AtBatDraftRow> = load_at_bat_draft(conn, game_pk).ok().flatten();

    let Some(draft) = &draft_opt else { return None };

    let Ok(pc) = serde_json::from_str::<crate::PitchCount>(&draft.pitch_count_json) else {
        ui.emit(UiEvent::Error(
            "Invalid pitch_count_json in at_bat_draft. Ignoring draft.".to_string(),
        ));
        return None;
    };

    let prev_inning = state.inning;
    let prev_half = state.half;

    let draft_inning = draft.inning as u32;
    let draft_half = if draft.half_inning == "Bottom" {
        HalfInning::Bottom
    } else {
        HalfInning::Top
    };

    state.inning = draft_inning;
    state.half = draft_half;

    if prev_inning != draft_inning || prev_half != draft_half {
        state.outs = 0;
        state.on_1b = None;
        state.on_2b = None;
        state.on_3b = None;
    }

    state.current_batter_id = draft.batter_id;
    state.current_batter_jersey_no = None;
    state.current_batter_first_name = None;
    state.current_batter_last_name = None;
    state.current_batter_order = None;
    state.current_batter_position = None;

    state.current_pitcher_id = draft.pitcher_id;
    state.current_pitcher_jersey_no = None;
    state.current_pitcher_first_name = None;
    state.current_pitcher_last_name = None;

    state.pitch_count = pc;

    // Ricostruzione statistiche pitcher dal draft
    if let Some(pid) = state.current_pitcher_id {
        let stats = state.pitcher_stats.entry(pid).or_default();

        for pitch in &state.pitch_count.sequence {
            match pitch {
                Pitch::Ball => stats.balls += 1,
                _ => stats.strikes += 1,
            }
        }
    }

    ui.emit(UiEvent::Line(
        "(Resume) Restored in-progress at-bat draft.".to_string(),
    ));

    draft_opt
}
