# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.12.0] - Unreleased

### Added

- Added support for multiple roster positions for each player.
  - Accepted values: `P`, `C`, `1B`, `2B`, `3B`, `SS`, `LF`, `CF`, `RF`, `IF`, `OF`, `DH`.
  - Multiple values are stored as a normalized comma-separated string, for example `P,C,IF`.
- Added WBSC-style `BAT/THROW` notation for player import/export.
  - Examples: `R/R`, `R/L`, `L/R`, `S/S`.
  - The value on the left of `/` is the batting side.
  - The value on the right of `/` is the throwing hand.
- Added automatic database migration for legacy player data.
- Added automatic migration of legacy Linux application data directory.

### Changed

- Renamed player handedness terminology from `pitch` to `throw`.
- Renamed the player database column from `pitch` to `throw`.
- Changed throwing hand values from `RHP`, `LHP`, `SHP` to `R`, `L`, `S`.
- Changed the `players.position` field from numeric defensive-position values to roster-position strings.
  - Legacy values are migrated as follows:
    - `1` → `P`
    - `2` → `C`
    - `3` → `1B`
    - `4` → `2B`
    - `5` → `3B`
    - `6` → `SS`
    - `7` → `LF`
    - `8` → `CF`
    - `9` → `RF`
    - `10` → `DH`
- CSV player import/export now uses `;` as field separator.
- CSV player import/export now uses this format:

  ```text
  team_name;number;away_number;first_name;last_name;position;bat_throw
  ```

- JSON player import/export now uses this format:

  ```json
  {
    "team_name": "Rimini Baseball",
    "number": 12,
    "away_number": 9,
    "first_name": "Mario",
    "last_name": "Rossi",
    "position": "P,C,IF",
    "bat_throw": "R/R"
  }
  ```

- Linux application data directory moved from:

  ```text
  ~/.bs_scorer
  ```

  to the XDG-compliant path:

  ```text
  ~/.local/share/bs_scoring
  ```

  or:

  ```text
  $XDG_DATA_HOME/bs_scoring
  ```

  when `XDG_DATA_HOME` is defined.

- Database filenames were renamed from `baseball_scorer*.db` to `bs_scoring*.db`.

### Fixed

- Existing Linux databases are migrated automatically from the legacy application directory to the new XDG-compliant data directory.
- Legacy player `pitch` values are migrated from `RHP/LHP/SHP` to `R/L/S`.
- Legacy numeric player positions are migrated to roster-position notation.
- Home and away jersey numbers are now both unique per team.
- Player import/export is now aligned with the new player model.

### Migration Notes

- Existing databases are migrated automatically on startup.
- Legacy JSON imports using `pitch` are still accepted for compatibility and converted to `throw` internally.
- `bat_throw` is the preferred import/export format from v0.12.0 onward.
- The lineup defensive positions used during Play Ball remain unchanged: `1`–`9`, plus `DH` when applicable.

---

## [0.11.5] - 2026-06-04

### Added

- Added downloadable CSV import template for players.
- Added downloadable JSON import template for players.
- Included sample records and required fields in generated templates.

### Fixed

- Added support for teams without a league in player edit/delete workflows.
- JSON import now rejects invalid non-integer `away_number` values instead of silently defaulting to the home jersey number.
- Player selection in Play Ball lineup entry now uses roster index/player ID instead of jersey number where needed, avoiding ambiguity with duplicate away jersey numbers.
- Lineup import now detects ambiguous duplicate display jersey numbers and reports a validation error instead of overwriting players in the roster map.

---

## [0.11.4] - 2026-06-04

### Fixed

- Play Ball now uses the correct jersey number based on team side.
- Home teams use the home jersey number.
- Away teams use the away jersey number.
- Fixed lineup and roster rendering when away jersey numbers differ from home jersey numbers.

---

## [0.11.3] - 2026-06-03

### Added

- Added separate home and away jersey numbers for player creation.
- Added `away_number` support to player CSV/JSON import and export.
- Added support for jersey number `0`.

### Changed

- If the away jersey number is left blank, it defaults to the home jersey number.
- Refactored player creation internals to support the expanded player model.

---

## [0.11.2] - 2026-06-03

### Changed

- Updated Player Management edit/delete workflows: users now select a league first, then a team from that league, then manage only players from that team.
- Added a `No league` path for teams with `league_id = NULL`.
- After editing or deleting a player, the refreshed team roster is shown again until the user selects `0`.
- Pressing ENTER on the player-selection prompt behaves like `0` and goes back.

