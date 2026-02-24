use crate::models::play_ball::{LineupSide, PlayBallGameContext, PlayBallGate};
use crate::models::types::GameStatus;
use rusqlite::{Connection, params};

pub fn list_pregame_games(conn: &Connection) -> rusqlite::Result<Vec<PlayBallGameContext>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.game_id, g.game_date, g.venue,
                t1.id as away_team_id, t1.name as away_team, t1.abbreviation as away_abbr,
                t2.id as home_team_id, t2.name as home_team, t2.abbreviation as home_abbr
         FROM games g
         JOIN teams t1 ON g.away_team_id = t1.id
         JOIN teams t2 ON g.home_team_id = t2.id
         WHERE g.status = 1
         ORDER BY g.game_date DESC, g.id DESC",
    )?;

    let v = stmt
        .query_map([], |row| {
            Ok(PlayBallGameContext {
                id: row.get(0)?,
                game_id: row.get(1)?,
                game_date: row.get(2)?,
                venue: row.get(3)?,

                away_team_id: row.get(4)?,
                away_team_name: row.get(5)?,
                away_team_abbr: row.get(6)?,

                home_team_id: row.get(7)?,
                home_team_name: row.get(8)?,
                home_team_abbr: row.get(9)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(v)
}

pub fn gate_check_lineups(
    conn: &Connection,
    game_id: &str,
    away_team_id: i64,
    home_team_id: i64,
) -> rusqlite::Result<PlayBallGate> {
    let (at_uses_dh, ht_uses_dh): (bool, bool) = conn.query_row(
        "SELECT at_uses_dh, ht_uses_dh
         FROM games
         WHERE game_id = ?1",
        params![game_id],
        |r| Ok((r.get::<_, i64>(0)? != 0, r.get::<_, i64>(1)? != 0)),
    )?;

    let away_required = if at_uses_dh { 10 } else { 9 };
    let home_required = if ht_uses_dh { 10 } else { 9 };

    let away_found = starting_lineup_count(conn, game_id, away_team_id)?;
    if away_found != away_required {
        return Ok(PlayBallGate::InvalidLineup {
            side: LineupSide::Away,
            required: away_required,
            found: away_found,
        });
    }

    let home_found = starting_lineup_count(conn, game_id, home_team_id)?;
    if home_found != home_required {
        return Ok(PlayBallGate::InvalidLineup {
            side: LineupSide::Home,
            required: home_required,
            found: home_found,
        });
    }

    Ok(PlayBallGate::Ready)
}

fn starting_lineup_count(conn: &Connection, game_id: &str, team_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(1)
         FROM game_lineups
         WHERE game_id = ?1
           AND team_id = ?2
           AND is_starting = 1",
        params![game_id, team_id],
        |r| r.get(0),
    )
}

pub fn set_game_status(
    conn: &mut Connection,
    game_id: &str,
    status: GameStatus,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE games SET status = ?2 WHERE game_id = ?1",
        params![game_id, status.to_i64()],
    )?;
    Ok(changed == 1)
}
