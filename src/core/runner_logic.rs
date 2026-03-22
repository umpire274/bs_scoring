//! Unified runner movement logic.
//!
//! This module consolidates all runner advancement logic that was previously
//! duplicated across `play_ball_reducer.rs` and `play_ball_apply.rs`.
//! It provides a single source of truth for:
//! - Hit advancement (with optional overrides)
//! - Walk advancement (forced advancement)
//! - Movement record generation for DB persistence

use crate::db::runner_movements::RunnerMovementInsert;
use crate::models::game_state::{BatterOrder, GameState};
use crate::models::runner::{RunnerDest, RunnerOverride};
use crate::models::types::HalfInning;
use std::collections::HashMap;

// ─── Score helpers ────────────────────────────────────────────────────────────

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

// ─── Base helpers ─────────────────────────────────────────────────────────────

fn base_str(b: u8) -> &'static str {
    match b {
        1 => "1B",
        2 => "2B",
        3 => "3B",
        _ => "HOME",
    }
}

/// Place a runner at `dest` (1/2/3) or score (>=4).
fn place_runner(
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

/// Resolve where a runner ends up: override if present, else auto-advance.
fn resolve_dest(
    order: BatterOrder,
    auto_dest: u8,
    override_map: &HashMap<BatterOrder, RunnerDest>,
) -> (u8, bool) {
    // Returns (dest_base_number, is_override)
    match override_map.get(&order) {
        Some(RunnerDest::First) => (1, true),
        Some(RunnerDest::Second) => (2, true),
        Some(RunnerDest::Third) => (3, true),
        Some(RunnerDest::Score) => (4, true), // 4 = score
        None => (auto_dest, false),
    }
}

/// Compute the end-base string and scored flag for a movement row.
fn effective_end(dest: u8) -> (&'static str, bool) {
    if dest > 3 {
        ("HOME", true)
    } else {
        (base_str(dest), false)
    }
}

// ─── Hit advancement ─────────────────────────────────────────────────────────

/// Result of applying hit advancement to the game state.
pub struct HitResult {
    /// Runner movement rows to persist in the DB.
    pub movements: Vec<RunnerMovementInsert>,
    /// Total runs scored on this play.
    pub runs_scored: u32,
}

/// Apply hit advancement with optional per-runner overrides.
///
/// Mutates `state` (base occupancy, score, hit counter) and returns
/// movement records for DB persistence.
///
/// The `batter_order` is who just hit; they go to `bases` base by default.
/// Each `RunnerOverride` explicitly places a runner already on base.
/// Any runner NOT mentioned in overrides advances automatically (`current_base + bases`).
pub fn apply_hit(
    state: &mut GameState,
    batter_order: BatterOrder,
    bases: u8,
    overrides: &[RunnerOverride],
) -> HitResult {
    let half_str = state.half.as_str();
    let inning = state.inning;

    // Snapshot current base occupants before clearing
    let snapshot = BaseSnapshot {
        on_1b: state.on_1b,
        on_2b: state.on_2b,
        on_3b: state.on_3b,
    };

    // Clear bases
    state.on_1b = None;
    state.on_2b = None;
    state.on_3b = None;

    let override_map: HashMap<BatterOrder, RunnerDest> =
        overrides.iter().map(|r| (r.order, r.dest)).collect();

    let mut runs_scored: u32 = 0;

    // Move runners already on base (order: 3B first to avoid collisions)
    for (runner, current_base) in snapshot.runners_descending() {
        let (dest, _is_override) =
            resolve_dest(runner, current_base + bases, &override_map);
        place_runner(
            runner,
            dest,
            &mut runs_scored,
            &mut state.on_1b,
            &mut state.on_2b,
            &mut state.on_3b,
        );
    }

    // Place batter
    place_runner(
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

    // Build movement rows
    let movements = build_movements_from_snapshot(
        &snapshot,
        batter_order,
        bases,
        &override_map,
        inning,
        half_str,
    );

    HitResult {
        movements,
        runs_scored,
    }
}

/// Build movement rows from a pre-mutation snapshot.
///
/// This is the single source of truth for both live play and replay.
/// Called from `apply_hit()` and also directly when movements are needed
/// without mutating state (e.g. for the live PA path).
pub fn build_movements_from_snapshot(
    snapshot: &BaseSnapshot,
    batter_order: BatterOrder,
    bases: u8,
    override_map: &HashMap<BatterOrder, RunnerDest>,
    inning: u32,
    half_str: &str,
) -> Vec<RunnerMovementInsert> {
    let mut movements = Vec::with_capacity(4);

    // Existing runners (3B → 2B → 1B)
    for (order, current_base) in snapshot.runners_descending() {
        let (dest, is_override) = resolve_dest(order, current_base + bases, override_map);
        let (end, scored) = effective_end(dest);
        let adv = if is_override { "hit_override" } else { "hit_auto" };
        movements.push(make_movement(
            order,
            base_str(current_base),
            end,
            scored,
            adv,
            inning,
            half_str,
        ));
    }

    // Batter
    let (batter_end, batter_scored) = effective_end(bases);
    movements.push(make_movement(
        batter_order,
        "BAT",
        batter_end,
        batter_scored,
        "hit_auto",
        inning,
        half_str,
    ));

    movements
}

// ─── Walk advancement ─────────────────────────────────────────────────────────

/// Result of applying walk advancement.
pub struct WalkResult {
    pub movements: Vec<RunnerMovementInsert>,
    pub runs_scored: u32,
}

/// Apply walk advancement to the game state.
///
/// Forces runners ahead if bases are occupied ahead of them.
/// Returns movement records for DB persistence.
pub fn apply_walk(state: &mut GameState, batter_order: BatterOrder) -> WalkResult {
    let half_str = state.half.as_str();
    let inning = state.inning;
    let mut movements = Vec::with_capacity(4);
    let mut runs_scored: u32 = 0;

    // Bases loaded: runner from 3B scores
    if state.on_1b.is_some() && state.on_2b.is_some() && state.on_3b.is_some() {
        let r3 = state.on_3b.unwrap_or(0);
        movements.push(make_movement(r3, "3B", "HOME", true, "walk", inning, half_str));
        state.on_3b = None;

        runs_scored += 1;
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
    if state.on_1b.is_some() && state.on_2b.is_some() {
        let r2 = state.on_2b.unwrap_or(0);
        movements.push(make_movement(r2, "2B", "3B", false, "walk", inning, half_str));
        state.on_3b = state.on_2b;
    }

    if state.on_1b.is_some() {
        let r1 = state.on_1b.unwrap_or(0);
        movements.push(make_movement(r1, "1B", "2B", false, "walk", inning, half_str));
        state.on_2b = state.on_1b;
    }

    // Batter takes first
    movements.push(make_movement(
        batter_order,
        "BAT",
        "1B",
        false,
        "walk",
        inning,
        half_str,
    ));
    state.on_1b = Some(batter_order);

    WalkResult {
        movements,
        runs_scored,
    }
}

// ─── Base snapshot ─────────────────────────────────────────────────────────────

/// Snapshot of base occupancy before a play mutates the state.
#[derive(Debug, Clone)]
pub struct BaseSnapshot {
    pub on_1b: Option<BatterOrder>,
    pub on_2b: Option<BatterOrder>,
    pub on_3b: Option<BatterOrder>,
}

impl BaseSnapshot {
    /// Create a snapshot from the current game state.
    pub fn from_state(state: &GameState) -> Self {
        Self {
            on_1b: state.on_1b,
            on_2b: state.on_2b,
            on_3b: state.on_3b,
        }
    }

    /// Iterate runners in descending base order (3B, 2B, 1B).
    /// Returns (batting_order, base_number) for occupied bases.
    pub fn runners_descending(&self) -> Vec<(BatterOrder, u8)> {
        let mut runners = Vec::with_capacity(3);
        if let Some(order) = self.on_3b {
            runners.push((order, 3));
        }
        if let Some(order) = self.on_2b {
            runners.push((order, 2));
        }
        if let Some(order) = self.on_1b {
            runners.push((order, 1));
        }
        runners
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_movement(
    batter_order: BatterOrder,
    start: &'static str,
    end: &'static str,
    scored: bool,
    advancement_type: &'static str,
    inning: u32,
    half_str: &str,
) -> RunnerMovementInsert {
    RunnerMovementInsert {
        game_id: 0,          // filled in by engine loop
        pa_seq: None,        // linked to PA by engine after PA insert
        game_event_id: None, // standalone runner movement
        inning,
        half_inning: half_str.to_string(),
        runner_id: None, // no player_id in reducer; batter_order is the identity
        batter_order,
        start_base: start,
        end_base: end,
        advancement_type,
        is_out: false,
        scored,
        is_earned: true,
    }
}

// ─── Override validation ──────────────────────────────────────────────────────

/// Validate runner overrides before applying a hit.
///
/// Checks:
/// 1. No two overrides target the same base.
/// 2. No override sends a runner to a base already occupied by a non-moved runner.
pub fn validate_runner_overrides(
    state: &GameState,
    _batter_order: u8,
    overrides: &[RunnerOverride],
) -> Result<(), String> {
    use std::collections::HashSet;

    // Collect all destination bases claimed by overrides (excluding Score).
    let mut claimed: HashSet<u8> = HashSet::new();

    for ro in overrides {
        let dest_base: Option<u8> = match ro.dest {
            RunnerDest::First => Some(1),
            RunnerDest::Second => Some(2),
            RunnerDest::Third => Some(3),
            RunnerDest::Score => None, // multiple runners can score
        };
        if let Some(b) = dest_base
            && !claimed.insert(b)
        {
            return Err(format!(
                "Two runners cannot end up on the same base ({}B)",
                b
            ));
        }
    }

    // Check that no override destination is occupied by a runner who is NOT
    // in the overrides list.
    let moved_orders: HashSet<u8> = overrides.iter().map(|r| r.order).collect();

    let check_base = |base_occupant: Option<u8>, base_no: u8| -> Result<(), String> {
        if let Some(occupant) = base_occupant
            && claimed.contains(&base_no)
            && !moved_orders.contains(&occupant)
        {
            return Err(format!(
                "Runner {} on {}B would be overwritten — add an explicit override for them",
                occupant, base_no
            ));
        }
        Ok(())
    };

    check_base(state.on_1b, 1)?;
    check_base(state.on_2b, 2)?;
    check_base(state.on_3b, 3)?;

    Ok(())
}
