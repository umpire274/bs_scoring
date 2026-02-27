use crate::Pitch;
use crate::models::types::{GameStatus, HalfInning};
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

        pitcher_id: i64,
        pitcher_jersey_no: i32,
        pitcher_first_name: String,
        pitcher_last_name: String,
    },

    PitchThrown {
        pitcher_id: i64,
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
            DomainEvent::PitchThrown { .. } => "pitch_thrown",
            DomainEvent::PitcherChanged { .. } => "pitcher_changed",
            _ => "",
        }
    }
}
