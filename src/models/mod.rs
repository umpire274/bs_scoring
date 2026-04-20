//! Domain model: value types used across the engine, DB, and UI layers.
//!
//! This module deliberately contains **no I/O and no UI**. Persistence lives
//! in `crate::db`, game logic in `crate::engine`, and presentation in
//! `crate::ui` and `crate::cli`.

pub mod events;
pub mod field_zone;
pub mod game_state;
pub mod plate_appearance;
pub mod player_traits;
pub mod runner;
pub mod scoring;
pub mod session;
pub mod types;
pub mod umpires;
