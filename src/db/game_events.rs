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
