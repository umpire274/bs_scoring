# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.7] - 2026-03-12

### Changed (Refactoring / Internal)

- **`HalfInning` now lives exclusively in `models::types`** — removed the stale
  `pub(crate) use crate::HalfInning` re-export from `models::play_ball` that caused
  ambiguity. All modules now import it from one canonical location.
- **Removed legacy dead-code from `models/types.rs`** — deleted `Game`, `GameTeam`,
  `GamePlayer`, `BaseRunner`, and `RunnerAdvancement` structs that were marked
  `#[allow(dead_code)]` and had no callers. The live engine uses `models/plate_appearance.rs`
  and `models/play_ball.rs` exclusively.
- **`PlateAppearance` re-export fixed in `lib.rs`** — was incorrectly pointing to the
  legacy (dead) struct in `models::types`; now correctly re-exports the live
  `models::plate_appearance::PlateAppearance`.
- **`GameStatus` now implements `TryFrom<i64>` and `From<GameStatus> for i64`** —
  the existing `from_i64`/`to_i64` helpers are kept as thin wrappers for
  backward compatibility.
- **`Position::from_number` now handles `10 => DesignatedHitter`** — previously
  the DH case was only handled in `from_db_value`, causing inconsistency.
- **`find_order_for_batter` rewritten as a single SQL query** — replaced the
  O(n) loop (up to 9 sequential DB round-trips) with one direct lookup against
  `game_lineups`.
- **`ApplyResult` now implements `Default`** — internal `empty_result()` free
  function replaced by `ApplyResult::default()`.
- **`player_traits.rs` no longer depends on strum** — `PitchHand` and `BatSide`
  now provide `all()` static slice helpers; `Display` and `FromStr` are
  implemented by hand. The `strum` and `strum_macros` crates have been removed
  from `Cargo.toml`.
- **`utils/cli.rs` `CliSelectable` trait decoupled from strum** — replaced
  `IntoEnumIterator` bound with a new `all_variants() -> &'static [Self]`
  method on the trait.
- **Migration v13 added as a no-op placeholder** — closes the gap between v12
  and v14 in the migration chain, preventing confusion when auditing schema
  history.



### Added

- New live log message format for new at-bat:
    ```text
    <inning><↑/↓> <outs> At bat: <order>. <firstname> <lastname> (#<jersey> <position>)
    ```
  Example:

    ```text
    3↓ 1 out At bat: 5. Sam Garcia (#7 DH)
    ```

- Added support for **DH (Designated Hitter)** as defensive position.
- Added helper `Position::from_db_value()` to correctly parse position codes from DB.

### Changed

- Restored `batter_order` as **numeric (u8)** instead of `String`.
- Database schema updated: `plate_appearances.batter_order` is now stored as **INTEGER**.
- `defensive_position` is now treated as **TEXT** (`P`, `C`, `1B`, `2B`, `3B`, `SS`, `LF`, `CF`, `RF`, `DH`).
- Improved **live log readability** by aligning formatting with scoreboard information.
- Updated **replay log layout** to display batting order instead of sequential plate appearance id.
- Unified formatting of player information between scoreboard and log.

### Fixed

- Walk (BB) runner advancement logic.
- Correct forced advancement of runners.
- Correct scoring when bases are loaded.
- Fixed mismatch between **live state** and **replay reconstruction** for walk events.
- Fixed `apply_plate_appearance_core()` walk handling.
- Fixed pitcher pitch counting inconsistencies between live mode and replay.
- Fixed scoreboard base occupancy not updating correctly after BB.
- Fixed conversion errors when reading `DH` as defensive position.

### Internal

- Refactored lineup lookup helpers:
- `get_batter_by_order`
- `get_batter_order_and_position`
- Added `apply_walk_advancement()` helper to centralize forced runner movement logic.
- Reduced unnecessary `.clone()` calls on `Option<u8>`.
-

---

## [0.7.5] - 2026-03-11

### Added

- Added `batter_order` field to the `plate_appearances` table.
- Introduced `type BatterOrder = String` to support flexible batting order representations (future DH support).
- New utility: **Game Management → Utilities → Refactor Batter Order** to rebuild batting order for existing games.

### Changed

- Renamed database table `plate_appearances_compact` → `plate_appearances`.
- Refactored engine to use `BatterOrder` instead of numeric batting order types.
- Replay log now displays `batter_order` instead of internal sequence (`seq`).

### UI Improvements

- **Scoreboard layout improved**
    - Batter line now shows batting order and field position:
      ```
      <order>. <firstname> <lastname> (#<jersey> <position>)
      ```
    - Pitcher statistics aligned on the right side:
      ```
      (P <total_pitches>: <strikes>-<balls>)
      ```
    - Improved alignment and spacing of player information.

- **Replay log visualization redesigned**
    - Replay output now grouped by half-inning.
    - Batting order is displayed instead of internal PA sequence numbers.
    - Outs are shown only when they change, improving readability.
    - Log layout optimized for better scanning during replay.

### Fixed

- Fixed SQL syntax error in `append_plate_appearance()` INSERT statement.
- Fixed edge cases during game resume where batter information could be missing.
- Fixed pitch counting inconsistencies for hits (`H`, `2H`, `HR`) during replay reconstruction.

### Internal

- Introduced `plate_appearances` as the main persisted PA structure.
- Updated multiple modules (`engine`, `db`, `reducer`) to support the new `batter_order` model.

---

## [0.7.4] — 2026-03-10

### Added

- Added `pitch` field to players (`LHP`, `RHP`, `SHP`) to record pitching hand.
- Added `bat` field to players (`L`, `R`, `S`) to record batting side.
- Support for `pitch` and `bat` in:
    - player creation
    - player editing
    - CSV import/export
    - JSON import/export.

- Introduced `FieldZone` enum to represent field hit zones used by official scorers.
- Added support for recording hit zones (e.g. `1b ll`, `2b cf`, `hr rc`).
- Hit zones are persisted in the database and used during replay reconstruction.

### Improved

- Refactored CLI enum selection using reusable helpers.
- Simplified CLI logic for selecting enum values.
- Improved roster display to include pitching and batting handedness.

### Database

- Updated `players` table schema to include:
    - `pitch`
    - `bat`

### Internal

- Added `models/field_zone.rs`.
- Improved replay formatting compatibility for legacy pitch sequences.
- Minor CLI and formatting improvements.

---

## [0.7.3] - 2026-03-09

### Added

- Added a new **Help** panel in the right-side TUI layout, positioned below **Scoreboard**
- Added initial help content for:
    - **pitch commands**
    - **hit commands**
- Added **log navigation help** in the Help box:
    - `↑ / ↓`
    - `PgUp / PgDn`
    - `Home / End`

### Changed

- Refactored the right-side TUI layout to split the area into:
    - **Scoreboard**
    - **Help**
- Improved TUI usability by making the **Log** panel scrollable via keyboard
- Added viewport-aware scroll clamping for the log panel to prevent overscrolling

### Technical

- Reused and completed the existing log scroll infrastructure in `TuiUi`
- Added viewport-based scroll normalization during render
- Kept automatic scroll-to-bottom behavior when new log lines are appended

---

## [0.7.2] - 2026-03-09

### Added

- New hit commands for plate appearance outcomes:
    - `1B` – Single
    - `2B` – Double
    - `3B` – Triple
    - `HR` – Home Run
- Simplified runner advancement model:
    - runners advance automatically according to hit type
- Automatic scoring for hits and home runs
- Pitch sequence persistence now supports hit steps (`1B`, `2B`, `3B`, `HR`)
- New `PlateAppearanceStep` enum to represent pitch-by-pitch sequences including hits
- Dynamic inning-by-inning scoreboard linescore
- Hit totals (`H`) column in the scoreboard
- Linescore adapts automatically to extra innings

### Changed

- `PlateAppearance.pitches_sequence` now stores `Vec<PlateAppearanceStep>` instead of raw pitches
- Replay engine updated to parse and render hit steps inside pitch sequences
- Scoreboard layout expanded to support dynamic inning columns
- `apply_pitch` and `apply_hit_command` refactored to share sequence-building logic
- Linescore rendering logic refactored to support right-aligned `R/H/E` columns
- Scoreboard width expanded to support larger inning displays

### Fixed

- Correct pitch count when a hit command ends a plate appearance
- Replay log now correctly displays pitch sequences including hit steps
- Scoreboard rendering alignment issues when runs or hits exceed single digits

### Refactor

