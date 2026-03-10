use crate::commands::types::EngineCommand;
use crate::models::events::{DomainEvent, OutRecordedData, StatusChangedData, StrikeoutKind};
use crate::models::play_ball::{GameState, HalfInning};
use crate::models::types::{GameStatus, Pitch};
use crate::ui::events::UiEvent;

#[derive(Debug, Clone)]
pub struct PersistedEvent {
    pub inning: u32,
    pub half: HalfInning,
    pub event: DomainEvent,
    pub description: String,
}

pub struct ApplyResult {
    pub events: Vec<UiEvent>,
    /// Events to persist in `game_events` (administrative / low-frequency).
    pub persisted: Vec<PersistedEvent>,
    /// Events to apply to the in-memory state (scoreboard) but NOT persist.
    pub applied: Vec<DomainEvent>,
    /// Optional: compact 1-row-per-batter record persisted at end of PA.
    pub plate_appearance: Option<crate::models::plate_appearance::PlateAppearance>,
    pub exit: bool,
    pub status_change: Option<GameStatus>,
    pub needs_next_at_bat: bool,
}

fn empty_result() -> ApplyResult {
    ApplyResult {
        events: vec![],
        persisted: vec![],
        applied: vec![],
        plate_appearance: None,
        exit: false,
        status_change: None,
        needs_next_at_bat: false,
    }
}

pub fn apply_engine_command(state: &mut GameState, cmd: EngineCommand) -> ApplyResult {
    match cmd {
        EngineCommand::Exit => ApplyResult {
            exit: true,
            ..empty_result()
        },

        // NOTE: PLAYBALL is handled in the engine layer because it requires DB lookups
        EngineCommand::PlayBall => ApplyResult {
            events: vec![UiEvent::Error(
                "PLAYBALL must be handled by the engine (DB-backed).".to_string(),
            )],
            ..empty_result()
        },

        EngineCommand::SetStatus(status) => {
            let msg = format!("{} Game set to {}.", status.icon(), status);

            ApplyResult {
                events: vec![UiEvent::Line(msg.clone())],
                persisted: vec![PersistedEvent {
                    inning: state.inning,
                    half: state.half,
                    event: DomainEvent::StatusChanged(StatusChangedData { to: status }),
                    description: msg,
                }],
                applied: vec![],
                plate_appearance: None,
                exit: true,
                status_change: Some(status),
                needs_next_at_bat: false,
            }
        }

        // ✅ NEW: pitch command (0.6.7 baseline)
        EngineCommand::Pitch(pitch) => apply_pitch(state, pitch),

        EngineCommand::Single { zone } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::Single { zone },
            "1B",
        ),

        EngineCommand::Double { zone } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::Double { zone },
            "2B",
        ),

        EngineCommand::Triple { zone } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::Triple { zone },
            "3B",
        ),

        EngineCommand::HomeRun { zone } => apply_hit_command(
            state,
            crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { zone },
            "HR",
        ),

        EngineCommand::Unknown(s) => ApplyResult {
            events: vec![UiEvent::Error(format!("Unknown command: {s}"))],
            ..empty_result()
        },
    }
}

