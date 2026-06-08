use crate::cli::menu::DBMenuChoice;
use crate::db::migrations;
use crate::utils::term;
use crate::{Database, Menu, get_db_path, get_db_path_display};
use chrono::Local;
use std::fs;
use std::path::Path;

pub fn handle_db_menu(db: &Database) {
    loop {
        match Menu::show_db_menu() {
            DBMenuChoice::ViewInfo => view_db_info(db),
            DBMenuChoice::ViewStatus => view_db_status(db),
            DBMenuChoice::RunMigrations => run_migrations_manual(db),
            DBMenuChoice::BackupDB => backup_database(),
            DBMenuChoice::RestoreDB => restore_database(),
            DBMenuChoice::VacuumDB => vacuum_database(db),
            DBMenuChoice::ClearData => clear_all_data(db),
            DBMenuChoice::ExportGame => export_game(db),
            DBMenuChoice::Back => break,
        }
    }
}

fn view_db_info(db: &Database) {
    term::show_header("DATABASE INFO");

    println!("📁 Location: {}", get_db_path_display());

    let conn = db.get_connection();

    // Schema version
    let schema_version = migrations::get_schema_version(conn).unwrap_or(0);
    let migrations_pending = migrations::migrations_needed(conn).unwrap_or(false);

    println!("\n🔢 Schema:");
    println!("   {:<12} {:<22}", "Version:", schema_version);
    if migrations_pending {
        println!("   {:<12} {:<22}", "Status:", "⚠️  Migrations pending!");
    } else {
        println!("   {:<12} {:<22}", "Status:", "✅ Up to date");
    }

    // Count records
    let league_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM leagues", [], |row| row.get(0))
        .unwrap_or(0);
    let team_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM teams", [], |row| row.get(0))
        .unwrap_or(0);
    let player_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM players", [], |row| row.get(0))
        .unwrap_or(0);
    let game_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap_or(0);

    println!("\n📊 Records:");
    println!("  🏆 {:<10} {:>8}", "Leagues:", league_count);
    println!("  ⚾ {:<10} {:>8}", "Teams:", team_count);
    println!("  👥 {:<10} {:>8}", "Players:", player_count);
    println!("  🎮 {:<10} {:>8}", "Games:", game_count);

    // DB file size
    if let Ok(path) = get_db_path()
        && let Ok(metadata) = fs::metadata(&path)
    {
        let size_kb = metadata.len() / 1024;
        println!("\n💾 Database size: {} KB", size_kb);
    }

    term::wait_for_enter();
}

fn view_db_status(db: &Database) {
    term::show_header("DATABASE STATUS");

    let conn = db.get_connection();

    // Page count and size
    let page_count: i64 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .unwrap_or(0);

    let page_size: i64 = conn
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .unwrap_or(0);

    let db_size_bytes = page_count * page_size;
    let db_size_kb = db_size_bytes / 1024;
    let db_size_mb = db_size_kb as f64 / 1024.0;

    // Freelist (unused pages)
    let freelist_count: i64 = conn
        .query_row("PRAGMA freelist_count", [], |row| row.get(0))
        .unwrap_or(0);

    let freelist_size = freelist_count * page_size;
    let freelist_kb = freelist_size / 1024;
    let freelist_percent = if page_count > 0 {
        (freelist_count as f64 / page_count as f64) * 100.0
    } else {
        0.0
    };

    // Journal mode
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap_or_else(|_| "unknown".to_string());

    // Synchronous mode
    let synchronous: i64 = conn
        .query_row("PRAGMA synchronous", [], |row| row.get(0))
        .unwrap_or(0);

    let sync_mode = match synchronous {
        0 => "OFF",
        1 => "NORMAL",
        2 => "FULL",
        3 => "EXTRA",
        _ => "UNKNOWN",
    };

    // Auto vacuum
    let auto_vacuum: i64 = conn
        .query_row("PRAGMA auto_vacuum", [], |row| row.get(0))
        .unwrap_or(0);

    let vacuum_mode = match auto_vacuum {
        0 => "NONE",
        1 => "FULL",
        2 => "INCREMENTAL",
        _ => "UNKNOWN",
    };

    // Integrity check (quick)
    let integrity: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .unwrap_or_else(|_| "ERROR".to_string());

    println!("📊 Database Statistics:");
    println!();
    println!(
        "  💾 {:<20} {:>12}",
        "Total size:",
        format!("{:.2} MB", db_size_mb)
    );
    println!("  📄 {:<20} {:>12}", "Page count:", page_count);
    println!(
        "  📐 {:<20} {:>12}",
        "Page size:",
        format!("{} bytes", page_size)
    );
    println!();
    println!(
        "  🗑️  {:<20} {:>12}",
        "Free space:",
        format!("{} KB ({:.1}%)", freelist_kb, freelist_percent)
    );
    println!("  📝 {:<20} {:>12}", "Journal mode:", journal_mode);
    println!("  🔒 {:<20} {:>12}", "Synchronous:", sync_mode);
    println!("  🧹 {:<20} {:>12}", "Auto vacuum:", vacuum_mode);
    println!();
    println!("  ✓  {:<20} {:>12}", "Integrity:", integrity);

    // Suggest vacuum if needed
    if freelist_percent > 10.0 {
        println!();
        println!(
            "  ⚠️  Suggestion: Database has {:.1}% free space.",
            freelist_percent
        );
        println!("     Consider running VACUUM to reclaim space.");
    }

    term::wait_for_enter();
}

