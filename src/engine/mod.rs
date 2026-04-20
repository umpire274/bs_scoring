//! Game engine: pure game logic — no I/O, no UI.
//!
//! - `commands` — internal engine commands produced by parsing user scoring
//!   notation and consumed by the reducer.
//! - `notation` — parser for the compact scoring-notation (`CommandParser`).
//! - `scoring` — scoring-rules helpers (batter-out kinds, resolution).
//! - `runners` — base-runner movement logic.
//! - `apply` — applies a single `EngineCommand` onto the `GameState`.
//! - `reducer` — higher-level reducer stitching a plate-appearance together.
//! - `play_ball` — top-level play-by-play loop orchestrating engine + UI.

pub mod apply;
pub mod commands;
pub(crate) mod helpers;
pub mod notation;
pub mod play_ball;
pub mod reducer;
pub mod runners;
pub mod scoring;

pub(crate) use helpers::{get_fielder, get_foul_flag, get_sequence, parse_outcome_json};
