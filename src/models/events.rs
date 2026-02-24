use crate::models::types::{GameStatus, HalfInning};
use serde::{Deserialize, Serialize};

/// Persisted, replayable domain events for the Play Ball engine.
///
/// These events are stored in the `game_events` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// One out recorded in the current half-inning.
    OutRecorded(OutRecordedData),

    /// The side has changed (Top <-> Bottom), possibly advancing the inning.
    SideChange(SideChangeData),

    /// Game status changed (Regulation, Suspended, ...).
    StatusChanged(StatusChangedData),
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
            DomainEvent::OutRecorded(_) => "out_recorded",
            DomainEvent::SideChange(_) => "side_change",
            DomainEvent::StatusChanged(_) => "status_changed",
        }
    }
}
