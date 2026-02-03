use crate::models::types::Position;
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Team {
    pub id: Option<i64>,
    pub name: String,
    pub league_id: Option<i64>,
    pub city: Option<String>,
    pub abbreviation: Option<String>,
    pub founded_year: Option<i32>,
}

#[allow(dead_code)]
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

impl Team {
    pub fn new(
        name: String,
        league_id: Option<i64>,
        city: Option<String>,
        abbreviation: Option<String>,
        founded_year: Option<i32>,
    ) -> Self {
        Team {
            id: None,
            name,
            league_id,
            city,
            abbreviation,
            founded_year,
        }
    }

    /// Create a new team
    pub fn create(&mut self, conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT INTO teams (name, league_id, city, abbreviation, founded_year) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.name,
                self.league_id,
                self.city,
                self.abbreviation,
                self.founded_year
            ],
        )?;

        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    /// Get team by ID
    #[allow(dead_code)]
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Team> {
        let mut stmt = conn.prepare(
            "SELECT id, name, league_id, city, abbreviation, founded_year 
             FROM teams WHERE id = ?1",
        )?;

        stmt.query_row(params![id], |row| {
            Ok(Team {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                league_id: row.get(2)?,
                city: row.get(3)?,
                abbreviation: row.get(4)?,
                founded_year: row.get(5)?,
            })
        })
    }

    /// Get all teams
    pub fn get_all(conn: &Connection) -> Result<Vec<Team>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, league_id, city, abbreviation, founded_year 
             FROM teams ORDER BY name",
        )?;

        let teams = stmt.query_map([], |row| {
            Ok(Team {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                league_id: row.get(2)?,
                city: row.get(3)?,
                abbreviation: row.get(4)?,
                founded_year: row.get(5)?,
            })
        })?;

        teams.collect()
    }

    /// Get teams by league
    #[allow(dead_code)]
    pub fn get_by_league(conn: &Connection, league_id: i64) -> Result<Vec<Team>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, league_id, city, abbreviation, founded_year 
             FROM teams WHERE league_id = ?1 ORDER BY name",
        )?;

        let teams = stmt.query_map(params![league_id], |row| {
            Ok(Team {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                league_id: row.get(2)?,
                city: row.get(3)?,
                abbreviation: row.get(4)?,
                founded_year: row.get(5)?,
            })
        })?;

        teams.collect()
    }

    /// Update team
    #[allow(dead_code)]
    pub fn update(&self, conn: &Connection) -> Result<()> {
        if let Some(id) = self.id {
            conn.execute(
                "UPDATE teams SET name = ?1, league_id = ?2, city = ?3, 
                 abbreviation = ?4, founded_year = ?5 WHERE id = ?6",
                params![
                    self.name,
                    self.league_id,
                    self.city,
                    self.abbreviation,
                    self.founded_year,
                    id
                ],
            )?;
        }
        Ok(())
    }

    /// Delete team
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        // First delete all players
        conn.execute("DELETE FROM players WHERE team_id = ?1", params![id])?;
        // Then delete team
        conn.execute("DELETE FROM teams WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Get team's roster (players)
    #[allow(dead_code)]
    pub fn get_roster(&self, conn: &Connection) -> Result<Vec<Player>> {
        if let Some(team_id) = self.id {
            Player::get_by_team(conn, team_id)
        } else {
            Ok(Vec::new())
        }
    }
}

#[allow(dead_code)]
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

        stmt.query_row(params![id], |row| {
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
        })
    }

    /// Get all players for a team
    pub fn get_by_team(conn: &Connection, team_id: i64) -> Result<Vec<Player>> {
        let mut stmt = conn.prepare(
            "SELECT id, team_id, number, name, position, batting_order, is_active 
             FROM players WHERE team_id = ?1 AND is_active = 1 
             ORDER BY batting_order, number",
        )?;

        let players = stmt.query_map(params![team_id], |row| {
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
        })?;

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
    use crate::models::database::Database;

    #[test]
    fn test_team_crud() {
        let db = Database::new(":memory:").unwrap();
        db.init_schema().unwrap();
        let conn = db.get_connection();

        let mut team = Team::new(
            "Red Sox".to_string(),
            None,
            Some("Boston".to_string()),
            Some("BOS".to_string()),
            Some(1901),
        );
        let id = team.create(conn).unwrap();
        assert!(id > 0);

        let retrieved = Team::get_by_id(conn, id).unwrap();
        assert_eq!(retrieved.name, "Red Sox");
    }

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

        let roster = team.get_roster(conn).unwrap();
        assert_eq!(roster.len(), 1);
    }
}
