# ⚾ Baseball Scorer - v0.2.5

A comprehensive baseball and softball scoring application with SQLite persistence, official scoring symbols support,
cross-platform compatibility, and professional database management.

## 🆕 What's New in v0.2.5

- ✅ **Database Migration System**: Automatic and manual schema migrations with version tracking
- ✅ **Meta Table**: Application metadata storage (schema version, backup/restore dates)
- ✅ **Incremental Migrations**: Safe, tracked database schema updates
- ✅ **Migration Interface**: Manual migration execution via DB management menu

### Recent Versions

**v0.2.4** - Complete database management suite with backup/restore, VACUUM, status monitoring, and game export  
**v0.2.3** - CLI commands refactoring, main menu loop extraction, DB management menu  
**v0.2.2** - Library support and standard Rust project structure

## 📁 Project Structure

```
bs_scoring/
├── Cargo.toml              # Package configuration with lib + bin
├── README.md
├── CHANGELOG.md
├── SCORING_GUIDE.md
├── STRUCTURE.md
└── src/                    # All source code
    ├── lib.rs             # Library interface
    ├── main.rs            # CLI application entry point (minimal)
    ├── cli/               # Command-line interface
    │   ├── mod.rs
    │   └── commands/      # Command handlers
    │       ├── db.rs      # Database management
    │       ├── game.rs    # Game operations
    │       ├── leagues.rs # League management
    │       ├── main_menu.rs # Main menu loop
    │       ├── statistics.rs # Statistics display
    │       └── team.rs    # Team management
    ├── core/              # Business logic
    │   ├── menu.rs        # Menu system
    │   └── parser.rs      # Scoring notation parser
    ├── db/                # Database layer
    │   ├── config.rs      # Cross-platform paths + setup_db()
    │   ├── database.rs    # SQLite schema and operations
    │   ├── league.rs      # League CRUD
    │   ├── team.rs        # Team and Player CRUD
    │   └── migrations.rs  # Schema migration system (NEW v0.2.5)
    ├── models/            # Data types
    │   └── types.rs       # Game scoring types
    └── utils/             # Utilities
        └── cli.rs         # CLI helper functions
```

## 🚀 Installation

### Prerequisites

