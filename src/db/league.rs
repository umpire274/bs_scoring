use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct League {
    pub id: Option<i64>,
    pub name: String,
    pub season: Option<String>,
    pub description: Option<String>,
}

impl League {
    pub fn new(name: String, season: Option<String>, description: Option<String>) -> Self {
        League {
            id: None,
            name,
            season,
            description,
        }
    }

    /// Helper function to map a database row to a League struct
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(League {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            season: row.get(2)?,
            description: row.get(3)?,
        })
    }

    /// Create a new league in the database
    pub fn create(&mut self, conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT INTO leagues (name, season, description) VALUES (?1, ?2, ?3)",
            params![self.name, self.season, self.description],
        )?;

        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    /// Get a league by ID
    #[allow(dead_code)]
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<League> {
        let mut stmt =
            conn.prepare("SELECT id, name, season, description FROM leagues WHERE id = ?1")?;

        stmt.query_row(params![id], Self::from_row)
    }

    /// Get all leagues
    pub fn get_all(conn: &Connection) -> Result<Vec<League>> {
        let mut stmt =
            conn.prepare("SELECT id, name, season, description FROM leagues ORDER BY name")?;

        let leagues = stmt.query_map([], Self::from_row)?;

        leagues.collect()
    }

    /// Update league
    pub fn update(&self, conn: &Connection) -> Result<()> {
        if let Some(id) = self.id {
            conn.execute(
                "UPDATE leagues SET name = ?1, season = ?2, description = ?3 WHERE id = ?4",
                params![self.name, self.season, self.description, id],
            )?;
        }
        Ok(())
    }

    /// Delete league
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM leagues WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::database::Database;

    #[test]
    fn test_league_crud() {
        let db = Database::new(":memory:").unwrap();
        db.init_schema().unwrap();
        let conn = db.get_connection();

        // Create
        let mut league = League::new(
            "MLB".to_string(),
            Some("2026".to_string()),
            Some("Major League Baseball".to_string()),
        );
        let id = league.create(conn).unwrap();
        assert!(id > 0);

        // Read
        let retrieved = League::get_by_id(conn, id).unwrap();
        assert_eq!(retrieved.name, "MLB");

        // Update
        let mut updated = retrieved.clone();
        updated.season = Some("2027".to_string());
        updated.update(conn).unwrap();

        // Delete
        League::delete(conn, id).unwrap();
    }
}
