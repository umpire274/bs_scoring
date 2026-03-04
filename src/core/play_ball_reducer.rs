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

            // ✅ cambio half-inning => si azzerano le basi
            state.on_1b = false;
            state.on_2b = false;
            state.on_3b = false;

            // ✅ invalida PA corrente (battitore) perché inizia una nuova fase offensiva
            state.current_batter_id = None;
            state.current_batter_jersey_no = None;
            state.current_batter_first_name = None;
            state.current_batter_last_name = None;

            // ✅ reset del count della PA (balls/strikes + sequence)
            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();
        }

        DomainEvent::StatusChanged(_) => {
            // Prompt state is not affected by status.
        }
        DomainEvent::GameStarted => {
            state.started = true;
            state.inning = 1;
            state.half = HalfInning::Top; // away bats first
            state.outs = 0;

            // reset PA count
            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();
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

            // Batter
            state.current_batter_id = Some(*batter_id);
            state.current_batter_jersey_no = Some(*batter_jersey_no);
            state.current_batter_first_name = Some(batter_first_name.clone());
            state.current_batter_last_name = Some(batter_last_name.clone());

            // Pitcher
            state.current_pitcher_id = Some(*pitcher_id);
            state.current_pitcher_jersey_no = Some(*pitcher_jersey_no);
            state.current_pitcher_first_name = Some(pitcher_first_name.clone());
            state.current_pitcher_last_name = Some(pitcher_last_name.clone());

            // ✅ PA count reset (solo balls/strikes/sequence)
            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();

            state.current_pitch_count = *state.pitcher_pitch_counts.get(pitcher_id).unwrap_or(&0);
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
            // Pitch count del pitcher (persistente per pitcher)
            let entry = state.pitcher_pitch_counts.entry(*pitcher_id).or_insert(0);
            *entry = entry.saturating_add(1);

            // Pitch count del pitcher corrente
            if state.current_pitcher_id == Some(*pitcher_id) {
                state.current_pitch_count = *entry;
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
                    if state.pitch_count.strikes < 2 {
                        state.pitch_count.strikes = state.pitch_count.strikes.saturating_add(1);
                    }
                }
                Pitch::FoulBunt => {
                    state.pitch_count.strikes = state.pitch_count.strikes.saturating_add(1);
                }
                Pitch::InPlay | Pitch::HittedBy => {}
            }

            // opzionale: clamp per UI pulita
            if state.pitch_count.balls > 4 {
                state.pitch_count.balls = 4;
            }
            if state.pitch_count.strikes > 3 {
                state.pitch_count.strikes = 3;
            }
        }

        DomainEvent::CountReset => {
            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();
        }

        DomainEvent::WalkIssued { .. } => {}
        DomainEvent::Strikeout { .. } => {}

        DomainEvent::OutRecorded(data) => {
            state.outs = data.outs_after;
        }

        DomainEvent::RunnerToFirst { .. } => {
            state.on_1b = true;
        }
    }
}
