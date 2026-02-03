use crate::cli::commands::{db, game, leagues, statistics, team};
use crate::{Database, MainMenuChoice, Menu};

pub fn main_menu(db: &Database) {
    // Main menu loop
    loop {
        match Menu::show_main_menu() {
            MainMenuChoice::NewGame => game::handle_new_game(db),
            MainMenuChoice::ManageLeagues => leagues::handle_league_menu(db),
            MainMenuChoice::ManageTeams => team::handle_team_menu(db),
            MainMenuChoice::Statistics => statistics::handle_statistics(db),
            MainMenuChoice::ManageDB => db::handle_db_menu(db),
            MainMenuChoice::Exit => {
                println!("\n👋 Thank you for using Baseball Scorer!");
                println!("⚾ Play Ball!\n");
                break;
            }
        }
    }
}