---

## [0.11.1] - 2026-04-21

### Added

- Added `engine::commands::kind` with `CommandKind`, `CommandFamily`, `CommandKind::family()`, and `CommandKind::canonical_name()`.

### Changed

- Refactored the scoring-command vocabulary around a single `CommandKind` enum.
- Removed parallel sub-enums from token, segment, and validator layers.
- Simplified lexer recognition for parameter-less verbs.
- Improved TUI scoreboard highlighting, count styling, outs indicator, and linescore layout.
- Bumped version from `0.11.0` to `0.11.1`.

### Notes

- Internal refactor only: accepted grammar, engine behavior, and public API remain unchanged.

---

## [0.11.0] - 2026-04-21

### Added

- First stable release of the v0.11.0 milestone.
- Consolidated the v0.11.0-alpha1 structural refactor and v0.11.0-alpha2 grammar refactor.

### Changed

- Rebuilt the scoring-command parser on a two-stage pipeline:
  - stateless grammar layer;
  - state-aware validator.
- Reorganized `src/`:
  - `core/` absorbed into `engine/`;
  - top-level `commands/` moved under `engine/commands/`;
  - `cli/commands/` renamed to `cli/screens/`;
  - `utils/cli.rs` renamed to `utils/term.rs`;
  - `ui/cli.rs` renamed to `ui/cli_impl.rs`.

### Fixed

- Fixed composite defensive-play state consistency.
- Fixed fielder's-choice scoring to home.
- Fixed steal/action mixing validation.
- Fixed fielding-sequence lexer accepting fielder `0`.
- Fixed resumed-game double scoring on walk/hit movements.
- Fixed inning-bucket updates for HOME composite and steal movements.
- Fixed cross-half steal-home scoring.

---

## [0.10.6] - 2026-04-16

### Added

- Added support for defensive out commands without explicit batting-order prefix for the current batter.
- Added support for unassisted outs.
- Added support for fielder's choice commands with explicit destination base.
- Added support for composed defensive-play commands.
- Added resume support for new defensive-play outcomes.
- Added command history recall in the TUI Command input.

### Changed

- Updated defensive-play engine flow and replay handling.
- Reworked TUI Command panel behavior.
- Updated `SCORING_GUIDE.md`.

### Fixed

- Fixed defensive-play parsing, resume/replay, scoreboard restoration, and live scoreboard updates for several defensive outcomes.
- Fixed inning-by-inning score updates for stolen home.
- Fixed infield-fly validation.

---

## [0.10.5-bugfix] - 2026-04-13

### Fixed

- Fixed handling of nullable `game_time` in game lookup.
- Prevented failures when loading legacy games with NULL time values.
- Ensured matchup/date/venue are correctly displayed and exported for migrated data.
- Removed unused variable warnings in umpire evaluation summary rendering.

---

## [0.10.5] - 2026-04-12

### Added

- Enhanced umpire evaluation summary view with game date, game time, and venue.

### Changed

- Refactored game lookup cache using a `GameInfo` struct.
- Improved CLI rendering for umpire reports.

---

## [0.10.4] - 2026-04-12

### Added

- Added CSV and JSON export for umpire evaluation reports.
- Added interactive umpire history navigation.

### Changed

- Reused league-based umpire filtering workflow for report export.
- Improved umpire evaluation data access and summary rendering.

### Fixed

- Fixed several umpire history and report rendering issues.

---

## [0.10.3] - 2026-04-10

### Changed

- Refactored umpire history handling into helper functions.
- Improved interactive browsing of umpire evaluations.

### Fixed

- Fixed selection consistency in filtered umpire lists.

---

## [0.10.2] - 2026-04-09

### Added

- Added batter-out scoring commands.
- Added support for multiple defensive assists.
- Added new plate-appearance outcomes and sequence steps.

### Changed

- Extended Play Ball command parsing and replay/state rebuild logic.

### Fixed

- Fixed scoreboard, DB persistence, resume rendering, and pitcher stats for batter-out outcomes.

---

## [0.10.1] - 2026-03-23

### Added

- Added Umpire Supervisor module.
- Added umpire CRUD, game assignments, evaluations, and career statistics.

---

## [0.10.0] - 2026-03-20

### Changed

- Major architecture refactor.
- Improved database access and runner logic.

---

## [0.9.x]

### Notes

- Earlier 0.9.x releases introduced the TUI Help panel, focus system, runner movement persistence, steal command support, and module split groundwork.