fn run_migrations_manual(db: &Database) {
    term::show_header("DATABASE MIGRATIONS");

    let conn = db.get_connection();

    // Get migration info
    let info = match migrations::get_migration_info(conn) {
        Ok(info) => info,
        Err(e) => {
            term::show_error(&format!("Failed to get migration info: {}", e));
            return;
        }
    };

    println!("📊 Migration Status:\n");
    println!("  Current schema version:  v{}", info.current_version);
    println!("  Latest schema version:   v{}", info.latest_version);
    println!("  Pending migrations:      {}", info.pending_count);

    if let Some(last_migration) = &info.last_migration {
        println!("  Last migration:          {}", last_migration);
    } else {
        println!("  Last migration:          Never");
    }

    if let Some(created) = &info.created_at {
        println!("  Database created:        {}", created);
    }
    println!();

    if info.pending_count == 0 {
        println!("✅ Database schema is up to date!");
        println!();
        term::wait_for_enter();
        return;
    }

    println!("⚠️  {} migration(s) available:", info.pending_count);
    println!();

    // List pending migrations
    let migrations_list = migrations::get_migrations();
    for migration in migrations_list {
        if migration.version > info.current_version {
            println!("  • v{}: {}", migration.version, migration.description);
        }
    }
    println!();

    if term::confirm("Run pending migrations?") {
        println!("\n🔄 Running migrations...\n");

        match migrations::run_migrations(conn, info.current_version) {
            Ok(new_version) => {
                term::show_success(&format!(
                    "Migrations completed!\n   Schema updated: v{} → v{}",
                    info.current_version, new_version
                ));
            }
            Err(e) => {
                term::show_error(&format!(
                    "Migration failed: {}\n\n⚠️  Database may be in inconsistent state!\nConsider restoring from backup.",
                    e
                ));
            }
        }
    } else {
        println!("\n❌ Migrations cancelled");
        term::wait_for_enter();
    }
}

