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
) -> Vec<crate::db::runner_movements::RunnerMovementInsert> {
    use crate::db::runner_movements::RunnerMovementInsert;

    let half_str = match state.half {
        HalfInning::Top    => "Top",
        HalfInning::Bottom => "Bottom",
    };
    let inning = state.inning;

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
    let resolve = |order: BatterOrder, auto_dest: u8,
                       runs_scored: &mut u32,
                       on_1b: &mut Option<BatterOrder>,
                       on_2b: &mut Option<BatterOrder>,
                       on_3b: &mut Option<BatterOrder>| {
        match override_map.get(&order) {
            Some(RunnerDest::First) => place_runner_with_order(order, 1, runs_scored, on_1b, on_2b, on_3b),
            Some(RunnerDest::Second) => place_runner_with_order(order, 2, runs_scored, on_1b, on_2b, on_3b),
            Some(RunnerDest::Third) => place_runner_with_order(order, 3, runs_scored, on_1b, on_2b, on_3b),
            Some(RunnerDest::Score) => *runs_scored += 1,
            None => place_runner_with_order(order, auto_dest, runs_scored, on_1b, on_2b, on_3b),
        }
    };

    // Move runners already on base (order: 3B first to avoid collisions)
    if let Some(order) = runner_on_3b {
        resolve(order, 3 + bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);
    }
    if let Some(order) = runner_on_2b {
        resolve(order, 2 + bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);
    }
    if let Some(order) = runner_on_1b {
        resolve(order, 1 + bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);
    }

    // Place batter (no override allowed on batter — enforced at parse time)
    place_runner_with_order(batter_order, bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);

    add_runs_to_score(state, runs_scored);

    // Hits counter
    match state.half {
        HalfInning::Top => state.score.away_hits += 1,
        HalfInning::Bottom => state.score.home_hits += 1,
    }

    // Build RunnerMovementInsert rows from snapshot + resolved destinations.
    let base_str = |b: u8| -> &'static str {
        match b { 1 => "1B", 2 => "2B", 3 => "3B", _ => "HOME" }
    };
    let effective_end = |order: BatterOrder, start_base_n: u8| -> (&'static str, bool) {
        let raw_dest = start_base_n + bases;
        match override_map.get(&order) {
            Some(RunnerDest::First)  => ("1B",   false),
            Some(RunnerDest::Second) => ("2B",   false),
            Some(RunnerDest::Third)  => ("3B",   false),
            Some(RunnerDest::Score)  => ("HOME", true),
            None => {
                if raw_dest > 3 { ("HOME", true) }
                else            { (base_str(raw_dest), false) }
            }
        }
    };

    let mut movements: Vec<RunnerMovementInsert> = vec![];

    let push_rm = |movements: &mut Vec<RunnerMovementInsert>,
                   border: BatterOrder,
                   start: &'static str,
                   end: &'static str,
                   scored: bool,
                   adv_type: &'static str| {
        movements.push(RunnerMovementInsert {
            game_id: 0,           // filled by engine
            pa_seq: None,         // filled by engine after PA insert
            game_event_id: None,
            inning,
            half_inning: half_str.to_string(),
            runner_id: None,      // no player_id in reducer; batter_order is the identity
            batter_order: border,
            start_base: start,
            end_base: end,
            advancement_type: adv_type,
            is_out: false,
            scored,
            is_earned: true,
        });
    };

    if let Some(order) = runner_on_3b {
        let (end, scored) = effective_end(order, 3);
        let adv = if override_map.contains_key(&order) { "hit_override" } else { "hit_auto" };
        push_rm(&mut movements, order, "3B", end, scored, adv);
    }
    if let Some(order) = runner_on_2b {
        let (end, scored) = effective_end(order, 2);
        let adv = if override_map.contains_key(&order) { "hit_override" } else { "hit_auto" };
        push_rm(&mut movements, order, "2B", end, scored, adv);
    }
    if let Some(order) = runner_on_1b {
        let (end, scored) = effective_end(order, 1);
        let adv = if override_map.contains_key(&order) { "hit_override" } else { "hit_auto" };
        push_rm(&mut movements, order, "1B", end, scored, adv);
    }
    // Batter
    {
        let raw_end = bases;
        let (end, scored) = if raw_end > 3 { ("HOME", true) } else { (base_str(raw_end), false) };
        push_rm(&mut movements, batter_order, "BAT", end, scored, "hit_auto");
    }

    movements
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
        place_runner_with_order(order, 3 + bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);
    }
    if let Some(order) = runner_on_2b {
        place_runner_with_order(order, 2 + bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);
    }
    if let Some(order) = runner_on_1b {
        place_runner_with_order(order, 1 + bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);
    }

    place_runner_with_order(batter_order, bases, &mut runs_scored, &mut state.on_1b, &mut state.on_2b, &mut state.on_3b);

    add_runs_to_score(state, runs_scored);

    match state.half {
        HalfInning::Top => state.score.away_hits += 1,
        HalfInning::Bottom => state.score.home_hits += 1,
    }
}