- Introduced `RheTotals` helper struct to simplify linescore rendering
- Reduced argument count in `render_linescore_row()` to satisfy Clippy constraints
- Replaced single-case `match` with `if` in run reconstruction logic
- Improved internal helper reuse for pitch sequence generation

### Internal

- Refactored scoreboard rendering pipeline in `tui.rs`
- Improved separation between scoreboard data extraction and rendering
- Cleaned up code to pass `cargo clippy -- -D warnings`

---

## [0.7.1] - 2026-03-06

### Changed

- Batting order is now fully deterministic on resume:
    - `*_next_batting_order` is reconstructed from `plate_appearances_compact`.
    - If an in-progress at-bat exists, the cursor is aligned using `at_bat_draft` (prevents repeating the same batter
      after restart).
- Removed all runtime and resume dependencies on persisted batting cursors.

### Database

- Migration v11: dropped obsolete `batting_cursors` table.

### Internal

- Removed cursor persistence (`load/upsert batting_cursors`) and related dead code.
- Simplified resume pipeline by keeping cursor reconstruction purely derived from persisted plate appearances + draft.

---

## [0.7.0] - 2026-03-05

### Added

- Deterministic reconstruction of game state from `plate_appearances_compact`.
- New replay mechanism that rebuilds `GameState` by applying plate appearances sequentially.
- Resume log now prints pitch sequence and outcome in scorer-friendly format:

    ```txt
  PA#5 1↑ [B, K, S, F, F, K] -> K
  PA#6 1↑ [B, B, B, B] -> BB
  PA#7 1↓ [K, S, B, X] -> In Play
    ```

### Changed

- Resume process no longer depends on `game_events` for rebuilding game state.
- Batting order progression is derived deterministically from plate appearances.
- Pitch counts for pitchers are reconstructed using `pitches_sequence`.
- Resume logic now correctly restores in-progress at-bats using `at_bat_draft`.
- Batting cursors are aligned on resume to avoid repeating the same batter after restart.

### Database

- `plate_appearances_compact` is now the authoritative source for gameplay reconstruction.

### Internal

- Introduced deterministic reducer logic for plate appearance replay.
- Improved resume handling to avoid repeating the same batter when resuming mid-at-bat.
- Simplified `play_ball` resume flow and removed redundant state restoration.
- Refactored `run_play_ball_engine()` resume flow into smaller helper functions.
- Removed duplicated draft and cursor restoration logic.
- Improved readability and maintainability of the resume pipeline.

---

## [0.6.9] - 2026-03-05

### Added

- Persisted pitch-by-pitch sequence for every plate appearance.
- New column `pitches_sequence` (JSON) in `plate_appearances_compact` storing `Vec<Pitch>`.

### Changed

- Resume log output now displays pitch sequence and final outcome in a compact scorer-friendly format.

  Example:
    ```txt
    PA#5 1↑ [B, K, S, F, F, K] -> K
    PA#6 1↑ [B, B, B, B] -> BB
    PA#7 1↓ [K, S, B, X] -> In Play
    ```

- Simplified plate appearance model:
- Removed redundant field `outs_before`.
- Field `outs_after` renamed to `outs`.

### Database

- Migration v10:
- Recreated `plate_appearances_compact` table without `outs_before`.
- Introduced `outs` column.
- Added `pitches_sequence` column for pitch history.
- Rebuilt index `idx_pa_compact_game_seq`.

### Internal

- Improved resume replay logging.
- Minor cleanup of plate appearance handling.

---

## [0.6.8] - 2026-03-04

### Added

- Automatic **next batter rotation** after Walk or Strikeout.
- Automatic **half-inning change after 3 outs**.
- `start_next_at_bat()` engine helper for lineup progression.
- `handle_three_outs_and_change_side()` to persist side changes and reset bases.
- Boot screen with **Unicode spinner animation** and structured startup messages.
- Database initialization status reporting (existing / new / migrations applied).

### Improved

- Pitch handling logic in `apply_pitch()`:
    - Walk issued automatically at **4 balls**.
    - Strikeout issued automatically at **3 strikes**.
- Pitcher pitch count tracking now persists **across innings**.
- Pitch count resets only when **pitcher changes**, not on inning change.
- Clearer separation between:
    - **Pitch count (balls/strikes per PA)**
    - **Pitcher total pitches thrown**
- Engine flow improved with `needs_next_at_bat` flag.
- Boot sequence now uses `anyhow::Result` for cleaner error propagation.

### Fixed

- Pitcher pitch count incorrectly resetting when inning changed.
- Incorrect scoreboard pitcher state after half-inning switch.
- Walk and Strikeout events not properly updating next batter.
- Dead-code warnings in engine helpers.
- Error propagation issues in database bootstrap.

### Internal

- Simplified database path handling (`get_db_path()`).
- Removed obsolete `PitchThrown` domain event.
- Improved reducer consistency in `apply_domain_event`.

---

## [0.6.7] - 2026-02-27

### ✨ Added

- Pitch count engine commands for the current plate appearance:
    - `b` (ball) with walk rule at 4 balls
    - `k` (called strike), `s` (swinging strike)
    - `f` (foul: counts as strike only if strikes < 2)
    - `fl` (foul bunt: can be strike 3)
- Persisted domain events for pitch tracking and count resets (event-sourced)

### 🧠 Engine

- Enforced baseball rules for balls/strikes thresholds:
    - 4 balls ⇒ batter awarded 1B (walk) and count reset
    - 3 strikes ⇒ batter out, outs increment, and count reset

### 📝 Docs

- Rewrote and extended `SCORING_GUIDE.md` with the new pitch commands and rules
-

---

## [0.6.6] - 2026-02-26

### ✨ Added

- Dynamic pitch count tracking per pitcher
    - New `DomainEvent::PitchThrown`
    - Pitch count stored per active pitcher
    - Counter resets automatically when pitcher changes
- New engine command: `p` / `pitch`
    - Persists `PitchThrown` event
    - Increments pitch count for current pitcher
- Scoreboard now displays live pitch count `(P xxx)`

### ♻️ Refactored

- Removed obsolete `at_bat_no` from `DomainEvent::AtBatStarted`
- Extended `AtBatStarted` to include:
    - Batter full identity (id, jersey, first/last name)
    - Pitcher full identity (id, jersey, first/last name)
- Fixed engine default path:
    - Persisted events are now properly reduced into `GameState`
    - Added `ui.set_state(&state)` after reducer application
- Ensured resume consistency:
    - Pitch count correctly rebuilt from persisted events
    - Batter and pitcher correctly restored after TUI restart

### 🧠 Architecture

- Strengthened event-sourced model:
    - All in-game state derived exclusively from persisted `DomainEvent`s
- Engine loop now consistently follows:
    - parse → apply → persist → reduce → update UI
- Eliminated state/UI desynchronization bug in default command path

### 🐛 Fixed

- `PitchThrown` was not being reduced into `GameState`
- Duplicate `GameStarted` persistence in earlier patch sequence
- Inconsistent scoreboard reconstruction after TUI restart

### 🎯 UX

- Live pitch counter displayed per active pitcher
- Scoreboard fully synchronized with in-memory state
- Resume behavior now deterministic and stable
-

---

## [0.6.5] - 2026-02-26

### ✨ Added

- New structured Scoreboard panel (36 columns layout)
    - Fixed-width panel with intelligent layout
    - Centered base diamond (2B on top, 1B/3B bottom aligned)
    - Dedicated status row (inning, half symbol, count, outs)
    - Batter display: `#<jersey> <First> <Last>`
    - Pitcher display with dynamic alignment and pitch count `(P xxx)`
- Introduced `PlayBallUiContext`
    - Decouples UI rendering from engine logic
    - Centralizes team abbreviations and scoreboard-related display data
- Unicode-aware layout helpers using `unicode-width`
    - `display_width`
    - `pad_right`
    - `pad_right_fit`
    - `fit_two_columns`
- Smart name formatting for scoreboard
    - Automatic abbreviation for long names (e.g. `#64 C. Petro`)

### ♻️ Refactored

- Removed prompt-based game state rendering (state now lives in scoreboard)
- Engine no longer passes team names directly to prompt
- `render_scoreboard` refactored to avoid width overflow and ellipsize artifacts
- Eliminated redundant padding of already width-safe diamond rows
- Improved column alignment using display-width-safe calculations

### 🧠 Architecture

- UI layer now state-driven (`ui.set_state(&GameState)`)
- Scoreboard rendering isolated from engine logic
- Prepared foundation for:
    - live base occupation tracking
    - ball/strike counter
    - dynamic runner display
    - inning-by-inning scoring

### 🎯 UX

