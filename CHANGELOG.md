# Changelog
## [0.11.3] - 2026-06-03

### Added
- Added separate home and away jersey numbers for player creation.
- If the away jersey number is left blank, it defaults to the home jersey number.

### Changed
- Jersey number validation now accepts `0` as a valid jersey number.
- Player CSV/JSON export now includes `away_number`; import remains backward-compatible with previous player formats.

## [0.11.2] - 2026-06-03

### Changed
- Updated Player Management edit/delete workflows: users now select a league first, then a team from that league, then manage only players from that team.
- After editing or deleting a player, the refreshed team roster is shown again until the user selects `0`.
- Pressing ENTER on the player-selection prompt now behaves like `0` and goes back.

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.11.1] - 2026-04-21

Internal refactor of the command vocabulary plus a scoreboard UX polish.
No change to the grammar accepted by the parser or to the engine
behaviour. Public API via `bs_scoring::*` is unchanged.

### Changed

- **Command-taxonomy refactor (internal).** The scoring-command
  pipeline now has a single source of truth for the list of verbs it
  accepts: the new `engine::commands::kind::CommandKind` enum, a flat
  26-variant enumeration with one variant per lexical verb shape, paired
  with a `CommandFamily` grouping. The parallel tag sub-enums that used
  to live in each pipeline layer (`HitVerbKind`, `PitchVerbKind`,
  `KeywordKind` in `tokens.rs`; `ControlKind`, `StatusKind`, `PitchKind`,
  `HitKind` in `segment.rs`) have been removed and their callers
  rewritten to reference `CommandKind` directly.
    - `TokenKind` collapses four of its variants (`HitVerb`,
      `PitchVerb`, `StealVerb`, `Keyword`) into a single
      `TokenKind::Verb(CommandKind)` carrying the parameter-less verb's
      kind.
    - `Segment::Control`, `Segment::Status`, `Segment::Pitch`, and
      `Segment::Hit` now carry `CommandKind` directly instead of a
      dedicated sub-enum.
    - `BatterOutKind` is kept (its variants carry fielder identifiers,
      not just a tag) and gains a `.command_kind()` helper that maps
      each variant to the corresponding `CommandKind`. Notably,
      `FlyOut { foul: true }` maps to `CommandKind::FoulFlyOut`, a
      distinct lexical verb.
    - The grammar's segment dispatcher now branches on
      `CommandKind::family()` rather than on the removed sub-enums.
    - `validator.rs` `Resolved::Pitch` / `Resolved::Hit` carry
      `CommandKind`; the hit coalescer and the `status_to_game` /
      `pitch_to_engine` helpers match on `CommandKind` with an
      `unreachable!()` catch-all that documents the family invariant.
- **Lexer micro-optimisation.** Removed three regexes
  (`RE_HIT_VERB`, `RE_PITCH_VERB`, `RE_STEAL_VERB`) that became dead
  weight after the refactor. Parameter-less verbs (hit, pitch, steal,
  control, status) are now recognised by a single exact-match `match`
  on lowercased text — faster and more readable than carrying a regex
  per one-letter keyword. Regexes retained only for verbs with numeric
  parameters (`o<n>`, `f<n>`, `l<n>`, `if<n>`, fielding sequences).
- **Scoreboard UX.** TUI scoreboard rewritten with dynamic highlighting
  and refined layout:
    - Batting team row highlighted with yellow + bold and a subtle
      left-edge marker so the active half-inning is obvious at a
      glance.
    - Linescore header reworked to render with `Line` / `Span` instead
      of a flat string, with the current inning emphasised via
      reversed style.
    - Balls / strikes count styled dynamically: a full count (3-2)
      renders reversed + bold, critical counts (3-1, 2-2) render
      yellow + bold. New helper `styled_count_span`.
    - Outs indicator now shows two dots (`○` / `●`) instead of three,
      with active outs in yellow + bold and inactive outs in dark
      gray. New helper `styled_outs_spans`.
    - Status line re-rendered with mixed styled spans and proper
      centring; the redundant inning indicator (`"4↑"`) removed.
    - Better spacing and alignment across scoreboard components.
- Bumped version from `0.11.0` to `0.11.1` in `Cargo.toml` and
  `Cargo.lock`.

### Added

- New public module `src/engine/commands/kind.rs` with `CommandKind`,
  `CommandFamily`, `CommandKind::family()`, and
  `CommandKind::canonical_name()`. The module is the single documented
  reference for anyone asking "which commands does the parser accept?"
  and the place to add new commands in future releases.

### Notes

- This release is an internal refactor. No change to accepted grammar,
  engine behaviour, diagnostic messages, or on-disk data. The 83 unit
  tests of the command pipeline (`tokens`, `segment`, `grammar::mod`,
  `validator`, `parser` facade) pass unchanged; 4 new tests in
  `kind.rs` verify the invariants of the new taxonomy.

---

## [v0.11.0] - 2026-04-21

