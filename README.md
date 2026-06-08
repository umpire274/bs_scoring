# ⚾ BS Scoring — v0.12.0

BS Scoring is a terminal-based baseball and softball scoring application written in Rust.
It provides SQLite persistence, Play Ball game scoring, deterministic resume, lineup management,
player/team/league management, and umpire supervisor tools.

## Highlights in v0.12.0

### Player model update

v0.12.0 introduces a WBSC-compatible player data model:

- throwing hand is now named `throw`, not `pitch`;
- throwing values are now `R`, `L`, `S` instead of `RHP`, `LHP`, `SHP`;
- players can have multiple roster positions such as `P,C,IF`;
- CSV/JSON player import/export uses `bat_throw` notation such as `R/R`;
- CSV player files use `;` as separator;
- both home and away jersey numbers are unique per team.

### Storage update

On Linux, the application now uses the XDG data directory:

```text
~/.local/share/bs_scoring
```

or:

```text
$XDG_DATA_HOME/bs_scoring
```

when `XDG_DATA_HOME` is defined.

Existing data from the legacy directory is migrated automatically:

```text
~/.bs_scorer
```

Database files are now named `bs_scoring*.db` instead of `baseball_scorer*.db`.

## Main Features

- League, team, player, and game management.
- Player import/export in CSV and JSON.
- Downloadable CSV and JSON player import templates.
- Separate home and away jersey numbers.
- Play Ball mode with pitch-by-pitch command input.
- Runner advancement overrides.
- Steal support.
- Fielder's choice and composite defensive-play support.
- Deterministic game resume from persisted data.
- TUI scoreboard with game state, count, outs, score, and lineup context.
- Umpire Supervisor module with assignments, evaluations, and reports.
- SQLite persistence with automatic migrations.

## Installation

Clone the repository and build with Cargo:

```bash
git clone https://github.com/umpire274/bs_scoring.git
cd bs_scoring
cargo build --release
```

Run the application:

```bash
cargo run --release
```

Or run the compiled binary:

```bash
./target/release/bs_scoring
```

## Data Location

### Linux

```text
$XDG_DATA_HOME/bs_scoring
```

or, when `XDG_DATA_HOME` is not set:

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

## Player Import/Export Format

### CSV

CSV player files use `;` as field separator.

Header:

```text
team_name;number;away_number;first_name;last_name;position;bat_throw
```

Example:

```text
team_name;number;away_number;first_name;last_name;position;bat_throw
Rimini Baseball;12;9;Mario;Rossi;P,C,IF;R/R
San Marino;0;0;Luca;Bianchi;IF,OF,DH;L/R
```

### JSON

Example:

```json
[
  {
    "team_name": "Rimini Baseball",
    "number": 12,
    "away_number": 9,
    "first_name": "Mario",
    "last_name": "Rossi",
    "position": "P,C,IF",
    "bat_throw": "R/R"
  },
  {
    "team_name": "San Marino",
    "number": 0,
    "away_number": 0,
    "first_name": "Luca",
    "last_name": "Bianchi",
    "position": "IF,OF,DH",
    "bat_throw": "L/R"
  }
]
```

### `position`

The `position` field describes roster capabilities, not the lineup defensive position.
It may contain one or more comma-separated values.

Allowed values:

```text
P,C,1B,2B,3B,SS,LF,CF,RF,IF,OF,DH
```

Examples:

```text
P
P,C,IF
IF,OF,DH
LF,CF,RF
```

### `bat_throw`

The `bat_throw` field follows the standard `BAT/THROW` notation.

Allowed values:

```text
R/R
R/L
R/S
L/R
L/L
L/S
S/R
S/L
S/S
```

The value before `/` is the batting side. The value after `/` is the throwing hand.

## Play Ball Command Examples

Start a game:

```text
playball
```

Pitch sequence:

```text
b, k, f, s
```

Single by the current batter:

```text
h
```

Single by batting-order slot 5:

```text
5 h
```

Runner steals second:

```text
6 st 2b
```

Hit with runner override:

```text
5 h lf, 3 sc
```

Composite defensive play:

```text
5 l6, 3 64, 4 43
```

See `SCORING_GUIDE.md` for the full command reference.

## Project Structure

Main source folders:

```text
src/
├── cli/       # Menu screens and user-facing workflows
├── db/        # SQLite persistence and migrations
├── engine/    # Game engine, command parser, reducer, scoring logic
├── models/    # Domain models and pure data types
├── ui/        # Terminal UI implementations
└── utils/     # Terminal, time, normalization, and boot helpers
```

See `STRUCTURE.md` for the detailed architecture.

## Development

Run checks:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

Build release:

```bash
cargo build --release
```

## License

MIT.
