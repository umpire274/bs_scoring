use crate::commands::engine_parser::parse_engine_commands;
use crate::commands::types::EngineCommand;
use crate::core::play_ball::set_game_status;
use crate::core::play_ball_apply::apply_engine_command;
use crate::core::play_ball_reducer::{apply_domain_event, apply_plate_appearance_row};
use crate::db::at_bat_draft::{
    AtBatDraftRow, clear_at_bat_draft, load_at_bat_draft, upsert_at_bat_draft,
};
use crate::db::game_events::{GameEventRow, append_game_event, list_game_events};
use crate::db::plate_appearances_compact::{
    PlateAppearanceRow, append_plate_appearance, list_plate_appearances,
};
use crate::models::events::{DomainEvent, SideChangeData};
use crate::models::play_ball::{GameState, OutcomeSymbol};
use crate::ui::Ui;
use crate::ui::events::UiEvent;
use crate::{HalfInning, Pitch};
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

            // If a PA was completed, persist a single compact record
            if let Some(pa) = &result.plate_appearance {
                if let Err(e) = append_plate_appearance(conn, game_pk, pa) {
                    ui.emit(UiEvent::Error(format!(
                        "Failed to append plate appearance: {e}"
                    )));
                } else {
                    // PA is over, clear the draft now.
                    let _ = clear_at_bat_draft(conn, game_pk);
                }
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

/// After resume/replay, ensure `GameState` contains the batter/pitcher display fields
/// used by the TUI scoreboard (jersey + names).
fn hydrate_current_matchup(
    conn: &Connection,
    game_id: &str,
    state: &mut GameState,
    away_team_id: i64,
    home_team_id: i64,
) -> rusqlite::Result<()> {
    // Determine batting/fielding teams based on current half.
    let (batting_team_id, fielding_team_id, next_order) = match state.half {
        HalfInning::Top => (away_team_id, home_team_id, state.away_next_batting_order),
        HalfInning::Bottom => (home_team_id, away_team_id, state.home_next_batting_order),
    };

    // Batter: hydrate by id if present (draft), otherwise from batting order cursor.
    if let Some(bid) = state.current_batter_id {
        if let Ok((num, first, last)) = get_player_basic(conn, bid) {
            state.current_batter_jersey_no = Some(num);
            state.current_batter_first_name = Some(first);
            state.current_batter_last_name = Some(last);
        }
    } else if (1..=9).contains(&next_order)
        && let Ok((batter_id, _abbr, jersey_no, first, last)) =
            get_batter_by_order(conn, game_id, batting_team_id, next_order)
    {
        state.current_batter_id = Some(batter_id);
        state.current_batter_jersey_no = Some(jersey_no);
        state.current_batter_first_name = Some(first);
        state.current_batter_last_name = Some(last);
    }

    // Pitcher: hydrate by id if present (replay/draft), otherwise from starting pitcher.
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

fn find_order_for_batter(
    conn: &mut Connection,
    game_id: &str,
    batting_team_id: i64,
    batter_id: i64,
) -> Option<u8> {
    for order in 1..=9 {
        match get_batter_by_order(conn, game_id, batting_team_id, order) {
            Ok((bid, _, _, _, _)) if bid == batter_id => {
                return Some(order);
            }
            _ => continue,
        }
    }

    None
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

    // 5) Start next at-bat (NOT persisted anymore)
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
    state.on_1b = false;
    state.on_2b = false;
    state.on_3b = false;

    true
}

fn half_symbol(half: &str) -> char {
    match half {
        "Top" | "top" => '↑',
        "Bottom" | "bottom" => '↓',
        _ => '?',
    }
}

/// Formatta la sequenza come: [B, K, S, F]
fn format_pitch_sequence(seq: &[Pitch]) -> String {
    let inner = seq
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}]", inner)
}

fn outcome_symbol_from_outcome_type(outcome_type: &str) -> OutcomeSymbol {
    match outcome_type {
        // Adatta queste stringhe ai valori REALI che salvi nel DB
        "walk" | "bb" => OutcomeSymbol::Walk,
        "strikeout" | "k" => OutcomeSymbol::Strikeout,
        "in_play" | "inplay" => OutcomeSymbol::InPlay,
        "out" => OutcomeSymbol::Out,
        "single" | "1b" => OutcomeSymbol::Single,
        "double" | "2b" => OutcomeSymbol::Double,
        "triple" | "3b" => OutcomeSymbol::Triple,
        "home_run" | "hr" => OutcomeSymbol::HomeRun,
        _ => OutcomeSymbol::Out,
    }
}

fn replay_admin_logs(ui: &mut dyn Ui, rows: &[GameEventRow]) {
    for r in rows {
        if let Some(desc) = &r.description {
            ui.emit(UiEvent::Line(desc.clone()));
        }
    }
}

fn replay_plate_appearances_and_log(
    ui: &mut dyn Ui,
    state: &mut GameState,
    pas: &[PlateAppearanceRow],
) {
    for pa in pas {
        // 1) Source of truth: ricostruisci state
        apply_plate_appearance_row(state, pa);

        match pa.half_inning.as_str() {
            "Top" => state.away_next_batting_order = bump_order(state.away_next_batting_order),
            "Bottom" => state.home_next_batting_order = bump_order(state.home_next_batting_order),
            _ => {}
        }

        // 2) Log scorer-friendly: [B, K, ...] -> K
        let seq: Vec<Pitch> = serde_json::from_str(&pa.pitches_sequence).unwrap_or_default();
        let seq_text = format_pitch_sequence(&seq);
        let outcome_sym = outcome_symbol_from_outcome_type(&pa.outcome_type);

        ui.emit(UiEvent::Line(format!(
            "PA#{}, {}{}, outs {}, {} → {}",
            pa.seq,
            pa.inning,
            half_symbol(&pa.half_inning),
            pa.outs,
            seq_text,
            outcome_sym
        )));
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
        state.on_1b = false;
        state.on_2b = false;
        state.on_3b = false;
    }

    state.current_batter_id = draft.batter_id;
    state.current_pitcher_id = draft.pitcher_id;
    state.pitch_count = pc;

    if let Some(pid) = state.current_pitcher_id {
        let n = state.pitch_count.sequence.len() as u32;
        if n > 0 {
            let entry = state.pitcher_pitch_counts.entry(pid).or_insert(0);
            *entry = entry.saturating_add(n);
            state.current_pitch_count = *entry;
        }
    }

    ui.emit(UiEvent::Line(
        "(Resume) Restored in-progress at-bat draft.".to_string(),
    ));

    draft_opt
}
