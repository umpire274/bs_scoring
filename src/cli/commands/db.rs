use crate::core::menu::DBMenuChoice;
use crate::utils::cli;
use crate::{get_db_path, get_db_path_display, Database, Menu};

pub fn handle_db_menu(db: &Database) {
    loop {
        match Menu::show_db_menu() {
            DBMenuChoice::ViewInfo => view_db_info(db),
            DBMenuChoice::BackupDB => backup_database(db),
            DBMenuChoice::RestoreDB => restore_database(db),
            DBMenuChoice::ClearData => clear_all_data(db),
            DBMenuChoice::ChangeLocation => change_db_location(),
            DBMenuChoice::Back => break,
        }
    }
}

fn view_db_info(db: &Database) {
    cli::show_header("DATABASE INFO");

    println!("📁 Location: {}", get_db_path_display());

    let conn = db.get_connection();

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
        && let Ok(metadata) = std::fs::metadata(&path)
    {
        let size_kb = metadata.len() / 1024;
        println!("\n💾 Database size: {} KB", size_kb);
    }

    cli::wait_for_enter();
}

fn backup_database(_db: &Database) {
    cli::show_header("BACKUP DATABASE");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

fn restore_database(_db: &Database) {
    cli::show_header("RESTORE DATABASE");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

fn clear_all_data(db: &Database) {
    cli::show_header("CLEAR ALL DATA");

    println!("⚠️  WARNING: This will delete ALL data from the database!");
    println!("This action CANNOT be undone.\n");

    if cli::confirm("Are you sure you want to clear all data?") {
        if cli::confirm("Are you REALLY sure? Type 'y' again to confirm") {
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

            cli::show_success("All data cleared successfully!");
        } else {
            println!("\n❌ Operation cancelled.");
            cli::wait_for_enter();
        }
    } else {
        println!("\n❌ Operation cancelled.");
        cli::wait_for_enter();
    }
}

fn change_db_location() {
    cli::show_header("CHANGE DB LOCATION");
    println!("🚧 Feature under development...\n");
    println!("Current location: {}", get_db_path_display());
    cli::wait_for_enter();
}
