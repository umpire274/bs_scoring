use crate::models::events::DomainEvent;
use crate::models::play_ball::GameState;
use crate::{HalfInning, Pitch};

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
        DomainEvent::AtBatStarted {
            batter_id,
            batter_jersey_no,
            batter_first_name,
            batter_last_name,
            pitcher_id,
            pitcher_jersey_no,
            pitcher_first_name,
            pitcher_last_name,
            ..
        } => {
            state.started = true;
            state.current_pitch_count = 0;

            state.current_batter_id = Some(*batter_id);
            state.current_batter_jersey_no = Some(*batter_jersey_no);
            state.current_batter_first_name = Some(batter_first_name.clone());
            state.current_batter_last_name = Some(batter_last_name.clone());

            state.current_pitcher_id = Some(*pitcher_id);
            state.current_pitcher_jersey_no = Some(*pitcher_jersey_no);
            state.current_pitcher_first_name = Some(pitcher_first_name.clone());
            state.current_pitcher_last_name = Some(pitcher_last_name.clone());
        }

        DomainEvent::PitchThrown { pitcher_id } => {
            // incrementa solo se è il pitcher attuale
            if state.current_pitcher_id == Some(*pitcher_id) {
                state.current_pitch_count = state.current_pitch_count.saturating_add(1);
            }
        }

        DomainEvent::PitcherChanged {
            pitcher_id,
            pitcher_jersey_no,
            pitcher_first_name,
            pitcher_last_name,
        } => {
            state.current_pitcher_id = Some(*pitcher_id);
            state.current_pitcher_jersey_no = Some(*pitcher_jersey_no);
            state.current_pitcher_first_name = Some(pitcher_first_name.clone());
            state.current_pitcher_last_name = Some(pitcher_last_name.clone());

            // reset count per nuovo pitcher
            state.current_pitch_count = 0;
        }

        DomainEvent::PitchRecorded {
            pitcher_id, pitch, ..
        } => {
            // Pitch count del pitcher corrente (solo se coerente)
            if state.current_pitcher_id == Some(*pitcher_id) {
                state.current_pitch_count = state.current_pitch_count.saturating_add(1);
            }

            // Sequenza lanci per PA
            state.pitch_count.sequence.push(pitch.clone());

            match pitch {
                Pitch::Ball => {
                    state.pitch_count.balls = state.pitch_count.balls.saturating_add(1);
                }
                Pitch::CalledStrike | Pitch::SwingingStrike => {
                    state.pitch_count.strikes = state.pitch_count.strikes.saturating_add(1);
                }
                Pitch::Foul => {
                    // F = strike solo se strikes < 2
                    if state.pitch_count.strikes < 2 {
                        state.pitch_count.strikes = state.pitch_count.strikes.saturating_add(1);
                    }
                }
                Pitch::FoulBunt => {
                    // FL = strike sempre
                    state.pitch_count.strikes = state.pitch_count.strikes.saturating_add(1);
                }
                Pitch::InPlay | Pitch::HittedBy => {
                    // 0.6.7: non gestiti ancora (ma restano in sequence)
                }
            }
        }

        DomainEvent::CountReset => {
            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();
        }

        DomainEvent::WalkIssued { .. } => {
            // L’effetto su count/basi lo gestiamo con CountReset + RunnerToFirst (eventi separati).
        }

        DomainEvent::Strikeout { .. } => {
            // L'out è gestito da OutRecorded; reset count da CountReset.
        }

        DomainEvent::OutRecorded(data) => {
            state.outs = data.outs_after;
        }

        DomainEvent::RunnerToFirst { .. } => {
            // MVP: mark 1B occupied
            state.on_1b = true;
        }
    }
}
