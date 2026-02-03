//! Baseball Scorer Library
//!
//! A comprehensive baseball and softball scoring library with SQLite persistence,
//! official scoring symbols support, and cross-platform compatibility.
//!
//! # Features
//!
//! - **Database Layer**: SQLite-backed persistence for leagues, teams, players, and games
//! - **Scoring System**: Full support for official baseball scoring notation
//! - **Cross-Platform**: Windows, macOS, and Linux support with platform-specific data paths
//! - **Type Safety**: Strongly-typed models for all baseball entities
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
pub mod core;
pub mod db;
pub mod models;
pub mod utils;

// Re-export commonly used items for convenience
pub use db::config::{get_app_data_dir, get_db_path, get_db_path_display, setup_db};
pub use db::database::Database;
pub use db::league::League;
pub use db::migrations::{get_schema_version, migrations_needed, run_migrations};
pub use db::team::{Player, Team};

pub use core::menu::{LeagueMenuChoice, MainMenuChoice, Menu, TeamMenuChoice};
pub use core::parser::CommandParser;

pub use models::types::{
    AdvancedPlay, Base, BaseRunner, Error, Game, GamePlayer, GameTeam, HalfInning, HitType,
    OutType, Pitch, PitchCount, PlateAppearance, PlateAppearanceResult, Position,
    RunnerAdvancement, Walk,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");
