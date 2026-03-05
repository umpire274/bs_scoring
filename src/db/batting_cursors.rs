use rusqlite::{Connection, params};

#[derive(Debug, Clone)]
pub struct BattingCursorsRow {
    pub game_id: i64,
    pub away_next_batting_order: i64,
    pub home_next_batting_order: i64,
    pub updated_at: Option<String>,
}

pub fn load_batting_cursors(
    conn: &Connection,
    game_pk: i64,
) -> rusqlite::Result<Option<BattingCursorsRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT game_id, away_next_batting_order, home_next_batting_order, updated_at
        FROM batting_cursors
        WHERE game_id = ?1
        LIMIT 1
        "#,
    )?;

    let row = stmt.query_row(params![game_pk], |r| {
        Ok(BattingCursorsRow {
            game_id: r.get(0)?,
            away_next_batting_order: r.get(1)?,
            home_next_batting_order: r.get(2)?,
            updated_at: r.get(3)?,
        })
    });

    match row {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn upsert_batting_cursors(
    conn: &Connection,
    game_pk: i64,
    away_next_batting_order: u8,
    home_next_batting_order: u8,
) -> rusqlite::Result<()> {
    conn.execute(
        r#"
        INSERT INTO batting_cursors (game_id, away_next_batting_order, home_next_batting_order, updated_at)
        VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
        ON CONFLICT(game_id) DO UPDATE SET
            away_next_batting_order = excluded.away_next_batting_order,
            home_next_batting_order = excluded.home_next_batting_order,
            updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            game_pk,
            away_next_batting_order as i64,
            home_next_batting_order as i64,
        ],
    )?;

    Ok(())
}
