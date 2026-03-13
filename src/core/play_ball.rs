//! Deprecated — moved to `db::game_queries` in v0.8.1.
//!
//! Re-exported here only to avoid breaking any external crate that may
//! have depended on these paths. Will be removed in a future version.

#[deprecated(since = "0.8.1", note = "use `crate::db::game_queries` instead")]
pub use crate::db::game_queries::{gate_check_lineups, list_playable_games, set_game_status};
