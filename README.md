# ⚾ Baseball Scorer — v0.10.1

A comprehensive baseball and softball scoring TUI application with SQLite
persistence, pitch-by-pitch tracking, runner advancement overrides, steal
support, deterministic game resume, and umpire supervisor tools.

## 🆕 What's New in v0.10.1

- ✅ **Umpire Supervisor module** — new top-level menu with umpire registry (CRUD), game crew assignment (configurable
  2/3/4/6-man crews), post-game evaluation report cards (8 scored categories, 1–10 scale), and career history/statistics
- ✅ **Umpire ↔ League association** — N:N relationship; each umpire can work in multiple leagues, selectable at creation
  and editable later
- ✅ **Game picker for umpire operations** — assign and evaluate functions use the same game selection list as Play
  Ball (excludes finished/cancelled/forfeited games)
- ✅ **Schema v18** — three new tables: `umpires`, `game_umpires`, `umpire_evaluations`, plus `umpire_leagues` junction
  table

### Recent Versions

| Version | Highlights                                                            |
|---------|-----------------------------------------------------------------------|
| 0.10.1  | Umpire Supervisor module, crew assignments, evaluations, career stats |
| 0.10.0  | Architecture refactor, DB optimisation, unified runner logic          |
| 0.9.3   | Scrollable Help panel, Tab focus system, shortcuts bar                |
| 0.9.2   | `runner_movements` rebuilt (v16); steal replay fixed                  |
| 0.9.1   | Steal command, Unicode panic fix, override collision validation       |
| 0.9.0   | Module split, runner override persistence (v15)                       |

## 📁 Project Structure

```
bs_scoring/
├── Cargo.toml                # Package configuration ([lib] + [[bin]])
├── README.md
├── CHANGELOG.md
├── SCORING_GUIDE.md          # Scorer command reference
├── STRUCTURE.md              # Detailed architecture
└── src/
    ├── lib.rs                # Library entry point / public re-exports
    ├── main.rs               # Binary entry point
    │
    ├── models/               # Pure data types — no I/O, no DB
    │   ├── types.rs          # HalfInning, Pitch, GameStatus, Score, Position
    │   ├── game_state.rs     # GameState, BatterOrder, PitchStats
    │   ├── runner.rs         # RunnerDest, RunnerOverride
    │   ├── session.rs        # PlayBallGameContext, PlayBallGate, LineupSide
    │   ├── plate_appearance.rs # PlateAppearance, PlateAppearanceOutcome
    │   ├── events.rs         # DomainEvent, PersistedEvent
    │   ├── field_zone.rs     # FieldZone (LF, CF, RF, …)
    │   ├── player_traits.rs  # PitchHand, BatSide
    │   ├── play_ball.rs      # Compatibility re-export shim
    │   └── scoring/          # Full scoring notation types (parser only)
    │       └── types.rs      # HitType, OutType, Walk, AdvancedPlay, …
    │
    ├── commands/             # Input parsing
    │   ├── types.rs          # EngineCommand enum
    │   └── engine_parser.rs  # parse_engine_commands()
    │
    ├── core/                 # Game logic
    │   ├── runner_logic.rs   # ★ Unified runner movement logic (NEW v0.10.0)
    │   ├── play_ball_apply.rs # EngineCommand → ApplyResult
    │   ├── play_ball_reducer.rs # DomainEvent / PA → GameState mutations
    │   ├── menu.rs           # Menu system
    │   ├── parser.rs         # Scoring notation parser (reference)
    │   └── play_ball.rs      # Deprecated re-export shim
    │
    ├── engine/
    │   └── play_ball.rs      # Main game loop: I/O, DB persistence, state drive
    │
    ├── db/                   # SQLite persistence layer
    │   ├── database.rs       # Connection management + WAL + PRAGMAs
    │   ├── migrations.rs     # Schema versioning (v1–v16)
    │   ├── game_queries.rs   # Playable games, lineup gate-check, status
    │   ├── plate_appearances.rs # plate_appearances CRUD
    │   ├── runner_movements.rs  # runner_movements CRUD
    │   ├── game_events.rs    # game_events log (admin/info only)
    │   ├── at_bat_draft.rs   # In-progress PA draft (resume support)
    │   ├── league.rs         # League CRUD
    │   ├── team.rs           # Team CRUD
    │   ├── player.rs         # Player CRUD
    │   └── config.rs         # Cross-platform DB path
    │
    ├── ui/                   # UI abstractions
    │   ├── tui.rs            # Terminal UI (ratatui) — scoreboard, log, help
    │   ├── cli.rs            # Plain-text CLI fallback
    │   ├── events.rs         # UiEvent definitions
    │   ├── context.rs        # PlayBallUiContext
    │   ├── factory.rs        # UI backend selection
    │   └── app.rs            # App-level UI state
    │
    ├── cli/                  # CLI command handlers (menu actions)
    │   └── commands/
    │       ├── main_menu.rs
    │       ├── game.rs       # Game creation, listing, editing, lineups
    │       ├── play_ball.rs  # Play Ball session launcher
    │       ├── players.rs    # Player management + import/export
    │       ├── team.rs       # Team management
    │       ├── leagues.rs    # League management
    │       ├── statistics.rs # Statistics display
    │       ├── umpire_supervisor.rs # Umpire Supervisor module
    │       └── db.rs         # Database management utilities
    │
    └── utils/
        ├── boot.rs           # App initialization + banner
        └── cli.rs            # Input helpers, CliSelectable trait
```

