# 🎯 BS Scoring v0.9.2 – Project Structure

## 📂 Directory layout

```
bs_scoring/
│
├── Cargo.toml                  # Package manifest ([lib] + [[bin]])
├── Cargo.lock
├── README.md
├── CHANGELOG.md
├── SCORING_GUIDE.md            # Scorer command reference
├── STRUCTURE.md                # This file
├── .gitignore
│
└── src/
    ├── lib.rs                  # Library entry point / public re-exports
    ├── main.rs                 # Binary entry point
    │
    ├── models/                 # Pure data types — no I/O, no DB
    │   ├── types.rs            # HalfInning, Pitch, GameStatus, Score, Position, …
    │   ├── game_state.rs       # GameState, BatterOrder, PitchStats        [v0.9.0]
    │   ├── runner.rs           # RunnerDest, RunnerOverride                 [v0.9.0]
    │   ├── session.rs          # PlayBallGameContext, PlayBallGate,         [v0.9.0]
    │   │                       #   LineupSide
    │   ├── play_ball.rs        # ⚠ Compatibility shim — re-exports from    [v0.9.0]
    │   │                       #   game_state, runner, session
    │   ├── plate_appearance.rs # PlateAppearance, PlateAppearanceOutcome, …
    │   ├── events.rs           # DomainEvent, PersistedEvent               [v0.9.0]
    │   ├── field_zone.rs       # FieldZone (LF, CF, RF, …)
    │   ├── player_traits.rs    # PitchHand, BatSide
    │   └── scoring/            # Full scoring notation types               [v0.9.0]
    │       ├── mod.rs
    │       └── types.rs        # HitType, OutType, Walk, AdvancedPlay,
    │                           #   PlateAppearanceResult, Base, ScoringError
    │                           #   (used by core/parser.rs; not by live engine)
    │
    ├── commands/               # Input parsing
    │   ├── types.rs            # EngineCommand enum (incl. StealBase)      [v0.9.1]
    │   └── engine_parser.rs    # parse_engine_commands()
    │                           #   handles "6 h, 5 2b" and "6 st 2b" syntax
    │
    ├── core/                   # Game logic
    │   ├── menu.rs             # COBOL-style menu system
    │   ├── parser.rs           # Scoring notation parser (legacy / reference)
    │   ├── play_ball.rs        # ⚠ Deprecated shim — re-exports from      [v0.9.0]
    │   │                       #   db/game_queries
    │   ├── play_ball_apply.rs  # EngineCommand → ApplyResult
    │   │                       #   apply_steal() lives here                [v0.9.1]
    │   └── play_ball_reducer.rs# DomainEvent / PA → GameState mutations
    │                           #   apply_hit_with_overrides() lives here
    │
    ├── engine/
    │   └── play_ball.rs        # Main game loop: I/O, DB persistence, state drive
    │
    ├── db/                     # SQLite persistence layer
    │   ├── database.rs         # Connection management
    │   ├── migrations.rs       # Schema versioning (v1–v16)                [v0.9.2]
    │   ├── game_queries.rs     # list_playable_games, gate_check_lineups,  [v0.9.0]
    │   │                       #   set_game_status
    │   ├── plate_appearances.rs# plate_appearances CRUD                    [v0.9.0]
    │   │                       #   append_plate_appearance returns seq i64  [v0.9.2]
    │   ├── runner_movements.rs # runner_movements CRUD                     [v0.9.2]
    │   │                       #   append_runner_movement, list_runner_movements
    │   ├── game_events.rs      # game_events log CRUD (admin/info only)    [v0.9.2]
    │   ├── at_bat_draft.rs     # In-progress PA draft (resume support)
    │   ├── league.rs           # League CRUD
    │   ├── team.rs             # Team CRUD
    │   ├── player.rs           # Player CRUD
    │   └── config.rs           # Cross-platform DB path
    │
    ├── ui/                     # UI abstractions
    │   ├── events.rs           # UiEvent definitions
    │   ├── context.rs          # PlayBallUiContext (team names, …)
    │   ├── factory.rs          # UI backend selection
    │   ├── app.rs              # App-level UI trait
    │   ├── cli.rs              # Plain-text CLI backend
    │   └── tui.rs              # Terminal UI (ratatui) backend
    │
    ├── cli/                    # CLI command handlers (menu actions)
    │   └── commands/
    │       ├── main_menu.rs
    │       ├── game.rs
    │       ├── play_ball.rs
    │       ├── leagues.rs
    │       ├── team.rs
    │       ├── players.rs
    │       ├── statistics.rs
    │       └── db.rs
    │
    └── utils/
        ├── boot.rs             # App initialization
        └── cli.rs              # CliSelectable trait, choose_enum helpers
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   main.rs / cli/                         │
│            Menu-driven CLI application                   │
└───────────────────────────┬─────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                    engine/play_ball.rs                   │
│  Main game loop: reads input, drives state, writes DB   │
└──────┬─────────────────┬──────────────────┬─────────────┘
       │                 │                  │
       ▼                 ▼                  ▼
┌────────────┐  ┌─────────────────┐  ┌─────────────────────┐
│ commands/  │  │     core/       │  │        db/          │
│            │  │                 │  │                     │
│ engine_    │  │ play_ball_      │  │ plate_appearances.rs│
│ parser.rs  │  │ apply.rs        │  │ game_events.rs      │
│            │  │                 │  │ game_queries.rs     │
│ "6 h, 5 2b"│  │ play_ball_      │  │ at_bat_draft.rs     │
│ "6 st 2b"  │  │ reducer.rs      │  │                     │
│            │  │                 │  │ SQLite (v15)        │
└────────────┘  └────────┬────────┘  └─────────────────────┘
                         │
                         ▼
               ┌──────────────────────┐
               │       models/        │
               │                      │
               │ game_state.rs        │
               │   GameState          │
               │   on_1b/2b/3b:       │
               │   Option<BatterOrder>│
               │                      │
               │ runner.rs            │
               │   RunnerDest         │
               │   RunnerOverride     │
               │                      │
               │ session.rs           │
               │   PlayBallGameContext│
               │   PlayBallGate       │
               │                      │
               │ plate_appearance.rs  │
               │ events.rs            │
               └──────────────────────┘
```

