# 🎯 Baseball Scorer v0.2.2 - Structure Overview

## 📦 Complete Package Contents

**Archive:** `bs_scoring-v0.2.2.zip` (33KB)

## 📂 Directory Structure

```
bs_scoring/
│
├── 📄 Configuration & Documentation
│   ├── Cargo.toml              # Package manifest with [lib] and [[bin]]
│   ├── Cargo.lock              # Dependency lock file
│   ├── README.md               # Main documentation
│   ├── CHANGELOG.md            # Version history (v0.2.2 entry added)
│   ├── SCORING_GUIDE.md        # Baseball scoring symbols reference
│   ├── RELEASE.md              # Git release instructions
│   ├── .gitignore              # Git ignore patterns
│   └── .gitmodules             # Git submodules (if any)
│
├── 🔧 GitHub Actions
│   └── .github/
│       └── workflows/
│           └── rust.yml        # CI/CD pipeline
│
└── 📁 Source Code (src/)
    │
    ├── lib.rs                  # 🆕 Library interface (v0.2.2)
    ├── main.rs                 # CLI application entry point
    │
    ├── core/                   # Business logic modules
    │   ├── mod.rs
    │   ├── menu.rs             # COBOL-style menu system
    │   └── parser.rs           # Scoring notation parser (K, 6-3, HR, etc.)
    │
    ├── db/                     # Database layer
    │   ├── mod.rs
    │   ├── config.rs           # 🆕 Cross-platform path management (v0.2.1)
    │   ├── database.rs         # SQLite schema and initialization
    │   ├── league.rs           # League CRUD operations
    │   └── team.rs             # Team and Player CRUD operations
    │
    └── models/                 # Data types and structures
        ├── mod.rs
        └── types.rs            # Game scoring types (Hit, Out, Walk, etc.)
```

## 🔑 Key Changes from v0.2.1

### ✅ New in v0.2.2

1. **Library Support (`src/lib.rs`)**
   - Public API for code reusability
   - Re-exports common types and functions
   - Module documentation with examples
   - Enables use as dependency in other projects

2. **Standard Rust Structure**
   - All code now in `src/` directory
   - `src/main.rs` (binary entry point)
   - `src/lib.rs` (library entry point)
   - Follows official Rust project layout

3. **Enhanced Cargo.toml**
   ```toml
   [lib]
   name = "bs_scoring"
   path = "src/lib.rs"
   
   [[bin]]
   name = "bs_scoring"
   path = "src/main.rs"
   ```

4. **Metadata Additions**
   - Authors, description, license
   - Repository URL
   - Keywords and categories
   - Ready for crates.io publishing

## 📊 Module Overview

### Core Modules

| Module | Purpose | Lines | Key Types |
|--------|---------|-------|-----------|
| `core::menu` | Menu navigation | ~300 | Menu, MenuChoice enums |
| `core::parser` | Scoring parser | ~280 | CommandParser |
| `db::config` | Path management | ~90 | get_db_path(), get_app_data_dir() |
| `db::database` | SQLite schema | ~180 | Database |
| `db::league` | League CRUD | ~120 | League |
| `db::team` | Team/Player CRUD | ~280 | Team, Player |
| `models::types` | Game types | ~310 | Hit, Out, Walk, Position, etc. |

### Database Locations

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\bs_scorer\baseball_scorer.db` |
| macOS | `$HOME/.bs_scorer/baseball_scorer.db` |
| Linux | `$HOME/.bs_scorer/baseball_scorer.db` |

## 🎨 Architecture Diagram

```
┌─────────────────────────────────────────────┐
│           CLI Application (main.rs)          │
│  - Menu-driven interface                     │
│  - User interaction                          │
└─────────────────────┬───────────────────────┘
                      │
                      ↓
┌─────────────────────────────────────────────┐
│          Library (lib.rs) - PUBLIC API       │
│  - Re-exports all modules                    │
│  - Documentation                              │
└───────┬─────────────┬─────────────┬─────────┘
        │             │             │
        ↓             ↓             ↓
┌──────────┐  ┌─────────────┐  ┌──────────┐
│   core   │  │     db      │  │  models  │
│          │  │             │  │          │
│ • menu   │  │ • config    │  │ • types  │
│ • parser │  │ • database  │  │          │
│          │  │ • league    │  │          │
│          │  │ • team      │  │          │
└──────────┘  └─────────────┘  └──────────┘
                      │
                      ↓
              ┌──────────────┐
              │   SQLite DB   │
              │ (cross-platform)│
              └──────────────┘
```

## 🚀 Usage Examples

### As a Binary

```bash
cargo build --release
./target/release/bs_scoring
```

### As a Library

```rust
use bs_scoring::{Database, League, get_db_path};

fn main() {
    let db_path = get_db_path().unwrap();
    let db = Database::new(&db_path.to_string_lossy()).unwrap();
    db.init_schema().unwrap();
    
    let mut league = League::new(
        "MLB".to_string(),
        Some("2026".to_string()),
        None
    );
    league.create(db.get_connection()).unwrap();
}
```

## 📈 Version History

- **v0.2.2** (2026-02-03): Library support + standard structure
- **v0.2.1** (2026-02-03): Cross-platform DB paths
- **v0.2.0** (2026-02-03): SQLite + menu system
- **v0.1.0** (2026-02-01): Initial CLI scoring

## 🔜 Next Steps (v0.3.0)

Planned features:
- Live game scoring interface
- Pitch-by-pitch tracking
- Complete roster management
- Real-time game state display
- Player statistics module

---

**Built with Rust 🦀**  
**Play Ball! ⚾**