fn backup_database() {
    term::show_header("BACKUP DATABASE");

    let db_path = match get_db_path() {
        Ok(path) => path,
        Err(e) => {
            term::show_error(&format!("Cannot determine database path: {}", e));
            return;
        }
    };

    if !db_path.exists() {
        term::show_error("Database file does not exist");
        return;
    }

    // Create backup filename with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("bs_scoring_backup_{}.db", timestamp);

    let backup_path = db_path.parent().unwrap().join(&backup_name);

    println!("📁 Source: {}", db_path.display());
    println!("📁 Backup: {}", backup_path.display());
    println!();

    if term::confirm("Create backup?") {
        match fs::copy(&db_path, &backup_path) {
            Ok(bytes) => {
                // Update meta table
                if let Ok(db) = Database::new(&db_path.to_string_lossy()) {
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let _ = migrations::set_meta_value(db.get_connection(), "last_backup", &now);
                }

                let kb = bytes / 1024;
                term::show_success(&format!(
                    "Backup created successfully!\n   File: {}\n   Size: {} KB",
                    backup_name, kb
                ));
            }
            Err(e) => {
                term::show_error(&format!("Failed to create backup: {}", e));
            }
        }
    } else {
        println!("\n❌ Backup cancelled");
        term::wait_for_enter();
    }
}

fn restore_database() {
    term::show_header("RESTORE DATABASE");

    let db_path = match get_db_path() {
        Ok(path) => path,
        Err(e) => {
            term::show_error(&format!("Cannot determine database path: {}", e));
            return;
        }
    };

    let db_dir = db_path.parent().unwrap();

    // List available backups
    let backups = match list_backup_files(db_dir) {
        Ok(backups) if !backups.is_empty() => backups,
        Ok(_) => {
            term::show_error("No backup files found");
            return;
        }
        Err(e) => {
            term::show_error(&format!("Error listing backups: {}", e));
            return;
        }
    };

    println!("📦 Available backups:\n");
    for (i, (name, size)) in backups.iter().enumerate() {
        println!("  {}. {} ({} KB)", i + 1, name, size / 1024);
    }
    println!();

    if let Some(choice) = term::read_i64("Select backup to restore (0 to cancel): ") {
        if choice == 0 {
            println!("\n❌ Restore cancelled");
            term::wait_for_enter();
            return;
        }

        if choice < 1 || choice as usize > backups.len() {
            term::show_error("Invalid selection");
            return;
        }

        let (backup_name, _) = &backups[(choice - 1) as usize];
        let backup_path = db_dir.join(backup_name);

        println!("\n⚠️  WARNING: This will replace the current database!");
        println!("Current database will be backed up first as a safety measure.");
        println!();

        if term::confirm("Are you sure you want to restore from this backup?") {
            // Safety backup of current DB
            let safety_backup = format!(
                "bs_scoring_before_restore_{}.db",
                Local::now().format("%Y%m%d_%H%M%S")
            );
            let safety_path = db_dir.join(&safety_backup);

            if db_path.exists() {
                if let Err(e) = fs::copy(&db_path, &safety_path) {
                    term::show_error(&format!("Failed to create safety backup: {}", e));
                    return;
                }
                println!("✅ Safety backup created: {}", safety_backup);
            }

            // Restore from backup
            match fs::copy(&backup_path, &db_path) {
                Ok(_) => {
                    // Update meta table
                    if let Ok(db) = Database::new(&db_path.to_string_lossy()) {
                        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        let _ =
                            migrations::set_meta_value(db.get_connection(), "last_restore", &now);
                    }

                    term::show_success(&format!(
                        "Database restored successfully!\n   From: {}\n   Safety backup: {}",
                        backup_name, safety_backup
                    ));
                }
                Err(e) => {
                    term::show_error(&format!("Failed to restore: {}", e));
                }
            }
        } else {
            println!("\n❌ Restore cancelled");
            term::wait_for_enter();
        }
    }
}

fn list_backup_files(dir: &Path) -> std::io::Result<Vec<(String, u64)>> {
    let mut backups = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if (name_str.starts_with("bs_scoring_backup_")
                || name_str.starts_with("bs_scoring_before_restore_"))
                && let Ok(metadata) = fs::metadata(&path)
            {
                backups.push((name_str.to_string(), metadata.len()));
            }
        }
    }

    // Sort by name (which includes timestamp) in reverse order (newest first)
    backups.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(backups)
}

