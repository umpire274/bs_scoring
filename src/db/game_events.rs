use crate::models::events::DomainEvent;
use crate::models::types::HalfInning;
use rusqlite::{Connection, params};

#[derive(Debug, Clone)]
pub struct GameEventRow {
    pub id: i64,
    pub inning: i64,
    pub half_inning: String,
    pub event_type: String,
    pub event_data: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

pub fn list_game_events(conn: &Connection, game_pk: i64) -> rusqlite::Result<Vec<GameEventRow>> {
    let mut stmt = conn.prepare(
        r#"
            SELECT id,
                   inning,
                   half_inning,
                   event_type,
                   event_data,
                   description,
                   created_at
            FROM game_events
            WHERE game_id = ?1
            ORDER BY id ASC
            "#,
    )?;

    let rows = stmt
        .query_map(params![game_pk], |r| {
            Ok(GameEventRow {
                id: r.get(0)?,
                inning: r.get(1)?,
                half_inning: r.get(2)?,
                event_type: r.get(3)?,
                event_data: r.get(4)?,
                description: r.get(5)?,
                created_at: r.get(6)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(rows)
}

pub fn append_game_event(
    conn: &Connection,
    game_pk: i64,
    inning: u32,
    half: HalfInning,
    event: &DomainEvent,
    description: &str,
) -> rusqlite::Result<()> {
    // Skip persisting noisy pitch-by-pitch events to keep DB compact.
    // These are still applied to in-memory state; resume uses `at_bat_draft` + summary events.
    match event {
        DomainEvent::PitchRecorded { .. } => return Ok(()),
        DomainEvent::CountReset => return Ok(()),
        _ => {}
    }

    let half_str = match half {
        HalfInning::Top => "Top",
        HalfInning::Bottom => "Bottom",
    };

    let data = serde_json::to_string(event).ok();

    conn.execute(
        "INSERT INTO game_events (game_id, at_bat_id, inning, half_inning, event_type, event_data, description)\
         VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6)",
        params![
            game_pk,
            inning as i64,
            half_str,
            event.event_type(),
            data,
            description
        ],
    )?;

    Ok(())
}

pub fn get_lineup_batter_by_order(
    conn: &Connection,
    game_id_str: &str,
    team_id: i64,
    batting_order: i32,
) -> rusqlite::Result<(i64, String, String)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT p.id, p.first_name, p.last_name
        FROM game_lineups gl
        JOIN players p ON gl.player_id = p.id
        WHERE gl.game_id = ?1
          AND gl.team_id = ?2
          AND gl.is_starting = 1
          AND gl.batting_order = ?3
        LIMIT 1
        "#,
    )?;

    stmt.query_row(
        rusqlite::params![game_id_str, team_id, batting_order],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}
