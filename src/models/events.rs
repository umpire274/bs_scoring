use crate::models::types::{GameStatus, HalfInning};
use crate::{Pitch, Position};
use serde::{Deserialize, Serialize};

/// Persisted, replayable domain events for the Play Ball engine.
///
/// These events are stored in the `game_events` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    /// The side has changed (Top <-> Bottom), possibly advancing the inning.
    SideChange(SideChangeData),

    /// Game status changed (Regulation, Suspended, ...).
    StatusChanged(StatusChangedData),

    GameStarted,

    AtBatStarted {
        team_abbrv: String,
        batting_team_id: i64,

        batter_id: i64,
        batter_jersey_no: i32,
        batter_first_name: String,
        batter_last_name: String,
        batter_order: u8,
        batter_position: Position,

        pitcher_id: i64,
        pitcher_jersey_no: i32,
        pitcher_first_name: String,
        pitcher_last_name: String,
    },

    PitcherChanged {
        pitcher_id: i64,
        pitcher_jersey_no: i32,
        pitcher_first_name: String,
        pitcher_last_name: String,
    },

    PitchRecorded {
        pitcher_id: i64,
        batter_id: i64,
        pitch: Pitch,
    },

    /// Summary: how many pitches were thrown in the last completed at-bat.
    /// Persisted to reconstruct pitcher pitch counts without logging every pitch.
    AtBatPitchesCount {
        pitcher_id: i64,
        pitches: u32,
    },

    CountReset,

    WalkIssued {
        batter_id: i64,
    },

    Strikeout {
        batter_id: i64,
        kind: StrikeoutKind,
    },

    OutRecorded(OutRecordedData),

    RunnerToFirst {
        runner_id: i64,
        runner_jersey_no: i32,
        runner_first_name: String,
        runner_last_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrikeoutKind {
    Called,   // K
    Swinging, // S
    FoulBunt, // FL (quando fa il terzo strike)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutRecordedData {
    pub outs_before: u8,
    pub outs_after: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideChangeData {
    pub inning: u32,
    pub half: HalfInning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChangedData {
    pub to: GameStatus,
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::SideChange(_) => "side_change",
            DomainEvent::StatusChanged(_) => "status_changed",
            DomainEvent::GameStarted => "game_started",
            DomainEvent::AtBatStarted { .. } => "at_bat_started",
            DomainEvent::PitcherChanged { .. } => "pitcher_changed",
            DomainEvent::PitchRecorded { .. } => "pitch_recorded",
            DomainEvent::AtBatPitchesCount { .. } => "at_bat_pitches_count",
            DomainEvent::CountReset => "count_reset",
            DomainEvent::WalkIssued { .. } => "walk_issued",
            DomainEvent::Strikeout { .. } => "strikeout",
            DomainEvent::OutRecorded(_) => "out_recorded",
            DomainEvent::RunnerToFirst { .. } => "runner_to_first",
        }
    }
}

// ─── Persisted event ─────────────────────────────────────────────────────────

/// A domain event bundled with its context, ready to be written to `game_events`.
///
/// Produced by `apply_engine_command()` and consumed by the engine loop,
/// which calls `append_game_event()` for each one.
#[derive(Debug, Clone)]
pub struct PersistedEvent {
    pub inning: u32,
    pub half: HalfInning,
    pub event: DomainEvent,
    pub description: String,
}
