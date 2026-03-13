//! Baseball Scorer Library
//!
//! A comprehensive baseball and softball scoring library with SQLite persistence,
//! official scoring symbols support, and cross-platform compatibility.
//!
//! # Example
//!
//! ```no_run
//! use bs_scoring::db::database::Database;
//! use bs_scoring::db::config::get_db_path;
//!
//! let db_path = get_db_path().unwrap();
//! let db = Database::new(&db_path.to_string_lossy()).unwrap();
//! db.init_schema().unwrap();
//! ```

pub mod cli;
pub mod commands;
pub mod core;
pub mod db;
pub mod engine;
pub mod models;
pub mod ui;
pub mod utils;

// ─── DB / infrastructure ─────────────────────────────────────────────────────
pub use db::config::{get_app_data_dir, get_db_path, get_db_path_display, setup_db};
pub use db::database::Database;
pub use db::league::League;
pub use db::migrations::{get_schema_version, migrations_needed, run_migrations};
pub use db::player::Player;
pub use db::team::Team;

// ─── Menu / CLI ───────────────────────────────────────────────────────────────
pub use core::menu::{
    DBMenuChoice, GameMenuChoice, LeagueMenuChoice, MainMenuChoice, Menu, PlayerMenuChoice,
    TeamMenuChoice,
};
pub use core::parser::CommandParser;

// ─── Primitive domain types (engine + DB layers) ─────────────────────────────
pub use models::types::{GameStatus, HalfInning, Pitch, PitchCount, Position, Score};

// ─── Live game model ─────────────────────────────────────────────────────────
pub use models::game_state::{BatterOrder, GameState, PitchStats};
pub use models::plate_appearance::PlateAppearance;
pub use models::runner::{RunnerDest, RunnerOverride};
pub use models::session::{LineupSide, PlayBallGameContext, PlayBallGate};

// ─── Full scoring notation (parser / future engine) ──────────────────────────
pub use models::scoring::{AdvancedPlay, Base, HitType, OutType, PlateAppearanceResult, Walk};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name  
pub const NAME: &str = env!("CARGO_PKG_NAME");
