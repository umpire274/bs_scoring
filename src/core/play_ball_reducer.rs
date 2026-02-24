use crate::models::events::DomainEvent;
use crate::models::play_ball::GameState;

/// Apply a persisted DomainEvent to the in-memory GameState.
///
/// This is used to rebuild the state when resuming a game.
pub fn apply_domain_event(state: &mut GameState, ev: &DomainEvent) {
    match ev {
        DomainEvent::OutRecorded(d) => {
            state.outs = d.outs_after;
        }
        DomainEvent::SideChange(d) => {
            state.inning = d.inning;
            state.half = d.half;
            state.outs = 0;
        }
        DomainEvent::StatusChanged(_) => {
            // Prompt state is not affected by status.
        }
    }
}
