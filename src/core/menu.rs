use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub enum MainMenuChoice {
    NewGame,
    ManageLeagues,
    ManageTeams,
    Statistics,
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

pub struct Menu;

impl Menu {
    /// Display main menu and get user choice
    pub fn show_main_menu() -> MainMenuChoice {
        loop {
            Self::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║      ⚾ BASEBALL SCORER - MAIN MENU        ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. 🆕 New Game");
            println!("  2. 🏆 Manage Leagues");
            println!("  3. ⚾ Manage Teams");
            println!("  4. 📊 Statistics");
            println!("  5. 🚪 Exit");
            println!();
            print!("Select an option (1-5): ");
            io::stdout().flush().unwrap();

            let choice = Self::read_choice();
            match choice {
                1 => return MainMenuChoice::NewGame,
                2 => return MainMenuChoice::ManageLeagues,
                3 => return MainMenuChoice::ManageTeams,
                4 => return MainMenuChoice::Statistics,
                5 => return MainMenuChoice::Exit,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    Self::wait_for_enter();
                }
            }
        }
    }

    /// Display league management menu
    pub fn show_league_menu() -> LeagueMenuChoice {
        loop {
            Self::clear_screen();
            println!("╔════════════════════════════════════════════╗");
            println!("║         🏆 LEAGUE MANAGEMENT               ║");
            println!("╚════════════════════════════════════════════╝");
            println!();
            println!("  1. ➕ Create New League");
            println!("  2. 📋 View Leagues");
            println!("  3. ✏️  Edit League");
            println!("  4. 🗑️  Delete League");
            println!("  5. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-5): ");
            io::stdout().flush().unwrap();

            let choice = Self::read_choice();
            match choice {
                1 => return LeagueMenuChoice::CreateLeague,
                2 => return LeagueMenuChoice::ViewLeagues,
                3 => return LeagueMenuChoice::EditLeague,
                4 => return LeagueMenuChoice::DeleteLeague,
                5 => return LeagueMenuChoice::Back,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    Self::wait_for_enter();
                }
            }
        }
    }

    /// Display team management menu
    pub fn show_team_menu() -> TeamMenuChoice {
        loop {
            Self::clear_screen();
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
            println!("  7. 🔙 Back to Main Menu");
            println!();
            print!("Select an option (1-7): ");
            io::stdout().flush().unwrap();

            let choice = Self::read_choice();
            match choice {
                1 => return TeamMenuChoice::CreateTeam,
                2 => return TeamMenuChoice::ViewTeams,
                3 => return TeamMenuChoice::EditTeam,
                4 => return TeamMenuChoice::ManageRoster,
                5 => return TeamMenuChoice::ImportTeam,
                6 => return TeamMenuChoice::DeleteTeam,
                7 => return TeamMenuChoice::Back,
                _ => {
                    println!("\n❌ Invalid choice. Press ENTER to continue...");
                    Self::wait_for_enter();
                }
            }
        }
    }

    /// Clear the screen (works on most terminals)
    fn clear_screen() {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    /// Read a numeric choice from user
    fn read_choice() -> u32 {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().parse().unwrap_or(0)
    }

    /// Read a string input from user
    pub fn read_string(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    /// Read an optional string (can be empty)
    pub fn read_optional_string(prompt: &str) -> Option<String> {
        let input = Self::read_string(prompt);
        if input.is_empty() { None } else { Some(input) }
    }

    /// Read an integer
    pub fn read_i32(prompt: &str) -> Option<i32> {
        let input = Self::read_string(prompt);
        input.parse().ok()
    }

    /// Read an i64
    pub fn read_i64(prompt: &str) -> Option<i64> {
        let input = Self::read_string(prompt);
        input.parse().ok()
    }

    /// Wait for user to press enter
    pub fn wait_for_enter() {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }

    /// Display a success message
    pub fn show_success(message: &str) {
        println!("\n✅ {}", message);
        println!("\nPress ENTER to continue...");
        Self::wait_for_enter();
    }

    /// Display an error message
    pub fn show_error(message: &str) {
        println!("\n❌ Error: {}", message);
        println!("\nPress ENTER to continue...");
        Self::wait_for_enter();
    }

    /// Confirm action
    pub fn confirm(message: &str) -> bool {
        print!("{} (y/n): ", message);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_lowercase() == "y"
    }

    /// Display a header
    pub fn show_header(title: &str) {
        println!("\n╔═══════════════════════════════════════════════════╗");
        println!("║ {: ^50}║", title);
        println!("╚═══════════════════════════════════════════════════╝\n");
    }

    /// Display a list item
    pub fn show_list_item(index: usize, item: &str) {
        println!("  {}. {}", index, item);
    }

    /// Show a table separator
    pub fn show_separator() {
        println!("  ─────────────────────────────────────────────────");
    }
}
