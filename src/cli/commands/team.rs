use crate::{Database, League, Menu, Team, TeamMenuChoice};
use crate::utils::cli;

pub fn handle_team_menu(db: &Database) {
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
    cli::show_header("CREATE NEW TEAM");

    let name = cli::read_string("Team name: ");
    if name.is_empty() {
        cli::show_error("Name is required!");
        return;
    }

    let city = cli::read_optional_string("City [optional]: ");
    let abbreviation = cli::read_optional_string("Abbreviation (e.g. BOS) [optional]: ");
    let founded_year = cli::read_i32("Founded year [optional]: ");

    // Optional: select league
    let league_id = match League::get_all(db.get_connection()) {
        Ok(leagues) if !leagues.is_empty() => {
            println!("\nAvailable leagues:");
            for (i, league) in leagues.iter().enumerate() {
                cli::show_list_item(i + 1, &league.name);
            }
            cli::show_list_item(0, "No league");

            match cli::read_i64("\nLeague (0 for none): ") {
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
            cli::show_success(&format!("Team created successfully! ID: {}", id));
        }
        Err(e) => {
            cli::show_error(&format!("Error creating: {}", e));
        }
    }
}

fn view_teams(db: &Database) {
    cli::show_header("VIEW TEAMS");

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
                    cli::show_separator();
                }
            }
            cli::wait_for_enter();
        }
        Err(e) => {
            cli::show_error(&format!("Error loading: {}", e));
        }
    }
}

fn edit_team(_db: &Database) {
    cli::show_header("EDIT TEAM");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

fn manage_roster(_db: &Database) {
    cli::show_header("MANAGE ROSTER");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

fn import_team(_db: &Database) {
    cli::show_header("IMPORT TEAM");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

fn delete_team(db: &Database) {
    cli::show_header("DELETE TEAM");

    match Team::get_all(db.get_connection()) {
        Ok(teams) => {
            if teams.is_empty() {
                cli::show_error("No teams available");
                return;
            }

            for (i, team) in teams.iter().enumerate() {
                cli::show_list_item(i + 1, &team.name);
            }

            if let Some(choice) = cli::read_i64("\nSelect team to delete: ") {
                if choice < 1 || choice as usize > teams.len() {
                    cli::show_error("Invalid selection");
                    return;
                }

                let team = &teams[(choice - 1) as usize];

                if cli::confirm(&format!("Are you sure you want to delete '{}'?", team.name))
                    && let Some(id) = team.id
                {
                    match Team::delete(db.get_connection(), id) {
                        Ok(_) => cli::show_success("Team deleted!"),
                        Err(e) => cli::show_error(&format!("Error: {}", e)),
                    }
                }
            }
        }
        Err(e) => cli::show_error(&format!("Error: {}", e)),
    }
}
