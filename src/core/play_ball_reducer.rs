use crate::HalfInning;
use crate::models::events::DomainEvent;
use crate::models::play_ball::GameState;

/// Apply a persisted DomainEvent to the in-memory GameState.
///
/// This is used to rebuild the state when resuming a game.
pub fn apply_domain_event(state: &mut GameState, ev: &DomainEvent) {
    match ev {
        DomainEvent::SideChange(d) => {
            state.inning = d.inning;
            state.half = d.half;
            state.outs = 0;
        }
        DomainEvent::StatusChanged(_) => {
            // Prompt state is not affected by status.
        }
        DomainEvent::GameStarted => {
            state.started = true;
            state.inning = 1;
            state.half = HalfInning::Top; // away bats first
            state.outs = 0;
        }
        DomainEvent::AtBatStarted { at_bat_no, .. } => {
            state.at_bat_no = *at_bat_no;
        }
    }
}
