use crate::models::play_ball::{LineupSide, PlayBallGameContext, PlayBallGate};
use crate::models::types::GameStatus;
use rusqlite::{Connection, params};

pub fn list_playable_games(conn: &Connection) -> rusqlite::Result<Vec<PlayBallGameContext>> {
    use crate::models::types::GameStatus;

    let excluded = [
        GameStatus::Regulation.to_i64(),
        GameStatus::Cancelled.to_i64(),
        GameStatus::Forfeited.to_i64(),
    ];

    let mut stmt = conn.prepare(
        r#"
        SELECT g.id, g.game_id, g.game_date, g.venue,
               t1.id as away_team_id, t1.name as away_team, t1.abbreviation as away_abbr,
               t2.id as home_team_id, t2.name as home_team, t2.abbreviation as home_abbr,
               g.status
        FROM games g
        JOIN teams t1 ON g.away_team_id = t1.id
        JOIN teams t2 ON g.home_team_id = t2.id
        WHERE g.status NOT IN (?1, ?2, ?3)
        ORDER BY g.game_date DESC, g.id DESC
        "#,
    )?;

    let mut rows = stmt.query(rusqlite::params![excluded[0], excluded[1], excluded[2]])?;
    let mut v = Vec::new();

    while let Some(row) = rows.next()? {
        let status_i64: i64 = row.get(10)?;
        let status = GameStatus::from_i64(status_i64).unwrap_or(GameStatus::Pregame);

        v.push(PlayBallGameContext {
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

            status,
        });
    }

    // Diagnostic: if no games found, log statuses present to help debugging
    if v.is_empty() {
        let mut sstmt =
            conn.prepare("SELECT status, COUNT(1) FROM games GROUP BY status ORDER BY status")?;
        let mut srows = sstmt.query([])?;
        let mut diagnostics = Vec::new();
        while let Some(srow) = srows.next()? {
            let st: i64 = srow.get(0)?;
            let cnt: i64 = srow.get(1)?;
            diagnostics.push(format!("status {} => {} games", st, cnt));
        }
        // Use eprintln to not interfere with CLI/TUI; the caller can decide how to show it.
        eprintln!(
            "list_pregame_games: no playable games found. DB status summary: {:?}",
            diagnostics
        );
    }

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
