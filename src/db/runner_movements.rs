//! Persistence layer for `runner_movements` — one row per runner per play.
//!
//! Every time a runner changes base (hit advancement, walk, steal, future plays)
//! a row is written here. This is the authoritative source for replay of base
//! state, separate from the plate appearance outcomes stored in `plate_appearances`.

use rusqlite::{Connection, Result, params};

// ─── Row type (for read / replay) ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RunnerMovementRow {
    pub id: i64,
    pub game_id: i64,
    /// PA sequence number — NULL for non-PA events (steal).
    pub pa_seq: Option<i64>,
    /// game_events row id — NULL for PA movements.
    pub game_event_id: Option<i64>,
    pub inning: i64,
    pub half_inning: String,
    /// Player id — None when identity not resolved at scoring time.
    pub runner_id: Option<i64>,
    pub batter_order: u8,
    /// `"BAT"`, `"1B"`, `"2B"`, `"3B"`
    pub start_base: String,
    /// `"1B"`, `"2B"`, `"3B"`, `"HOME"`, `"OUT"`
    pub end_base: String,
    /// `"hit_auto"`, `"hit_override"`, `"walk"`, `"steal"`, …
    pub advancement_type: String,
    pub is_out: bool,
    pub scored: bool,
    pub is_earned: bool,
}

// ─── Write helpers ────────────────────────────────────────────────────────────

/// Parameters for a single runner movement row.
pub struct RunnerMovementInsert {
    pub game_id: i64,
    pub pa_seq: Option<i64>,
    pub game_event_id: Option<i64>,
    pub inning: u32,
    pub half_inning: String,
    /// Player id — use None when identity not resolved.
    pub runner_id: Option<i64>,
    pub batter_order: u8,
    pub start_base: &'static str,
    pub end_base: &'static str,
    pub advancement_type: &'static str,
    pub is_out: bool,
    pub scored: bool,
    pub is_earned: bool,
}

/// Insert a single runner movement row. Returns the new row id.
pub fn append_runner_movement(
    conn: &Connection,
    m: &RunnerMovementInsert,
) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO runner_movements (
            game_id, pa_seq, game_event_id,
            inning, half_inning,
            runner_id, batter_order,
            start_base, end_base, advancement_type,
            is_out, scored, is_earned
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            m.game_id,
            m.pa_seq,
            m.game_event_id,
            m.inning as i64,
            m.half_inning,
            m.runner_id,
            m.batter_order as i64,
            m.start_base,
            m.end_base,
            m.advancement_type,
            m.is_out as i64,
            m.scored as i64,
            m.is_earned as i64,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

// ─── Read helpers ─────────────────────────────────────────────────────────────

/// Load all runner movements for a game, ordered for replay.
pub fn list_runner_movements(
    conn: &Connection,
    game_pk: i64,
) -> Result<Vec<RunnerMovementRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, game_id, pa_seq, game_event_id,
               inning, half_inning,
               runner_id, batter_order,
               start_base, end_base, advancement_type,
               is_out, scored, is_earned
        FROM runner_movements
        WHERE game_id = ?1
        ORDER BY inning ASC,
                 CASE half_inning WHEN 'Top' THEN 0 ELSE 1 END ASC,
                 COALESCE(pa_seq, 999999) ASC,
                 COALESCE(game_event_id, 999999) ASC,
                 id ASC
        "#,
    )?;

    let rows = stmt
        .query_map(params![game_pk], |r| {
            Ok(RunnerMovementRow {
                id:               r.get(0)?,
                game_id:          r.get(1)?,
                pa_seq:           r.get(2)?,
                game_event_id:    r.get(3)?,
                inning:           r.get(4)?,
                half_inning:      r.get(5)?,
                runner_id:        r.get(6)?,
                batter_order:     r.get(7)?,
                start_base:       r.get(8)?,
                end_base:         r.get(9)?,
                advancement_type: r.get(10)?,
                is_out:           r.get::<_, i64>(11)? != 0,
                scored:           r.get::<_, i64>(12)? != 0,
                is_earned:        r.get::<_, i64>(13)? != 0,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(rows)
}
