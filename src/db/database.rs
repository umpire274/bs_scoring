use crate::db::migrations;
use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create or open the database
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Database { conn })
    }

    /// Initialize database schema
    pub fn init_schema(&self) -> Result<()> {
        // Initialize meta table first
        migrations::init_meta_table(&self.conn)?;

        // Check if this is a new database
        let is_new_db = migrations::get_schema_version(&self.conn)? == 0;

        if is_new_db {
            // Set creation timestamp
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            migrations::set_meta_value(&self.conn, "created_at", &now)?;
            migrations::set_meta_value(&self.conn, "app_version", crate::VERSION)?;
        }

        // Leagues table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS leagues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                season TEXT,
                description TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Teams table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS teams (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                league_id INTEGER,
                city TEXT,
                abbreviation TEXT,
                founded_year INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (league_id) REFERENCES leagues(id)
            )",
            [],
        )?;

        // Players table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                team_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                name TEXT NOT NULL,
                position INTEGER NOT NULL,
                batting_order INTEGER,
                is_active BOOLEAN DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (team_id) REFERENCES teams(id),
                UNIQUE(team_id, number)
            )",
            [],
        )?;

        // Games table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id TEXT NOT NULL UNIQUE,
                home_team_id INTEGER NOT NULL,
                away_team_id INTEGER NOT NULL,
                venue TEXT,
                game_date DATE NOT NULL,
                league_id INTEGER,
                home_score INTEGER DEFAULT 0,
                away_score INTEGER DEFAULT 0,
                home_hits INTEGER DEFAULT 0,
                away_hits INTEGER DEFAULT 0,
                home_errors INTEGER DEFAULT 0,
                away_errors INTEGER DEFAULT 0,
                status TEXT DEFAULT 'in_progress',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (home_team_id) REFERENCES teams(id),
                FOREIGN KEY (away_team_id) REFERENCES teams(id),
                FOREIGN KEY (league_id) REFERENCES leagues(id)
            )",
            [],
        )?;

        // Create indexes for better performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_games_date ON games(game_date)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_players_team ON players(team_id)",
            [],
        )?;

        // Create new tables
        self.conn.execute(
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

        self.conn.execute(
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

        self.conn.execute(
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

        self.conn.execute(
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
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_at_bats_game ON at_bats(game_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pitches_at_bat ON pitches(at_bat_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runner_movements_at_bat ON runner_movements(at_bat_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_game_events_game ON game_events(game_id)",
            [],
        )?;

        // Run any pending migrations
        let current_version = migrations::get_schema_version(&self.conn)?;
        if current_version < migrations::CURRENT_SCHEMA_VERSION {
            println!("\n🔄 Database migrations needed...");
            migrations::run_migrations(&self.conn, current_version)?;
        } else if is_new_db {
            // For new DB, set initial version
            migrations::set_meta_value(
                &self.conn,
                "schema_version",
                &migrations::CURRENT_SCHEMA_VERSION.to_string(),
            )?;
        }

        Ok(())
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
}
