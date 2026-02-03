mod core;
mod models;

use core::menu::{LeagueMenuChoice, MainMenuChoice, Menu, TeamMenuChoice};
use models::database::Database;
use models::league::League;
use models::team::Team;

const DB_PATH: &str = "baseball_scorer.db";

fn main() {
    // Initialize database
    let db = match Database::new(DB_PATH) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("❌ Error opening database: {}", e);
            return;
        }
    };

    if let Err(e) = db.init_schema() {
        eprintln!("❌ Error initializing database: {}", e);
        return;
    }

    println!("✅ Database initialized: {}", DB_PATH);
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Main menu loop
    loop {
        match Menu::show_main_menu() {
            MainMenuChoice::NewGame => handle_new_game(&db),
            MainMenuChoice::ManageLeagues => handle_league_menu(&db),
            MainMenuChoice::ManageTeams => handle_team_menu(&db),
            MainMenuChoice::Statistics => handle_statistics(&db),
            MainMenuChoice::Exit => {
                println!("\n👋 Thank you for using Baseball Scorer!");
                println!("⚾ Play Ball!\n");
                break;
            }
        }
    }
}

fn handle_new_game(db: &Database) {
    Menu::show_header("NEW GAME");

    let conn = db.get_connection();

    // List available teams
    match Team::get_all(conn) {
        Ok(teams) => {
            if teams.is_empty() {
                Menu::show_error("No teams available. Create teams first!");
                return;
            }

            println!("Available teams:\n");
            for (i, team) in teams.iter().enumerate() {
                Menu::show_list_item(
                    i + 1,
                    &format!(
                        "{} {}",
                        team.name,
                        team.city
                            .as_ref()
                            .map(|c| format!("({})", c))
                            .unwrap_or_default()
                    ),
                );
            }
            println!();

            // Select away team
            if let Some(away_idx) = Menu::read_i64("\nAway team (number): ") {
                if away_idx < 1 || away_idx as usize > teams.len() {
                    Menu::show_error("Invalid selection");
                    return;
                }

                // Select home team
                if let Some(home_idx) = Menu::read_i64("Home team (number): ") {
                    if home_idx < 1 || home_idx as usize > teams.len() {
                        Menu::show_error("Invalid selection");
                        return;
                    }

                    if away_idx == home_idx {
                        Menu::show_error("Teams must be different!");
                        return;
                    }

                    let venue = Menu::read_string("Venue: ");

                    Menu::show_success(&format!(
                        "Game created: {} @ {} - {}",
                        teams[(away_idx - 1) as usize].name,
                        teams[(home_idx - 1) as usize].name,
                        venue
                    ));

                    // TODO: Launch game scoring interface
                    println!("\n🚧 Scoring interface under development...");
                    Menu::wait_for_enter();
                }
            }
        }
        Err(e) => {
            Menu::show_error(&format!("Error loading teams: {}", e));
        }
    }
}

fn handle_league_menu(db: &Database) {
    loop {
        match Menu::show_league_menu() {
            LeagueMenuChoice::CreateLeague => create_league(db),
            LeagueMenuChoice::ViewLeagues => view_leagues(db),
            LeagueMenuChoice::EditLeague => edit_league(db),
            LeagueMenuChoice::DeleteLeague => delete_league(db),
            LeagueMenuChoice::Back => break,
        }
    }
}

fn create_league(db: &Database) {
    Menu::show_header("CREATE NEW LEAGUE");

    let name = Menu::read_string("League name: ");
    if name.is_empty() {
        Menu::show_error("Name is required!");
        return;
    }

    let season = Menu::read_optional_string("Season (e.g. 2026) [optional]: ");
    let description = Menu::read_optional_string("Description [optional]: ");

    let mut league = League::new(name, season, description);

    match league.create(db.get_connection()) {
        Ok(id) => {
            Menu::show_success(&format!("League created successfully! ID: {}", id));
        }
        Err(e) => {
            Menu::show_error(&format!("Error creating: {}", e));
        }
    }
}

fn view_leagues(db: &Database) {
    Menu::show_header("VIEW LEAGUES");

    match League::get_all(db.get_connection()) {
        Ok(leagues) => {
            if leagues.is_empty() {
                println!("📭 No leagues found.\n");
            } else {
                for league in leagues {
                    println!(
                        "  🏆 {} - {}",
                        league.name,
                        league.season.unwrap_or("N/A".to_string())
                    );
                    if let Some(desc) = league.description {
                        println!("     {}", desc);
                    }
                    Menu::show_separator();
                }
            }
            Menu::wait_for_enter();
        }
        Err(e) => {
            Menu::show_error(&format!("Error loading: {}", e));
        }
    }
}

fn edit_league(db: &Database) {
    Menu::show_header("EDIT LEAGUE");

    match League::get_all(db.get_connection()) {
        Ok(leagues) => {
            if leagues.is_empty() {
                Menu::show_error("No league disponibile");
                return;
            }

            for (i, league) in leagues.iter().enumerate() {
                Menu::show_list_item(i + 1, &league.name);
            }

            if let Some(choice) = Menu::read_i64("\nSelect league to edit: ") {
                if choice < 1 || choice as usize > leagues.len() {
                    Menu::show_error("Invalid selection");
                    return;
                }

                let mut league = leagues[(choice - 1) as usize].clone();

                league.name = Menu::read_string(&format!("Name [{}]: ", league.name));
                league.season = Menu::read_optional_string("Stagione: ");
                league.description = Menu::read_optional_string("Descrizione: ");

                match league.update(db.get_connection()) {
                    Ok(_) => Menu::show_success("League updated!"),
                    Err(e) => Menu::show_error(&format!("Error: {}", e)),
                }
            }
        }
        Err(e) => Menu::show_error(&format!("Error: {}", e)),
    }
}

