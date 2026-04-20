# ⚾ Baseball Scorer — v0.11.0-alpha2

A comprehensive baseball and softball scoring TUI application with SQLite
persistence, pitch-by-pitch tracking, runner advancement overrides, steal
support, deterministic game resume, and umpire supervisor tools.

## 🆕 What's New in v0.11.0

`v0.11.0` is being delivered incrementally as a series of alphas. No
`v0.11.0` final has been cut yet; the two alphas below represent the work
landed so far.

### v0.11.0-alpha2 — Grammar refactor

- ✅ **Regex-assisted command parser** — the scoring-command parser has been
  rebuilt on top of a formal grammar. Lexical recognition uses `regex` 1.11;
  segment parsing is a small handwritten recursive descent.
- ✅ **Stateless-then-stateful pipeline** — parsing splits into two clear
  stages: a syntactic pass that produces `Vec<Segment>` with no access to
  the game state, and a validator that applies state-dependent rules
  (batter-slot coherence, runner presence, infield-fly preconditions,
  too-many-outs, mutual exclusions).
- ✅ **Subject-always grammar** — the batting-order subject is mandatory on
  every action segment, with one documented exception: verbs whose shape
  cannot be confused with a lone digit (hit verbs, multi-character
  batter-outs, fielder's choice) may omit the subject, in which case it
  defaults to the current batter.
- ✅ **Order-independent segments** — a play like `5 l6, 3 64, 4 43` and its
  permutations (`3 64, 5 l6, 4 43`, `4 43, 5 l6, 3 64`) all produce the
  same triple play. The parser no longer privileges the first token.
- ✅ **Diagnostic errors with segment index** — every error is surfaced as
  `error at segment N: '<text>': <reason>`. Multiple errors in the same
  line are accumulated and reported together rather than short-circuiting
  on the first.
- ✅ **130 new unit tests** covering token classification, segment parsing,
  semantic validation, order invariance, and diagnostic quality.

### v0.11.0-alpha1 — Structural refactor

- ✅ **`core/` absorbed into `engine/`** — game logic now lives in a single
  coherent subtree; `runner_logic.rs`, `play_ball_apply.rs`,
  `play_ball_reducer.rs`, `parser.rs`, and `scoring/` moved under
  `src/engine/`.
- ✅ **Top-level `commands/` removed** — moved under
  `src/engine/commands/`, eliminating the ambiguity with
  `src/cli/commands/`.
- ✅ **`cli/commands/` renamed to `cli/screens/`** — these files are
  user-flow screens, not engine commands; the new name removes the clash.
- ✅ **Anti-homonym renames** — `utils/cli.rs` → `utils/term.rs` and
  `ui/cli.rs` → `ui/cli_impl.rs` remove two module-name collisions that
  made `use` sites hard to read.
- ✅ **Deprecated shims removed** — `core/play_ball.rs` and
  `models/play_ball.rs` (compatibility re-exports since v0.8.1) deleted.
- ✅ **Public API preserved** — the top-level re-exports from
  `bs_scoring::*` (`Database`, `Menu`, `GameState`, `Player`, `Team`,
  `League`, scoring types, …) are unchanged.

### Recent Versions

| Version       | Highlights                                                                       |
|---------------|----------------------------------------------------------------------------------|
| 0.11.0-alpha2 | Grammar refactor: regex-assisted parser, subject-always rule, error accumulation |
| 0.11.0-alpha1 | Structural refactor of `src/`: engine/ absorbs core/, cli/screens/, renames      |
| 0.10.6        | Defensive plays (composite), unassisted out, fielder's choice, TUI history       |
| 0.10.5-bugfix | Nullable `game_time` handling, summary-table cleanup                             |
| 0.10.5        | Umpire evaluation summary enriched with date / time / venue                      |
| 0.10.4        | Umpire reports CSV/JSON export, `GameInfo` refactor, interactive history         |
| 0.10.3        | Umpire history helper decomposition, interactive report selection                |
| 0.10.2        | Batter-out commands: ground/fly/foul-fly/line-out/infield-fly                    |
| 0.10.1        | Umpire Supervisor module, crew assignments, evaluations, career stats            |
| 0.10.0        | Architecture refactor, DB optimisation, unified runner logic                     |
| 0.9.3         | Scrollable Help panel, Tab focus system, shortcuts bar                           |
| 0.9.2         | `runner_movements` rebuilt (v16); steal replay fixed                             |
| 0.9.1         | Steal command, Unicode panic fix, override collision validation                  |
| 0.9.0         | Module split, runner override persistence (v15)                                  |

## 📁 Project Structure

```
bs_scoring/
├── Cargo.toml                # Package configuration ([lib] + [[bin]])
├── README.md
├── CHANGELOG.md
├── SCORING_GUIDE.md          # Scorer command reference
├── STRUCTURE.md              # Detailed architecture
├── RELEASE.md                # Release process
└── src/
    ├── lib.rs                # Library entry point / public re-exports
    ├── main.rs               # Binary entry point
    │
    ├── models/               # Pure data types — no I/O, no DB, no UI
    │   ├── types.rs          # HalfInning, Pitch, GameStatus, Score, Position
    │   ├── game_state.rs     # GameState, BatterOrder, PitchStats
    │   ├── runner.rs         # RunnerDest, RunnerOverride
    │   ├── session.rs        # PlayBallGameContext, PlayBallGate, LineupSide
    │   ├── plate_appearance.rs # PlateAppearance, PlateAppearanceOutcome
    │   ├── events.rs         # DomainEvent, PersistedEvent
    │   ├── field_zone.rs     # FieldZone (LF, CF, RF, …)
    │   ├── player_traits.rs  # PitchHand, BatSide
    │   ├── umpires.rs        # Umpire types, evaluation rows
    │   └── scoring/          # Full scoring-notation value types
    │       └── types.rs      # HitType, OutType, Walk, AdvancedPlay, …
    │
    ├── engine/               # Game logic — no I/O, no UI
    │   ├── commands/         # Scoring-command pipeline [v0.11.0]
    │   │   ├── types.rs      # EngineCommand enum
    │   │   ├── errors.rs     # ParseError / ValidationError / CommandError
    │   │   ├── grammar/      # Stateless syntactic layer
    │   │   │   ├── tokens.rs # Regex lazy + TokenKind classifier
    │   │   │   └── segment.rs# Segment + parse_segment + parse_line
    │   │   ├── validator.rs  # State-aware validation + coalescing
    │   │   └── parser.rs     # Facade: parse_engine_commands(line, state)
    │   ├── scoring/          # Batter-out / defensive-play helpers
    │   │   ├── batter_outs.rs
    │   │   └── resolve_batter_out.rs
    │   ├── notation.rs       # Full scoring-notation parser (reference)
    │   ├── runners.rs        # Unified runner-movement logic
    │   ├── apply.rs          # EngineCommand → ApplyResult
    │   ├── reducer.rs        # DomainEvent / PA → GameState mutations
    │   ├── helpers.rs        # Shared internal helpers
    │   └── play_ball.rs      # Main game loop: I/O, DB persistence, drive
    │
    ├── db/                   # SQLite persistence layer
    │   ├── database.rs       # Connection management + WAL + PRAGMAs
    │   ├── migrations.rs     # Schema versioning
    │   ├── game_queries.rs   # Playable games, lineup gate-check, status
    │   ├── plate_appearances.rs # plate_appearances CRUD
    │   ├── runner_movements.rs  # runner_movements CRUD
    │   ├── game_events.rs    # game_events log (admin/info only)
    │   ├── at_bat_draft.rs   # In-progress PA draft (resume support)
    │   ├── league.rs         # League CRUD
    │   ├── team.rs           # Team CRUD
    │   ├── player.rs         # Player CRUD
    │   ├── umpire.rs         # Umpire + evaluation CRUD
    │   └── config.rs         # Cross-platform DB path
    │
    ├── ui/                   # UI abstractions (Ui trait + backends)
    │   ├── tui.rs            # Terminal UI (ratatui) — scoreboard, log, help
    │   ├── cli_impl.rs       # Plain-text CLI backend
    │   ├── events.rs         # UiEvent definitions
    │   ├── context.rs        # PlayBallUiContext
    │   ├── factory.rs        # UI backend selection
    │   └── app.rs            # App-level UI state
    │
    ├── cli/                  # User-facing command-line layer
    │   ├── menu.rs           # Menu-choice enums
    │   └── screens/          # Menu-entry handlers
    │       ├── main_menu.rs
    │       ├── game.rs       # Game creation, listing, editing, lineups
    │       ├── play_ball.rs  # Play Ball session launcher
    │       ├── players.rs    # Player management + import/export
    │       ├── team.rs       # Team management
    │       ├── leagues.rs    # League management
    │       ├── statistics.rs # Statistics display
    │       ├── umpire_supervisor.rs # Umpire Supervisor module
    │       ├── export.rs     # Export helpers
    │       └── db.rs         # Database management utilities
    │
    └── utils/
        ├── boot.rs           # App initialization + banner
        ├── term.rs           # Terminal helpers, CliSelectable trait
        ├── normalize.rs      # slugify / filename normalization
        └── time.rs           # Export-timestamp helpers
```

## 🚀 Installation

### Prerequisites

- Rust 1.86+ (for edition 2024 and the current `ratatui` major) — install
  from [rustup.rs](https://rustup.rs/)

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
2. Initializes SQLite database with WAL mode
3. Runs all migrations to create schema
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

### Scorer Commands (summary)

Under v0.11.0-alpha2 the batting-order subject is mandatory on action
segments, with one exception: on verbs whose shape cannot be confused with
a lone digit (hits, multi-character batter-outs, fielder's choice) the
subject may be omitted and defaults to the current batter.

| Command          | Description                                         |
|------------------|-----------------------------------------------------|
| `playball`       | Start the game                                      |
| `b`              | Ball                                                |
| `k`              | Called strike                                       |
| `s`              | Swinging strike                                     |
| `f`              | Foul                                                |
| `fl`             | Foul bunt (counts as strike even with 2 strikes)    |
| `5 h [zone]`     | Batter #5 singles (subject required; zone optional) |
| `5 2h [zone]`    | Batter #5 doubles                                   |
| `5 3h [zone]`    | Batter #5 triples                                   |
| `5 hr [zone]`    | Batter #5 home runs                                 |
| `5 h lf, 3 sc`   | Batter #5 singles to LF; runner #3 scores           |
| `5 63`           | Batter #5 ground out 6-3                            |
| `5 l6`           | Batter #5 line out to SS                            |
| `5 o6 1b`        | Batter #5 safe at 1B on fielder's choice by SS      |
| `4 46, 5 o4 1b`  | Runner #4 out 4-6, batter #5 safe at 1B on FC       |
| `5 st 2b`        | Runner #5 steals second                             |
| `b, 5 st 2b`     | Ball pitched, runner #5 steals second               |
| `regular`        | End game (regulation)                               |
| `exit` or `quit` | Return to menu                                      |

For the complete grammar, including every edge case and the list of
diagnostic errors the parser will emit, see
[SCORING_GUIDE.md](SCORING_GUIDE.md).

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
  5. 📤 Export Umpire Reports   — CSV / JSON export
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

| Version          | Key Features                                                                    |
|------------------|---------------------------------------------------------------------------------|
| 0.11.0-alpha2    | Grammar refactor, regex-assisted parser, order-independent segments, 130+ tests |
| 0.11.0-alpha1    | Structural refactor: engine/ absorbs core/, cli/screens/, anti-homonym renames  |
| 0.10.6           | Composite defensive plays, unassisted out, fielder's choice, TUI history        |
| 0.10.5 / -bugfix | Umpire summary with date/time/venue, nullable game_time fix                     |
| 0.10.4           | Umpire reports CSV/JSON export, `GameInfo` struct, interactive history UX       |
| 0.10.3           | Umpire history helper decomposition, interactive report selection               |
| 0.10.2           | Batter-out commands: ground / fly / foul-fly / line-out / infield-fly           |
| 0.10.1           | Umpire Supervisor: registry, crew assignment, evaluations, career history       |
| 0.10.0           | Unified runner logic, WAL mode, migration-only schema, model helpers            |
| 0.9.x            | TUI Help/focus system, steal command, runner_movements v16, module split        |
| 0.8.0            | Runner overrides by batting order, `Option<BatterOrder>` on bases               |
| 0.7.x            | Compact PA persistence, deterministic resume, TUI scoreboard                    |
| 0.6.x            | Pitch-by-pitch tracking, pitch count, strike/ball logic                         |
| 0.4.x            | Pre-game lineup editing, DH support, `GameStatus` enum                          |
| 0.3.x            | Player management, CSV/JSON import-export                                       |
| 0.2.x            | SQLite persistence, menu system, schema migrations                              |
| 0.1.0            | Initial CLI scoring                                                             |

## 🚀 Roadmap

- Mid-game substitutions (pinch hitters/runners, defensive replacements)
- Caught stealing, errors, sacrifice plays
- Player statistics (AVG, ERA, OPS, WHIP)
- League standings and season summaries
- Game export/import

## 📚 Documentation

- [CHANGELOG.md](CHANGELOG.md) — Complete version history
- [SCORING_GUIDE.md](SCORING_GUIDE.md) — Command grammar and diagnostics
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

**Version:** 0.11.0-alpha2
**Schema:** v18
**Edition:** Rust 2024
**Author:** Alessandro Maestri

**Play Ball! ⚾**
