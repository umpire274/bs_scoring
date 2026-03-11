use crate::utils;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub enum MainMenuChoice {
    ManageGames,
    ManageLeagues,
    ManageTeams,
    ManagePlayers,
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
    ImportTeam,
    DeleteTeam,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub enum DBMenuChoice {
    ViewInfo,
    ViewStatus,
    RunMigrations,
    BackupDB,
    RestoreDB,
    VacuumDB,
    ClearData,
    ExportGame,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerMenuChoice {
    AddPlayer,
    ListPlayers,
    UpdatePlayer,
    DeletePlayer,
    ChangeTeam,
    ImportExport,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub enum GameMenuChoice {
    NewGame,
    ListGames,
    EditGame,
    PlayBall,
    Utilities,
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
            println!("  1. 🎮 Game Management");
            println!("  2. 🏆 Leagues Management");
            println!("  3. ⚾ Teams Management");
            println!("  4. 👥 Player Management");
            println!("  5. 📊 Statistics");
            println!("  6. 💾 Manage DB");
            println!();
            println!("  0. 🚪 Exit");
            println!();
            print!("Select an option (1-6 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return MainMenuChoice::ManageGames,
                2 => return MainMenuChoice::ManageLeagues,
                3 => return MainMenuChoice::ManageTeams,
                4 => return MainMenuChoice::ManagePlayers,
                5 => return MainMenuChoice::Statistics,
                6 => return MainMenuChoice::ManageDB,
                0 => return MainMenuChoice::Exit,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    utils::cli::wait_for_enter();
                }
            }
        }
    }

    /// Display game management menu
    pub fn show_game_menu() -> GameMenuChoice {
        loop {
            utils::cli::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║           🎮  GAME MANAGEMENT              ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. 🆕 New Game");
            println!("  2. 📋 List Games");
            println!("  3. ✏️  Edit Game");
            println!("  4. ⚾ Play Ball!");
            println!("  5. 🧰 Utilities");
            println!();
            println!("  0. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-4 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return GameMenuChoice::NewGame,
                2 => return GameMenuChoice::ListGames,
                3 => return GameMenuChoice::EditGame,
                4 => return GameMenuChoice::PlayBall,
                5 => return GameMenuChoice::Utilities,
                0 => return GameMenuChoice::Back,
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
            println!("╔═════════════════════════════════════════════╗");
            println!("║          🏆  LEAGUES MANAGEMENT             ║");
            println!("╚═════════════════════════════════════════════╝");
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
            println!("║           ⚾  TEAMS MANAGEMENT             ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. ➕ Create New Team");
            println!("  2. 📋 View Teams");
            println!("  3. ✏️  Edit Team");
            println!("  4. 📥 Import Team (JSON/CSV)");
            println!("  5. 🗑️  Delete Team");
            println!();
            println!("  0. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-5 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return TeamMenuChoice::CreateTeam,
                2 => return TeamMenuChoice::ViewTeams,
                3 => return TeamMenuChoice::EditTeam,
                4 => return TeamMenuChoice::ImportTeam,
                5 => return TeamMenuChoice::DeleteTeam,
                0 => return TeamMenuChoice::Back,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    utils::cli::wait_for_enter();
                }
            }
        }
    }

    /// Display player management menu
    pub fn show_player_menu() -> PlayerMenuChoice {
        loop {
            utils::cli::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║          👥  PLAYER MANAGEMENT             ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. ➕ Add New Player");
            println!("  2. 📋 List All Players");
            println!("  3. ✏️  Update Player");
            println!("  4. 🗑️  Delete Player");
            println!("  5. 🔄 Change Team");
            println!("  6. 📥 Import/Export Players (JSON/CSV)");
            println!();
            println!("  0. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-6 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return PlayerMenuChoice::AddPlayer,
                2 => return PlayerMenuChoice::ListPlayers,
                3 => return PlayerMenuChoice::UpdatePlayer,
                4 => return PlayerMenuChoice::DeletePlayer,
                5 => return PlayerMenuChoice::ChangeTeam,
                6 => return PlayerMenuChoice::ImportExport,
                0 => return PlayerMenuChoice::Back,
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
            println!("║          💾  DATABASE MANAGEMENT           ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. 📋 View DB Info");
            println!("  2. 🔍 View DB Status");
            println!("  3. 🔄 Run Migrations");
            println!("  4. 💾 Backup Database");
            println!("  5. 📥 Restore Database");
            println!("  6. 🧹 Vacuum Database");
            println!("  7. 🗑️  Clear All Data");
            println!("  8. 📤 Export Game");
            println!();
            println!("  0. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-8 or 0): ");
            io::stdout().flush().unwrap();

            let choice = utils::cli::read_choice();
            match choice {
                1 => return DBMenuChoice::ViewInfo,
                2 => return DBMenuChoice::ViewStatus,
                3 => return DBMenuChoice::RunMigrations,
                4 => return DBMenuChoice::BackupDB,
                5 => return DBMenuChoice::RestoreDB,
                6 => return DBMenuChoice::VacuumDB,
                7 => return DBMenuChoice::ClearData,
                8 => return DBMenuChoice::ExportGame,
                0 => return DBMenuChoice::Back,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    utils::cli::wait_for_enter();
                }
            }
        }
    }
}
