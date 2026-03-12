use crate::db::plate_appearances::PlateAppearanceRow;
use crate::engine::play_ball::{bump_order, parse_pa_sequence};
use crate::models::events::{DomainEvent, StrikeoutKind};
use crate::models::plate_appearance::PlateAppearanceStep;
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

            state.on_1b = false;
            state.on_2b = false;
            state.on_3b = false;

            state.current_batter_id = None;
            state.current_batter_jersey_no = None;
            state.current_batter_first_name = None;
            state.current_batter_last_name = None;
            state.current_batter_order = None;
            state.current_batter_position = None;

            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();
        }

        DomainEvent::StatusChanged(_) => {}

        DomainEvent::GameStarted => {
            state.started = true;
            state.inning = 1;
            state.half = HalfInning::Top;
            state.outs = 0;

            state.current_batter_id = None;
            state.current_batter_jersey_no = None;
            state.current_batter_first_name = None;
            state.current_batter_last_name = None;
            state.current_batter_order = None;
            state.current_batter_position = None;

            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();
        }

        DomainEvent::AtBatStarted {
            batter_id,
            batter_jersey_no,
            batter_first_name,
            batter_last_name,
            batter_order,
            batter_position,
            pitcher_id,
            pitcher_jersey_no,
            pitcher_first_name,
            pitcher_last_name,
            ..
        } => {
            state.started = true;

            state.current_batter_id = Some(*batter_id);
            state.current_batter_jersey_no = Some(*batter_jersey_no);
            state.current_batter_first_name = Some(batter_first_name.clone());
            state.current_batter_last_name = Some(batter_last_name.clone());
            state.current_batter_order = Some(*batter_order);
            state.current_batter_position = Some(*batter_position);

            state.current_pitcher_id = Some(*pitcher_id);
            state.current_pitcher_jersey_no = Some(*pitcher_jersey_no);
            state.current_pitcher_first_name = Some(pitcher_first_name.clone());
            state.current_pitcher_last_name = Some(pitcher_last_name.clone());

            state.pitch_count.balls = 0;
            state.pitch_count.strikes = 0;
            state.pitch_count.sequence.clear();

            state.pitcher_stats.entry(*pitcher_id).or_default();
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

            state.pitcher_stats.entry(*pitcher_id).or_default();
        }

        DomainEvent::PitchRecorded {
            pitcher_id, pitch, ..
        } => {
            let stats = state.pitcher_stats.entry(*pitcher_id).or_default();

            match pitch {
                Pitch::Ball => stats.balls += 1,
                _ => stats.strikes += 1,
            }

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

            if state.pitch_count.balls > 4 {
                state.pitch_count.balls = 4;
            }
            if state.pitch_count.strikes > 3 {
                state.pitch_count.strikes = 3;
            }
        }

        DomainEvent::AtBatPitchesCount { .. } => {}

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
            apply_walk_advancement(state);
        }
    }
}

fn place_runner(
    dest: u8,
    runs_scored: &mut u32,
    on_1b: &mut bool,
    on_2b: &mut bool,
    on_3b: &mut bool,
) {
    if dest >= 4 {
        *runs_scored += 1;
        return;
    }

    match dest {
        1 => *on_1b = true,
        2 => *on_2b = true,
        3 => *on_3b = true,
        _ => {}
    }
}

fn ensure_inning(vec: &mut Vec<u16>, inning: u32) {
    let idx = inning as usize;
    if vec.len() < idx {
        vec.resize(idx, 0);
    }
}

