use crate::utils;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub enum MainMenuChoice {
    NewGame,
    ManageLeagues,
    ManageTeams,
    Statistics,
    ManageDB,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub enum LeagueMenuChoice {
    CreateLeague,
    ViewLeagues,
    EditLeague,
    DeleteLeague,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub enum TeamMenuChoice {
    CreateTeam,
    ViewTeams,
    EditTeam,
    ManageRoster,
    ImportTeam,
    DeleteTeam,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub enum DBMenuChoice {
    ViewInfo,
    BackupDB,
    RestoreDB,
    ClearData,
    ChangeLocation,
    Back,
}

pub struct Menu;

impl Menu {
    /// Display main menu and get user choice
    pub fn show_main_menu() -> MainMenuChoice {
        loop {
            utils::cli::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║  ⚾  BASEBALL/SOFTBALL SCORER - MAIN MENU  ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. 🆕 New Game");
            println!("  2. 🏆 Manage Leagues");
            println!("  3. ⚾ Manage Teams");
            println!("  4. 📊 Statistics");
            println!("  5. 💾 Manage DB");
            println!();
            println!("  0. 🚪 Exit");
            println!();
            print!("Select an option (1-5 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return MainMenuChoice::NewGame,
                2 => return MainMenuChoice::ManageLeagues,
                3 => return MainMenuChoice::ManageTeams,
                4 => return MainMenuChoice::Statistics,
                5 => return MainMenuChoice::ManageDB,
                0 => return MainMenuChoice::Exit,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    utils::cli::wait_for_enter();
                }
            }
        }
    }

    /// Display league management menu
    pub fn show_league_menu() -> LeagueMenuChoice {
        loop {
            utils::cli::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║         🏆 LEAGUE MANAGEMENT               ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. ➕ Create New League");
            println!("  2. 📋 View Leagues");
            println!("  3. ✏️  Edit League");
            println!("  4. 🗑️  Delete League");
            println!();
            println!("  0. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-4 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return LeagueMenuChoice::CreateLeague,
                2 => return LeagueMenuChoice::ViewLeagues,
                3 => return LeagueMenuChoice::EditLeague,
                4 => return LeagueMenuChoice::DeleteLeague,
                0 => return LeagueMenuChoice::Back,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    utils::cli::wait_for_enter();
                }
            }
        }
    }

    /// Display team management menu
    pub fn show_team_menu() -> TeamMenuChoice {
        loop {
            utils::cli::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║         ⚾ TEAM MANAGEMENT                 ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. ➕ Create New Team");
            println!("  2. 📋 View Teams");
            println!("  3. ✏️  Edit Team");
            println!("  4. 👥 Manage Roster");
            println!("  5. 📥 Import Team (JSON/CSV)");
            println!("  6. 🗑️  Delete Team");
            println!();
            println!("  0. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-6 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return TeamMenuChoice::CreateTeam,
                2 => return TeamMenuChoice::ViewTeams,
                3 => return TeamMenuChoice::EditTeam,
                4 => return TeamMenuChoice::ManageRoster,
                5 => return TeamMenuChoice::ImportTeam,
                6 => return TeamMenuChoice::DeleteTeam,
                0 => return TeamMenuChoice::Back,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    utils::cli::wait_for_enter();
                }
            }
        }
    }

    pub fn show_db_menu() -> DBMenuChoice {
        loop {
            utils::cli::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║         💾 DATABASE MANAGEMENT             ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. 📋 View DB Info");
            println!("  2. 💾 Backup Database");
            println!("  3. 📥 Restore Database");
            println!("  4. 🗑️  Clear All Data");
            println!("  5. 📁 Change DB Location");
            println!();
            println!("  0. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-5 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return DBMenuChoice::ViewInfo,
                2 => return DBMenuChoice::BackupDB,
                3 => return DBMenuChoice::RestoreDB,
                4 => return DBMenuChoice::ClearData,
                5 => return DBMenuChoice::ChangeLocation,
                0 => return DBMenuChoice::Back,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    utils::cli::wait_for_enter();
                }
            }
        }
    }
}
