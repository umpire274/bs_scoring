use crate::models::player_traits::{BatSide, ThrowHand};
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: Option<i64>,
    pub team_id: i64,
    pub number: i32,
    pub away_number: i32,
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub throw: Option<ThrowHand>,
    pub bat: Option<BatSide>,
    pub is_active: bool,
}

pub struct NewPlayer {
    pub team_id: i64,
    pub number: i32,
    pub away_number: i32,
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub throw: Option<ThrowHand>,
    pub bat: Option<BatSide>,
}

impl Player {
    pub fn new(data: NewPlayer) -> Self {
        Player {
            id: None,
            team_id: data.team_id,
            number: data.number,
            away_number: data.away_number,
            first_name: data.first_name,
            last_name: data.last_name,
            position: data.position,
            throw: data.throw,
            bat: data.bat,
            is_active: true,
        }
    }

    /// Return the jersey number to display/use for a home or away game.
    pub fn jersey_number(&self, is_home_team: bool) -> i32 {
        if is_home_team {
            self.number
        } else {
            self.away_number
        }
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Helper function to map a database row to a Player struct
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        let away_number: i32 = row.get(9).or_else(|_| row.get(2))?;

        Ok(Player {
            id: Some(row.get(0)?),
            team_id: row.get(1)?,
            number: row.get(2)?,
            away_number,
            first_name: row.get(3)?,
            last_name: row.get(4)?,
            position: row.get(5)?,
            throw: row.get(6).ok().and_then(|s: String| ThrowHand::parse(&s)),
            bat: row.get(7).ok().and_then(|s: String| BatSide::parse(&s)),
            is_active: row.get(8)?,
        })
    }

    /// Helper to map a database row with team_name to (Player, String).
    /// Expects columns: id, team_id, number, first_name, last_name, position, throw, bat,
    /// currently backed by the legacy `pitch` database column.
    pub fn from_row_with_team(row: &rusqlite::Row) -> Result<(Self, String)> {
        let player = Self::from_row(row)?;
        let team_name: String = row.get(10)?;
        Ok((player, team_name))
    }

    /// Create a new player
    pub fn create(&mut self, conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT INTO players (team_id, number, first_name, last_name, position, throw, bat, is_active, away_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                self.team_id,
                self.number,
                self.first_name,
                self.last_name,
                self.position,
                self.throw.map(|p| p.as_str().to_string()),
                self.bat.map(|b| b.as_str().to_string()),
                self.is_active,
                self.away_number
            ],
        )?;

        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    /// Get player by ID
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Player> {
        let mut stmt = conn.prepare(
            "SELECT id, team_id, number, first_name, last_name, position, bat, throw, is_active,
                    COALESCE(away_number, number) AS away_number
             FROM players WHERE id = ?1",
        )?;

        stmt.query_row(params![id], Self::from_row)
    }

    /// Get all players for a team
    pub fn get_by_team(conn: &Connection, team_id: i64) -> Result<Vec<Player>> {
        let mut stmt = conn.prepare(
            "SELECT id, team_id, number, first_name, last_name, position, bat, throw, is_active,
                    COALESCE(away_number, number) AS away_number
             FROM players WHERE team_id = ?1 AND is_active = 1
             ORDER BY number",
        )?;

        let players = stmt.query_map(params![team_id], Self::from_row)?;

        players.collect()
    }

    /// Update player
    pub fn update(&self, conn: &Connection) -> Result<()> {
        if let Some(id) = self.id {
            conn.execute(
                "UPDATE players SET team_id = ?1, number = ?2, first_name = ?3, last_name = ?4,
                 position = ?5, throw = ?6, bat = ?7, is_active = ?8, away_number = ?9 WHERE id = ?10",
                params![
                    self.team_id,
                    self.number,
                    self.first_name,
                    self.last_name,
                    self.position,
                    self.throw.map(|p| p.as_str().to_string()),
                    self.bat.map(|b| b.as_str().to_string()),
                    self.is_active,
                    self.away_number,
                    id
                ],
            )?;
        }
        Ok(())
    }

    /// Delete player
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM players WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::database::Database;
    use crate::db::team::Team;

    #[test]
    fn test_player_crud() {
        let db = Database::new(":memory:").unwrap();
        db.init_schema().unwrap();
        let conn = db.get_connection();

        let mut team = Team::new("Yankees".to_string(), None, None, None, None);
        let team_id = team.create(conn).unwrap();

        let mut player = Player::new(NewPlayer {
            team_id,
            number: 99,
            away_number: 99,
            first_name: "Aaron".to_string(),
            last_name: "Judge".to_string(),
            position: "RF".to_string(),
            throw: Some(ThrowHand::R),
            bat: Some(BatSide::R),
        });
        let player_id = player.create(conn).unwrap();
        assert!(player_id > 0);

        let roster = Player::get_by_team(conn, team_id).unwrap();
        assert_eq!(roster.len(), 1);
        assert_eq!(roster[0].full_name(), "Aaron Judge");
    }
}