fn apply_hit_advancement(state: &mut GameState, bases: u8) {
    let mut runs_scored: u32 = 0;

    let runner_on_1b = state.on_1b;
    let runner_on_2b = state.on_2b;
    let runner_on_3b = state.on_3b;

    // reset basi
    state.on_1b = false;
    state.on_2b = false;
    state.on_3b = false;

    // muovi corridori già in base
    if runner_on_3b {
        place_runner(
            3 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }

    if runner_on_2b {
        place_runner(
            2 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }

    if runner_on_1b {
        place_runner(
            1 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }

    // muovi battitore
    place_runner(
        bases,
        &mut runs_scored,
        &mut state.on_1b,
        &mut state.on_2b,
        &mut state.on_3b,
    );

    // aggiorna punteggio
    match state.half {
        HalfInning::Top => {
            state.score.away += runs_scored as u16;

            ensure_inning(&mut state.score.away_innings, state.inning);
            let idx = (state.inning - 1) as usize;
            state.score.away_innings[idx] += runs_scored as u16;

            state.score.away_hits += 1; // ogni hit conta come 1, indipendentemente da single/double/triple/hr
        }

        HalfInning::Bottom => {
            state.score.home += runs_scored as u16;

            ensure_inning(&mut state.score.home_innings, state.inning);
            let idx = (state.inning - 1) as usize;
            state.score.home_innings[idx] += runs_scored as u16;

            state.score.home_hits += 1; // ogni hit conta come 1, indipendentemente da single/double/triple/hr
        }
    }
}

fn apply_walk_advancement(state: &mut GameState) {
    // Bases loaded: runner from 3B scores
    if state.on_1b && state.on_2b && state.on_3b {
        match state.half {
            HalfInning::Top => {
                state.score.away = state.score.away.saturating_add(1);
                ensure_inning(&mut state.score.away_innings, state.inning);
                let idx = (state.inning - 1) as usize;
                state.score.away_innings[idx] = state.score.away_innings[idx].saturating_add(1);
            }
            HalfInning::Bottom => {
                state.score.home = state.score.home.saturating_add(1);
                ensure_inning(&mut state.score.home_innings, state.inning);
                let idx = (state.inning - 1) as usize;
                state.score.home_innings[idx] = state.score.home_innings[idx].saturating_add(1);
            }
        }
    }

    // Forced advancements
    if state.on_1b && state.on_2b {
        state.on_3b = true;
    }

    if state.on_1b {
        state.on_2b = true;
    }

    state.on_1b = true;
}

fn apply_plate_appearance_core(
    state: &mut GameState,
    pa: &crate::models::plate_appearance::PlateAppearance,
    recount_pitcher_stats_from_sequence: bool,
    add_terminal_live_pitch: bool,
) {
    // Align inning / half
    if state.inning != pa.inning || state.half != pa.half {
        state.inning = pa.inning;
        state.half = pa.half;

        state.outs = pa.outs;
        state.on_1b = false;
        state.on_2b = false;
        state.on_3b = false;

        state.current_batter_id = None;
        state.current_batter_jersey_no = None;
        state.current_batter_first_name = None;
        state.current_batter_last_name = None;
        state.current_batter_order = None;
        state.current_batter_position = None;
    }

    // Pitcher stats update from actual pitch sequence
    if recount_pitcher_stats_from_sequence {
        let stats = state.pitcher_stats.entry(pa.pitcher_id).or_default();

        for step in &pa.pitches_sequence {
            match step {
                PlateAppearanceStep::Pitch(Pitch::Ball) => {
                    stats.balls = stats.balls.saturating_add(1);
                }

                PlateAppearanceStep::Pitch(_) => {
                    stats.strikes = stats.strikes.saturating_add(1);
                }

                PlateAppearanceStep::Single
                | PlateAppearanceStep::Double
                | PlateAppearanceStep::Triple
                | PlateAppearanceStep::HomeRun => {
                    stats.strikes = stats.strikes.saturating_add(1);
                }

                PlateAppearanceStep::Walk
                | PlateAppearanceStep::Strikeout
                | PlateAppearanceStep::Out => {}
            }
        }
    } else {
        let stats = state.pitcher_stats.entry(pa.pitcher_id).or_default();

        if add_terminal_live_pitch {
            stats.strikes = stats.strikes.saturating_add(1);
        }
    }

    state.current_pitcher_id = Some(pa.pitcher_id);

    // Outcome effects
    match &pa.outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Walk => {
            apply_walk_advancement(state);
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Strikeout(_)
        | crate::models::plate_appearance::PlateAppearanceOutcome::Out => {
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. } => {
            apply_hit_advancement(state, 1);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. } => {
            apply_hit_advancement(state, 2);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. } => {
            apply_hit_advancement(state, 3);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. } => {
            apply_hit_advancement(state, 4);
            state.outs = pa.outs;
        }
    }

    // Advance batting order
    match pa.half {
        HalfInning::Top => {
            state.away_next_batting_order = bump_order(state.away_next_batting_order);
        }
        HalfInning::Bottom => {
            state.home_next_batting_order = bump_order(state.home_next_batting_order);
        }
    }

    // End of PA => reset count UI
    state.pitch_count.balls = 0;
    state.pitch_count.strikes = 0;
    state.pitch_count.sequence.clear();
}

/// Replay / deterministic rebuild:
/// add the full PA pitch count because we're reconstructing from zero.
pub fn apply_plate_appearance(
    state: &mut GameState,
    pa: &crate::models::plate_appearance::PlateAppearance,
) {
    apply_plate_appearance_core(state, pa, true, false);
}

/// Live game flow:
/// only add pitches that were NOT already counted by live PitchRecorded events.
pub fn apply_live_plate_appearance(
    state: &mut GameState,
    pa: &crate::models::plate_appearance::PlateAppearance,
) {
    let add_terminal_live_pitch = matches!(
        &pa.outcome,
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. }
            | crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. }
            | crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. }
            | crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. }
    );

    apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch);
}