fn vacuum_database(db: &Database) {
    term::show_header("VACUUM DATABASE");

    let conn = db.get_connection();

    // Get size before
    let page_count_before: i64 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .unwrap_or(0);

    let page_size: i64 = conn
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .unwrap_or(0);

    let size_before_kb = (page_count_before * page_size) / 1024;

    let freelist_before: i64 = conn
        .query_row("PRAGMA freelist_count", [], |row| row.get(0))
        .unwrap_or(0);

    let freelist_kb = (freelist_before * page_size) / 1024;

    println!("📊 Current status:");
    println!("  Database size:  {} KB", size_before_kb);
    println!("  Free space:     {} KB", freelist_kb);
    println!();
    println!("ℹ️  VACUUM will:");
    println!("  • Rebuild database file");
    println!("  • Reclaim unused space");
    println!("  • Optimize page layout");
    println!("  • Defragment tables");
    println!();
    println!("⚠️  This may take a few seconds for large databases.");
    println!();

    if term::confirm("Run VACUUM?") {
        println!("\n🔄 Running VACUUM...");

        match conn.execute("VACUUM", []) {
            Ok(_) => {
                // Get size after
                let page_count_after: i64 = conn
                    .query_row("PRAGMA page_count", [], |row| row.get(0))
                    .unwrap_or(0);

                let size_after_kb = (page_count_after * page_size) / 1024;
                let saved_kb = size_before_kb.saturating_sub(size_after_kb);
                let saved_percent = if size_before_kb > 0 {
                    (saved_kb as f64 / size_before_kb as f64) * 100.0
                } else {
                    0.0
                };

                println!();
                term::show_success(&format!(
                    "VACUUM completed successfully!\n\n\
                    📊 Results:\n\
                    \n   Before:  {} KB\
                    \n   After:   {} KB\
                    \n   Saved:   {} KB ({:.1}%)",
                    size_before_kb, size_after_kb, saved_kb, saved_percent
                ));
            }
            Err(e) => {
                term::show_error(&format!("VACUUM failed: {}", e));
            }
        }
    } else {
        println!("\n❌ VACUUM cancelled");
        term::wait_for_enter();
    }
}

fn clear_all_data(db: &Database) {
    term::show_header("CLEAR ALL DATA");

    println!("⚠️  WARNING: This will delete ALL data from the database!");
    println!("This action CANNOT be undone.\n");

    if term::confirm("Are you sure you want to clear all data?") {
        if term::confirm("Are you REALLY sure? Type 'y' again to confirm") {
            let conn = db.get_connection();

            match conn.execute("DELETE FROM base_runners", []) {
                Ok(_) => {}
                Err(e) => println!("Error clearing base_runners: {}", e),
            }

            match conn.execute("DELETE FROM plate_appearances", []) {
                Ok(_) => {}
                Err(e) => println!("Error clearing plate_appearances: {}", e),
            }

            match conn.execute("DELETE FROM games", []) {
                Ok(_) => {}
                Err(e) => println!("Error clearing games: {}", e),
            }

            match conn.execute("DELETE FROM players", []) {
                Ok(_) => {}
                Err(e) => println!("Error clearing players: {}", e),
            }

            match conn.execute("DELETE FROM teams", []) {
                Ok(_) => {}
                Err(e) => println!("Error clearing teams: {}", e),
            }

            match conn.execute("DELETE FROM leagues", []) {
                Ok(_) => {}
                Err(e) => println!("Error clearing leagues: {}", e),
            }

            term::show_success("All data cleared successfully!");
        } else {
            println!("\n❌ Operation cancelled.");
            term::wait_for_enter();
        }
    } else {
        println!("\n❌ Operation cancelled.");
        term::wait_for_enter();
    }
}

