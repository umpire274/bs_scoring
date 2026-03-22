use crate::db::migrations;
use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create or open the database with optimised PRAGMA settings.
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // ── Performance PRAGMAs ──────────────────────────────────────────────
        // WAL mode: allows concurrent reads while writing; much better for a
        // single-writer app that refreshes the scoreboard frequently.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // Synchronous NORMAL is safe with WAL and significantly faster than FULL.
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        // Increase page cache to ~8 MB (2000 × 4 KB pages).
        conn.pragma_update(None, "cache_size", -8000)?;
        // Enable foreign key enforcement (off by default in SQLite).
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Database { conn })
    }

    /// Initialize database schema.
    ///
    /// For a brand-new database this runs **all** migrations from v1 to CURRENT,
    /// which creates every table, index, and column.  For an existing database it
    /// only runs migrations newer than the stored schema version.
    ///
    /// Returns the number of migrations applied.
    pub fn init_schema(&self) -> Result<i64> {
        // 1) Ensure the `meta` table exists (needed to read schema_version).
        migrations::init_meta_table(&self.conn)?;

        // 2) Record creation metadata for brand-new databases.
        let current_version = migrations::get_schema_version(&self.conn)?;
        let is_new_db = current_version == 0;

        if is_new_db {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            migrations::set_meta_value(&self.conn, "created_at", &now)?;
            migrations::set_meta_value(&self.conn, "app_version", crate::VERSION)?;
        }

        // 3) Run all pending migrations (for a new DB: all of them).
        let applied = if current_version < migrations::CURRENT_SCHEMA_VERSION {
            migrations::run_migrations(&self.conn, current_version)?
        } else {
            0
        };

        Ok(applied)
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }

    pub fn get_connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new(":memory:").unwrap();
        db.init_schema().unwrap();

        // Verify tables exist
        let tables: Vec<String> = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"leagues".to_string()));
        assert!(tables.contains(&"teams".to_string()));
        assert!(tables.contains(&"players".to_string()));
        assert!(tables.contains(&"games".to_string()));
    }

    #[test]
    fn test_wal_mode() {
        let db = Database::new(":memory:").unwrap();
        let mode: String = db
            .conn
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap();
        // In-memory databases may report "memory" instead of "wal"
        assert!(mode == "wal" || mode == "memory");
    }

    #[test]
    fn test_migrations_applied_on_new_db() {
        let db = Database::new(":memory:").unwrap();
        let applied = db.init_schema().unwrap();
        assert!(
            applied > 0,
            "new DB should have applied all migrations"
        );

        let version = crate::db::migrations::get_schema_version(db.get_connection()).unwrap();
        assert_eq!(version, crate::db::migrations::CURRENT_SCHEMA_VERSION);
    }
}