fn parse_hit_outcome_data(raw: Option<&str>) -> crate::models::plate_appearance::HitOutcomeData {
    serde_json::from_str(raw.unwrap_or(r#"{"zone":null}"#))
        .unwrap_or(crate::models::plate_appearance::HitOutcomeData { zone: None })
}

/// Apply a compact, persisted Plate Appearance row to the in-memory GameState.
///
/// This is used on resume to rebuild the game without replaying pitch-by-pitch.
pub fn apply_plate_appearance_row(state: &mut GameState, row: &PlateAppearanceRow) {
    let outcome = match row.outcome_type.as_str() {
        "walk" => crate::models::plate_appearance::PlateAppearanceOutcome::Walk,

        "strikeout" => {
            let kind: StrikeoutKind =
                serde_json::from_str(row.outcome_data.as_deref().unwrap_or("null"))
                    .unwrap_or(StrikeoutKind::Called);

            crate::models::plate_appearance::PlateAppearanceOutcome::Strikeout(kind)
        }

        "out" => crate::models::plate_appearance::PlateAppearanceOutcome::Out,

        "single" => {
            let data = parse_hit_outcome_data(row.outcome_data.as_deref());
            crate::models::plate_appearance::PlateAppearanceOutcome::Single { zone: data.zone }
        }

        "double" => {
            let data = parse_hit_outcome_data(row.outcome_data.as_deref());
            crate::models::plate_appearance::PlateAppearanceOutcome::Double { zone: data.zone }
        }

        "triple" => {
            let data = parse_hit_outcome_data(row.outcome_data.as_deref());
            crate::models::plate_appearance::PlateAppearanceOutcome::Triple { zone: data.zone }
        }

        "home_run" => {
            let data = parse_hit_outcome_data(row.outcome_data.as_deref());
            crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { zone: data.zone }
        }

        _ => crate::models::plate_appearance::PlateAppearanceOutcome::Out,
    };

    let seq: Vec<PlateAppearanceStep> = parse_pa_sequence(&row.pitches_sequence);

    let pa = crate::models::plate_appearance::PlateAppearance {
        inning: row.inning as u32,
        half: if row.half_inning == "Bottom" {
            HalfInning::Bottom
        } else {
            HalfInning::Top
        },
        batter_id: row.batter_id,
        batter_order: row.batter_order,
        pitcher_id: row.pitcher_id,
        pitches: row.pitches as u32,
        pitches_sequence: seq,
        outcome,
        outs: row.outs as u8,
    };

    apply_plate_appearance(state, &pa);
}