fn apply_pitch(state: &mut GameState, pitch: Pitch) -> ApplyResult {
    let Some(batter_id) = state.current_batter_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter. Use PLAYBALL (or resume the game) first.".to_string(),
            )],
            ..empty_result()
        };
    };

    let Some(pitcher_id) = state.current_pitcher_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active pitcher in state (cannot record pitch).".to_string(),
            )],
            ..empty_result()
        };
    };

    // Count BEFORE applying this pitch
    let balls_before = state.pitch_count.balls;
    let strikes_before = state.pitch_count.strikes;

    // Compute count AFTER applying this pitch (domain rules)
    let mut balls_after = balls_before;
    let mut strikes_after = strikes_before;

    match pitch {
        Pitch::Ball => {
            balls_after = balls_after.saturating_add(1);
        }
        Pitch::CalledStrike | Pitch::SwingingStrike => {
            strikes_after = strikes_after.saturating_add(1);
        }
        Pitch::Foul => {
            // counts as strike only if strikes < 2
            if strikes_after < 2 {
                strikes_after = strikes_after.saturating_add(1);
            }
        }
        Pitch::FoulBunt => {
            // ALWAYS counts as strike (can be K on strike 3)
            strikes_after = strikes_after.saturating_add(1);
        }
        Pitch::InPlay | Pitch::HittedBy => {
            // reserved for future use (no count changes here)
        }
    }

    let mut events_ui = vec![UiEvent::Line(format!("Pitch: {}", pitch))];
    let mut applied: Vec<DomainEvent> = vec![DomainEvent::PitchRecorded {
        pitcher_id,
        batter_id,
        pitch: pitch.clone(),
    }];

    let mut needs_next_at_bat = false;
    let mut plate_appearance: Option<crate::models::plate_appearance::PlateAppearance> = None;

    // This pitch counts as one more pitch in the PA
    let pitches_in_pa = state.pitch_count.sequence.len() as u32 + 1;

    // Shared final PA sequence = current sequence + this pitch
    let mut final_sequence = build_pa_sequence(state);
    final_sequence.push(crate::models::plate_appearance::PlateAppearanceStep::Pitch(
        pitch.clone(),
    ));

    // Helper closure to finalize a PA without duplicating struct construction
    let finalize_pa = |outcome: crate::models::plate_appearance::PlateAppearanceOutcome,
                       outs: u8|
     -> crate::models::plate_appearance::PlateAppearance {
        crate::models::plate_appearance::PlateAppearance {
            inning: state.inning,
            half: state.half,
            batter_id,
            pitcher_id,
            pitches: pitches_in_pa,
            pitches_sequence: final_sequence.clone(),
            outcome,
            outs,
        }
    };

    // Walk: 4 balls and strikes < 3
    if balls_after >= 4 && strikes_after < 3 {
        applied.push(DomainEvent::WalkIssued { batter_id });

        let runner_jersey_no = state.current_batter_jersey_no.unwrap_or(0);
        let runner_first_name = state
            .current_batter_first_name
            .as_deref()
            .unwrap_or("-")
            .to_string();
        let runner_last_name = state
            .current_batter_last_name
            .as_deref()
            .unwrap_or("-")
            .to_string();

        applied.push(DomainEvent::RunnerToFirst {
            runner_id: batter_id,
            runner_jersey_no,
            runner_first_name,
            runner_last_name,
        });

        applied.push(DomainEvent::CountReset);

        events_ui.push(UiEvent::Line("BB: batter to 1B".to_string()));
        needs_next_at_bat = true;

        plate_appearance = Some(finalize_pa(
            crate::models::plate_appearance::PlateAppearanceOutcome::Walk,
            state.outs,
        ));
    }
    // Strikeout: 3 strikes before 4 balls
    else if strikes_after >= 3 && balls_after < 4 {
        let kind = match pitch {
            Pitch::CalledStrike => StrikeoutKind::Called,
            Pitch::SwingingStrike => StrikeoutKind::Swinging,
            Pitch::FoulBunt => StrikeoutKind::FoulBunt,
            // safety fallback
            _ => StrikeoutKind::Called,
        };

        applied.push(DomainEvent::Strikeout {
            batter_id,
            kind: kind.clone(),
        });

        applied.push(DomainEvent::OutRecorded(OutRecordedData {
            outs_before: state.outs,
            outs_after: state.outs.saturating_add(1),
        }));

        applied.push(DomainEvent::CountReset);

        events_ui.push(UiEvent::Line("K: batter out".to_string()));
        needs_next_at_bat = true;

        plate_appearance = Some(finalize_pa(
            crate::models::plate_appearance::PlateAppearanceOutcome::Strikeout(kind),
            state.outs.saturating_add(1),
        ));
    } else if matches!(pitch, Pitch::InPlay | Pitch::HittedBy) {
        events_ui.push(UiEvent::Line("Note: X/H not implemented yet".to_string()));
    }

    ApplyResult {
        events: events_ui,
        persisted: vec![],
        applied,
        plate_appearance,
        exit: false,
        status_change: None,
        needs_next_at_bat,
    }
}

fn apply_hit_command(
    state: &mut GameState,
    outcome: crate::models::plate_appearance::PlateAppearanceOutcome,
    label: &str,
) -> ApplyResult {
    let Some(batter_id) = state.current_batter_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active batter. Use PLAYBALL (or resume the game) first.".to_string(),
            )],
            ..empty_result()
        };
    };

    let Some(pitcher_id) = state.current_pitcher_id else {
        return ApplyResult {
            events: vec![UiEvent::Error(
                "No active pitcher in state (cannot record hit).".to_string(),
            )],
            ..empty_result()
        };
    };

    let final_step = match &outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::Single
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::Double
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::Triple
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. } => {
            crate::models::plate_appearance::PlateAppearanceStep::HomeRun
        }
        _ => {
            return ApplyResult {
                events: vec![UiEvent::Error(
                    "Invalid hit outcome passed to apply_hit_command.".to_string(),
                )],
                ..empty_result()
            };
        }
    };

    let mut final_sequence = build_pa_sequence(state);
    final_sequence.push(final_step);

    let pitches_in_pa = final_sequence.len() as u32;

    let plate_appearance = crate::models::plate_appearance::PlateAppearance {
        inning: state.inning,
        half: state.half,
        batter_id,
        pitcher_id,
        pitches: pitches_in_pa,
        pitches_sequence: final_sequence,
        outcome: outcome.clone(),
        outs: state.outs,
    };

    let zone = match &outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { zone }
        | crate::models::plate_appearance::PlateAppearanceOutcome::Double { zone }
        | crate::models::plate_appearance::PlateAppearanceOutcome::Triple { zone }
        | crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { zone } => *zone,
        _ => None,
    };

    let human_label = match label {
        "1B" => "Single",
        "2B" => "Double",
        "3B" => "Triple",
        "HR" => "Home run",
        _ => label,
    };

    let message = if let Some(z) = zone {
        format!("{human_label} to {}", z.as_str())
    } else {
        human_label.to_string()
    };

    ApplyResult {
        events: vec![UiEvent::Line(message)],
        persisted: vec![],
        applied: vec![DomainEvent::CountReset],
        plate_appearance: Some(plate_appearance),
        exit: false,
        status_change: None,
        needs_next_at_bat: true,
    }
}

fn build_pa_sequence(
    state: &GameState,
) -> Vec<crate::models::plate_appearance::PlateAppearanceStep> {
    state
        .pitch_count
        .sequence
        .iter()
        .cloned()
        .map(crate::models::plate_appearance::PlateAppearanceStep::Pitch)
        .collect()
}
