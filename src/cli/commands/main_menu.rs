use crate::cli::commands::{db, game, leagues, players, statistics, team, umpire_supervisor};
use crate::{Database, MainMenuChoice, Menu};

pub fn run_main_menu(db: &mut Database) {
    loop {
        match Menu::show_main_menu() {
            MainMenuChoice::ManageGames => game::handle_game_menu(db),
            MainMenuChoice::ManageLeagues => leagues::handle_league_menu(db),
            MainMenuChoice::ManageTeams => team::handle_team_menu(db),
            MainMenuChoice::ManagePlayers => players::handle_player_menu(db),
            MainMenuChoice::Statistics => statistics::handle_statistics(db),
            MainMenuChoice::UmpireSupervisor => {
                umpire_supervisor::handle_umpire_supervisor_menu(db)
            }
            MainMenuChoice::ManageDB => db::handle_db_menu(db),
            MainMenuChoice::Exit => {
                println!("\n👋 Thank you for using Baseball Scorer!");
                println!("⚾ Play Ball!\n");
                break;
            }
        }
    }
}
