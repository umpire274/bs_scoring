# 🎯 BS Scoring v0.12.0 — Project Structure

This document describes the project layout and the main architectural boundaries of BS Scoring.

## Directory Layout

```text
bs_scoring/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── CHANGELOG.md
├── SCORING_GUIDE.md
├── STRUCTURE.md
├── RELEASE.md
└── src/
    ├── lib.rs
    ├── main.rs
    │
    ├── models/
    │   ├── types.rs
    │   ├── game_state.rs
    │   ├── runner.rs
    │   ├── session.rs
    │   ├── plate_appearance.rs
    │   ├── events.rs
    │   ├── field_zone.rs
    │   ├── player_traits.rs
    │   ├── umpires.rs
    │   └── scoring/
    │       ├── mod.rs
    │       └── types.rs
    │
    ├── engine/
    │   ├── commands/
    │   │   ├── kind.rs
    │   │   ├── types.rs
    │   │   ├── errors.rs
    │   │   ├── grammar/
    │   │   │   ├── mod.rs
    │   │   │   ├── tokens.rs
    │   │   │   └── segment.rs
    │   │   ├── validator.rs
    │   │   └── parser.rs
    │   ├── scoring/
    │   │   ├── batter_outs.rs
    │   │   └── resolve_batter_out.rs
    │   ├── notation.rs
    │   ├── runners.rs
    │   ├── apply.rs
    │   ├── reducer.rs
    │   ├── helpers.rs
    │   └── play_ball.rs
    │
    ├── db/
    │   ├── database.rs
    │   ├── migrations.rs
    │   ├── config.rs
    │   ├── game_queries.rs
    │   ├── plate_appearances.rs
    │   ├── runner_movements.rs
    │   ├── game_events.rs
    │   ├── at_bat_draft.rs
    │   ├── league.rs
    │   ├── team.rs
    │   ├── player.rs
    │   └── umpire.rs
    │
    ├── ui/
    │   ├── events.rs
    │   ├── context.rs
    │   ├── factory.rs
    │   ├── app.rs
    │   ├── cli_impl.rs
    │   └── tui.rs
    │
    ├── cli/
    │   ├── menu.rs
    │   └── screens/
    │       ├── main_menu.rs
    │       ├── game.rs
    │       ├── play_ball.rs
    │       ├── leagues.rs
    │       ├── team.rs
    │       ├── players.rs
    │       ├── statistics.rs
    │       ├── db.rs
    │       ├── export.rs
    │       └── umpire_supervisor.rs
    │
    └── utils/
        ├── boot.rs
        ├── term.rs
        ├── normalize.rs
        └── time.rs
```

## Architectural Boundaries

### `models/`

Pure domain and data types.

Important files:

- `types.rs` — game-level types such as `HalfInning`, `Pitch`, `PitchCount`, `GameStatus`, `Score`, and lineup defensive `Position`.
- `player_traits.rs` — player roster traits:
  - `BatSide`
  - `ThrowHand`
  - `PlayerFieldPosition`
  - `parse_bat_throw()`
  - `parse_player_positions()`
- `game_state.rs` — in-memory game state.
- `runner.rs` — base runner destinations and overrides.
- `plate_appearance.rs` — plate-appearance records and replay sequence types.
- `events.rs` — domain events emitted by the engine.

### `engine/`

Game logic and command processing.

The engine owns:

- command parsing;
- semantic validation;
- command application;
- runner movement logic;
- replay/reducer logic;
- Play Ball state driving.

The command pipeline is split into:

| Stage | Module | Responsibility |
|---|---|---|
| Lexical/syntactic | `engine/commands/grammar` | Parse raw text into segments |
| Semantic | `engine/commands/validator.rs` | Validate against `GameState` |
| Facade | `engine/commands/parser.rs` | Public parser entry point |
| Application | `engine/apply.rs` | Apply `EngineCommand` values |
| Replay | `engine/reducer.rs` | Rebuild `GameState` from persisted data |

### `db/`

SQLite persistence layer.

Important files:

- `database.rs` — connection setup, WAL, PRAGMAs.
- `migrations.rs` — schema versioning and automatic data migrations.
- `config.rs` — cross-platform database path.
- `player.rs` — Player CRUD.
- `team.rs` — Team CRUD.
- `league.rs` — League CRUD.
- `game_queries.rs` — playable games and lineup gate checks.
- `plate_appearances.rs` and `runner_movements.rs` — scoring persistence.
- `at_bat_draft.rs` — in-progress plate appearance resume support.
- `umpire.rs` — umpires and evaluations.

## Player Model in v0.12.0

The v0.12.0 player model separates roster data from lineup/scoring data.

### Player roster positions

`players.position` is a normalized comma-separated string of roster capabilities.

Examples:

```text
P
P,C,IF
IF,OF,DH
LF,CF,RF
```

Allowed values:

```text
P,C,1B,2B,3B,SS,LF,CF,RF,IF,OF,DH
```

This is independent from lineup defensive positions, which remain numeric `1`–`9` plus `DH` where applicable.

### Batting and throwing

The database stores batting and throwing separately:

```text
bat   = R | L | S
throw = R | L | S
```

Import/export uses the WBSC-style combined field:

```text
bat_throw = BAT/THROW
```

Examples:

```text
R/R
L/R
S/L
```

### Jersey numbers

Each player has:

```text
number       # home jersey number
away_number  # away jersey number
```

Both are unique within the same team.

## Database Storage

### Linux

```text
$XDG_DATA_HOME/bs_scoring
```

or:

```text
~/.local/share/bs_scoring
```

### macOS

```text
~/Library/Application Support/bs_scoring
```

### Windows

```text
%LOCALAPPDATA%\bs_scoring
```

with fallback to:

```text
%APPDATA%\bs_scoring
```

Legacy Linux data in `~/.bs_scorer` is migrated automatically.

## Migration Responsibilities

`db/migrations.rs` owns schema and data migrations, including:

- legacy player `pitch` column migration to `throw`;
- conversion of `RHP/LHP/SHP` to `R/L/S`;
- conversion of numeric player positions to roster-position codes;
- database filename migration from `baseball_scorer*.db` to `bs_scoring*.db`;
- enforcement of home/away jersey uniqueness.

## CLI Layer

`cli/screens/` contains menu workflows.

Important screens:

- `players.rs` — player CRUD, import/export, templates.
- `game.rs` — game creation and lineup workflows.
- `play_ball.rs` — entry into live game mode.
- `team.rs` and `leagues.rs` — organization management.
- `umpire_supervisor.rs` — umpire evaluation workflows.

## UI Layer

`ui/` contains display abstractions and backends:

- `cli_impl.rs` — plain text output.
- `tui.rs` — Ratatui scoreboard and live game UI.
- `events.rs` — UI event types.
- `factory.rs` — backend selection.

## Development Checks

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```
