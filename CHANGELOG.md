# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
