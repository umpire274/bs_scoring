use chrono::Local;
use rusqlite::{Connection, Result};

/// Current schema version - increment this when adding migrations
pub const CURRENT_SCHEMA_VERSION: i64 = 1;

/// Migration structure
pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub up: fn(&Connection) -> Result<()>,
}

/// Get all migrations in order
pub fn get_migrations() -> Vec<Migration> {
    vec![
        // Migration 1: Initial schema (already exists from init_schema)
        Migration {
            version: 1,
            description: "Initial schema with leagues, teams, players, games",
            up: migration_v1,
        },
        // Future migrations will be added here
        // Migration {
        //     version: 2,
        //     description: "Add new column to teams table",
        //     up: migration_v2,
        // },
    ]
}

/// Migration v1: Initial schema (noop - already handled by init_schema)
fn migration_v1(_conn: &Connection) -> Result<()> {
    // Initial schema already created by init_schema()
    // This is just a placeholder for version tracking
    Ok(())
}

// Example future migration (commented out)
/*
fn migration_v2(conn: &Connection) -> Result<()> {
    // Add new column
    conn.execute(
        "ALTER TABLE teams ADD COLUMN logo_url TEXT",
        [],
    )?;

    // Add index
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_teams_logo ON teams(logo_url)",
        [],
    )?;

    Ok(())
}
*/

/// Run pending migrations
pub fn run_migrations(conn: &Connection, current_version: i64) -> Result<i64> {
    let migrations = get_migrations();
    let mut applied_count = 0;

    for migration in migrations {
        if migration.version > current_version {
            println!(
                "🔄 Applying migration v{}: {}",
                migration.version, migration.description
            );

            // Run migration
            (migration.up)(conn)?;

            // Update schema version
            set_meta_value(conn, "schema_version", &migration.version.to_string())?;
            set_meta_value(
                conn,
                "last_migration",
                &Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            )?;

            applied_count += 1;
            println!("✅ Migration v{} applied successfully", migration.version);
        }
    }

    if applied_count > 0 {
        println!("\n✅ {} migration(s) applied", applied_count);
    }

    Ok(CURRENT_SCHEMA_VERSION)
}

/// Initialize meta table
pub fn init_meta_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// Get meta value
pub fn get_meta_value(conn: &Connection, key: &str) -> Result<Option<String>> {
    match conn.query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
        row.get(0)
    }) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Set meta value
pub fn set_meta_value(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value, updated_at)
         VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        [key, value],
    )?;
    Ok(())
}

/// Get current schema version from DB
pub fn get_schema_version(conn: &Connection) -> Result<i64> {
    match get_meta_value(conn, "schema_version")? {
        Some(version_str) => version_str
            .parse()
            .map_err(|_| rusqlite::Error::InvalidQuery),
        None => Ok(0), // No version = pristine DB
    }
}

/// Check if migrations are needed
pub fn migrations_needed(conn: &Connection) -> Result<bool> {
    let current = get_schema_version(conn)?;
    Ok(current < CURRENT_SCHEMA_VERSION)
}

/// Get migration info for display
pub fn get_migration_info(conn: &Connection) -> Result<MigrationInfo> {
    let current_version = get_schema_version(conn)?;
    let pending_count = (CURRENT_SCHEMA_VERSION - current_version).max(0);

    let last_migration = get_meta_value(conn, "last_migration")?;
    let created_at = get_meta_value(conn, "created_at")?;

    Ok(MigrationInfo {
        current_version,
        latest_version: CURRENT_SCHEMA_VERSION,
        pending_count,
        last_migration,
        created_at,
    })
}

pub struct MigrationInfo {
    pub current_version: i64,
    pub latest_version: i64,
    pub pending_count: i64,
    pub last_migration: Option<String>,
    pub created_at: Option<String>,
}
