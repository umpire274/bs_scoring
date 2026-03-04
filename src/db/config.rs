use crate::{Database, utils};
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;

/// Initialize database with proper error handling and user feedback
///
/// This function:
/// - Determines the platform-specific database path
/// - Creates or opens the database file
/// - Initializes the schema (creates tables if needed)
/// - Provides clear feedback to the user
///
/// Returns the initialized Database or exits the program on error
pub fn setup_db() -> Result<(Database, String, utils::boot::DbBootStatus)> {
    // 1) DB path
    let db_path: PathBuf = get_db_path().context("determining database path")?;
    let db_exists = db_path.exists();

    // 2) Open database (boot step)
    utils::boot::boot_step(1, 3, "Opening database", || Ok(()))?;

    let db = Database::new(&db_path.to_string_lossy())
        .with_context(|| format!("opening database at {}", db_path.display()))?;

    // 3) Init schema + migrations (boot step)
    utils::boot::boot_step(2, 3, "Checking schema", || Ok(()))?;

    db.init_schema().context("initializing database schema")?;

    // 4) Ready (boot step)
    let status = if db_exists {
        utils::boot::DbBootStatus::ReadyExisting
    } else {
        utils::boot::DbBootStatus::ReadyNew
    };

    utils::boot::boot_step(3, 3, "Database ready", || Ok(()))?;

    Ok((db, db_path.to_string_lossy().to_string(), status))
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
pub fn get_db_path() -> Result<PathBuf> {
    let app_dir = get_app_data_dir().map_err(|e| anyhow!("{e}"))?;
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