---

## 🔑 Key design decisions

### Model split (v0.9.0)

Prior to v0.9.0, `models/play_ball.rs` contained everything related to live
game state. It has been split into focused modules:

| Module                    | Contents                                                                                 |
|---------------------------|------------------------------------------------------------------------------------------|
| `models/game_state.rs`    | `GameState`, `BatterOrder`, `PitchStats`                                                 |
| `models/runner.rs`        | `RunnerDest`, `RunnerOverride`                                                           |
| `models/session.rs`       | `PlayBallGameContext`, `PlayBallGate`, `LineupSide`                                      |
| `models/scoring/types.rs` | Full scoring notation types (`HitType`, `OutType`, etc.) — used only by `core/parser.rs` |
| `models/play_ball.rs`     | Compatibility shim — re-exports from the above modules                                   |

Similarly, `core/play_ball.rs` (DB queries) has been moved to `db/game_queries.rs`
and kept as a deprecated re-export shim.

### Runner identity on bases (`Option<BatterOrder>`)

`GameState.on_1b/on_2b/on_3b` are `Option<BatterOrder>` (since v0.8.0).
The engine knows *who* is on each base by batting-order slot, not just
*whether* a base is occupied. This enables explicit runner overrides and
steal command validation.

The UI still displays bases as occupied/empty (`◆` / `◇`).

### Runner override flow

```
Input: "6 h, 5 2b"
  └─ parse_engine_commands()
       └─ Single { zone: None, runner_overrides: [{ order: 5, dest: Second }] }
            └─ apply_hit_command()
                 ├─ validate_runner_overrides()   ← collision check before state change
                 └─ PlateAppearance { ..., runner_overrides: [...] }
                      └─ apply_live_plate_appearance()
                           └─ apply_hit_with_overrides(state, batter=6, bases=1, overrides)
                                • runner on 2B was order=5 → override: stays on 2B
                                • batter #6 → automatic: goes to 1B
```

### Steal flow (v0.9.1)

```
Input: "k, 6 st 2b"
  └─ parse_engine_commands()
       ├─ Pitch(CalledStrike)
       └─ StealBase { order: 6, dest: Second }
            └─ apply_steal()
                 ├─ validate: on_1b == Some(6)?   ← error if runner not on source base
                 ├─ state.on_1b = None
                 ├─ state.on_2b = Some(6)
                 └─ persisted: DomainEvent::StolenBase { order: 6, dest: Second, … }
```

