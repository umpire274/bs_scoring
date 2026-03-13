use crate::db::plate_appearances::PlateAppearanceRow;
use crate::engine::play_ball::{bump_order, parse_pa_sequence};
use crate::models::events::{DomainEvent, StrikeoutKind};
use crate::models::plate_appearance::PlateAppearanceStep;
use crate::models::play_ball::{BatterOrder, GameState, RunnerDest, RunnerOverride};
use crate::models::types::{HalfInning, Pitch};

/// Apply a persisted DomainEvent to the in-memory GameState.
pub fn apply_domain_event(state: &mut GameState, ev: &DomainEvent) {
    match ev {
        DomainEvent::SideChange(d) => {
            state.inning = d.inning;
            state.half = d.half;
            state.outs = 0;

            state.on_1b = None;
            state.on_2b = None;
            state.on_3b = None;

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

// ─── Base placement helpers ───────────────────────────────────────────────────

/// Place a runner (identified by batting order) at `dest` (1/2/3) or score (>=4).
fn place_runner_with_order(
    order: BatterOrder,
    dest: u8,
    runs_scored: &mut u32,
    on_1b: &mut Option<BatterOrder>,
    on_2b: &mut Option<BatterOrder>,
    on_3b: &mut Option<BatterOrder>,
) {
    if dest >= 4 {
        *runs_scored += 1;
        return;
    }
    match dest {
        1 => *on_1b = Some(order),
        2 => *on_2b = Some(order),
        3 => *on_3b = Some(order),
        _ => {}
    }
}

fn ensure_inning(vec: &mut Vec<u16>, inning: u32) {
    let idx = inning as usize;
    if vec.len() < idx {
        vec.resize(idx, 0);
    }
}

fn add_runs_to_score(state: &mut GameState, runs: u32) {
    if runs == 0 {
        return;
    }
    match state.half {
        HalfInning::Top => {
            state.score.away += runs as u16;
            ensure_inning(&mut state.score.away_innings, state.inning);
            let idx = (state.inning - 1) as usize;
            state.score.away_innings[idx] += runs as u16;
        }
        HalfInning::Bottom => {
            state.score.home += runs as u16;
            ensure_inning(&mut state.score.home_innings, state.inning);
            let idx = (state.inning - 1) as usize;
            state.score.home_innings[idx] += runs as u16;
        }
    }
}

// ─── Hit advancement ─────────────────────────────────────────────────────────

/// Apply hit advancement with optional per-runner overrides.
///
/// The `batter_order` is who just hit (they go to `bases` base by default).
/// Each `RunnerOverride` explicitly places a runner already on base.
/// Any runner NOT mentioned in overrides uses the automatic advance (`current_base + bases`).
///
/// # Override semantics
/// - `RunnerDest::First/Second/Third` → place runner on that base
/// - `RunnerDest::Score` → runner scores (run++)
pub fn apply_hit_with_overrides(
    state: &mut GameState,
    batter_order: BatterOrder,
    bases: u8,
    overrides: &[RunnerOverride],
) {
    let mut runs_scored: u32 = 0;

    // Snapshot current base occupants before clearing
    let runner_on_1b: Option<BatterOrder> = state.on_1b;
    let runner_on_2b: Option<BatterOrder> = state.on_2b;
    let runner_on_3b: Option<BatterOrder> = state.on_3b;

    state.on_1b = None;
    state.on_2b = None;
    state.on_3b = None;

    // Build a quick lookup: batting_order → explicit dest
    let override_map: std::collections::HashMap<BatterOrder, RunnerDest> =
        overrides.iter().map(|r| (r.order, r.dest)).collect();

    // Helper: resolve destination for a runner already on base
    let resolve = |order: BatterOrder,
                   auto_dest: u8,
                   runs_scored: &mut u32,
                   on_1b: &mut Option<BatterOrder>,
                   on_2b: &mut Option<BatterOrder>,
                   on_3b: &mut Option<BatterOrder>| {
        match override_map.get(&order) {
            Some(RunnerDest::First) => {
                place_runner_with_order(order, 1, runs_scored, on_1b, on_2b, on_3b)
            }
            Some(RunnerDest::Second) => {
                place_runner_with_order(order, 2, runs_scored, on_1b, on_2b, on_3b)
            }
            Some(RunnerDest::Third) => {
                place_runner_with_order(order, 3, runs_scored, on_1b, on_2b, on_3b)
            }
            Some(RunnerDest::Score) => *runs_scored += 1,
            None => place_runner_with_order(order, auto_dest, runs_scored, on_1b, on_2b, on_3b),
        }
    };

    // Move runners already on base (order: 3B first to avoid collisions)
    if let Some(order) = runner_on_3b {
        resolve(
            order,
            3 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }
    if let Some(order) = runner_on_2b {
        resolve(
            order,
            2 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }
    if let Some(order) = runner_on_1b {
        resolve(
            order,
            1 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }

    // Place batter (no override allowed on batter — enforced at parse time)
    place_runner_with_order(
        batter_order,
        bases,
        &mut runs_scored,
        &mut state.on_1b,
        &mut state.on_2b,
        &mut state.on_3b,
    );

    add_runs_to_score(state, runs_scored);

    // Hits counter
    match state.half {
        HalfInning::Top => state.score.away_hits += 1,
        HalfInning::Bottom => state.score.home_hits += 1,
    }
}

/// Legacy automatic-only hit advancement (used by PA replay where we don't have override data).
pub fn apply_hit_advancement(state: &mut GameState, bases: u8) {
    // Use a synthetic batter order that won't clash (replay path doesn't track identities).
    // We pass order=0 which is never a real batting order; the batter just goes to `bases`.
    let batter_order: BatterOrder = 0;

    let runner_on_1b = state.on_1b;
    let runner_on_2b = state.on_2b;
    let runner_on_3b = state.on_3b;

    let mut runs_scored: u32 = 0;

    state.on_1b = None;
    state.on_2b = None;
    state.on_3b = None;

    if let Some(order) = runner_on_3b {
        place_runner_with_order(
            order,
            3 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }
    if let Some(order) = runner_on_2b {
        place_runner_with_order(
            order,
            2 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }
    if let Some(order) = runner_on_1b {
        place_runner_with_order(
            order,
            1 + bases,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }

    place_runner_with_order(
        batter_order,
        bases,
        &mut runs_scored,
        &mut state.on_1b,
        &mut state.on_2b,
        &mut state.on_3b,
    );

    add_runs_to_score(state, runs_scored);

    match state.half {
        HalfInning::Top => state.score.away_hits += 1,
        HalfInning::Bottom => state.score.home_hits += 1,
    }
}

// ─── Walk advancement ─────────────────────────────────────────────────────────

fn apply_walk_advancement(state: &mut GameState) {
    // Bases loaded: runner from 3B scores
    if state.on_1b.is_some() && state.on_2b.is_some() && state.on_3b.is_some() {
        // Runner on 3B scores
        state.on_3b = None;
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

    // Forced advancements (shift runners up if forced)
    if state.on_1b.is_some() && state.on_2b.is_some() {
        // on_3b was either None or just cleared above
        state.on_3b = state.on_2b;
    }

    if state.on_1b.is_some() {
        state.on_2b = state.on_1b;
    }

    // Batter takes first — use current batter order if available, else a placeholder
    let batter_order = state.current_batter_order.unwrap_or(0);
    state.on_1b = Some(batter_order);
}

// ─── PA replay ───────────────────────────────────────────────────────────────

fn apply_plate_appearance_core(
    state: &mut GameState,
    pa: &crate::models::plate_appearance::PlateAppearance,
    recount_pitcher_stats_from_sequence: bool,
    add_terminal_live_pitch: bool,
    apply_walk_base_advancement: bool,
) {
    // Align inning / half
    if state.inning != pa.inning || state.half != pa.half {
        state.inning = pa.inning;
        state.half = pa.half;

        state.outs = pa.outs;
        state.on_1b = None;
        state.on_2b = None;
        state.on_3b = None;

        state.current_batter_id = None;
        state.current_batter_jersey_no = None;
        state.current_batter_first_name = None;
        state.current_batter_last_name = None;
        state.current_batter_order = None;
        state.current_batter_position = None;
    }

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

    // Outcome effects — replay uses automatic advancement (no override data stored yet)
    match &pa.outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Walk => {
            if apply_walk_base_advancement {
                apply_walk_advancement(state);
            }
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Strikeout(_)
        | crate::models::plate_appearance::PlateAppearanceOutcome::Out => {
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. } => {
            apply_hit_with_overrides(state, pa.batter_order, 1, &pa.runner_overrides);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. } => {
            apply_hit_with_overrides(state, pa.batter_order, 2, &pa.runner_overrides);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. } => {
            apply_hit_with_overrides(state, pa.batter_order, 3, &pa.runner_overrides);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. } => {
            apply_hit_with_overrides(state, pa.batter_order, 4, &pa.runner_overrides);
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

    state.pitch_count.balls = 0;
    state.pitch_count.strikes = 0;
    state.pitch_count.sequence.clear();
}

pub fn apply_plate_appearance(
    state: &mut GameState,
    pa: &crate::models::plate_appearance::PlateAppearance,
) {
    apply_plate_appearance_core(state, pa, true, false, true);
}

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

    apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
}

fn parse_hit_outcome_data(raw: Option<&str>) -> crate::models::plate_appearance::HitOutcomeData {
    serde_json::from_str(raw.unwrap_or(r#"{"zone":null}"#))
        .unwrap_or(crate::models::plate_appearance::HitOutcomeData { zone: None })
}

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
        runner_overrides: vec![], // legacy rows have no override data
    };

    apply_plate_appearance(state, &pa);
}
