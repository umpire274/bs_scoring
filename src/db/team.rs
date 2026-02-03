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

    /// Helper function to map a database row to a Team struct
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Team {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            league_id: row.get(2)?,
            city: row.get(3)?,
            abbreviation: row.get(4)?,
            founded_year: row.get(5)?,
        })
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
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Team> {
        let mut stmt = conn.prepare(
            "SELECT id, name, league_id, city, abbreviation, founded_year
             FROM teams WHERE id = ?1",
        )?;

        stmt.query_row(params![id], Self::from_row)
    }

    /// Get all teams
    pub fn get_all(conn: &Connection) -> Result<Vec<Team>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, league_id, city, abbreviation, founded_year
             FROM teams ORDER BY name",
        )?;

        let teams = stmt.query_map([], Self::from_row)?;

        teams.collect()
    }

    /// Get teams by league
    pub fn get_by_league(conn: &Connection, league_id: i64) -> Result<Vec<Team>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, league_id, city, abbreviation, founded_year
             FROM teams WHERE league_id = ?1 ORDER BY name",
        )?;

        let teams = stmt.query_map(params![league_id], Self::from_row)?;

        teams.collect()
    }

    /// Update team
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
    pub fn get_roster(&self, conn: &Connection) -> Result<Vec<crate::db::player::Player>> {
        if let Some(team_id) = self.id {
            crate::db::player::Player::get_by_team(conn, team_id)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::database::Database;

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
}