- Prompt simplified to `> `
- Clean separation between:
    - Log
    - Scoreboard
    - Command input
- Removed ellipsis artifacts caused by over-padding
- More broadcast-style scoreboard layout

---

## [0.6.1] - 2026-02-24

### ✨ Added

- First real scoring engine command: `playball`
    - Starts the game when no previous events exist
    - Persists `GameStarted` event
    - Persists first `AtBatStarted` event
    - Automatically loads AWAY leadoff batter from lineup
    - Logs:
      `At bat: <TEAM> #<jersey> <First> <Last>`
- Automatic transition from `Pregame` → `InProgress` when entering Play Ball engine
- Centralized UI factory (`create_ui()`) removing duplicated TUI/CLI initialization
- Updated `SCORING_GUIDE.md` to match currently supported engine commands

### ♻️ Refactored

- Removed experimental `out` command from engine and parser
- Simplified Play Ball entry flow (removed explicit start confirmation)
- Cleaned lineup gate-check logic
- Eliminated duplicated event persistence blocks inside engine loop
- Improved event persistence flow consistency

### 🧠 Architecture

- Improved event-sourced engine consistency
- Introduced proper **current batter tracking** in `GameState`
    - replaced ambiguous `at_bat_no`
    - added `current_batter_id`
    - added `current_batter_jersey_no`
- Domain event model prepared for full inning engine evolution
- Engine persists domain events before reducer/UI execution

### 🎯 UX

- Immediate engine start when lineups are valid
- Cleaner Play Ball workflow
- Correct batter identification using jersey number instead of batting order

---

## [0.6.0] - 2026-02-24

### 🚀 Added

- Ratatui-based TUI interface with:
    - Fixed command prompt at bottom
    - Scrollable game log
    - Keyboard navigation (Up/Down, PgUp/PgDn, Home/End)
- Event-sourced Play Ball engine
- Persistent `game_events` logging
- Game state replay on resume
- DomainEvent model with JSON payload support
- Status display in Play Ball game selection list
- New `list_playable_games` function (replaces pregame-only filtering)

### ♻️ Refactored

- Decoupled UI layer from engine logic
- Introduced reducer pattern for GameState reconstruction
- Split engine responsibilities: apply / persist / render
- Reorganized Play Ball flow to support resume without snapshots

### 🧠 Architecture

- Play Ball now uses persistent event log instead of in-memory state
- Games can be safely suspended and resumed
- Foundation prepared for full scoring command expansion

### 🎯 UX Improvements

- Game status icon and label shown in Play Ball selection list
- Improved filtering: excludes only Regulation, Cancelled, Forfeited games

---

## [0.5.0] - 2026-02-23

### 🚀 Added

- Initial implementation of the **Play Ball engine**
- Dynamic game prompt:
    - Inning indicator (↑ Top / ↓ Bottom)
    - Inning number
    - Outs counter
    - Live score display
    - Team abbreviations
- `GameState` model (inning, half, outs, score)
- Engine command loop with support for:
    - Multiple commands separated by commas
    - Case-insensitive input
- Engine control commands:
    - `exit` / `quit`
- Game status commands (available at any time):
    - `regular` → Regulation game
    - `post` → Postponed
    - `cancel` → Cancelled
    - `susp` → Suspended
    - `forf` → Forfeited
    - `protest` → Protested
- Unified `set_game_status()` function using `GameStatus` enum
- Removed legacy `start_game_live()` function
- Added `GameStatus::icon()` support
- Rewritten `SCORING_GUIDE.md` (engine command documentation)

### 🧹 Refactored

- Replaced status magic numbers with `GameStatus` enum everywhere
- Centralized prompt rendering via `print_prompt()`
- Improved separation between CLI and core engine

### 🗑 Removed

- Deprecated `start_game_live()` function
- Unused `has_starting_lineup()` helper

---

## [0.4.3] - 2026-02-23

### ✨ Added

- Import Teams feature (Main Menu → Teams Management → Import Teams)
    - Supports CSV and JSON formats
    - Transaction-safe bulk import
    - Automatic upsert logic (update if existing, insert otherwise)
    - League name resolution during import
- Interactive import workflow with validation and rollback on error

### 🔄 Improved

- Team editing now correctly loads full team data via `get_by_id`
- Edit prompts display current database values as defaults
- Pressing ENTER now preserves existing values correctly
- Optional fields support explicit clearing via `none`

### 🛠 Fixed

- Prevented accidental nullification of team fields during edit
- Corrected incomplete struct cloning during team modification
- Improved error handling during bulk operations (no partial imports)

### 🧱 Internal

- Refactored DB interaction patterns for safer transactional handling
- Improved separation between read-only and write operations in Teams Management

---

## [0.4.2] - 2026-02-12

### Added

- **Player Import/Export System**:
    - Import players from CSV files
    - Import players from JSON files
    - Export players to CSV files
    - Export players to JSON files
    - Auto-creation of teams during import
    - Comprehensive error handling and reporting
    - Format: `team,number,first_name,last_name,position`

- **Import/Export Menu**:
    - New submenu: Player Management → Import/Export Players
    - CSV format support with header detection
    - JSON format support with validation
    - Batch import with progress reporting
    - Export with automatic formatting

- **Interactive lineup editing** for games in **Pre-Game** status
    - Swap two batting spots without recreating the entire lineup
    - Replace a lineup spot with any eligible roster player
- **Dynamic bench detection** (roster − current lineup)
- **Transaction-safe lineup updates**

### Changed

- **Database Schema v5 (Migration)**:
    - **Modified `players` table**:
        - **Removed** `batting_order` field (now managed in game_lineups)
        - **Removed** `name` field (split into first_name and last_name)
        - **Added** `first_name TEXT NOT NULL`
        - **Added** `last_name TEXT NOT NULL`
    - Automatic name splitting during migration (split on first space)
    - Maintains backward compatibility

- **Player Model Updates**:
    - `Player::new()` now takes `first_name` and `last_name` separately
    - Added `full_name()` method: returns "FirstName LastName"
    - Removed `batting_order` from all operations
    - Updated all database queries and display functions

- **Player Management UI**:
    - Add Player: separate fields for first/last name
    - List Players: displays full name via `full_name()`
    - Update Player: separate first/last name updates
    - All displays use `full_name()` method
    - Removed batting order prompts

- **Lineup editing** no longer requires full lineup re-entry
- Replace logic **now updates** the existing lineup slot directly (no bench persistence)
- **Swap logic** now exchanges players between spots instead of modifying batting_order
- Improved borrow handling with scoped statements to allow safe transactions

### 🛠 Fixed

- Resolved UNIQUE constraint violations during lineup swap
- Resolved CHECK constraint violations on `batting_order`
- Corrected transaction mutability handling (`&mut Connection`)
- Eliminated duplicate index conflicts during replace operations

### 🧱 Internal

- Introduced helper functions for:
    - Lineup printing
    - Roster-based bench computation
    - Safe transactional replace and swap
- Improved separation between read-only and write operations

### Technical Details

- **Migration v5** (`src/db/migrations.rs`):
    - Table recreation approach for schema changes
    - Name splitting logic: `substr(name, 1, instr(name, ' ') - 1)` for first_name
    - Handles edge cases: single-word names, empty last names
    - Preserves all existing player data

- **Import Functions**:
    - `import_csv()`: Line-by-line CSV parsing with validation
    - `import_json()`: JSON array parsing with field validation
    - `get_or_create_team()`: Auto-creates missing teams
    - Error tracking per line/player
    - Progress reporting during import

- **Export Functions**:
    - `export_csv()`: Generates CSV with header row
    - `export_json()`: Pretty-printed JSON output
    - Uses `serde_json` for JSON serialization
    - File path validation

