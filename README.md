# ⚾ Baseball Scorer - v0.4.0

A comprehensive baseball and softball scoring application with SQLite persistence, official scoring symbols support,
cross-platform compatibility, professional database management, and complete lineup management with DH support.

## 🆕 What's New in v0.4.0

- ✅ **Complete Lineup Entry**: Full lineup management during game creation with validation
- ✅ **Designated Hitter Support**: Independent DH option for each team with proper pitcher handling
- ✅ **Game Time Field**: Record both date and time for each game
- ✅ **Custom Game IDs**: Option to use custom game identifiers
- ✅ **Schema v3**: New `game_lineups` table for complete lineup tracking

### Recent Versions

**v0.3.1** - Complete CLI menu system, game metadata management  
**v0.3.0** - Game management system with metadata tracking  
**v0.2.5** - Database migration system with meta table  
**v0.2.4** - Complete database management suite

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
    │       ├── game.rs    # Game operations + LINEUP ENTRY (NEW v0.4.0)
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
    │   ├── player.rs      # Player operations
    │   └── migrations.rs  # Schema migration system (v3 in v0.4.0)
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
4. Run migrations to latest version (v3)
5. Display database location

**Database Locations:**

- **Windows**: `%LOCALAPPDATA%\bs_scorer\baseball_scorer.db`
- **macOS/Linux**: `$HOME/.bs_scorer/baseball_scorer.db`

## 🎮 Main Menu

```
╔════════════════════════════════════════════╗
║  ⚾  BASEBALL/SOFTBALL SCORER - MAIN MENU  ║
╚════════════════════════════════════════════╝

  1. 🎮 Manage Games
  2. 🏆 Manage Leagues
  3. ⚾ Manage Teams
  4. 📊 Statistics
  5. 💾 Manage DB

  0. 🚪 Exit

Select an option (1-5 or 0):
```

## 🎮 Game Management (Enhanced in v0.4.0)

```
GAME MANAGEMENT
═══════════════════════════════════
  1. 🆕 New Game          ← ENHANCED with full lineup entry
  2. 📋 List Games
  3. ✏️  Edit Game
  4. ⚾ Play Ball!

  0. 🔙 Back to Main Menu
```

### Creating a New Game (v0.4.0 Workflow)

The new game creation process guides you through:

#### 1. **Game Metadata**

- **Game ID**: Auto-generated or custom (e.g., `B00A1AAAR0111`)
- **Teams**: Select away and home teams
- **Date**: Game date (YYYY-MM-DD, default today)
- **Time**: Game time (HH:MM, default now)
- **Venue**: Game location (required)

#### 2. **Away Team Lineup**

1. Check roster (minimum 12 players required)
2. Choose DH option (Y/N)
3. For each batting position (1-9 or 1-10 if DH):
    - Enter jersey number
    - Assign defensive position (1-9 or DH)
