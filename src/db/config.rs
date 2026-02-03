use std::{fs, process};
use std::path::PathBuf;
use crate::Database;

/// Initialize database with proper error handling and user feedback
///
/// This function:
/// - Determines the platform-specific database path
/// - Creates or opens the database file
/// - Initializes the schema (creates tables if needed)
/// - Provides clear feedback to the user
///
/// Returns the initialized Database or exits the program on error
pub fn setup_db() -> Database {
    // Get platform-specific database path
    let db_path = match get_db_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("❌ Error determining database path: {}", e);
            process::exit(1);
        }
    };

    println!("\n📁 Database location: {}", db_path.display());

    let db_exists = db_path.exists();

    // Open or create database
    let db = match Database::new(&db_path.to_string_lossy()) {
        Ok(db) => {
            if db_exists {
                println!("✅ Existing database opened");
            } else {
                println!("🆕 New database created");
            }
            db
        }
        Err(e) => {
            eprintln!("❌ Error with database file: {}", e);
            eprintln!("   Path: {}", db_path.display());
            if !db_exists {
                eprintln!("   (Database file does not exist and could not be created)");
            } else {
                eprintln!("   (Database file exists but could not be opened)");
            }
            process::exit(1);
        }
    };

    // Initialize schema (IF NOT EXISTS = doesn't delete existing data)
    if let Err(e) = db.init_schema() {
        eprintln!("❌ Error initializing database schema: {}", e);
        process::exit(1);
    }

    if db_exists {
        println!("✅ Database schema verified");
    } else {
        println!("✅ Database schema initialized");
    }

    db
}
/// Get the application data directory based on the operating system
pub fn get_app_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base_dir = if cfg!(target_os = "windows") {
        // Windows: %LOCALAPPDATA%\bs_scorer
        let local_appdata = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("APPDATA"))
            .map_err(|_| "Could not find LOCALAPPDATA or APPDATA environment variable")?;
        PathBuf::from(local_appdata).join("bs_scorer")
    } else {
        // macOS and Linux: $HOME/.bs_scorer
        let home = std::env::var("HOME").map_err(|_| "Could not find HOME environment variable")?;
        PathBuf::from(home).join(".bs_scorer")
    };

    // Create directory if it doesn't exist
    if !base_dir.exists() {
        fs::create_dir_all(&base_dir)?;
    }

    Ok(base_dir)
}

/// Get the full path to the database file
pub fn get_db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_dir = get_app_data_dir()?;
    Ok(app_dir.join("baseball_scorer.db"))
}

/// Get a display-friendly path string for showing to users
pub fn get_db_path_display() -> String {
    match get_db_path() {
        Ok(path) => path.display().to_string(),
        Err(_) => "baseball_scorer.db".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_dir_creation() {
        let dir = get_app_data_dir();
        assert!(dir.is_ok());

        if let Ok(path) = dir {
            assert!(path.exists());

            // Verify it contains "bs_scorer"
            assert!(path.to_string_lossy().contains("bs_scorer"));
        }
    }

    #[test]
    fn test_db_path_has_correct_name() {
        if let Ok(path) = get_db_path() {
            assert_eq!(path.file_name().unwrap(), "baseball_scorer.db");
        }
    }

    #[test]
    fn test_platform_specific_path() {
        let dir = get_app_data_dir().unwrap();
        let path_str = dir.to_string_lossy();

        if cfg!(target_os = "windows") {
            // Should contain AppData or LOCALAPPDATA
            assert!(path_str.contains("AppData") || path_str.contains("APPDATA"));
        } else {
            // Should be in home directory and start with dot
            assert!(path_str.contains(".bs_scorer"));
        }
    }
}