- **Format Specifications**:
    - **CSV**: `team,number,first_name,last_name,position`
    - **JSON**: Array of objects with fields: team, number, first_name, last_name, position
    - Position: numeric value 1-9 (1=P, 2=C, 3=1B, etc.)
    - Number: 1-99
    - Team: full team name (auto-created if doesn't exist)

### File Changes

- `src/db/player.rs`: Complete rewrite for first_name/last_name
- `src/db/migrations.rs`: Added migration_v5
- `src/cli/commands/players.rs`: Complete rewrite with import/export
- `src/core/menu.rs`: Added ImportExport to PlayerMenuChoice
- `src/cli/commands/game.rs`: Updated all player.name → player.full_name()
- `examples/players_example.csv`: Sample CSV file
- `examples/players_example.json`: Sample JSON file

### Breaking Changes

- **Database migration required (v4 → v5)**
- **Player struct changed**:
    - Old: `name: String, batting_order: Option<i32>`
    - New: `first_name: String, last_name: String`
- **Player::new() signature changed**
- All code using `player.name` must use `player.full_name()`

### Example Import/Export

**CSV Example:**

```csv
team,number,first_name,last_name,position
Bologna,5,John,Smith,6
Modena,10,Carlos,Rodriguez,3
```

**JSON Example:**

```json
[
  {
    "team": "Bologna",
    "number": 5,
    "first_name": "John",
    "last_name": "Smith",
    "position": 6
  }
]
```

### Migration Notes

**Name Splitting Logic:**

- "John Smith" → first_name="John", last_name="Smith"
- "John" → first_name="John", last_name=""
- "John Paul Jones" → first_name="John", last_name="Paul Jones"

**Batting Order:**

- Previously stored in players table (per-player default)
- Now only in game_lineups table (per-game specific)
- More flexible: players can bat in different positions per game

---

**Migration Path**: v0.4.1 → v0.4.2

- Automatic schema migration v4 → v5
- All player names split automatically
- batting_order removed (use game_lineups instead)
- Backup recommended before upgrade

---

## [0.4.1] - 2026-02-11

### Added

- **GameStatus Enum**:
    - New enum for game status tracking
    - `Pregame = 1` - Game created, lineups can be freely edited
    - `InProgress = 2` - Game started, lineup changes are substitutions
    - `Finished = 3` - Game completed
    - Display trait implementation for user-friendly output
    - Conversion methods: `from_i64()`, `to_i64()`, `as_str()`

- **Edit Lineups Functionality**:
    - Access via: Main Menu → Game Management → Edit Game → Edit Lineups
    - Shows only games with status = Pregame
    - Select game from list of pre-game games
    - Choose team (Away or Home) to edit
    - View current lineup before editing
    - Complete lineup re-entry using same interface as game creation
    - Changes are NOT substitutions (pre-game modifications)
    - Old lineup completely replaced with new one

- **Pre-Game Lineup Management**:
    - Freely modify lineups before game starts
    - No substitution tracking for pre-game changes
    - Players can be moved, swapped, or replaced without restrictions
    - Useful for last-minute roster adjustments

### Changed

- **Database Schema v4 (Migration)**:
    - **Modified `games` table**:
        - Changed `status` field from TEXT to INTEGER
        - **Removed** `current_inning` and `current_half` fields
            - These are now derived from `at_bats` table
            - Current inning = MAX(inning) from at_bats for this game
            - No redundant data storage
        - Default status value: 1 (Pregame)
        - Data migration:
            - 'not_started'/'pregame' → 1
            - 'in_progress' → 2
            - 'completed'/'finished' → 3
    - Migration uses table recreation approach (SQLite limitation)
    - Automatic conversion of existing games
    - Added index on `game_date` for performance

- **Game Status Display**:
    - Updated `list_games()` to use new GameStatus enum
    - Status icons:
        - 🆕 Pregame (was "not started")
        - ▶️ In Progress
        - ✅ Finished (was "completed")
    - Removed "suspended" status (not used)
    - Better visual consistency

- **Inning Display**:
    - Removed from game list (no longer stored in games table)
    - Current inning/half now derived from at_bats table
    - Cleaner games table schema (no redundant data)

### Fixed

- **Optional Fields Handling**:
    - `current_inning` and `current_half` properly handled as Option<>
    - No more errors when these fields are NULL
    - Graceful fallback to "-" in display

### Technical Details

- **GameStatus Enum** (`src/models/types.rs`):
    - Implements Debug, Clone, Copy, Serialize, Deserialize, PartialEq
    - Numeric representation matches database INTEGER values
    - Type-safe status handling throughout codebase

- **Migration v4** (`src/db/migrations.rs`):
    - Complete table recreation (SQLite doesn't support ALTER COLUMN TYPE)
    - Steps:
        1. Create `games_new` with INTEGER status
        2. Copy data with CASE conversion
        3. Drop old `games` table
        4. Rename `games_new` to `games`
    - Safe migration with data preservation
    - Recreates indexes after table swap

- **Edit Lineups Flow**:
    1. Query games WHERE status = 1 (Pregame only)
    2. Display available games with metadata
    3. User selects game and team
    4. Load current lineup from game_lineups table
    5. Display current lineup for review
    6. Re-enter complete lineup using `insert_team_lineup()`
    7. DELETE old lineup entries
    8. INSERT new lineup entries
    9. Transaction ensures atomicity

### User Experience

**Before v0.4.1:**

```
Create game → Enter lineups → Cannot edit before game starts
```

**After v0.4.1:**

```
Create game → Enter lineups → Edit if needed → Play Ball!
                    ↑                ↑
                    └─ Can modify freely while status = Pregame
```

### Breaking Changes

- **Database Schema**: Requires migration from v3 to v4
    - Status field type changed: TEXT → INTEGER
    - Automatic migration on first run
    - Existing status values converted automatically
    - Backup recommended before upgrade

### Database Migration Details

**Status Value Mapping:**

```
OLD (TEXT)           → NEW (INTEGER)
─────────────────────────────────────
'not_started'        → 1 (Pregame)
'pregame'            → 1 (Pregame)
'in_progress'        → 2 (InProgress)
'completed'          → 3 (Finished)
'finished'           → 3 (Finished)
(any other value)    → 1 (Pregame, default)
```

### Known Limitations

- Lineup editing only available for Pregame games
- Once game status changes to InProgress (via Play Ball), lineup editing becomes substitutions
- Complete lineup replacement only (no individual player swap yet)
- Substitution tracking not yet implemented (coming in v0.5.0+)

### Future Enhancements (v0.5.0+)

- Play Ball interface (changes status to InProgress)
- Mid-game substitutions with tracking
- Individual player position/order changes
- Substitution history and reporting
- Lineup comparison before/after changes

---

**Migration Path**: v0.4.0 → v0.4.1

- Automatic schema migration v3 → v4 on startup
- Status field converted from TEXT to INTEGER
- All existing games preserved
- Backup recommended before upgrade

**Next Version**: v0.5.0 will implement "Play Ball!" interface and set status to InProgress

---

## [0.4.0] - 2026-02-11

### Added

- **Complete Lineup Entry System**:
    - Interactive lineup creation for both teams during game setup
    - Full roster validation (minimum 12 players required)
    - Jersey number-based player selection
    - Defensive position assignment (1-9 or DH)
    - Visual lineup confirmation with option to restart
    - Real-time validation: unique positions, unique players

- **Designated Hitter (DH) Support**:
    - Independent DH option for each team
    - **Without DH**: 9 players (pitcher bats in lineup)
    - **With DH**: 9 batters + pitcher (position 10, informational only)
    - Pitcher always defensive position 1 when DH is used
    - DH can bat in any position 1-9 (manager's choice)
    - Proper tracking in database with `at_uses_dh` and `ht_uses_dh` flags

- **Enhanced Game Metadata**:
    - Custom Game ID support (or auto-generated default)
    - Game time field (HH:MM) in addition to date
    - Auto-generated ID format: `GAME_YYYYMMDD_HHMMSS_AWAY_vs_HOME`
    - User can override with custom ID (e.g., `B00A1AAAR0111`)

- **Database Schema v3 (Migration)**:
    - **Modified `games` table**:
        - Added `game_time TEXT` - Time of game (HH:MM format)
        - Added `at_uses_dh BOOLEAN DEFAULT 0` - Away team DH flag
        - Added `ht_uses_dh BOOLEAN DEFAULT 0` - Home team DH flag
        - `current_inning` and `current_half` left NULL until game starts

    - **New `game_lineups` table**:
        - Complete starting lineup tracking
        - Fields: `game_id`, `team_id`, `player_id`, `batting_order` (1-10), `defensive_position`
        - Support for substitution tracking: `is_starting`, `substituted_at_inning`, `substituted_at_half`
        - Primary key: (game_id, team_id, batting_order)
        - Foreign keys with referential integrity
        - Indexes on game_id and team_id for performance
        - Ready for future substitution implementation

- **Position Display Support**:
    - Implemented `fmt::Display` trait for `Position` enum
    - Position abbreviations: P, C, 1B, 2B, 3B, SS, LF, CF, RF
    - Used in roster display during lineup entry

- **Application Icons**:
    - 4 professional icon variants (SVG format, 1024x1024)
    - **v1**: Baseball field with vertical pencil (realistic, sports-focused)
    - **v2**: Scorecard with diagonal pencil (traditional, vintage)
    - **v3**: Modern minimalist with blue gradient (clean, contemporary)
    - **v4**: Professional app icon style (recommended, versatile)
    - Complete conversion guide for PNG, .ico, .icns formats
    - Integration instructions for Windows, macOS, Linux

### Changed

- **Game Creation Workflow**:
    - **Old**: Select teams → Date → Venue → Done
    - **New**: Game ID → Teams → Date → Time → Venue → Away Lineup → Home Lineup → Confirm
    - Much more comprehensive pre-game setup
    - All lineup information captured before game starts
    - Better preparation for "Play Ball!" scoring interface

- **Lineup Entry Logic**:
    - Always 9 batting positions (regardless of DH)
    - Pitcher in position 10 is informational only when DH used
    - Clear prompts: "Defensive position (1-9 or DH)" when applicable
    - Removed redundant position 10 batting entry
    - Pitcher can be same player as one in lineup (rare but legal)

- **Database Insert Queries**:
    - Removed `current_inning` and `current_half` from game creation
    - These fields now remain NULL until "Play Ball!" starts
    - More semantically correct: NULL = game not started
    - Values will be populated when scoring begins (v0.5.0)

### Fixed

- **Compilation Errors**:
    - Added `Display` trait implementation for `Position` enum
    - Fixed temporary value lifetime issue in lineup display
    - Corrected defensive position variable usage
    - All compiler warnings resolved

- **DH Logic Corrections**:
    - Fixed double "position 10" prompt bug
    - Corrected loop to always be 1-9 (not 1-10)
    - Pitcher entry now clearly labeled as informational
    - Removed incorrect "already in lineup" check for pitcher

### Technical Details

- **Helper Functions Added** (`src/cli/commands/game.rs`):
    - `insert_team_lineup()` - Interactive lineup entry with full validation
    - `display_lineup()` - Pretty-print lineup for user confirmation
    - `save_lineup()` - Persist lineup to database with transaction safety

- **Migration v3** (`src/db/migrations.rs`):
    - Automatic migration from schema v2 to v3
    - Safe ALTER TABLE operations
    - New table creation with proper constraints
    - Index creation for query optimization

- **Validation Rules**:
    - Minimum 12 players in roster before lineup entry
    - Each defensive position 1-9 used exactly once
    - No duplicate players in batting lineup
    - Jersey numbers must exist in team roster
    - Proper error messages for all validation failures

### Documentation

- **New Files**:
    - `ICONS_README.md` - Complete icon usage guide
    - `FIX_DH_LOGIC.md` - DH implementation explanation
    - `FIX_REMOVE_CURRENT_INNING.md` - Rationale for field changes
    - `BUGFIX.md` - Compilation error fixes
    - `IMPLEMENTATION_GUIDE_v0.4.0.md` - Technical implementation details

- **Updated Files**:
    - `README.md` - Updated for v0.4.0 features
    - `CHANGELOG.md` - This file

### Breaking Changes

- **Database Schema**: Requires migration from v2 to v3
    - Automatic migration on first run after upgrade
    - Existing games compatible but without lineups
    - Recommended: Backup database before upgrading
    - Use "Manage DB > Backup Database" feature

### Known Limitations

- Lineup entry required for all new games (cannot skip)
- No lineup editing after creation (coming in v0.5.0)
- Pitcher position must be assigned even when DH not used
- Substitutions not yet implemented (planned for v0.5.0+)

### Future Enhancements (v0.5.0+)

- "Play Ball!" live scoring interface
- Mid-game substitutions
- Lineup editing
- Player position validation
- Lineup templates for quick entry
- Import/export lineups

---

**Migration Path**: v0.3.1 → v0.4.0

- Automatic schema migration on startup
- Backup recommended before upgrade
- All existing data preserved

**Next Version**: v0.5.0 will implement the "Play Ball!" scoring interface

---

## [0.3.0] - 2026-02-03

### Added

- **Game Management System**:
    - New main menu option: "1. Manage Games" (replaces "New Game")
    - Complete game lifecycle management interface
    - Game metadata creation and tracking

- **Game Management Menu**:
    - **New Game**: Create game with metadata only
        - Select away and home teams
        - Set venue (required)
        - Set game date (defaults to today)
        - Auto-generate unique game ID
        - Status: 'not_started' until scoring begins

    - **List Games**: View all games with details
        - Sorted by date (newest first)
        - Shows: date, teams, score, venue, status, inning
        - Status icons: 🆕 Not Started, ▶️ In Progress, ✅ Completed, ⏸️ Suspended
        - Game ID display for reference

    - **Edit Game**: Modify game metadata (placeholder for v0.3.1)
        - Future: Edit date, venue, teams, metadata

    - **Play Ball!**: Launch scoring interface (placeholder for v0.4.0)
        - Future: Pitch-by-pitch scoring
        - Future: Real-time display
        - Future: Runner tracking

- **Database Schema Redesign (Migration v2)**:
    - **Removed obsolete tables**:
        - `plate_appearances` (too simplistic)
        - `base_runners` (insufficient granularity)

    - **New comprehensive tables**:
        - `at_bats`: Complete plate appearance tracking
            - State before: outs, runners on base (1B, 2B, 3B)
            - Player IDs: batter, pitcher, base runners
            - Result: type and details (JSON)
            - State after: outs, runs, RBIs
            - Enables complete game reconstruction

        - `pitches`: Individual pitch tracking
            - Pitch-by-pitch sequence with numbering
            - Count before each pitch (balls, strikes)
            - Pitch type: BALL, CALLED_STRIKE, SWINGING_STRIKE, FOUL, IN_PLAY, HBP
            - In-play result if applicable
            - Foundation for pitch analytics

        - `runner_movements`: Base runner tracking
            - Per at-bat runner advancement
            - Start and end base for each runner
            - Advancement type: BATTED_BALL, STOLEN_BASE, WILD_PITCH, etc.
            - Out tracking: caught stealing, picked off, force out
            - Earned run tracking for ERA calculation

        - `game_events`: Special events tracking
            - Substitutions, injuries, delays
            - Ejections, challenges, protests
            - Flexible event data in JSON
            - Links to at-bat if applicable

### Changed

- **Main Menu Structure**:
    - Option 1: "New Game" → "Manage Games"
    - Positions unchanged for options 2-6
    - Better organization and workflow clarity

- **Game Creation Workflow**:
    - **Before v0.3.0**: Direct scoring start
    - **After v0.3.0**: Two-phase approach
        1. Create game metadata (New Game)
        2. Start scoring later (Play Ball!)
    - Allows game setup without immediate scoring
    - Better for scheduling and planning

- **Database Philosophy**:
    - From: Simplified plate appearance summary
    - To: Granular pitch-by-pitch tracking
    - Captures complete game state at every moment
    - Supports advanced analytics and sabermetrics

- **Game ID Generation**:
    - Auto-generated with timestamp
    - Format: `GAME_YYYYMMDD_HHMMSS_AWAY_vs_HOME`
    - Includes team abbreviations for clarity
    - Unique and sortable

### Improved

- **Game State Tracking**:
    - Complete snapshot before/after each at-bat
    - All base runners identified by player ID
    - Foreign key relationships maintain integrity
    - Comprehensive event history

- **Scalability for Analytics**:
    - Pitch-by-pitch data enables:
        - Pitch count tracking
        - Strike zone analysis
        - Batter tendencies
        - Pitcher repertoire analysis

    - At-bat data enables:
        - Batting average (AVG)
        - On-base percentage (OBP)
        - Slugging percentage (SLG)
        - Runs batted in (RBI)

    - Runner movement data enables:
        - Stolen base statistics
        - Base running efficiency
        - Earned run average (ERA) calculation
        - Defensive efficiency metrics

- **Database Normalization**:
    - Proper foreign keys to players table
    - Referential integrity enforced
    - CHECK constraints validate data
    - Indexes for query performance

### Technical Details

#### Migration v2 (Automatic)

```sql
-- Drop old tables
DROP TABLE IF EXISTS base_runners;
DROP TABLE IF EXISTS plate_appearances;

-- Create new tables
CREATE TABLE at_bats
(...
);
CREATE TABLE pitches
(...
);
CREATE TABLE runner_movements
(...
);
CREATE TABLE game_events
(...
);

-- Create indexes
CREATE INDEX idx_at_bats_game ON at_bats (game_id);
CREATE INDEX idx_pitches_at_bat ON pitches (at_bat_id);
CREATE INDEX idx_runner_movements_at_bat ON runner_movements (at_bat_id);
CREATE INDEX idx_game_events_game ON game_events (game_id);
```

#### Schema Version

- Incremented: v1 → v2
- Automatic migration on app startup
- Manual execution: Manage DB → Run Migrations

#### Table Relationships

```
games
  ├── at_bats (1:N)
  │   ├── pitches (1:N)
  │   └── runner_movements (1:N)
  └── game_events (1:N)

players
  ├── at_bats.batter_id (1:N)
  ├── at_bats.pitcher_id (1:N)
  ├── at_bats.runner_on_* (1:N)
  └── runner_movements.runner_id (1:N)
```

#### At-Bat State Capture

```rust
// Before at-bat
outs_before: 0 - 2
runner_on_first: Option
runner_on_second: Option
runner_on_third: Option

// After at-bat
outs_after: 0 - 3
runs_scored: i32
rbis: i32
```

#### Pitch Sequence Example

```
Pitch 1: Count 0-0, BALL        → 1-0
Pitch 2: Count 1-0, CALLED_STRIKE → 1-1
Pitch 3: Count 1-1, FOUL        → 1-2
Pitch 4: Count 1-2, IN_PLAY     → Result: SINGLE
```

### Files Added

- None (only modified existing files)

### Files Modified

- `src/core/menu.rs`: Added GameMenuChoice enum, show_game_menu()
- `src/cli/commands/game.rs`: Complete rewrite with game management
- `src/cli/commands/main_menu.rs`: Updated routing for ManageGames
- `src/db/migrations.rs`: Added migration_v2() for schema redesign
- `src/db/database.rs`: Removed old table creation code
- `src/lib.rs`: Updated re-exports for GameMenuChoice
- `Cargo.toml`: Version bump to 0.3.0

### Breaking Changes

- **Database Schema**: Migration v2 drops `plate_appearances` and `base_runners`
    - Impact: Any existing game data will be lost
    - Mitigation: Backup database before upgrading (automatic prompt)
    - New installations: No impact

### Migration Notes

- **Automatic**: Runs on first app start after upgrade
- **Manual option**: Manage DB → Run Migrations
- **Data loss warning**: Old game scoring data cannot be migrated
- **Recommendation**: Fresh start for v0.3.0+

### Deprecation Notice

- Old `PlateAppearance` and `BaseRunner` types in models/types.rs
    - Still present for backward compatibility
    - Will be removed in v0.4.0
    - Not used in new game scoring system

### Future Roadmap (v0.4.0+)

- **Play Ball! Interface**:
    - Pitch-by-pitch input
    - Real-time score display
    - Base diagram visualization
    - Substitution handling
    - Game state persistence

- **Analytics Dashboard**:
    - Player statistics
    - Pitch analytics
    - Team performance metrics
    - Historical comparisons

### Developer Notes

**Creating a Game:**

```rust
// Insert into games table
game_id: auto -generated unique ID
status: 'not_started'
current_inning: 1
current_half: 'Top'
```

**Recording an At-Bat:**

```rust
// 1. Create at_bats record with state before
// 2. Add pitches in sequence
// 3. Record runner movements
// 4. Update at_bats with state after
// 5. Update games table scores
```

**Query Example - Get Game Summary:**

```sql
SELECT COUNT(DISTINCT ab.id) as plate_appearances,
       SUM(ab.runs_scored)   as runs,
       COUNT(DISTINCT p.id)  as total_pitches
FROM at_bats ab
         LEFT JOIN pitches p ON ab.id = p.at_bat_id
WHERE ab.game_id = ?
```

---

## [0.2.6] - 2026-02-03

### Added

- **Player Management System**:
    - New main menu option: "4. Manage Players"
    - Complete CRUD operations for players
    - Dedicated player management interface

- **Player Management Menu**:
    - **Add New Player**: Create players with team assignment
        - Input: Name, jersey number (1-99), defensive position (1-9)
        - Optional batting order
        - Automatic team selection from available teams
        - Validation for number and position

    - **List All Players**: View all players with filtering
        - Filter by team or view all
        - Display: Number, name, team, position, batting order
        - Sorted by team name and jersey number
        - Professional table format with aligned columns

    - **Update Player**: Modify player information
        - Update name, jersey number, position, batting order
        - Keep existing values (press ENTER to skip)
        - Validation on all inputs

    - **Delete Player**: Remove players with confirmation
        - Safety confirmation before deletion
        - Shows player details before confirming

    - **Change Team**: Transfer players between teams
        - List all players with current teams
        - Select new team from available teams
        - Prevents assignment to same team
        - Updates player record atomically

### Changed

- **Main Menu Structure**:
    - Expanded from 5 to 6 main options
    - Added "4. Manage Players" between Teams and Statistics
    - Renumbered: Statistics (4→5), Manage DB (5→6)
    - Updated menu prompt: "Select an option (1-6 or 0)"

- **Code Organization**:
    - **Refactored Player Module**: Extracted from `team.rs` to dedicated `player.rs`
    - `src/db/player.rs`: Complete Player struct and implementation (NEW)
    - `src/db/team.rs`: Now contains only Team-related code
    - Better separation of concerns and scalability
    - Consistent with League pattern (one file per entity)

- **Player CRUD in Separate Module**:
    - `src/cli/commands/players.rs`: Player command handlers (NEW)
    - All player operations in dedicated module
    - Clean imports and dependencies

### Improved

- **DRY Principle**:
    - Created `Player::from_row_with_team()` helper method
    - Reuses existing `Player::from_row()` for consistency
    - Eliminates code duplication in player listing
    - Single source of truth for player mapping

- **Helper Functions**:
    - `display_player_list()`: Centralized player display formatting
    - Used in update, delete, and change team operations
    - Consistent formatting across all player lists
    - Easy to modify display format in one place

- **Team Integration**:
    - `Team::get_roster()` now uses `player::Player` module
    - Clean cross-module references
    - Maintains existing team-player relationship

### Technical Details

- **Player Management Flow**:

```
  Main Menu → 4. Manage Players → Player Menu
    ├── 1. Add New Player    → Select Team → Enter Details
    ├── 2. List All Players  → Choose Filter → Display
    ├── 3. Update Player     → Select Player → Update Fields
    ├── 4. Delete Player     → Select Player → Confirm
    └── 5. Change Team       → Select Player → Select New Team
```

- **Player Validation**:
    - Jersey number: 1-99 range
    - Position: 1-9 (official baseball positions)
    - Name: Required, non-empty
    - Team: Must exist in database
    - Unique constraint: (team_id, number) pair

- **Database Integration**:
    - All operations use existing `players` table
    - Foreign key to `teams` table maintained
    - Cascading delete handled by Team::delete()
    - Active/inactive flag support (is_active)

- **Module Structure**:

```rust
  src/db/
├── player.rs (NEW) # Player struct + CRUD
└── team.rs # Team struct + CRUD (cleaned)

src/cli/commands/
└── players.rs (NEW) # Player management UI
```

### Files Added

- `src/db/player.rs`: Player entity with complete CRUD (NEW)
- `src/cli/commands/players.rs`: Player management commands (NEW)

### Files Modified

- `src/db/team.rs`: Removed Player code, kept only Team
- `src/db/mod.rs`: Added `pub mod player;`
- `src/core/menu.rs`: Added PlayerMenuChoice enum and show_player_menu()
- `src/cli/commands/mod.rs`: Added `pub mod players;`
- `src/cli/commands/main_menu.rs`: Integrated player menu handling
- `src/lib.rs`: Updated Player re-export from db::player
- `Cargo.toml`: Version bump to 0.2.6

### Developer Notes

**Player Entity Structure:**

```rust
pub struct Player {
    pub id: Option,
    pub team_id: i64,
    pub number: i32,
    pub name: String,
    pub position: Position,      // 1-9 enum
    pub batting_order: Option,
    pub is_active: bool,
}
```

**Helper Methods:**

```rust
Player::from_row(row)              // Map DB row to Player
Player::from_row_with_team(row)    // Map DB row to (Player, team_name)
```

**CRUD Operations:**

```rust
Player::create( & mut self , conn)    // Insert new player
Player::get_by_id(conn, id)        // Fetch by ID
Player::get_by_team(conn, team_id) // Fetch team roster
Player::update( & self , conn)        // Update existing
Player::delete(conn, id)           // Remove player
```

---

## [0.2.5] - 2026-02-03

### Added

- **Database Migration System**:
    - Automatic schema migration on application startup
    - Manual migration execution via DB management menu
    - Incremental migration support (only applies missing migrations)
    - Version tracking with detailed migration history
    - Migration descriptions for each schema change
    - Safe migration workflow with confirmations

- **Meta Table** (`meta`):
    - Centralized application metadata storage
    - `schema_version`: Current database schema version
    - `app_version`: Application version that created/updated DB
    - `created_at`: Database creation timestamp
    - `last_backup`: Last backup operation timestamp
    - `last_restore`: Last restore operation timestamp
    - `last_migration`: Last migration execution timestamp
    - Automatic timestamp updates on operations

- **Migration Management Interface**:
    - New menu option: "3. Run Migrations" in DB Management
    - View current and latest schema versions
    - List pending migrations with descriptions
    - Execute migrations manually on demand
    - Migration status display in "View DB Info"

- **Migration Module** (`src/db/migrations.rs`):
    - `CURRENT_SCHEMA_VERSION` constant for version control
    - `Migration` struct with version, description, and upgrade function
    - `get_migrations()`: Returns all available migrations
    - `run_migrations()`: Executes pending migrations incrementally
    - `get_schema_version()`: Retrieves current DB schema version
    - `migrations_needed()`: Checks if migrations are pending
    - `get_migration_info()`: Returns detailed migration status
    - Helper functions for meta table operations

### Changed

- **Database Initialization**:
    - `init_schema()` now creates meta table first
    - Checks for new database and sets creation metadata
    - Automatically runs pending migrations after table creation
    - Sets initial schema version for new databases
    - Displays migration progress during startup

- **Backup Operations**:
    - Records backup timestamp in meta table
    - Updates `last_backup` key automatically
    - Backup metadata persists across sessions

- **Restore Operations**:
    - Records restore timestamp in meta table
    - Updates `last_restore` key automatically
    - Restore metadata persists across sessions

- **View DB Info**:
    - Now displays current schema version
    - Shows migration status (up to date or pending)
    - Visual indicator for pending migrations (⚠️)

- **DB Management Menu**:
    - Expanded from 7 to 8 options
    - Added "3. Run Migrations" option
    - Renumbered existing options accordingly
    - Updated menu display and navigation

### Improved

- **Schema Evolution Support**:
    - Easy addition of new migrations
    - Clear migration history tracking
    - Safe incremental upgrades
    - No manual SQL execution needed

- **Database Metadata**:
    - Comprehensive application state tracking
    - Timestamp tracking for key operations
    - Version information for troubleshooting
    - Foundation for future analytics

- **Developer Experience**:
    - Simple migration addition workflow
    - Clear migration structure and patterns
    - Automatic version management
    - Built-in testing support

### Technical Details

- **Migration Version Control**:
    - Each migration has unique version number
    - Migrations applied in order (v1, v2, v3, ...)
    - Only missing migrations are executed
    - Version stored in meta table after each migration

- **Meta Table Schema**:

```sql
  CREATE TABLE meta
  (
      key        TEXT PRIMARY KEY,
      value      TEXT NOT NULL,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
  )
```

- **Migration Structure**:

```rust
  pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub up: fn(&Connection) -> Result,
}
```

- **Automatic Migration Flow**:
    1. App starts → setup_db() called
    2. Meta table created/verified
    3. Current schema version retrieved
    4. Compare with CURRENT_SCHEMA_VERSION
    5. If outdated → run pending migrations
    6. Update schema version in meta
    7. Continue application startup

- **Manual Migration Flow**:
    1. User selects "Run Migrations"
    2. Display current vs. latest version
    3. List pending migrations
    4. User confirms execution
    5. Apply migrations sequentially
    6. Update meta table
    7. Show completion summary

### Files Added

- `src/db/migrations.rs`: Complete migration system (NEW)

### Files Modified

- `src/db/mod.rs`: Export migrations module
- `src/db/database.rs`: Integration with migration system
- `src/core/menu.rs`: Added RunMigrations to DBMenuChoice
- `src/cli/commands/db.rs`: Implemented run_migrations_manual()
- `src/lib.rs`: Re-export migration functions
- `README.md`: Updated to v0.2.5 with migration documentation
- `Cargo.toml`: Version bump to 0.2.5

### Developer Guide

**Adding New Migrations:**

1. Increment `CURRENT_SCHEMA_VERSION` in `migrations.rs`
2. Add migration to `get_migrations()` vector
3. Implement migration function (e.g., `migration_v2`)
4. Test migration on development database
5. Users get automatic upgrade on next app start

**Example:**

```rust
// Step 1: Increment version
pub const CURRENT_SCHEMA_VERSION: i64 = 2;

// Step 2: Add to list
Migration {
version: 2,
description: "Add player statistics table",
up: migration_v2,
}

// Step 3: Implement
fn migration_v2(conn: &Connection) -> Result {
    conn.execute("CREATE TABLE stats (...)", [])?;
    Ok(())
}
```

---

## [0.2.4] - 2026-02-03

### Added

- **Complete Database Management Suite**:
    - **View DB Status**: Comprehensive database health monitoring
        - Database size (MB/KB) and page statistics
        - Free space analysis with percentage
        - Journal mode, synchronous mode, auto-vacuum settings
        - Quick integrity check
        - Smart suggestions (e.g., VACUUM when free space > 10%)

    - **Backup Database**: Safe database backup with timestamps
        - Creates timestamped backup files: `baseball_scorer_backup_YYYYMMDD_HHMMSS.db`
        - Shows source and destination paths
        - Displays backup size in KB
        - Confirmation before creating backup

    - **Restore Database**: Safe database restoration with safety backups
        - Lists all available backup files with dates and sizes
        - Creates automatic safety backup before restore
        - Double confirmation for safety
        - Shows detailed restore information

    - **Vacuum Database**: Database optimization and space reclamation
        - Shows current database size and free space
        - Explains VACUUM operation before execution
        - Displays before/after statistics
        - Calculates space saved (KB and percentage)
        - Improves database performance

    - **Export Game**: Export individual games to JSON or CSV
        - Lists all available games with scores and dates
        - JSON format: Detailed structured data
        - CSV format: Simplified format for Excel/spreadsheets
        - Exports to current working directory
        - Includes all plate appearances and game data

### Changed

- **Main Menu Loop Refactoring**:
    - Moved main menu loop from `main.rs` to `src/cli/commands/main_menu.rs`
    - `main.rs` reduced to absolute minimum (database setup + menu call)
    - Cleaner separation: initialization vs. application logic
    - Better code organization and maintainability

- **Database Management Menu Expanded**:
    - Renamed from 5 options to 7 options
    - Replaced "Change DB Location" with "Export Game"
    - Added "View DB Status" and "Vacuum Database"
    - Updated menu numbering: 1-7 options + 0 for Back
    - Professional menu layout with clear icons

### Improved

- **Database Monitoring**:
    - Comprehensive status information for troubleshooting
    - Performance metrics (page count, size, fragmentation)
    - Configuration display (journal, sync, vacuum modes)
    - Integrity verification

- **Database Maintenance**:
    - Complete backup/restore workflow
    - Space optimization with VACUUM
    - Safety measures (automatic backups before restore)
    - Clear user feedback at every step

- **Data Portability**:
    - Export games for external analysis
    - Multiple export formats (JSON, CSV)
    - Easy integration with other tools

### Technical Details

- **New Functions**:
    - `view_db_status()`: PRAGMA queries for DB metrics
    - `vacuum_database()`: Database optimization
    - `backup_database()`: Timestamped file copy
    - `restore_database()`: Safe restore with backups
    - `export_game_json()`: Structured game export
    - `export_game_csv()`: Tabular game export
    - `list_backup_files()`: Backup file discovery

- **SQLite Features Used**:
    - `PRAGMA page_count`, `PRAGMA page_size`
    - `PRAGMA freelist_count` (free space)
    - `PRAGMA journal_mode`, `PRAGMA synchronous`
    - `PRAGMA auto_vacuum`, `PRAGMA quick_check`
    - `VACUUM` command for optimization

- **File Naming Conventions**:
    - Backups: `baseball_scorer_backup_YYYYMMDD_HHMMSS.db`
    - Safety backups: `baseball_scorer_before_restore_YYYYMMDD_HHMMSS.db`
    - Exports: `{game_id}_export.json` or `{game_id}_export.csv`

### Files Modified

- `src/core/menu.rs`: Updated `DBMenuChoice` enum and menu display
- `src/cli/commands/db.rs`: Added 5 new database management functions
- `src/cli/commands/main_menu.rs`: NEW - Main menu loop extracted from main.rs
- `src/main.rs`: Simplified to minimal entry point
- `src/utils/cli.rs`: Added `read_choice_u32()` helper
- `Cargo.toml`: Version bump to 0.2.4
-

---

## [0.2.3] - 2026-02-03

### Added

- **Database Management Menu**:
    - New menu option "5. Manage DB"
    - View database information (location, record counts, size)
    - Clear all data functionality (with double confirmation)
    - Backup database placeholder (coming soon)
    - Restore database placeholder (coming soon)
    - Change DB location placeholder (coming soon)

- **CLI Commands Module (`src/cli/commands/`)**:
    - Separated command handlers into dedicated modules
    - `db.rs`: Database management commands
    - `game.rs`: Game-related commands
    - `leagues.rs`: League management commands
    - `statistics.rs`: Statistics display commands
    - `team.rs`: Team management commands
    - Better code organization and maintainability

- **Database Setup Helper (`setup_db()`)**:
    - Unified database initialization in `db/config.rs`
    - Clear error messages and user feedback
    - Handles path determination, file creation, and schema init
    - Exits cleanly on errors with detailed messages

### Changed

- **Menu System Improvements**:
    - Exit/Back option moved to `0` in all menus (was last option)
    - More intuitive and conventional UI pattern
    - Easier to add new menu options without renumbering
    - Updated prompts: "Select an option (1-5 or 0)"

- **UI Utilities Refactoring**:
    - Created `src/utils/cli.rs` for CLI helper functions
    - Moved functions from `Menu` impl to standalone utilities
    - Functions: `clear_screen()`, `read_choice()`, `show_header()`, etc.
    - Reusable across the entire application
    - Cleaner separation of concerns

- **main.rs Simplification**:
    - Reduced from ~400 lines to 25 lines
    - Database initialization delegated to `setup_db()`
    - Command handling delegated to `cli::commands` modules
    - Extremely clean and maintainable entry point

- **Database Info Display**:
    - Improved formatting with aligned columns
    - Uses `{:<width}` and `{:>width}` for perfect alignment
    - Professional table-like output

### Improved

- **Code Organization**:
    - Clear separation: UI utilities, menu logic, command handlers
    - Each command type in its own file
    - Easier to navigate and maintain
    - Better for testing individual components

- **Edition Update**:
    - Updated to Rust edition 2024 in Cargo.toml
    - Uses latest stable Rust features

### Technical Details

- All menus now use `utils::cli::` for helper functions
- Database row mapping uses helper functions (DRY principle)
- `setup_db()` uses `process::exit()` for clean error handling
- Consistent formatting throughout with `{:<10} {:>8}` patterns

---

## [0.2.2] - 2026-02-03

### Added

- **Library Support (`lib.rs`)**:
    - Created public library interface for code reusability
    - Re-exported commonly used types and functions
    - Added comprehensive module documentation
    - Enables integration with other Rust projects
    - Foundation for future GUI, API, or plugin development

### Changed

- **Standard Rust Project Structure**:
    - Moved `main.rs` → `src/main.rs`
    - Moved `core/` → `src/core/`
    - Moved `models/` → `src/models/`
    - All source code now under `src/` directory
    - Follows Rust best practices and conventions

- **Cargo.toml Enhancements**:
    - Added `[lib]` section for library compilation
    - Updated `[[bin]]` path to `src/main.rs`
    - Added metadata: authors, description, license, repository
    - Added keywords and categories for crates.io compatibility
    - Fixed edition to `2021` (was incorrectly `2024`)

- **Module System**:
    - `main.rs` now uses `bs_scoring::` imports from lib
    - Removed redundant module declarations
    - Cleaner separation between library and binary

### Improved

- **IDE and Tooling Support**:
    - Better autocomplete and navigation
    - Improved `cargo doc` documentation generation
    - Standard structure recognized by all Rust tools
    - Easier debugging and testing

---

## [0.2.1] - 2026-02-03

### Added

- **Cross-Platform Database Path Management**:
    - Windows: Database stored in `%LOCALAPPDATA%\bs_scorer\baseball_scorer.db`
    - macOS/Linux: Database stored in `$HOME/.bs_scorer/baseball_scorer.db`
    - Automatic directory creation on first run
    - Display database location on startup

### Changed

- **Project Structure Reorganization**:
    - Created `src/db/` directory for all database-related code
    - Moved `database.rs`, `league.rs`, `team.rs` to `src/db/`
    - Added `src/db/config.rs` for path management
    - `models/` now only contains `types.rs` (game scoring types)
    - Clearer separation between DB operations and game logic

### Fixed

- Clippy warnings for enum variant naming:
    - `Walk::IntentionalWalk` → `Walk::Intentional`
    - `Pitch::HitByPitch` → `Pitch::HittedBy`

---

## [0.2.0] - 2026-02-03

### Added

- **SQLite Database Integration**: Full persistence layer with rusqlite
    - `leagues` table for managing baseball/softball leagues
    - `teams` table with league association
    - `players` table with team rosters
    - `games` table for match tracking
    - `plate_appearances` table for detailed scoring
    - `base_runners` table for runner advancement tracking
    - Automatic database schema initialization
    - Indexed queries for performance optimization

- **COBOL-Style Menu System**: Classic terminal-based navigation
    - Main menu with 5 options (New Game, Manage Leagues, Manage Teams, Statistics, Exit)
    - League management submenu (Create, View, Edit, Delete)
    - Team management submenu (Create, View, Edit, Roster, Import, Delete)
    - Clean ASCII box-drawing UI
    - Input validation and confirmation dialogs

- **League Management (Full CRUD)**:
    - Create leagues with name, season, and description
    - View all leagues with formatted display
    - Edit league information
    - Delete leagues with confirmation

- **Team Management (Full CRUD)**:
    - Create teams with name, city, abbreviation, founded year
    - Link teams to leagues (optional)
    - View all teams with details
    - Delete teams (cascades to players)
    - Roster management foundation (in development)

- **Modular Architecture**:
    - `models/database.rs`: Database schema and initialization
    - `models/league.rs`: League CRUD operations
    - `models/team.rs`: Team and Player CRUD operations
    - `core/menu.rs`: Menu navigation system
    - Clear separation between DB models and game types

### Changed

- **Project Structure**: Reorganized into `core/` and `models/` modules
    - Moved parser to `core/parser.rs`
    - Moved types to `models/types.rs`
    - Added database-specific models in `models/`

- **Type System Refactor**:
    - Renamed `Team` → `GameTeam` in `types.rs` (for JSON/scoring)
    - Renamed `Player` → `GamePlayer` in `types.rs` (for JSON/scoring)
    - New `Team` and `Player` structs in `team.rs` (for database)
    - Prevents naming conflicts between DB and game scoring types

- **User Interface Language**: All messages translated to English
    - Menu labels and headers
    - Error messages and confirmations
    - Input prompts
    - Success notifications

- **Main Entry Point**: Complete rewrite of `main.rs`
    - Menu-driven flow instead of immediate game start
    - Database initialization on startup
    - Removed obsolete functions (setup_game, print_help, etc.)

### Fixed

- Struct conflicts between database models and game types
- Unused imports and dead code
- Inconsistent language in UI messages

### Removed

- Direct game start workflow (replaced with menu system)
- Obsolete helper functions from v0.1.0
- ~200 lines of legacy code

---

## [0.1.0] - 2026-02-01

### Added

- Initial CLI scoring application
- Baseball scoring symbols parser (K, 6-3, HR, BB, etc.)
- Plate appearance tracking
- JSON export functionality
- Comprehensive scoring guide documentation
- All defensive positions (1-9)
- Hit types: Single, Double, Triple, Home Run
- Out types: Strikeout, Groundout, Flyout, Lineout, etc.
- Advanced plays: Stolen Base, Wild Pitch, Balk, etc.

---

## Upcoming Features

### [0.3.0] - Planned

- [ ] Live game scoring interface
- [ ] Complete roster management with lineup builder
- [ ] Pitch-by-pitch tracking
- [ ] Base runner advancement tracking
- [ ] Real-time game state display
- [ ] Enhanced data validation

### [0.4.0] - Planned

- [ ] Player statistics module
- [ ] Batting average (AVG), On-base (OBP), Slugging (SLG)
- [ ] Pitcher ERA, WHIP, K/9
- [ ] League standings and rankings
- [ ] Season statistics aggregation

### [0.5.0] - Planned

- [ ] CSV/JSON import for teams and players
- [ ] PDF scorecard export
- [ ] ASCII art diamond visualization
- [ ] Game replay functionality
- [ ] Multi-season support

---

**Legend:**

- `Added` for new features
- `Changed` for changes in existing functionality
- `Deprecated` for soon-to-be removed features
- `Removed` for now removed features
- `Fixed` for any bug fixes
- `Security` in case of vulnerabilities