## 🚀 Installation

### Prerequisites

- Rust 1.85+ (for edition 2024) — install from [rustup.rs](https://rustup.rs/)

### Compilation

```bash
cd bs_scoring
cargo build --release
```

Executable: `target/release/bs_scoring` (or `bs_scoring.exe` on Windows)

## 📖 Usage

```bash
cargo run
# or
./target/release/bs_scoring
```

### First Run

1. Creates platform-specific database directory
2. Initialises SQLite database with WAL mode
3. Runs all migrations (v1→v18) to create schema
4. Displays database location and boot status

**Database Locations:**

| Platform    | Path                                          |
|-------------|-----------------------------------------------|
| Windows     | `%LOCALAPPDATA%\bs_scorer\baseball_scorer.db` |
| macOS/Linux | `$HOME/.bs_scorer/baseball_scorer.db`         |

## 🎮 Main Menu

```
╔════════════════════════════════════════════╗
║  ⚾  BASEBALL/SOFTBALL SCORER - MAIN MENU  ║
╚════════════════════════════════════════════╝

  1. 🎮 Game Management
  2. 🏆 Leagues Management
  3. ⚾ Teams Management
  4. 👥 Player Management
  5. 📊 Statistics
  6. 🧑‍⚖️ Umpire Supervisor
  7. 💾 Manage DB

  0. 🚪 Exit
```

## ⚾ Play Ball — Live Scoring

The TUI scoring interface provides:

- **Scoreboard** with line score, base diamond, count, outs, pitcher stats
- **Scrollable log** with replay on resume
- **Help panel** with command reference
- **Tab focus** between Log and Help panels

### Scorer Commands

| Command     | Description                         |
|-------------|-------------------------------------|
| `playball`  | Start the game                      |
| `b`         | Ball                                |
| `k`         | Called strike                       |
| `s`         | Swinging strike                     |
| `f`         | Foul                                |
| `fl`        | Foul bunt (counts as strike 3)      |
| `h [zone]`  | Single (optional field zone)        |
| `2h [zone]` | Double                              |
| `3h [zone]` | Triple                              |
| `hr [zone]` | Home run                            |
| `6 h, 5 2b` | Single by #6; runner #5 stays on 2B |
| `6 st 2b`   | Runner #6 steals second             |
| `regular`   | End game (regulation)               |
| `exit`      | Return to menu                      |

**Field zones:** `LL LF LC CF RC RF RL GLL LS MI RS GRL`

## 🗄️ Database Schema (v18)

### Core Tables

| Table                | Purpose                                                   |
|----------------------|-----------------------------------------------------------|
| `meta`               | App metadata and schema version                           |
| `leagues`            | League/championship information                           |
| `teams`              | Team data with league association                         |
| `players`            | Player roster (first/last name, position, handedness)     |
| `games`              | Game metadata (teams, venue, date, DH flags, status)      |
| `game_lineups`       | Starting lineups for both teams                           |
| `plate_appearances`  | Compact PA log (1 row per completed PA)                   |
| `runner_movements`   | Per-runner base movements (hit, walk, steal)              |
| `game_events`        | Administrative event log (start, status, side changes)    |
| `at_bat_draft`       | In-progress at-bat for resume support                     |
| `umpires`            | Umpire registry (name, license, level, contact)           |
| `game_umpires`       | Umpire crew assignments per game (HP, 1B, 2B, 3B, LF, RF) |
| `umpire_evaluations` | Post-game umpire report cards (8 categories, 1–10)        |
| `umpire_leagues`     | N:N umpire ↔ league association                           |
| `at_bats`            | Legacy detailed at-bat table                              |
| `pitches`            | Legacy pitch tracking table                               |

## 🧑‍⚖️ Umpire Supervisor

```
UMPIRE SUPERVISOR
  1. 👤 Manage Umpires          — CRUD + league association
  2. 📋 Assign Umpires to Game  — configurable crew (2/3/4/6)
  3. 📝 Evaluate Game           — report card per umpire
  4. 📊 Umpire History          — career stats & evaluations
```

### Evaluation Categories (1–10 scale)

| Category             | Applies to    |
|----------------------|---------------|
| Strike zone accuracy | HP only       |
| Safe/Out accuracy    | All positions |
| Positioning          | All positions |
| Timing               | All positions |
| Game management      | All positions |
| Professionalism      | All positions |
| Communication        | All positions |
| Hustle               | All positions |

## 💾 Database Management

```
DATABASE MANAGEMENT
  1. 📋 View DB Info
  2. 🔍 View DB Status
  3. 🔄 Run Migrations
  4. 💾 Backup Database
  5. 📥 Restore Database
  6. 🧹 Vacuum Database
  7. 🗑️  Clear All Data
  8. 📤 Export Game
```

## 📊 Features by Version

| Version | Key Features                                                              |
|---------|---------------------------------------------------------------------------|
| 0.10.1  | Umpire Supervisor: registry, crew assignment, evaluations, career history |
| 0.10.0  | Unified runner logic, WAL mode, migration-only schema, model helpers      |
| 0.9.x   | TUI Help/focus system, steal command, runner_movements v16, module split  |
| 0.8.0   | Runner overrides by batting order, `Option<BatterOrder>` on bases         |
| 0.7.x   | Compact PA persistence, deterministic resume, TUI scoreboard              |
| 0.6.x   | Pitch-by-pitch tracking, pitch count, strike/ball logic                   |
| 0.4.x   | Pre-game lineup editing, DH support, GameStatus enum                      |
| 0.3.x   | Player management, CSV/JSON import-export                                 |
| 0.2.x   | SQLite persistence, menu system, schema migrations                        |
| 0.1.0   | Initial CLI scoring                                                       |

## 🚀 Roadmap

- Mid-game substitutions (pinch hitters/runners, defensive replacements)
- Player statistics (AVG, ERA, OPS, WHIP)
- League standings and season summaries
- PDF scorecard generation
- Game export/import

## 📚 Documentation

- [CHANGELOG.md](CHANGELOG.md) — Complete version history
- [SCORING_GUIDE.md](SCORING_GUIDE.md) — Official scoring symbols
- [STRUCTURE.md](STRUCTURE.md) — Project architecture
- [RELEASE.md](RELEASE.md) — Release process

## 🤝 Contributing

Contributions welcome! Fork → feature branch → PR.

## 📄 License

MIT License — Free to use for your games! ⚾

## 🔗 Links

- **Repository**: https://github.com/umpire274/bs_scoring
- **Issues**: https://github.com/umpire274/bs_scoring/issues
- **Releases**: https://github.com/umpire274/bs_scoring/releases

---

**Version:** 0.10.1
**Schema:** v18
**Edition:** Rust 2024
**Author:** Alessandro Maestri

**Play Ball! ⚾**