First stable release of the v0.11.0 milestone. Promotes `v0.11.0-alpha2`
to final after the post-alpha fixes landed on
`v0.11.0-alpha2-fix_codex` (issues #55, #56, #59, #60, #61, #62, #64,
#66) were verified green. No functional code changes between
`v0.11.0-alpha2` + fixes and this release — the `v0.11.0` tag exists to
mark a clean, production-ready stopping point on the v0.11.0 line.

The highlights of the milestone, aggregated across alpha1 + alpha2 +
fixes:

- **Grammar refactor** (alpha2) — scoring-command parser rebuilt on a
  two-stage pipeline: stateless regex-assisted syntactic layer followed
  by a state-aware validator. Every error in a line is now reported at
  once with its 1-based segment index. Segments are order-independent.
- **Structural refactor** (alpha1) — `src/` reorganised: `core/` absorbed
  into `engine/`, top-level `commands/` moved under `engine/commands/`,
  `cli/commands/` renamed to `cli/screens/`, several anti-homonym
  renames. Public API via `bs_scoring::*` unchanged.
- **Composite-play state consistency** (#55) — live application and
  deterministic replay now converge on the same `GameState` for every
  composite defensive play, matching the `runner_movements` rows
  persisted to the DB.
- **FC-to-home scoring** (#56) — a batter reaching home directly on a
  fielder's choice now correctly credits the run.
- **Grammar and replay-path polish** (#59, #60, #61, #62, #64, #66) —
  steals can no longer be combined with end-of-PA actions; invalid
  fielder 0 in a fielding sequence is rejected at lex time; resumed
  games no longer double-apply walk / hit movements; HOME composite and
  steal-home movements correctly increment per-inning buckets and
  credit the right team when crossing half-innings.

### Added

- Nothing new beyond what already shipped in `v0.11.0-alpha2` and its
  follow-up fixes.

### Changed

- Version bumped from `0.11.0-alpha2` to `0.11.0` in `Cargo.toml` and
  `Cargo.lock`.
- Headers in `README.md`, `SCORING_GUIDE.md`, `STRUCTURE.md`, and
  `RELEASE.md` updated to reflect the final version.

---

## [v0.11.0-alpha2] - 2026-04-20

Second alpha of the v0.11.0 milestone. Ships the scoring-command grammar
refactor: the parser is rebuilt on top of a formal grammar with regex-assisted
lexical recognition, split into a stateless syntactic pass and a
state-dependent validator. Every error the line contains is now surfaced with
its segment index, rather than the parser stopping at the first problem.

### Added

- **`regex` crate dependency** (`regex = "1.11.1"`) for lexical token
  recognisers (fielding sequences, verb shapes, base / zone tags).
- **`engine/commands/errors.rs`** — new public types for diagnostics:
    - `ParseError` (empty segment, missing / disallowed subject, unknown verb,
      missing / invalid object, extra tokens, invalid fielding sequence, …)
    - `ValidationError` (batter-slot mismatch, runner not on base,
      advance without trigger, duplicate subject, infield-fly preconditions,
      too-many-outs, structural conflicts)
    - `CommandError` — wraps either kind with a 1-based segment index and the
      original segment text for `Display`.
- **`engine/commands/grammar/`** — stateless syntactic layer:
    - `tokens.rs` — `LazyLock<Regex>` patterns and a `TokenKind` classifier for
      every lexical shape (`Digit`, `HitVerb`, `PitchVerb`, `StealVerb`,
      `FcVerb`, `FlyVerb`, `LineVerb`, `InfieldFlyVerb`, `FieldingSeq`,
      `Zone`, `Base`, `Keyword`).
    - `segment.rs` — `Segment` enum (`Control`, `Status`, `Pitch`, `Hit`,
      `BatterOut`, `RunnerOut`, `FielderChoice`, `Steal`, `Advance`) and
      `parse_segment()` / `parse_line()` entry points.
- **`engine/commands/validator.rs`** — state-aware validation and coalescing.
  `validate(Vec<IndexedSegment>, &GameState)` accumulates every error and,
  on success, folds hit + advances into a single `EngineCommand::Single` /
  `Double` / `Triple` / `HomeRun`, defensive outs into a single
  `DefensivePlay`, and keeps pitches and steals as independent commands in
  their original segment order.
- **`engine/commands/parser.rs` facade** — `parse_engine_commands(line, &state)`
  composes the grammar and validator into a single entry point returning
  `Result<Vec<EngineCommand>, Vec<CommandError>>`.
- 83 new unit tests: 15 in `tokens`, 30 in `segment`, 7 in `grammar::mod`,
  24 in `validator`, 7 in the `parser` facade — covering happy paths,
  error diagnostics, order invariance, infield-fly preconditions, and every
  structural conflict rule.

### Changed

- **Subject-always grammar.** The batting-order subject is mandatory on every
  action segment, with one documented exception: verbs whose shape cannot be
  confused with a lone digit (hit verbs, multi-character batter-outs, FC) may
  omit the subject, which then defaults to the current batter. Single-digit
  unassisted (`5`), steals (`st`), and standalone runner-advances (`<n> <base>`)
  always require an explicit subject. Pitches and control / status keywords
  never accept a subject.
- **Order-independent segments.** Composite defensive plays, runner overrides
  after a hit, and mixed lines can be typed in any order. For example,
  `5 l6, 3 64, 4 43` / `3 64, 5 l6, 4 43` / `4 43, 5 l6, 3 64` all produce
  the same triple play.
- **Accumulated error reporting.** When a line contains multiple problems
  (syntactic or semantic), every one is emitted in a single pass as
  `error at segment N: '<text>': <reason>`, rather than the parser stopping
  at the first.
- **Game-loop wiring.** `engine::play_ball` now consumes the `Result`
  directly, emitting one `UiEvent::Error` per `CommandError` when the line
  fails validation.
- Subject checks now cross-reference the current batter slot. `5 h` when the
  current batter is `#8` is rejected with
  `batter slot #5 does not match current batter #8`.
- Runner-targeted segments (`<n> 64`, `<n> st 2b`, `<n> 2b`) now verify that
  the runner is actually on base. If the subject coincides with the current
  batter, a runner-out segment is silently demoted to a batter-out.
- Runner-advance overrides (`<n> <base>`) now require a triggering play
  (hit or FC) on the same line; a bare advance with no trigger is rejected.

### Fixed

- **FC-to-home run credit (#56)** — `apply_batter_fielders_choice` was
  a no-op for `RunnerDest::Score`, so a command like `5 o6 sc` recorded
  the plate appearance with `scored=true` in `runner_movements` but
  left `GameState.score` unchanged. The live scoreboard lagged by one
  run and deterministic replay reproduced the same error. Fix adds the
  run to the batting team's total and inning partial; base occupancy
  stays untouched (batter never was on base). Covered by 6 unit tests
  in `engine::apply::tests`.
- **Composite defensive-play state drift (#55)** — plays with explicit
  runner segments (runner outs, FC-safe advances on runners, triple
  plays) persisted correctly to `runner_movements` but the in-memory
  `GameState` did not mirror them. For instance `9 64, 1 o6 1b` with
  runner #9 on 1B and batter #1 at the plate produced
  `on_1b=Some(1), on_2b=Some(9)` — the tagged-out runner was
  erroneously "forced" to 2B by `apply_batter_fielders_choice`. The
  bug affected both live application and deterministic replay, and
  spanned every composite play shape, not just FC. Fix applies the
  same semantics to state that `runner_movements` records: clear every
  runner-out from its base, then place every FC-safe advance on its
  destination (crediting a run for `HOME`). The reducer's
  `FieldersChoice` branch was stripped to avoid double-applying the
  batter's placement on replay. Covered by 4 unit tests in
  `engine::apply::tests`.
- **Steal + end-of-PA action mixing not rejected (#59)** — `check_mixing`
  only blocked pitch + action combinations; a line such as `5 h, 3 st 2b`
  was silently accepted, causing `build_commands` to emit both a hit and a
  steal command and produce incorrect runner state. Added an explicit
  `has_steal && has_end_of_pa_action` guard that rejects any steal
  combined with a hit, batter-out, runner-out, fielder's-choice, or
  advance segment, and renamed the helper variable to the more descriptive
  `has_end_of_pa_action`. Covered by 3 new unit tests in
  `engine::commands::validator::tests` (`hit_and_steal_rejected`,
  `batter_out_and_steal_rejected`, `fc_and_steal_rejected`).
- **Fielding-sequence lexer accepts illegal fielder 0 (#60)** — both
  `RE_FIELDING_SEQ_COMPACT` (`^\d{2,}$`) and `RE_FIELDING_SEQ_DASHED`
  (`^\d(-\d)+$`) matched digits 0–9, so inputs like `60` or `6-0` were
  classified as valid `FieldingSeq` tokens and reached command-building
  with a generic error instead of a segment-specific parse error. Changed
  both patterns to `[1-9]` (`^[1-9]{2,}$` / `^[1-9](-[1-9])+$`). Also
  updated the vocabulary table in the module doc-comment. Covered by 1 new
  unit test `fielding_sequence_with_zero_is_unknown` in
  `engine::commands::grammar::tokens::tests`.
- **Resumed-game double-scoring on walk / hit movements (#61)** —
  `run_play_ball_engine` partitioned every non-steal `runner_movements`
  row into `composite_movements` and re-applied them through
  `apply_composite_state` during replay. This unintentionally re-applied
  `walk`, `hit_auto`, and `hit_override` rows that `apply_plate_appearance_row`
  had already applied, causing runs to be counted twice in resumed games.
- **Exclude hit and walk rows from composite replay pass (#62)** — fix
  for #61. The partition now uses an explicit whitelist of
  composite/defensive types (`ground_out`, `fly_out`, `line_out`,
  `infield_fly`, `unassisted_out`, `fielders_choice`) and discards all
  normal-PA movement rows (`hit_auto`, `hit_override`, `walk`, …),
  which are applied exclusively by `apply_plate_appearance_row`. This
  makes the replay pipeline authoritative on a row's `advancement_type`
  for deciding which path applies it, eliminating the double-apply.
- **Inning-bucket not incremented for HOME composite/steal movements (#64)** —
  Both `apply_composite_state` and `apply_steal_state` in the replay path
  incremented `state.score.away` / `state.score.home` directly via
  `saturating_add`, bypassing `add_runs_to_score`. As a result the
  per-inning `away_innings` / `home_innings` buckets were never updated,
  making the inning-by-inning line in the TUI inconsistent with the grand
  total after any FC-safe scoring or stolen-base-to-home in a resumed game.
  Both closures now call `add_runs_to_score(state, 1)`, which updates both
  the total and the correct inning bucket atomically, matching the live path.
- **Wrong team/inning bucket for steal-home across half-inning boundary (#66)** —
  After fix #62, both `apply_steal_state` and `apply_composite_state` called
  `add_runs_to_score(state, 1)`, which credits the run using `state.half` and
  `state.inning`. However, steals are linked to the last committed PA via
  `rm.pa_seq`, so a steal-home that occurred during the first at-bat of a new
  half could be replayed while `state.half` / `state.inning` still reflected the
  previous half-inning. The run was credited to the wrong team and inning bucket.
  Both closures now temporarily override `state.half` / `state.inning` with the
  values recorded in the `runner_movements` row (`rm.half_inning`, `rm.inning`)
  before calling `add_runs_to_score`, then restore the original state values.
  This makes the replay authoritative on the row's recorded context rather than
  the current traversal position.

### Removed

- **`EngineCommand::Unknown(String)`** variant — the new pipeline surfaces
  parse failures as `CommandError` values via the `Result` return type, so
  the catch-all string variant is no longer needed. The corresponding
  fallback branch in `engine::apply` has been deleted.
- Legacy ad-hoc parsers inside `engine/commands/parser.rs`
  (`parse_non_hit_command`, `parse_legacy_batter_out_command`,
  `parse_batter_token`, the standalone `parse_defensive_play_command` path).
  Their responsibilities are now covered by the grammar + validator pipeline.

### Migration notes

External consumers of `engine::commands::parser::parse_engine_commands` must
update their call sites:

```rust
// Before (v0.11.0-alpha1 and earlier):
let commands: Vec<EngineCommand> = parse_engine_commands(line);
for cmd in commands {
if let EngineCommand::Unknown(msg) = &cmd { /* handle */ }
/* … */
}

// After (v0.11.0-alpha2):
let commands = match parse_engine_commands(line, & state) {
Ok(cmds) => cmds,
Err(errors) => {
for err in errors {
ui.emit(UiEvent::Error(err.to_string()));
}
continue;
}
};
for cmd in commands { /* … */ }
```

Input strings that used to parse (possibly resulting in `EngineCommand::Unknown`)
but do not follow the subject-always grammar will now be rejected at input
time. In particular:

- `st 2b`, `2b`, `5 b` → now errors, previously handled in ad-hoc ways
- `5` alone → now an error; use `<n> 5` (e.g. `5 5`) for a batter unassisted
  by the third baseman

---

## [v0.11.0-alpha1] - 2026-04-20

First alpha of the v0.11.0 milestone. This alpha ships the structural refactor
of `src/` only; additional features scheduled for the v0.11.0 final release
will be delivered in subsequent alphas.

### Changed

- **Structural refactor** of `src/` for improved readability and maintainability.
  Public runtime behaviour is unchanged; the public API exposed by `lib.rs`
  preserves the same re-exports.
- Collapsed `src/core/` into `src/engine/`: game-logic files now live together.
    - `core/runner_logic.rs` → `engine/runners.rs`
    - `core/play_ball_apply.rs` → `engine/apply.rs`
    - `core/play_ball_reducer.rs` → `engine/reducer.rs`
    - `core/parser.rs` → `engine/notation.rs`
    - `core/scoring/` → `engine/scoring/`
- Removed the top-level `src/commands/` module (ambiguous with `src/cli/commands/`).
  Its contents were moved under `src/engine/commands/`:
    - `commands/engine_parser.rs` → `engine/commands/parser.rs`
    - `commands/types.rs` → `engine/commands/types.rs`
- Renamed `src/cli/commands/` → `src/cli/screens/` — these files are user-flow
  screens (new-game, list-games, play-ball, umpire-supervisor, …), not engine
  commands. This removes the clash with `engine/commands/`.
- Moved `core/menu.rs` → `cli/menu.rs` — menu-choice enums belong to the CLI
  layer.
- Renamed to remove module-name homonyms:
    - `utils/cli.rs` → `utils/term.rs` (terminal helpers)
    - `ui/cli.rs` → `ui/cli_impl.rs` (`Ui` trait CLI implementation)

### Removed

- `src/core/play_ball.rs` — deprecated compatibility re-export since v0.8.1,
  all callers already used `db::game_queries` directly.
- `src/models/play_ball.rs` — deprecated compatibility re-export since v0.8.1,
  all callers already used `models::game_state`, `models::runner`,
  `models::session` directly.

### Migration notes

External consumers that were still importing the compatibility paths removed in
this release need to update their `use` statements. The canonical paths are:

- `crate::core::*` → `crate::engine::*`
- `crate::commands::engine_parser` → `crate::engine::commands::parser`
- `crate::commands::types` → `crate::engine::commands::types`
- `crate::cli::commands::*` → `crate::cli::screens::*`
- `crate::core::menu::*` → `crate::cli::menu::*`
- `crate::utils::cli` → `crate::utils::term`
- `crate::ui::cli` → `crate::ui::cli_impl`

The top-level re-exports from `bs_scoring::*` (`Database`, `Menu`, `GameState`,
`Player`, `Team`, `League`, scoring types, …) are unchanged.

## [v0.10.6] - 2026-04-16

### Added

- Added support for defensive out commands without explicit batting-order prefix for the current batter:
    - `63`
    - `5`
    - `f9`
    - `ff2`
    - `l6`
    - `if4`
- Added support for unassisted outs (`UnassistedOut`) in parser, engine, persistence, resume, and TUI rendering.
- Added support for fielder's choice commands with explicit destination base:
    - `<order> o<n> <base>`
    - examples: `1 o6 1b`, `1 o5 2b`
- Added support for composed defensive-play commands, comma-separated, including out + fielder's choice combinations.
- Added resume support for new defensive-play outcomes:
    - ground out
    - fly out
    - foul fly out
    - line out
    - infield fly
    - unassisted out
    - fielder's choice
- Added command history recall in the TUI Command input using arrow navigation when Command has focus.

### Changed

- Updated defensive-play engine flow to normalize explicit runner targets that match the current batter into
  batter-target outcomes.
- Updated `apply_defensive_play_command()` to build consistent plate appearances, runner movements, and UI log messages
  for composed defensive plays.
- Updated `apply_plate_appearance_core()` to replay `FieldersChoice` and `UnassistedOut` correctly during
  resume/reconstruction.
- Updated plate-appearance DB serialization/deserialization to support all newly introduced defensive outcomes.
- Updated TUI help panel to document the currently supported command set.
- Updated `SCORING_GUIDE.md` to reflect the actual v0.10.6 grammar and feature set.
- Reworked TUI Command panel behavior:
    - kept it as a single-line input
    - added previous-command recall instead of multi-line visual history
    - preserved Log/Help scrolling with focus switching via `Tab`

### Fixed

- Fixed defensive-play parsing for commands like `63`, which were previously misread as invalid explicit batting-order
  prefixes.
- Fixed resume/replay so `FieldersChoice` no longer falls back to generic `Out`.
- Fixed scoreboard restoration after resume for fielder's-choice outcomes.
- Fixed live scoreboard updates for batter fielder's choice with forced runner advancement.
- Fixed inning-by-inning score updates for stolen home (`<order> st sc`) so both total score and inning partial score
  are updated consistently.
- Fixed resume consistency for stolen-home scoring.
- Fixed infield-fly validation so `IF` is accepted only in valid rule situations:
    - fewer than 2 outs
    - runners on 1B and 2B
- Fixed parser and engine support for legacy single-fielder outs such as `8 5`.
- Fixed defensive-play handling so explicit batter references like `1 o6 1b` are recognized correctly when the current
  batter is `#1`.

### Notes

- The current command grammar has grown significantly during v0.10.x.
- A broader grammar/parser refactor is planned for `v0.11.0-alpha1`, with a cleaner canonical command model and
  regex-assisted parsing.

---

## [0.10.5-bugfix] - 2026-04-13

### 🐛 Fixes

- Fix handling of nullable `game_time` in game lookup.
- Prevent failure when loading legacy games with NULL time values.
- Ensure matchup/date/venue are correctly displayed and exported for migrated data.
- Fixed unused variable warnings in umpire evaluation summary rendering.
- Removed unnecessary extraction of `game_time` and `venue` in summary view.
- Introduced dedicated helper for summary rendering to avoid unused data.

### 🧠 Refactor

- Added `extract_game_summary_info()` helper for lightweight game data access in summary view.
- Kept `extract_game_info()` for full-detail contexts (detail view and export).
- Removed explicit lifetimes in helper function signatures where they could be safely elided.

### 🎯 UX Improvements

- Simplified summary table layout for better readability in CLI.
- Reduced visual clutter by limiting summary data to essential fields.

---

## [0.10.5] - 2026-04-12

### ✨ Enhancements

- Enhanced umpire evaluation summary view:
    - Added game date (`game_date`)
    - Added game time (`game_time`)
    - Added venue (`venue`)
- Summary table now provides full contextual information for each evaluation:
    - matchup (`away_team @ home_team`)
    - date and time of the game
    - venue

### 🧠 Refactor

- Refactored game lookup cache:
    - replaced `HashMap<i64, String>` with `HashMap<i64, GameInfo>`
- Centralized access to game metadata via `GameInfo` struct
- Removed need for duplicated formatting logic across UI functions

### 🎯 UX Improvements

- Improved readability of CLI tables for umpire reports
- Reduced need for opening detailed view just to identify a game
- More professional and complete report visualization

---

## [0.10.4] - 2026-04-12

### Added

- Added `Export Umpire Reports` option to the Umpire Supervisor menu.
- Added CSV and JSON export for umpire evaluation reports.
- Export now includes all fields from `umpire_evaluations` plus derived `matchup` (`away_team @ home_team`).
- Export filenames now follow the format `<umpire-name>-<timestamp>.<csv|json>`.
- Export now prompts the user to choose the destination directory before writing CSV/JSON files.
- Export now includes game date time and venue for each evaluation report.
- Removed internal identifiers (`game_id`, `umpire_id`) from exported output.

### Improved

- Reused league-based umpire filtering workflow for report export.
- Improved umpire evaluation data access by enriching exported rows with matchup information.

### ✨ Enhancements

- Improved umpire evaluation summary view:
    - Added `Matchup` column (`away_team @ home_team`)
    - Better column alignment and readability in CLI output
- Added interactive navigation in umpire history:
    - Ability to view detailed report per game
    - Clean menu loop with exit on empty input / `E` / `X`

### 🧠 Refactor

- Introduced `GameInfo` struct for type-safe game retrieval
- Refactored `get_game_by_id` to return `Result<Option<GameInfo>>`
- Replaced tuple-based DB results with structured data
- Optimized matchup resolution using `HashMap<i64, String>` cache
- Eliminated redundant umpire fetch logic via helper (`fetch_umpire_or_notify`)
- Split UI logic into reusable helpers:
    - `print_umpire_header`
    - `print_umpire_evaluation_summary`
    - `print_umpire_evaluation_detail`

### 🐛 Fixes

- Fixed incorrect handling of `Result<Option<T>>` when loading game data
- Prevented potential panic / invalid access when game is not found
- Fixed duplicated DB lookups in umpire history workflow
- Corrected CLI rendering misalignment after adding matchup column

### 🎯 UX Improvements

- Improved CLI navigation flow in umpire history section
- Consistent handling of invalid input and empty selections
- Added fallback values (`"-"`) for missing game data

### 🚧 Notes

- Current implementation still uses N+1 queries for game lookup
  (planned optimization: batch query with `IN (...)`)

---

## [v0.10.3] - 2026-04-10

### Improved

- Refactored `handle_umpire_history()` to separate concerns using helper functions:
    - `print_umpire_header`
    - `print_umpire_evaluation_summary`
    - `print_umpire_evaluation_detail`
- Improved CLI usability when browsing umpire evaluations:
    - added interactive menu after summary table
    - users can now select a specific report by `Game ID`
    - textual evaluation fields are now accessible:
        - strengths
        - areas to improve
        - notes
- Enforced selection consistency: umpire must belong to the filtered league list.

### Fixed

- Fixed logical inconsistency where user could input an umpire ID not present in the filtered list.

---

## [v0.10.2] - 2026-04-09

### Added

- Added support for batter-out scoring commands in the Play Ball engine:
    - ground out with compact and hyphenated defensive sequences:
        - `63`
        - `6-3`
        - `862`
        - `8-6-2`
    - fly out:
        - `F<n>`
    - foul fly out:
        - `FF<n>`
    - line out:
        - `L<n>`
    - infield fly:
        - `IF<n>`
- Added support for multiple defensive assists in batter-out plays through fielding sequences.
- Added new `PlateAppearanceOutcome` variants for batter-out outcomes:
    - `GroundOut`
    - `FlyOut`
    - `LineOut`
    - `InfieldFly`
- Added new `PlateAppearanceStep` variants for batter-out terminal events:
    - `GroundOut`
    - `FlyOut`
    - `LineOut`
    - `InfieldFly`

### Changed

- Extended Play Ball engine command parsing to recognise batter-out commands with batting-order prefix.
- Updated live command application to process batter-out plays as completed plate appearances.
- Updated replay/resume rendering so batter-out plays are shown with specific labels instead of generic `OUT`.
- Improved `PlateAppearanceStep` display formatting to keep the pitch sequence compact:
    - `GO`
    - `FO`
    - `FFO`
    - `LO`
    - `IFF`
- Refactored plate appearance sequence construction so the terminal play step is appended consistently across:
    - pitch-based completed PA
    - hit outcomes
    - batter-out outcomes
- Updated replay/state rebuild logic to understand the new batter-out outcomes.
- Updated pitcher pitch/stat recount logic so batter-out plays in play correctly contribute the terminal pitch in both:
    - replay/recount mode
    - live application mode

### Fixed

- Fixed missing scoreboard updates after batter-out commands:
    - outs are now incremented correctly
    - count is reset correctly
    - next batter is started correctly
- Fixed missing DB persistence for batter-out plate appearances.
- Fixed resume output showing generic `OUT` for batter-out plays restored from DB.
- Fixed missing terminal plate-appearance step in persisted `pitches_sequence` for batter-out outcomes.
- Fixed live pitcher stats not counting the terminal pitch for:
    - ground out
    - fly out
    - foul fly out
    - line out
    - infield fly
- Fixed lowercase parsing support for batter-out commands such as:
    - `f9`
    - `ff3`
    - `l6`
    - `if4`

### Internal

- Centralized plate appearance sequence finalization to reduce duplicated logic.
- Kept legacy generic `Out` handling for backward compatibility with existing data.

---

## [0.10.1] - 2026-03-23

### Added

- **Umpire Supervisor module** — new top-level menu (voce 6, DB scala a 7) with
  four sub-functions:
    - **Manage Umpires (CRUD)**: add, list, edit, delete umpires with fields for
      first/last name, license number, level/classification, email, phone, notes.
      Edit shows full umpire list before prompting for ID.
    - **Assign Umpires to Game**: game selection via same picker used by Play Ball
      (excludes Regulation/Cancelled/Forfeited). Configurable crew size (2, 3, 4,
      or 6 umpires) with positions HP, 1B, 2B, 3B, LF, RF. Shows current
      assignments before editing. `UNIQUE(game_id, position)` prevents
      double-booking.
    - **Evaluate Game (Report Card)**: game selection via shared picker.
      Per-umpire evaluation form with 8 scored categories (1–10 scale): strike
      zone accuracy (HP only), safe/out accuracy, positioning, timing, game
      management, professionalism, communication, hustle. Computes average
      automatically; supervisor can override with manual overall. Free-text fields
      for strengths, areas to improve, and notes. Evaluation box dynamically sized
      to fit umpire name and position.
    - **Umpire History / Statistics**: career overview per umpire — tabular display
      of all evaluations with per-category scores and career average.

- **Umpire ↔ League association** (N:N):
    - Migration v18: `umpire_leagues` junction table with
      `PRIMARY KEY (umpire_id, league_id)`
    - During umpire creation: displays all registered leagues, user selects one or
      more by comma-separated IDs (e.g. `1,3`); validates that IDs exist
    - During umpire edit: shows current leagues, allows re-selection
    - Umpire list shows associated league names in a dedicated column
    - `set_umpire_leagues()`, `get_umpire_leagues()`, `add_umpire_league()`,
      `remove_umpire_league()` DB functions

- **Database schema v17–v18**:
    - v17: `umpires`, `game_umpires`, `umpire_evaluations` tables with indexes
    - v18: `umpire_leagues` junction table with indexes

- **`UmpirePosition` enum** — `HP`, `1B`, `2B`, `3B`, `LF`, `RF` with
  `crew(size)` method returning the appropriate positions for 2/3/4/6-man crews

- **`UmpireSupervisorMenuChoice` enum** — `ManageUmpires`, `AssignToGame`,
  `EvaluateGame`, `UmpireHistory`, `Back`

- **Utility helpers**: `read_i64_required()` (retry loop),
  `read_string_with_default()`, `read_optional_string_with_default()`

- **Shared game picker** (`select_game()`): reusable function that lists
  playable games with the same filter as Play Ball and returns the selected
  `PlayBallGameContext`

### Changed

- **Main menu** reordered: Umpire Supervisor = voce 6, Manage DB = voce 7
  (was 6). Prompt updated to `(1-7 or 0)`.
- `MainMenuChoice` enum gains `UmpireSupervisor` variant
- `lib.rs` re-exports `UmpireSupervisorMenuChoice`

---

## [0.10.0] - 2026-03-22

### Added

- **`core/runner_logic.rs`** — new module that unifies all runner advancement
  logic into a single source of truth. Provides:
    - `apply_hit()` — hit advancement with optional per-runner overrides,
      returns `HitResult` containing both state mutations and
      `RunnerMovementInsert` rows for DB persistence
    - `apply_walk()` — walk/BB forced advancement, returns `WalkResult`
      with movement rows
    - `build_movements_from_snapshot()` — generates movement rows from a
      pre-mutation `BaseSnapshot` without touching state (used by the live
      PA path)
    - `validate_runner_overrides()` — consolidated collision check
    - `BaseSnapshot` struct for capturing base occupancy before plays
- **`HalfInning` helper methods**:
    - `as_str()` → `"Top"` / `"Bottom"` (replaces 12+ manual match blocks)
    - `symbol()` → `'↑'` / `'↓'`
    - `from_str_loose()` — case-insensitive parse from DB strings
- **`PlateAppearanceOutcome` helper methods**:
    - `bases()` → number of bases on a hit (1–4), 0 for non-hits
    - `is_hit()` → true for Single/Double/Triple/HomeRun
    - `zone()` → extract `Option<FieldZone>` from any hit variant
    - `label()` → short symbol (`"H"`, `"2H"`, `"K"`, `"BB"`, …)
    - `display_label()` → human-readable (`"Single"`, `"Home run"`, …)
- **Database connection PRAGMAs** (set in `Database::new()`):
    - `journal_mode = WAL` — better write performance for single-writer apps
    - `synchronous = NORMAL` — safe with WAL, significantly faster than FULL
    - `cache_size = -8000` — ~8 MB page cache (was default ~2 MB)
    - `foreign_keys = ON` — FK enforcement enabled at connection time
- **New test** `test_migrations_applied_on_new_db` — verifies that a
  fresh `:memory:` database runs all migrations and reaches
  `CURRENT_SCHEMA_VERSION`

### Changed

- **`Database::init_schema()` simplified** — removed ~170 lines of
  `CREATE TABLE IF NOT EXISTS` that duplicated the migration chain.
  Now only creates the `meta` table, then delegates everything to
  `run_migrations()`. For a new database, all migrations (v1→v16) run
  in order; for an existing database, only pending migrations apply.
- **`migration_v1` is no longer a no-op** — now creates the foundational
  tables (`leagues`, `teams`, `players`, `games` + indexes) that were
  previously created by `init_schema()` directly.
- **`play_ball_apply.rs` rewritten** (655 → 530 lines):
    - Clean imports (removed 28 fully-qualified `crate::models::plate_appearance::*` paths)
    - `require_batter!` macro replaces 3 identical guard blocks
    - Hit command builder uses `PlateAppearanceOutcome::display_label()` and `zone()`
    - Override validation delegates to `runner_logic::validate_runner_overrides()`
- **`play_ball_reducer.rs` slimmed** (892 → 493 lines):
    - `apply_hit_with_overrides()` → delegates to `runner_logic::apply_hit()`
    - `apply_hit_advancement()` → delegates to `runner_logic::apply_hit()`
    - `apply_walk_advancement()` → delegates to `runner_logic::apply_walk()`
    - `build_hit_movements_from_snapshot()` → delegates to
      `runner_logic::build_movements_from_snapshot()`
    - Removed `place_runner_with_order()`, `ensure_inning()`,
      `add_runs_to_score()` (moved to `runner_logic`)
- **`HalfInning::as_str()` adopted** across `at_bat_draft.rs`,
  `game_events.rs`, `engine/play_ball.rs` — replaced manual
  `match half { Top => "Top", Bottom => "Bottom" }` patterns

### Removed

- ~250 lines of duplicated runner movement logic that existed in three
  separate places (`apply_hit_with_overrides`, `build_hit_movements_from_snapshot`,
  and inline walk movements in `apply_pitch`)
- ~170 lines of `CREATE TABLE` in `Database::init_schema()` that
  duplicated the migration chain
- Standalone `place_runner_with_order()`, `ensure_inning()`,
  `add_runs_to_score()` helper functions from `play_ball_reducer.rs`
  (consolidated into `runner_logic`)
- Standalone `half_symbol()` function from `engine/play_ball.rs`
  (replaced by `HalfInning::symbol()`)

---

## [0.9.3] - 2026-03-13

### Added

- **Scrollable Help panel**: the Help pane now supports independent scrolling
  (`help_scroll: u16` field in `TuiUi`).
- **Panel focus system**: `enum Focus { Log, Help }` added to `TuiUi`. `Tab`
  cycles focus between the Log and Help panels. All scroll keys (↑/↓,
  PgUp/PgDn, Home/End) act on the currently focused panel.
- **Shortcuts bar**: a fixed one-line bar between the main panels and the
  Command box shows the active focus and available navigation keys:
  ` - focus on:{Log|Help} - Tab:change focus ↑↓:scroll  PgUp/Dn:page  Home/End:top/bot`.
- **Focus indicator**: the title of the active panel shows `►`
  (e.g. `Log ►` or `Help ►`).
- Help content updated: added steal/walk/out to "Other commands"; removed
  "Navigation" section (now covered by the shortcuts bar).

### Changed

- `render_help` now accepts `scroll: u16` and `focused: bool` parameters.
- `render()` layout changed from 2 vertical zones to 3
  (`Min(1)` panels + `Length(1)` shortcuts bar + `Length(3)` command box).
- `clamp_scroll_to_viewport` updated to clamp both `scroll` and `help_scroll`.

---

## [0.9.2] - 2026-03-13

### Changed (architecture)

- **`runner_movements` table rebuilt** (migration v16): replaced legacy `at_bat_id`
  FK (pointing to unused `at_bats` table) with `pa_seq` (FK to
  `plate_appearances.seq`, NULL for non-PA events) and `game_event_id` (FK to
  `game_events.id`, NULL for PA movements). Added `inning`, `half_inning`,
  `game_id` columns. New `advancement_type` values: `hit_auto`, `hit_override`,
  `walk`, `steal`.
- **`game_events` scope clarified**: only administrative/informational events
  (game start, status changes, side changes, at-bat tracking, pitch recording,
  strikeouts, outs, walks). Runner base movements are no longer stored here.
- **Steal persistence moved to `runner_movements`**: `DomainEvent::StolenBase`
  removed; steals are now written as a `runner_movements` row
  (`advancement_type = "steal"`, `pa_seq = NULL`).
- **Hit advancements persisted in `runner_movements`**: every runner that moves
  on a hit (automatic or override) generates one row — `hit_auto` or
  `hit_override`.
- **Walk advancements persisted in `runner_movements`**: every forced advancement
  on a BB (batter to 1B, and any runners pushed up) generates one row per runner.
- **`append_plate_appearance` returns `i64`** (the `seq` of the inserted row)
  so the engine can link runner movements to the correct PA.
- **`apply_live_plate_appearance` returns `Vec<RunnerMovementInsert>`** — hit
  movements are computed from a pre-mutation base snapshot and returned for the
  engine to persist.
- **`apply_hit_with_overrides` returns `Vec<RunnerMovementInsert>`** — replay
  path discards the value with `let _ =`.

### Fixed

- **Steal replay on resume**: `replay_plate_appearances_and_log` now loads and
  interleaves standalone `runner_movements` rows (steals) in the correct
  inning/half order, applying them to the rebuilt `GameState`. Previously,
  steals appeared in the log (via `game_events`) but were NOT applied to base
  state on resume.

## [0.9.1] - 2026-03-13

### Added

- **Steal command** (`<order> st <base>`): scorer can record a successful stolen
  base mid-inning, standalone or combined with a pitch command
  (e.g. `k, 6 st 2b`). Valid destinations: `2b`, `3b`, `sc`/`score`/`home`.
  Stealing home increments the batting team's score.
- `EngineCommand::StealBase { order, dest }` variant
- `DomainEvent::StolenBase { order, runner_id, runner_first_name,
  runner_last_name, dest }` — persisted to `game_events` for replay on resume
- Validation: runner must be on the expected source base; `st 1b` and
  out-of-range order (0, >9) are rejected as `Unknown`

### Fixed

- **Unicode panic in compact override tokens**: `parse_runner_override_token`
  used byte-index slicing (`&compact[..1]`) which panics on non-ASCII input
  (e.g. `h, é2b`). Now uses `char_indices` to split at the first char boundary
  safely — non-ASCII leading chars return `None` without panicking.
- **Silent runner overwrite on conflicting destinations**: `apply_hit_command`
  now calls `validate_runner_overrides` before touching state. Returns an error
  if two overrides claim the same base, or if an override destination is already
  occupied by a runner not listed in the overrides (who would otherwise be
  silently evicted).

---

## [0.9.0] - 2026-03-13

### Changed — module refactor (no functional changes)

- `models/play_ball.rs` split into three focused modules:
    - `models/game_state.rs` — `GameState`, `BatterOrder`, `PitchStats`
    - `models/runner.rs` — `RunnerDest`, `RunnerOverride`
    - `models/session.rs` — `PlayBallGameContext`, `PlayBallGate`, `LineupSide`
- `models/play_ball.rs` kept as compatibility re-export shim
- Full-scoring domain types (`HitType`, `OutType`, `Walk`, `AdvancedPlay`,
  `PlateAppearanceResult`, `Base`, `ScoringError`) moved from `models/types.rs`
  to new `models/scoring/types.rs`; `core/parser.rs` updated accordingly
- `models/events.rs` gains `PersistedEvent` (previously defined inline in
  `core/play_ball_apply.rs`)
- `core/play_ball.rs` functions (`list_playable_games`, `gate_check_lineups`,
  `set_game_status`) moved to `db/game_queries.rs`; `core/play_ball.rs`
  kept as deprecated re-export shim
- `OutcomeSymbol` removed from `models/play_ball.rs` (was unused)
- `lib.rs` re-exports updated to reflect new module layout

### Fixed

- **Runner override replay correctness** (migration v15): `runner_overrides_json`
  column added to `plate_appearances`; `append_plate_appearance` now persists
  override data and `apply_plate_appearance_row` deserialises it on resume.
  Previously, any hit scored with explicit runner destinations would be replayed
  using automatic advancement after a game was reopened, producing a different
  base/score state than what was originally recorded.
- **Silent invalid override rejection**: hit commands with an unrecognisable
  trailing token (e.g. `6 h, 5 xx`) now return `Unknown` instead of silently
  executing a plain hit. Previously, `filter_map` dropped invalid tokens without
  any error, causing silent data loss.

## [0.8.0] - 2026-03-12

### Added

- **Runner override syntax** — lo scorer può ora specificare esplicitamente dove
  finisce ogni corridore dopo un hit, usando il batting order come identificatore:
    - `h` → singola, avanzamento automatico (comportamento precedente invariato)
    - `6 h, 5 2b` → il #6 batte singola; il corridore #5 rimane in 2a
    - `6 h, 5 2b, 3 sc` → singola; #5 → 2a; #3 segna
    - `4 2h, 2 sc` → doppia; il corridore #2 segna (anziché fermarsi in 4a)
    - Destinazioni valide: `1b`, `2b`, `3b`, `sc` / `score` / `home`
- `RunnerDest` e `RunnerOverride` aggiunti a `models/play_ball.rs`
- `apply_hit_with_overrides()` nel reducer sostituisce l'avanzamento automatico
  fisso; qualsiasi corridore senza override esplicito continua ad avanzare
  automaticamente in base al numero di basi colpite
- `GameState.on_1b/on_2b/on_3b` cambiati da `bool` a `Option<BatterOrder>` —
  il campo ora porta l'identità del corridore (batting order), non solo
  l'occupazione della base
- `PlateAppearance.runner_overrides` — gli override vengono persistiti nel PA
  compatto per un replay fedele al momento dell'inserimento dati
- Test unitari per il parser nella nuova sintassi (`engine_parser.rs`)
- **Runner override compact format** — i token runner accettano ora sia
  `7 sc` (con spazio) che `7sc` (senza spazio), per comodità di inserimento
  rapido (es. `9 h, 8 2b, 7sc, 6sc` con basi piene è ora valido)

### Changed

- `EngineCommand::Single/Double/Triple/HomeRun` hanno ora un campo
  `runner_overrides: Vec<RunnerOverride>` (breaking — solo interno)
- La UI (`tui.rs`) converte `Option<BatterOrder>` → `bool` per il diamond;
  la visualizzazione occupato/libero rimane invariata

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
