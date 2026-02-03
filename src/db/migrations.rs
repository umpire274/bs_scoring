use chrono::Local;
use rusqlite::{Connection, Result};

/// Current schema version - increment this when adding migrations
pub const CURRENT_SCHEMA_VERSION: i64 = 2;

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
        Migration {
            version: 2,
            description: "Remove old plate_appearances and base_runners, add new game scoring tables",
            up: migration_v2,
        },
    ]
}

/// Migration v1: Initial schema (noop - already handled by init_schema)
fn migration_v1(_conn: &Connection) -> Result<()> {
    // Initial schema already created by init_schema()
    // This is just a placeholder for version tracking
    Ok(())
}

fn migration_v2(conn: &Connection) -> Result<()> {
    // Drop old tables
    conn.execute("DROP TABLE IF EXISTS base_runners", [])?;
    conn.execute("DROP TABLE IF EXISTS plate_appearances", [])?;

    // Create new tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS at_bats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            inning INTEGER NOT NULL,
            half_inning TEXT NOT NULL CHECK(half_inning IN ('Top', 'Bottom')),
            batter_id INTEGER NOT NULL,
            pitcher_id INTEGER NOT NULL,
            outs_before INTEGER NOT NULL CHECK(outs_before BETWEEN 0 AND 2),
            runner_on_first INTEGER,
            runner_on_second INTEGER,
            runner_on_third INTEGER,
            result_type TEXT NOT NULL,
            result_detail TEXT,
            outs_after INTEGER NOT NULL CHECK(outs_after BETWEEN 0 AND 3),
            runs_scored INTEGER DEFAULT 0,
            rbis INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (game_id) REFERENCES games(id),
            FOREIGN KEY (batter_id) REFERENCES players(id),
            FOREIGN KEY (pitcher_id) REFERENCES players(id),
            FOREIGN KEY (runner_on_first) REFERENCES players(id),
            FOREIGN KEY (runner_on_second) REFERENCES players(id),
            FOREIGN KEY (runner_on_third) REFERENCES players(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS pitches (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            at_bat_id INTEGER NOT NULL,
            pitch_number INTEGER NOT NULL,
            balls_before INTEGER NOT NULL CHECK(balls_before BETWEEN 0 AND 3),
            strikes_before INTEGER NOT NULL CHECK(strikes_before BETWEEN 0 AND 2),
            pitch_type TEXT NOT NULL,
            in_play_result TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (at_bat_id) REFERENCES at_bats(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS runner_movements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            at_bat_id INTEGER NOT NULL,
            runner_id INTEGER NOT NULL,
            start_base TEXT NOT NULL CHECK(start_base IN ('1B', '2B', '3B', 'HOME')),
            end_base TEXT CHECK(end_base IN ('1B', '2B', '3B', 'HOME', 'OUT')),
            advancement_type TEXT NOT NULL,
            is_out BOOLEAN DEFAULT 0,
            out_type TEXT,
            scored BOOLEAN DEFAULT 0,
            is_earned BOOLEAN DEFAULT 1,
            FOREIGN KEY (at_bat_id) REFERENCES at_bats(id),
            FOREIGN KEY (runner_id) REFERENCES players(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            at_bat_id INTEGER,
            inning INTEGER NOT NULL,
            half_inning TEXT NOT NULL,
            event_type TEXT NOT NULL,
            event_data TEXT,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (game_id) REFERENCES games(id),
            FOREIGN KEY (at_bat_id) REFERENCES at_bats(id)
        )",
        [],
    )?;

    // Create indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_at_bats_game ON at_bats(game_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pitches_at_bat ON pitches(at_bat_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_runner_movements_at_bat ON runner_movements(at_bat_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_game_events_game ON game_events(game_id)",
        [],
    )?;

    Ok(())
}

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
