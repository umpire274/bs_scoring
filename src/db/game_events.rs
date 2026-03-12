use crate::engine::play_ball::bump_order;
use crate::models::events::DomainEvent;
use crate::models::types::HalfInning;
use rusqlite::{Connection, params};
use std::collections::HashMap;

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

pub fn refactor_batter_order(conn: &mut Connection) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    // 1) prendo l'elenco delle partite
    let game_ids: Vec<i64> = {
        let mut stmt_games = tx.prepare(
            r#"
            SELECT DISTINCT game_id
            FROM plate_appearances
            ORDER BY game_id
            "#,
        )?;

        stmt_games
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?
    };

    // statement UPDATE preparato una sola volta
    let mut stmt_update = tx.prepare(
        r#"
        UPDATE plate_appearances
        SET batter_order = ?1
        WHERE id = ?2
        "#,
    )?;

    for game_pk in game_ids {
        // 2) ricavo home/away team della partita
        let (game_id, away_team_id, home_team_id): (String, i64, i64) = {
            let mut stmt_game = tx.prepare(
                r#"
                SELECT game_id, away_team_id, home_team_id
                FROM games
                WHERE id = ?1
                LIMIT 1
                "#,
            )?;

            stmt_game.query_row([game_pk], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        };

        // 3) costruisco mappa player_id -> batter_order per ciascun team
        let (away_orders, home_orders): (HashMap<i64, u8>, HashMap<i64, u8>) = {
            let mut stmt_lineup = tx.prepare(
                r#"
                SELECT team_id, player_id, batting_order
                FROM game_lineups
                WHERE game_id = ?1
                  AND is_starting = 1
                ORDER BY team_id, batting_order
                "#,
            )?;

            let mut away = HashMap::new();
            let mut home = HashMap::new();

            let rows = stmt_lineup.query_map([&game_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?, // team_id
                    row.get::<_, i64>(1)?, // player_id
                    row.get::<_, i64>(2)?, // batting_order
                ))
            })?;

            for row in rows {
                let (team_id, player_id, batting_order) = row?;
                let order_u8 = batting_order as u8;

                if team_id == away_team_id {
                    away.insert(player_id, order_u8);
                } else if team_id == home_team_id {
                    home.insert(player_id, order_u8);
                }
            }

            (away, home)
        };

        // 4) cursori fallback sequenziali per half-inning
        let mut away_next: u8 = 1;
        let mut home_next: u8 = 1;

        // 5) itero tutte le PA in ordine cronologico
        let pa_rows: Vec<(i64, String, i64, i64)> = {
            let mut stmt_pa = tx.prepare(
                r#"
                SELECT
                    id,
                    half_inning,
                    batter_id,
                    COALESCE(NULLIF(batter_order,''),0) as batter_order
                FROM plate_appearances
                WHERE game_id = ?1
                ORDER BY seq
                "#,
            )?;

            stmt_pa
                .query_map([game_pk], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,    // id
                        row.get::<_, String>(1)?, // half_inning
                        row.get::<_, i64>(2)?,    // batter_id
                        row.get::<_, i64>(3)?,    // batter_order
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?
        };

        for (pa_id, half_inning, batter_id, current_batter_order) in pa_rows {
            // se già valorizzato, lo lascio stare
            if current_batter_order > 0 {
                continue;
            }

            let is_top = half_inning.eq_ignore_ascii_case("Top");

            let resolved_order: u8 = if is_top {
                if let Some(order) = away_orders.get(&batter_id) {
                    away_next = bump_order(*order);
                    *order
                } else {
                    let order = away_next;
                    away_next = bump_order(away_next);
                    order
                }
            } else if let Some(order) = home_orders.get(&batter_id) {
                home_next = bump_order(*order);
                *order
            } else {
                let order = home_next;
                home_next = bump_order(home_next);
                order
            };

            stmt_update.execute(params![resolved_order as i64, pa_id])?;
        }
    }

    drop(stmt_update);
    tx.commit()?;
    Ok(())
}