- Rust 1.85+ (for edition 2024) - Install from [rustup.rs](https://rustup.rs/)

### Compilation

```bash
cd bs_scoring
cargo build --release
```

Executable available at: `target/release/bs_scoring`

## 📖 Usage

```bash
cargo run
# or
./target/release/bs_scoring
```

### First Run

On first run, the application will:

1. Create platform-specific database directory
2. Initialize SQLite database
3. Create schema with all tables
4. Set initial schema version
5. Display database location

**Database Locations:**

- **Windows**: `%LOCALAPPDATA%\bs_scorer\baseball_scorer.db`
- **macOS/Linux**: `$HOME/.bs_scorer/baseball_scorer.db`

## 🎮 Main Menu

```
╔════════════════════════════════════════════╗
║  ⚾  BASEBALL/SOFTBALL SCORER - MAIN MENU  ║
╚════════════════════════════════════════════╝

  1. 🆕 New Game
  2. 🏆 Manage Leagues
  3. ⚾ Manage Teams
  4. 📊 Statistics
  5. 💾 Manage DB

  0. 🚪 Exit

Select an option (1-5 or 0):
```

## 💾 Database Management (NEW in v0.2.4-0.2.5)

Complete database lifecycle management:

```
DATABASE MANAGEMENT
═══════════════════════════════════
  1. 📋 View DB Info
  2. 🔍 View DB Status
  3. 🔄 Run Migrations      ← NEW v0.2.5
  4. 💾 Backup Database
  5. 📥 Restore Database
  6. 🧹 Vacuum Database
  7. 🗑️  Clear All Data
  8. 📤 Export Game

  0. 🔙 Back to Main Menu
```

### Features:

**View DB Info**

- Record counts (leagues, teams, players, games)
- Database file location and size
- Schema version and status

**View DB Status**

- Comprehensive health metrics
- Page statistics and fragmentation
- Journal mode, synchronous settings
- Integrity check with suggestions

**Run Migrations** ⭐ NEW

- Check current schema version
- List pending migrations
- Execute migrations manually
- Automatic on app startup if needed

**Backup Database**

- Timestamped backups: `baseball_scorer_backup_YYYYMMDD_HHMMSS.db`
- Size reporting
- Records backup date in metadata

**Restore Database**

- List available backups
- Automatic safety backup before restore
- Records restore date in metadata

**Vacuum Database**

- Reclaim unused space
- Optimize database performance
- Before/after statistics

**Export Game**

- JSON format (complete structured data)
- CSV format (Excel-compatible)

## 🔄 Database Migration System (v0.2.5)

### Automatic Migrations

Migrations run automatically on app startup when:

- Database schema version < current app version
- New migrations are available

### Manual Migrations

Execute via: `Main Menu > 5. Manage DB > 3. Run Migrations`

Shows:

- Current schema version
- Latest schema version
- Number of pending migrations
- List of migrations to be applied
- Confirmation before execution

### Migration Safety

- Backup recommended before migrations
- Version tracking in `meta` table
- Incremental application (only missing migrations)
- Detailed migration descriptions

### Meta Table

Stores application metadata:

- `schema_version`: Current database schema version
- `app_version`: Application version
- `created_at`: Database creation timestamp
- `last_backup`: Last backup timestamp
- `last_restore`: Last restore timestamp
- `last_migration`: Last migration timestamp

## 🏆 Manage Leagues

Create and manage leagues/championships:

- ➕ **Create League**: Name, season, description
- 📋 **View Leagues**: List all existing leagues
- ✏️ **Edit League**: Update information
- 🗑️ **Delete League**: Remove league (with confirmation)

**Example:**

```
League name: MLB
Season: 2026
Description: Major League Baseball
```

## ⚾ Manage Teams

Complete team management:

- ➕ **Create Team**: Name, city, abbreviation, founded year
- 📋 **View Teams**: List all teams with details
- ✏️ **Edit Team**: Update team information
- 👥 **Manage Roster**: Add/edit/remove players (in development)
- 📥 **Import Team**: From JSON/CSV (in development)
- 🗑️ **Delete Team**: Remove team and all players

**Example:**

```
Team name: Boston Red Sox
City: Boston
Abbreviation: BOS
Founded year: 1901
League: MLB (optional)
```

## 🗄️ Database Schema

### Core Tables

**meta** (NEW v0.2.5)

- Stores application metadata and schema version

**leagues**

- League/championship information

**teams**

- Team data with optional league association

**players**

- Player roster with positions and batting order

**games**

- Game records with scores and status

**plate_appearances**

- Detailed scoring of each at-bat

**base_runners**

- Runner advancement tracking

## 🎯 Scoring Symbols

*See [SCORING_GUIDE.md](SCORING_GUIDE.md) for complete reference*

### Quick Reference

**Hits:** 1B, 2B, 3B, HR, GRD  
**Outs:** K, KL, 6-3, F8, L9, P5, DP, TP  
**Walks:** BB, IBB, HBP  
**Errors:** E1-E9 (by position)  
**Advanced:** SB2, SB3, CS, WP, PB, BK, SF, SH

### Defensive Positions

1=Pitcher, 2=Catcher, 3=First Base, 4=Second Base, 5=Third Base,  
6=Shortstop, 7=Left Field, 8=Center Field, 9=Right Field

## 🔧 Development

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

### Adding Database Migrations

1. **Increment schema version** in `src/db/migrations.rs`:

```rust
   pub const CURRENT_SCHEMA_VERSION: i64 = 2; // was 1
```

2. **Add migration to list**:

```rust
   Migration {
version: 2,
description: "Add statistics table",
up: migration_v2,
}
```

3. **Implement migration function**:

```rust
   fn migration_v2(conn: &Connection) -> Result<()> {
    conn.execute("CREATE TABLE stats (...)", [])?;
    Ok(())
}
```

## 📊 Features by Version

| Version | Key Features                                   |
|---------|------------------------------------------------|
| 0.2.5   | Migration system, meta table, version tracking |
| 0.2.4   | DB backup/restore, VACUUM, status, export      |
| 0.2.3   | CLI refactor, DB management menu               |
| 0.2.2   | Library support, standard structure            |
| 0.2.1   | Cross-platform DB paths                        |
| 0.2.0   | SQLite persistence, menu system                |
| 0.1.0   | Initial CLI scoring tool                       |

## 🚀 Roadmap

### v0.3.0 (Planned)

- Live game scoring interface
- Pitch-by-pitch tracking
- Real-time score display
- Complete roster management

### v0.4.0 (Planned)

- Player statistics (AVG, ERA, OPS, WHIP)
- Team statistics and rankings
- League standings
- Season summaries

### Future

- Web interface
- Mobile app
- PDF scorecard generation
- Advanced analytics
- Multi-season support

## 📚 Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history
- [SCORING_GUIDE.md](SCORING_GUIDE.md) - Official scoring symbols
- [STRUCTURE.md](STRUCTURE.md) - Project architecture
- [RELEASE.md](RELEASE.md) - Release process

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

MIT License - Free to use for your games! ⚾

## 🔗 Links

- **Repository**: https://github.com/umpire274/bs_scoring
- **Issues**: https://github.com/umpire274/bs_scoring/issues
- **Releases**: https://github.com/umpire274/bs_scoring/releases

---

**Version:** 0.2.5  
**Edition:** Rust 2024  
**Author:** Alessandro Maestri

**Play Ball! ⚾**