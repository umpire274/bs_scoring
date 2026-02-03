use bs_scoring::cli::commands::{db, game, leagues, statistics, team};
use bs_scoring::core::menu::{MainMenuChoice, Menu};
use bs_scoring::setup_db;

fn main() {
    let db = setup_db();
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!();

    // Main menu loop
    loop {
        match Menu::show_main_menu() {
            MainMenuChoice::NewGame => game::handle_new_game(&db),
            MainMenuChoice::ManageLeagues => leagues::handle_league_menu(&db),
            MainMenuChoice::ManageTeams => team::handle_team_menu(&db),
            MainMenuChoice::Statistics => statistics::handle_statistics(&db),
            MainMenuChoice::ManageDB => db::handle_db_menu(&db),
            MainMenuChoice::Exit => {
                println!("\n👋 Thank you for using Baseball Scorer!");
                println!("⚾ Play Ball!\n");
                break;
            }
        }
    }
}
