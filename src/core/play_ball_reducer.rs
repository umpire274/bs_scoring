use crate::core::runner_logic;
use crate::db::plate_appearances::PlateAppearanceRow;
use crate::engine::play_ball::{bump_order, parse_pa_sequence};
use crate::models::events::{DomainEvent, StrikeoutKind};
use crate::models::game_state::{BatterOrder, GameState};
use crate::models::plate_appearance::PlateAppearanceStep;
use crate::models::runner::{RunnerDest, RunnerOverride};
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

        DomainEvent::RunnerToFirst { batter_order, .. } => {
            apply_walk_advancement(state, *batter_order);
        }
    }
}

// ─── Base placement helpers ───────────────────────────────────────────────────
// NOTE: State mutation and score tracking for hits and walks is now handled by
// `crate::core::runner_logic`. The functions below have been removed:
// - place_runner_with_order → runner_logic::place_runner
// - ensure_inning → runner_logic (internal)
// - add_runs_to_score → runner_logic (internal)

// ─── Hit advancement ─────────────────────────────────────────────────────────

/// Apply hit advancement with optional per-runner overrides.
///
/// Delegates to `runner_logic::apply_hit` — the single source of truth.
pub fn apply_hit_with_overrides(
    state: &mut GameState,
    batter_order: BatterOrder,
    bases: u8,
    overrides: &[RunnerOverride],
) -> Vec<crate::db::runner_movements::RunnerMovementInsert> {
    let result = runner_logic::apply_hit(state, batter_order, bases, overrides);
    result.movements
}

/// Legacy automatic-only hit advancement (used by PA replay where we don't have override data).
pub fn apply_hit_advancement(state: &mut GameState, bases: u8) {
    let batter_order: BatterOrder = 0;
    let _ = runner_logic::apply_hit(state, batter_order, bases, &[]);
}

// ─── Walk advancement ─────────────────────────────────────────────────────────

fn apply_walk_advancement(state: &mut GameState, batter_order: BatterOrder) {
    let _ = runner_logic::apply_walk(state, batter_order);
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
                apply_walk_advancement(state, pa.batter_order);
            }
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Strikeout(_)
        | crate::models::plate_appearance::PlateAppearanceOutcome::Out => {
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. } => {
            let _ = apply_hit_with_overrides(state, pa.batter_order, 1, &pa.runner_overrides);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. } => {
            let _ = apply_hit_with_overrides(state, pa.batter_order, 2, &pa.runner_overrides);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. } => {
            let _ = apply_hit_with_overrides(state, pa.batter_order, 3, &pa.runner_overrides);
            state.outs = pa.outs;
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. } => {
            let _ = apply_hit_with_overrides(state, pa.batter_order, 4, &pa.runner_overrides);
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
) -> Vec<crate::db::runner_movements::RunnerMovementInsert> {
    use crate::db::runner_movements::RunnerMovementInsert;

    // Snapshot bases before state mutation so we can build movement rows.
    let runner_on_1b = state.on_1b;
    let runner_on_2b = state.on_2b;
    let runner_on_3b = state.on_3b;
    let inning = state.inning;
    let half_str = match state.half {
        HalfInning::Top => "Top",
        HalfInning::Bottom => "Bottom",
    };

    let add_terminal_live_pitch = matches!(
        &pa.outcome,
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. }
            | crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. }
            | crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. }
            | crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. }
    );

    // For hits: apply_hit_with_overrides is called inside apply_plate_appearance_core
    // and now returns movements — but we can't intercept it there without bigger
    // refactor. Instead, call it directly here for hit outcomes and skip the core path.
    let movements: Vec<RunnerMovementInsert> = match &pa.outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { .. } => {
            apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
            // Rebuild movements from snapshot (state already mutated by core)
            build_hit_movements_from_snapshot(
                runner_on_1b,
                runner_on_2b,
                runner_on_3b,
                pa.batter_order,
                1,
                &pa.runner_overrides,
                inning,
                half_str,
            )
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. } => {
            apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
            build_hit_movements_from_snapshot(
                runner_on_1b,
                runner_on_2b,
                runner_on_3b,
                pa.batter_order,
                2,
                &pa.runner_overrides,
                inning,
                half_str,
            )
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. } => {
            apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
            build_hit_movements_from_snapshot(
                runner_on_1b,
                runner_on_2b,
                runner_on_3b,
                pa.batter_order,
                3,
                &pa.runner_overrides,
                inning,
                half_str,
            )
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. } => {
            apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
            build_hit_movements_from_snapshot(
                runner_on_1b,
                runner_on_2b,
                runner_on_3b,
                pa.batter_order,
                4,
                &pa.runner_overrides,
                inning,
                half_str,
            )
        }
        _ => {
            apply_plate_appearance_core(state, pa, false, false, false);
            vec![]
        }
    };

    movements
}

/// Build RunnerMovementInsert rows from a pre-mutation base snapshot.
/// Delegates to runner_logic::build_movements_from_snapshot.
#[allow(clippy::too_many_arguments)]
fn build_hit_movements_from_snapshot(
    runner_on_1b: Option<u8>,
    runner_on_2b: Option<u8>,
    runner_on_3b: Option<u8>,
    batter_order: u8,
    bases: u8,
    overrides: &[RunnerOverride],
    inning: u32,
    half_str: &str,
) -> Vec<crate::db::runner_movements::RunnerMovementInsert> {
    let snapshot = runner_logic::BaseSnapshot {
        on_1b: runner_on_1b,
        on_2b: runner_on_2b,
        on_3b: runner_on_3b,
    };
    let override_map: std::collections::HashMap<u8, RunnerDest> =
        overrides.iter().map(|r| (r.order, r.dest)).collect();
    runner_logic::build_movements_from_snapshot(
        &snapshot,
        batter_order,
        bases,
        &override_map,
        inning,
        half_str,
    )
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
        runner_overrides: row.runner_overrides(),
    };

    apply_plate_appearance(state, &pa);
}
