# 🎯 BS Scoring v0.8.0 – Project Structure

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
    │   ├── play_ball.rs        # GameState, BatterOrder, RunnerDest, RunnerOverride
    │   ├── plate_appearance.rs # PlateAppearance, PlateAppearanceOutcome, …
    │   ├── events.rs           # DomainEvent, UiEvent, PersistedEvent
    │   ├── field_zone.rs       # FieldZone (LF, CF, RF, …)
    │   └── player_traits.rs    # PitchHand, BatSide
    │
    ├── commands/               # Input parsing
    │   ├── types.rs            # EngineCommand enum
    │   └── engine_parser.rs    # parse_engine_commands() — handles "6 h, 5 2b" syntax
    │
    ├── core/                   # Game logic
    │   ├── menu.rs             # COBOL-style menu system
    │   ├── parser.rs           # Scoring notation parser (legacy / reference)
    │   ├── play_ball.rs        # Play Ball menu entry point
    │   ├── play_ball_apply.rs  # EngineCommand → ApplyResult (stateless transform)
    │   └── play_ball_reducer.rs# DomainEvent / PA → GameState mutations
    │                           #   apply_hit_with_overrides() lives here
    │
    ├── engine/
    │   └── play_ball.rs        # Main game loop: I/O, DB persistence, state drive
    │
    ├── db/                     # SQLite persistence layer
    │   ├── database.rs         # Connection management
    │   ├── migrations.rs       # Schema versioning (v1–v14)
    │   ├── plate_appearances.rs# plate_appearances_compact CRUD
    │   ├── game_events.rs      # game_events log CRUD
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
┌────────────┐  ┌─────────────────┐  ┌────────────────┐
│ commands/  │  │     core/       │  │      db/       │
│            │  │                 │  │                │
│ engine_    │  │ play_ball_      │  │ plate_         │
│ parser.rs  │  │ apply.rs        │  │ appearances.rs │
│            │  │                 │  │ game_events.rs │
│ "6 h, 5 2b"│  │ play_ball_      │  │ at_bat_draft.rs│
│  → Single{ │  │ reducer.rs      │  │                │
│  overrides}│  │                 │  │ SQLite (v14)   │
└────────────┘  │ apply_hit_with_ │  └────────────────┘
                │ overrides()     │
                └────────┬────────┘
                         │
                         ▼
               ┌──────────────────┐
               │   models/        │
               │                  │
               │ GameState        │
               │ on_1b/2b/3b:     │
               │ Option<BatterOrd>│
               │                  │
               │ RunnerOverride   │
               │ PlateAppearance  │
               └──────────────────┘
```

---

## 🔑 Key design decisions

### Runner identity on bases (`Option<BatterOrder>`)

From v0.8.0, `GameState.on_1b/on_2b/on_3b` are `Option<BatterOrder>` instead of `bool`.
The engine now knows *who* is on each base (by batting-order slot), not just *whether*
a base is occupied. This is what enables explicit runner overrides.

The UI still displays bases as occupied/empty (the diamond shows `◆` / `◇`).

### Runner override flow

```
Input: "6 h, 5 2b"
  └─ parse_engine_commands()
       └─ Single { zone: None, runner_overrides: [{ order: 5, dest: Second }] }
            └─ apply_hit_command()
                 └─ PlateAppearance { ..., runner_overrides: [...] }
                      └─ apply_live_plate_appearance()
                           └─ apply_hit_with_overrides(state, batter=6, bases=1, overrides)
                                • runner on 2B was order=5 → override: stays on 2B
                                • batter #6 → automatic: goes to 1B
```

### Two advancement paths

| Path | Function | Used for |
|------|----------|----------|
| With overrides | `apply_hit_with_overrides()` | Live scoring (v0.8.0+) |
| Automatic only | `apply_hit_advancement()` | PA replay from legacy rows |

### Compact plate appearance (resume model)

Every completed PA is persisted as a single row in `plate_appearances_compact`.
On resume, the game state is rebuilt by replaying these rows in order —
no pitch-by-pitch event log needed. Runner overrides are serialized into
the PA row so replay is faithful to the original scoring.

### DB schema version

Current: **v14** (migration chain v1–v13 + v14; v13 is a documented no-op).

---

## 📦 Dependencies

| Crate          | Purpose                                  |
|----------------|------------------------------------------|
| `rusqlite`     | SQLite bindings                          |
| `serde`        | Serialization framework                  |
| `serde_json`   | JSON for PA sequences and outcome data   |
| `ratatui`      | Terminal UI                              |
| `crossterm`    | Cross-platform terminal control          |
| `chrono`       | Date handling                            |
| `uuid`         | Game ID generation                       |

---

## 🗄️ Database locations

| Platform | Path                                               |
|----------|----------------------------------------------------|
| Windows  | `%LOCALAPPDATA%\bs_scorer\baseball_scorer.db`      |
| macOS    | `$HOME/.bs_scorer/baseball_scorer.db`              |
| Linux    | `$HOME/.bs_scorer/baseball_scorer.db`              |

---

## 📈 Version history (major milestones)

| Version | Highlights |
|---------|-----------|
| v0.8.0  | Runner overrides by batting order; `Option<BatterOrder>` on bases |
| v0.7.7  | Refactor pass: dead types removed, strum removed, migration gap fixed |
| v0.7.x  | Compact PA persistence, deterministic resume, TUI scoreboard |
| v0.6.x  | Pitch-by-pitch tracking, pitch count, strike/ball logic |
| v0.4.x  | Pre-game lineup editing, GameStatus enum |
| v0.3.x  | Player management, CSV/JSON import-export |
| v0.2.x  | SQLite persistence, menu system, schema migrations |
| v0.1.0  | Initial CLI scoring |

---

**Built with Rust 🦀 — Play Ball! ⚾**
