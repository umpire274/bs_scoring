# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