4. If DH used: Enter pitcher (position 10, doesn't bat)
5. Review complete lineup
6. Confirm or restart

#### 3. **Home Team Lineup**

(Same process as away team)

#### 4. **Confirmation**

Review all game details and lineups before saving.

### Lineup Entry Rules

**With DH (Designated Hitter):**

- 10 players in batting order
- Positions 1-9: Regular batters with defensive positions
- Position 10: Pitcher (defensive position 1, does NOT bat)
- DH can bat in any position 1-9
- DH defensive position: "DH" (does not field)

**Without DH:**

- 9 players in batting order
- All players bat and field
- Pitcher bats at his position in the order

**Validations:**

- Each defensive position (1-9) assigned exactly once
- Each player used only once in lineup
- Jersey numbers must exist in team roster
- Minimum 12 players in roster required

**Example Lineup with DH:**

```
═══════════════════════════════════════════════════
║              BOSTON RED SOX LINEUP                ║
═══════════════════════════════════════════════════

⚾ Designated Hitter: YES

  1.  #50 Mookie Betts           Pos 9 (RF)
  2.  #16 Andrew Benintendi      Pos 7 (LF)
  3.  #28 J.D. Martinez          DH
  4.  #15 Dustin Pedroia         Pos 4 (2B)
  5.  #2  Xander Bogaerts        Pos 6 (SS)
  6.  #11 Rafael Devers          Pos 5 (3B)
  7.  #36 Eduardo Núñez          Pos 3 (1B)
  8.  #23 Blake Swihart          Pos 2 (C)
  9.  #19 Jackie Bradley Jr.     Pos 8 (CF)
  10. #41 Chris Sale             P (does not bat)
```

## 🏆 Manage Leagues

Create and manage leagues/championships:

- ➕ **Create League**: Name, season, description
- 📋 **View Leagues**: List all existing leagues
- ✏️ **Edit League**: Update information
- 🗑️ **Delete League**: Remove league (with confirmation)

## ⚾ Manage Teams

Complete team management:

- ➕ **Create Team**: Name, city, abbreviation, founded year
- 📋 **View Teams**: List all teams with details
- ✏️ **Edit Team**: Update team information
- 👥 **Manage Roster**: Add/edit/remove players
    - ⚠️ **Important**: Need minimum 12 players to create games!
- 📥 **Import Team**: From JSON/CSV (in development)
- 🗑️ **Delete Team**: Remove team and all players

## 🗄️ Database Schema

### Core Tables

**meta**

- Application metadata and schema version tracking

**leagues**

- League/championship information

**teams**

- Team data with optional league association

**players**

- Player roster with positions and batting order

**games** (Enhanced in v0.4.0)

- Game metadata including:
    - `game_id`: Unique identifier
    - `game_date`: Date of game
    - `game_time`: Time of game (NEW v0.4.0)
    - `at_uses_dh`: Away team uses DH (NEW v0.4.0)
    - `ht_uses_dh`: Home team uses DH (NEW v0.4.0)
    - Status, scores, current inning

**game_lineups** (NEW in v0.4.0)

- Starting lineups for both teams:
    - `game_id`: Reference to game
    - `team_id`: Reference to team
    - `player_id`: Reference to player
    - `batting_order`: Position in order (1-10)
    - `defensive_position`: Field position (1-9 or "DH")
    - Substitution tracking (for future use)

**at_bats**

- Detailed scoring of each plate appearance

**pitches**

- Individual pitch tracking

**runner_movements**

- Base runner advancement tracking

**game_events**

- Special events (substitutions, delays, etc.)

## 💾 Database Management

Complete database lifecycle management:

```
DATABASE MANAGEMENT
═══════════════════════════════════
  1. 📋 View DB Info
  2. 🔍 View DB Status
  3. 🔄 Run Migrations      ← Includes v3 migration
  4. 💾 Backup Database
  5. 📥 Restore Database
  6. 🧹 Vacuum Database
  7. 🗑️  Clear All Data
  8. 📤 Export Game

  0. 🔙 Back to Main Menu
```

## 🔄 Database Migrations

### Schema Version 3 (v0.4.0)

**Changes:**

1. ALTER TABLE games:
    - Add `game_time TEXT`
    - Add `at_uses_dh BOOLEAN DEFAULT 0`
    - Add `ht_uses_dh BOOLEAN DEFAULT 0`

2. CREATE TABLE game_lineups:
    - Complete lineup tracking
    - Support for starting lineups
    - Ready for substitution tracking

**Migration Path:**

- Automatic on app startup
- Manual via "Manage DB > Run Migrations"
- Recommended: Backup database first

## 📊 Features by Version

| Version | Key Features                                 |
|---------|----------------------------------------------|
| 0.4.0   | Complete lineup entry, DH support, game time |
| 0.3.1   | Complete CLI menu structure                  |
| 0.3.0   | Game management system                       |
| 0.2.5   | Migration system, meta table                 |
| 0.2.4   | DB backup/restore, VACUUM, export            |
| 0.2.3   | CLI refactor, DB management menu             |
| 0.2.2   | Library support, standard structure          |

## 🚀 Roadmap

### v0.5.0 (Next)

- **Play Ball!** - Live game scoring interface
- Pitch-by-pitch tracking
- Real-time score display
- Base runner tracking
- Automatic lineup advancement

### v0.6.0 (Planned)

- Mid-game substitutions
- Pinch hitters/runners
- Defensive replacements
- Lineup editing

### Future

- Player statistics (AVG, ERA, OPS, WHIP)
- Team statistics and rankings
- League standings
- Season summaries
- Web interface
- PDF scorecard generation

## 📚 Documentation

- [CHANGELOG.md](CHANGELOG.md) - Complete version history
- [CHANGELOG_v0.4.0.md](CHANGELOG_v0.4.0.md) - Detailed v0.4.0 changes
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

**Version:** 0.4.0  
**Schema:** v3  
**Edition:** Rust 2024  
**Author:** Alessandro Maestri

**Play Ball! ⚾**