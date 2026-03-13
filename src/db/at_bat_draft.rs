use crate::PitchCount;
use crate::models::types::HalfInning;
use rusqlite::{Connection, params};

#[derive(Debug, Clone)]
pub struct AtBatDraftRow {
    pub game_id: i64,
    pub inning: i64,
    pub half_inning: String,
    pub batter_id: Option<i64>,
    pub pitcher_id: Option<i64>,
    pub pitch_count_json: String,
    pub updated_at: Option<String>,
}

pub fn load_at_bat_draft(
    conn: &Connection,
    game_pk: i64,
) -> rusqlite::Result<Option<AtBatDraftRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT game_id, inning, half_inning, batter_id, pitcher_id, pitch_count_json, updated_at
        FROM at_bat_draft
        WHERE game_id = ?1
        LIMIT 1
        "#,
    )?;

    let row = stmt.query_row(params![game_pk], |r| {
        Ok(AtBatDraftRow {
            game_id: r.get(0)?,
            inning: r.get(1)?,
            half_inning: r.get(2)?,
            batter_id: r.get(3)?,
            pitcher_id: r.get(4)?,
            pitch_count_json: r.get(5)?,
            updated_at: r.get(6)?,
        })
    });

    match row {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn upsert_at_bat_draft(
    conn: &Connection,
    game_pk: i64,
    inning: u32,
    half: HalfInning,
    batter_id: Option<i64>,
    pitcher_id: Option<i64>,
    pitch_count: &PitchCount,
) -> rusqlite::Result<()> {
    let half_str = match half {
        HalfInning::Top => "Top",
        HalfInning::Bottom => "Bottom",
    };

    let pitch_count_json = serde_json::to_string(pitch_count)
        .unwrap_or_else(|_| r#"{"balls":0,"strikes":0,"sequence":[]}"#.to_string());

    conn.execute(
        r#"
        INSERT INTO at_bat_draft (game_id, inning, half_inning, batter_id, pitcher_id, pitch_count_json, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
        ON CONFLICT(game_id) DO UPDATE SET
            inning = excluded.inning,
            half_inning = excluded.half_inning,
            batter_id = excluded.batter_id,
            pitcher_id = excluded.pitcher_id,
            pitch_count_json = excluded.pitch_count_json,
            updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            game_pk,
            inning as i64,
            half_str,
            batter_id,
            pitcher_id,
            pitch_count_json
        ],
    )?;

    Ok(())
}

pub fn clear_at_bat_draft(conn: &Connection, game_pk: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM at_bat_draft WHERE game_id = ?1",
        params![game_pk],
    )?;
    Ok(())
}