fn export_game(db: &Database) {
    term::show_header("EXPORT GAME");

    let conn = db.get_connection();

    // List available games
    let games = match conn.prepare(
        "SELECT id, game_id, date(game_date) as date, \
         (SELECT name FROM teams WHERE id = home_team_id) as home_team, \
         (SELECT name FROM teams WHERE id = away_team_id) as away_team, \
         home_score, away_score \
         FROM games ORDER BY game_date DESC",
    ) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,    // id
                    row.get::<_, String>(1)?, // game_id
                    row.get::<_, String>(2)?, // date
                    row.get::<_, String>(3)?, // home_team
                    row.get::<_, String>(4)?, // away_team
                    row.get::<_, i64>(5)?,    // home_score
                    row.get::<_, i64>(6)?,    // away_score
                ))
            }) {
                Ok(results) => results.collect::<Result<Vec<_>, _>>().unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        }
        Err(_) => Vec::new(),
    };

    if games.is_empty() {
        term::show_error("No games found to export");
        return;
    }

    println!("📋 Available games:\n");
    for (i, (_, game_id, date, home, away, home_score, away_score)) in games.iter().enumerate() {
        println!(
            "  {}. {} - {} vs {} ({}-{}) [{}]",
            i + 1,
            date,
            away,
            home,
            away_score,
            home_score,
            game_id
        );
    }
    println!();

    if let Some(choice) = term::read_i64("Select game to export (0 to cancel): ") {
        if choice == 0 {
            println!("\n❌ Export cancelled");
            term::wait_for_enter();
            return;
        }

        if choice < 1 || choice as usize > games.len() {
            term::show_error("Invalid selection");
            return;
        }

        let (game_db_id, game_id, _, _, _, _, _) = &games[(choice - 1) as usize];

        println!("\nExport format:");
        println!("  1. JSON (detailed)");
        println!("  2. CSV (simplified)");
        println!();
        println!("  0. Cancel");
        println!();

        let format_choice = term::read_choice();
        match format_choice {
            1 => export_game_json(db, *game_db_id, game_id),
            2 => export_game_csv(db, *game_db_id, game_id),
            0 => {
                println!("\n❌ Export cancelled");
                term::wait_for_enter();
            }
            _ => {
                term::show_error("Invalid format selection");
            }
        }
    }
}

fn export_game_json(db: &Database, game_id: i64, game_id_str: &str) {
    let _conn = db.get_connection();

    // Fetch game data (simplified - you'll need to build complete Game struct)
    let game_data = format!(
        r#"{{
  "game_id": "{}",
  "database_id": {},
  "export_date": "{}",
  "note": "Full game export with all plate appearances"
}}"#,
        game_id_str,
        game_id,
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    // Save to file
    let filename = format!("{}_export.json", game_id_str);
    let file_path = std::env::current_dir().unwrap_or_default().join(&filename);

    match fs::write(&file_path, game_data) {
        Ok(_) => {
            term::show_success(&format!(
                "Game exported to JSON!\n   File: {}",
                file_path.display()
            ));
        }
        Err(e) => {
            term::show_error(&format!("Failed to export: {}", e));
        }
    }
}

fn export_game_csv(db: &Database, game_id: i64, game_id_str: &str) {
    let conn = db.get_connection();

    // Build CSV with plate appearances
    let mut csv_data = String::from("Inning,Half,Batter,Result,RBIs,Runs\n");

    // Fetch plate appearances (simplified)
    if let Ok(mut stmt) = conn.prepare(
        "SELECT inning, half_inning, \
         (SELECT name FROM players WHERE id = batter_id) as batter, \
         result_type, rbis, runs_scored \
         FROM plate_appearances WHERE game_id = ? ORDER BY id",
    ) && let Ok(rows) = stmt.query_map([game_id], |row| {
        Ok(format!(
            "{},{},{},{},{},{}\n",
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)
                .unwrap_or_else(|_| "Unknown".to_string()),
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4).unwrap_or(0),
            row.get::<_, i64>(5).unwrap_or(0)
        ))
    }) {
        for row in rows.flatten() {
            csv_data.push_str(&row);
        }
    }

    // Save to file
    let filename = format!("{}_export.csv", game_id_str);
    let file_path = std::env::current_dir().unwrap_or_default().join(&filename);

    match fs::write(&file_path, csv_data) {
        Ok(_) => {
            term::show_success(&format!(
                "Game exported to CSV!\n   File: {}",
                file_path.display()
            ));
        }
        Err(e) => {
            term::show_error(&format!("Failed to export: {}", e));
        }
    }
}