### Two advancement paths

| Path           | Function                       | Used for                              |
|----------------|--------------------------------|---------------------------------------|
| With overrides | `apply_hit_with_overrides()`   | Live scoring                          |
| Replay from DB | `apply_plate_appearance_row()` | Resume — uses `runner_overrides_json` |

### Override validation

Before any state mutation, `validate_runner_overrides()` checks:

1. No two overrides (or batter destination) claim the same base.
2. No override targets a base occupied by a runner *not* listed in the
   overrides — they would otherwise be silently evicted.

Both conditions return an explicit error message to the scorer.

### DB schema version

Current: **v16**.

| Migration | Change                                                                                                                    |
|-----------|---------------------------------------------------------------------------------------------------------------------------|
| v14       | `plate_appearances_compact` → `plate_appearances` with `batter_order`                                                     |
| v15       | Add `runner_overrides_json TEXT NOT NULL DEFAULT '[]'` to `plate_appearances`                                             |
| v16       | Rebuild `runner_movements`: drop legacy `at_bat_id` FK, add `pa_seq`, `game_event_id`, `inning`, `half_inning`, `game_id` |

### `game_events` vs `runner_movements` responsibility

| Table              | What goes here                                                                                                                    |
|--------------------|-----------------------------------------------------------------------------------------------------------------------------------|
| `game_events`      | Administrative/informational: game start, status changes, side changes, at-bat tracking, pitch recording, strikeouts, outs, walks |
| `runner_movements` | Every physical base movement: hit advancement (auto or override), walk forced advancement, stolen base                            |

`runner_movements` rows are linked to a PA via `pa_seq` (for hit/walk) or standalone via `pa_seq = NULL` (for steal and
future events like wild pitch).

### Replay order

Resume reconstructs state from three sources in order:

1. `game_events` → log display only (admin events shown in scorelog)
2. `plate_appearances` → PA state (batting order, outs, score from hits/walks/outs)
3. `runner_movements` (standalone, `pa_seq IS NULL`) → interleaved with PAs by inning/half to restore base state for
   non-PA events (steals)

---

## 📦 Dependencies

| Crate        | Purpose                                |
|--------------|----------------------------------------|
| `rusqlite`   | SQLite bindings                        |
| `serde`      | Serialization framework                |
| `serde_json` | JSON for PA sequences and outcome data |
| `ratatui`    | Terminal UI                            |
| `crossterm`  | Cross-platform terminal control        |
| `chrono`     | Date handling                          |
| `uuid`       | Game ID generation                     |

---

## 🗄️ Database locations

| Platform | Path                                          |
|----------|-----------------------------------------------|
| Windows  | `%LOCALAPPDATA%\bs_scorer\baseball_scorer.db` |
| macOS    | `$HOME/.bs_scorer/baseball_scorer.db`         |
| Linux    | `$HOME/.bs_scorer/baseball_scorer.db`         |

---

## 📈 Version history (major milestones)

| Version | Highlights                                                                                                                                                      |
|---------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| v0.9.2  | `runner_movements` rebuilt (migration v16); steal/hit/walk movements persisted per-runner; steal replay fixed; `game_events` scope clarified to admin/info only |
| v0.9.1  | Steal command (`<order> st <dest>`); Unicode panic fix; runner collision validation                                                                             |
| v0.9.0  | Module split (game_state, runner, session, scoring); runner override persistence (migration v15); DB queries moved to db/game_queries                           |
| v0.8.0  | Runner overrides by batting order; `Option<BatterOrder>` on bases                                                                                               |
| v0.7.7  | Refactor pass: dead types removed, strum removed, migration gap fixed                                                                                           |
| v0.7.x  | Compact PA persistence, deterministic resume, TUI scoreboard                                                                                                    |
| v0.6.x  | Pitch-by-pitch tracking, pitch count, strike/ball logic                                                                                                         |
| v0.4.x  | Pre-game lineup editing, GameStatus enum                                                                                                                        |
| v0.3.x  | Player management, CSV/JSON import-export                                                                                                                       |
| v0.2.x  | SQLite persistence, menu system, schema migrations                                                                                                              |
| v0.1.0  | Initial CLI scoring                                                                                                                                             |

---

**Built with Rust 🦀 — Play Ball! ⚾**
