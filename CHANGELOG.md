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
  CREATE TABLE meta (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL,
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
