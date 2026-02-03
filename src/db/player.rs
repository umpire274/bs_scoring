use crate::models::types::Position;
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: Option<i64>,
    pub team_id: i64,
    pub number: i32,
    pub name: String,
    pub position: Position,
    pub batting_order: Option<i32>,
    pub is_active: bool,
}

impl Player {
    pub fn new(
        team_id: i64,
        number: i32,
        name: String,
        position: Position,
        batting_order: Option<i32>,
    ) -> Self {
        Player {
            id: None,
            team_id,
            number,
            name,
            position,
            batting_order,
            is_active: true,
        }
    }

    /// Helper function to map a database row to a Player struct
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let position_num: u8 = row.get(4)?;
        Ok(Player {
            id: Some(row.get(0)?),
            team_id: row.get(1)?,
            number: row.get(2)?,
            name: row.get(3)?,
            position: Position::from_number(position_num).unwrap_or(Position::RightField),
            batting_order: row.get(5)?,
            is_active: row.get(6)?,
        })
    }

    /// Helper to map a database row with team_name to (Player, String)
    /// Expects columns: id, team_id, number, name, position, batting_order, is_active, team_name
    pub fn from_row_with_team(row: &rusqlite::Row) -> rusqlite::Result<(Self, String)> {
        let player = Self::from_row(row)?;
        let team_name: String = row.get(7)?;
        Ok((player, team_name))
    }

    /// Create a new player
    pub fn create(&mut self, conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT INTO players (team_id, number, name, position, batting_order, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                self.team_id,
                self.number,
                self.name,
                self.position.to_number(),
                self.batting_order,
                self.is_active
            ],
        )?;

        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    /// Get player by ID
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Player> {
        let mut stmt = conn.prepare(
            "SELECT id, team_id, number, name, position, batting_order, is_active
             FROM players WHERE id = ?1",
        )?;

        stmt.query_row(params![id], Self::from_row)
    }

    /// Get all players for a team
    pub fn get_by_team(conn: &Connection, team_id: i64) -> Result<Vec<Player>> {
        let mut stmt = conn.prepare(
            "SELECT id, team_id, number, name, position, batting_order, is_active
             FROM players WHERE team_id = ?1 AND is_active = 1
             ORDER BY batting_order, number",
        )?;

        let players = stmt.query_map(params![team_id], Self::from_row)?;

        players.collect()
    }

    /// Update player
    pub fn update(&self, conn: &Connection) -> Result<()> {
        if let Some(id) = self.id {
            conn.execute(
                "UPDATE players SET team_id = ?1, number = ?2, name = ?3,
                 position = ?4, batting_order = ?5, is_active = ?6 WHERE id = ?7",
                params![
                    self.team_id,
                    self.number,
                    self.name,
                    self.position.to_number(),
                    self.batting_order,
                    self.is_active,
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

        let mut player = Player::new(
            team_id,
            99,
            "Aaron Judge".to_string(),
            Position::RightField,
            Some(1),
        );
        let player_id = player.create(conn).unwrap();
        assert!(player_id > 0);

        let roster = Player::get_by_team(conn, team_id).unwrap();
        assert_eq!(roster.len(), 1);
    }
}