fn delete_league(db: &Database) {
    Menu::show_header("DELETE LEAGUE");

    match League::get_all(db.get_connection()) {
        Ok(leagues) => {
            if leagues.is_empty() {
                Menu::show_error("No league disponibile");
                return;
            }

            for (i, league) in leagues.iter().enumerate() {
                Menu::show_list_item(i + 1, &league.name);
            }

            if let Some(choice) = Menu::read_i64("\nSelect league to delete: ") {
                if choice < 1 || choice as usize > leagues.len() {
                    Menu::show_error("Invalid selection");
                    return;
                }

                let league = &leagues[(choice - 1) as usize];

                if Menu::confirm(&format!(
                    "Are you sure you want to delete '{}'?",
                    league.name
                )) && let Some(id) = league.id
                {
                    match League::delete(db.get_connection(), id) {
                        Ok(_) => Menu::show_success("League deleted!"),
                        Err(e) => Menu::show_error(&format!("Error: {}", e)),
                    }
                }
            }
        }
        Err(e) => Menu::show_error(&format!("Error: {}", e)),
    }
}

fn handle_team_menu(db: &Database) {
    loop {
        match Menu::show_team_menu() {
            TeamMenuChoice::CreateTeam => create_team(db),
            TeamMenuChoice::ViewTeams => view_teams(db),
            TeamMenuChoice::EditTeam => edit_team(db),
            TeamMenuChoice::ManageRoster => manage_roster(db),
            TeamMenuChoice::ImportTeam => import_team(db),
            TeamMenuChoice::DeleteTeam => delete_team(db),
            TeamMenuChoice::Back => break,
        }
    }
}

fn create_team(db: &Database) {
    Menu::show_header("CREATE NEW TEAM");

    let name = Menu::read_string("Team name: ");
    if name.is_empty() {
        Menu::show_error("Name is required!");
        return;
    }

    let city = Menu::read_optional_string("City [optional]: ");
    let abbreviation = Menu::read_optional_string("Abbreviation (e.g. BOS) [optional]: ");
    let founded_year = Menu::read_i32("Founded year [optional]: ");

    // Optional: select league
    let league_id = match League::get_all(db.get_connection()) {
        Ok(leagues) if !leagues.is_empty() => {
            println!("\nAvailable leagues:");
            for (i, league) in leagues.iter().enumerate() {
                Menu::show_list_item(i + 1, &league.name);
            }
            Menu::show_list_item(0, "No league");

            match Menu::read_i64("\nLeague (0 for none): ") {
                Some(0) | None => None,
                Some(choice) if choice > 0 && choice as usize <= leagues.len() => {
                    leagues[(choice - 1) as usize].id
                }
                _ => None,
            }
        }
        _ => None,
    };

    let mut team = Team::new(name, league_id, city, abbreviation, founded_year);

    match team.create(db.get_connection()) {
        Ok(id) => {
            Menu::show_success(&format!("Team created successfully! ID: {}", id));
        }
        Err(e) => {
            Menu::show_error(&format!("Error creating: {}", e));
        }
    }
}

fn view_teams(db: &Database) {
    Menu::show_header("VIEW TEAMS");

    match Team::get_all(db.get_connection()) {
        Ok(teams) => {
            if teams.is_empty() {
                println!("📭 No teams found.\n");
            } else {
                for team in teams {
                    print!("  ⚾ {}", team.name);
                    if let Some(city) = team.city {
                        print!(" ({})", city);
                    }
                    if let Some(abbr) = team.abbreviation {
                        print!(" [{}]", abbr);
                    }
                    println!();
                    Menu::show_separator();
                }
            }
            Menu::wait_for_enter();
        }
        Err(e) => {
            Menu::show_error(&format!("Error loading: {}", e));
        }
    }
}

fn edit_team(_db: &Database) {
    Menu::show_header("EDIT TEAM");
    println!("🚧 Feature under development...\n");
    Menu::wait_for_enter();
}

fn manage_roster(_db: &Database) {
    Menu::show_header("MANAGE ROSTER");
    println!("🚧 Feature under development...\n");
    Menu::wait_for_enter();
}

fn import_team(_db: &Database) {
    Menu::show_header("IMPORT TEAM");
    println!("🚧 Feature under development...\n");
    Menu::wait_for_enter();
}

fn delete_team(db: &Database) {
    Menu::show_header("DELETE TEAM");

    match Team::get_all(db.get_connection()) {
        Ok(teams) => {
            if teams.is_empty() {
                Menu::show_error("No teams available");
                return;
            }

            for (i, team) in teams.iter().enumerate() {
                Menu::show_list_item(i + 1, &team.name);
            }

            if let Some(choice) = Menu::read_i64("\nSelect team to delete: ") {
                if choice < 1 || choice as usize > teams.len() {
                    Menu::show_error("Invalid selection");
                    return;
                }

                let team = &teams[(choice - 1) as usize];

                if Menu::confirm(&format!("Are you sure you want to delete '{}'?", team.name))
                    && let Some(id) = team.id
                {
                    match Team::delete(db.get_connection(), id) {
                        Ok(_) => Menu::show_success("Team deleted!"),
                        Err(e) => Menu::show_error(&format!("Error: {}", e)),
                    }
                }
            }
        }
        Err(e) => Menu::show_error(&format!("Error: {}", e)),
    }
}

fn handle_statistics(_db: &Database) {
    Menu::show_header("STATISTICS");
    println!("🚧 Statistics module under development...\n");
    println!("  Here you will be able to view:");
    println!("  - Player statistics");
    println!("  - Batting average, ERA, OPS");
    println!("  - League standings");
    println!("  - Game history\n");
    Menu::wait_for_enter();
}