// ─── Walk advancement ─────────────────────────────────────────────────────────

fn apply_walk_advancement(state: &mut GameState, batter_order: BatterOrder) {
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

    // Batter takes first
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
        HalfInning::Top    => "Top",
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
                runner_on_1b, runner_on_2b, runner_on_3b,
                pa.batter_order, 1, &pa.runner_overrides,
                inning, half_str,
            )
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Double { .. } => {
            apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
            build_hit_movements_from_snapshot(
                runner_on_1b, runner_on_2b, runner_on_3b,
                pa.batter_order, 2, &pa.runner_overrides,
                inning, half_str,
            )
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { .. } => {
            apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
            build_hit_movements_from_snapshot(
                runner_on_1b, runner_on_2b, runner_on_3b,
                pa.batter_order, 3, &pa.runner_overrides,
                inning, half_str,
            )
        }
        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { .. } => {
            apply_plate_appearance_core(state, pa, false, add_terminal_live_pitch, false);
            build_hit_movements_from_snapshot(
                runner_on_1b, runner_on_2b, runner_on_3b,
                pa.batter_order, 4, &pa.runner_overrides,
                inning, half_str,
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
/// Called only from apply_live_plate_appearance so we don't duplicate logic.
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
    use crate::db::runner_movements::RunnerMovementInsert;

    let override_map: std::collections::HashMap<u8, RunnerDest> =
        overrides.iter().map(|r| (r.order, r.dest)).collect();

    let base_str = |b: u8| -> &'static str {
        match b { 1 => "1B", 2 => "2B", 3 => "3B", _ => "HOME" }
    };

    let effective_end = |order: u8, start_base_n: u8| -> (&'static str, bool) {
        let raw = start_base_n + bases;
        match override_map.get(&order) {
            Some(RunnerDest::First)  => ("1B",   false),
            Some(RunnerDest::Second) => ("2B",   false),
            Some(RunnerDest::Third)  => ("3B",   false),
            Some(RunnerDest::Score)  => ("HOME", true),
            None => if raw > 3 { ("HOME", true) } else { (base_str(raw), false) },
        }
    };

    let mut movements = vec![];
    let push = |movements: &mut Vec<RunnerMovementInsert>, border: u8,
                start: &'static str, end: &'static str, scored: bool, adv: &'static str| {
        movements.push(RunnerMovementInsert {
            game_id: 0,
            pa_seq: None,
            game_event_id: None,
            inning,
            half_inning: half_str.to_string(),
            runner_id: None,
            batter_order: border,
            start_base: start,
            end_base: end,
            advancement_type: adv,
            is_out: false,
            scored,
            is_earned: true,
        });
    };

    if let Some(order) = runner_on_3b {
        let (end, scored) = effective_end(order, 3);
        let adv = if override_map.contains_key(&order) { "hit_override" } else { "hit_auto" };
        push(&mut movements, order, "3B", end, scored, adv);
    }
    if let Some(order) = runner_on_2b {
        let (end, scored) = effective_end(order, 2);
        let adv = if override_map.contains_key(&order) { "hit_override" } else { "hit_auto" };
        push(&mut movements, order, "2B", end, scored, adv);
    }
    if let Some(order) = runner_on_1b {
        let (end, scored) = effective_end(order, 1);
        let adv = if override_map.contains_key(&order) { "hit_override" } else { "hit_auto" };
        push(&mut movements, order, "1B", end, scored, adv);
    }
    // Batter
    {
        let raw = bases;
        let (end, scored) = if raw > 3 { ("HOME", true) } else { (base_str(raw), false) };
        push(&mut movements, batter_order, "BAT", end, scored, "hit_auto");
    }

    movements
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
