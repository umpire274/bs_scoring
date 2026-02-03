use crate::utils::cli;
use crate::{Database, League, LeagueMenuChoice, Menu};

pub fn handle_league_menu(db: &Database) {
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
    cli::show_header("CREATE NEW LEAGUE");

    let name = cli::read_string("League name: ");
    if name.is_empty() {
        cli::show_error("Name is required!");
        return;
    }

    let season = cli::read_optional_string("Season (e.g. 2026) [optional]: ");
    let description = cli::read_optional_string("Description [optional]: ");

    let mut league = League::new(name, season, description);

    match league.create(db.get_connection()) {
        Ok(id) => {
            cli::show_success(&format!("League created successfully! ID: {}", id));
        }
        Err(e) => {
            cli::show_error(&format!("Error creating: {}", e));
        }
    }
}

fn view_leagues(db: &Database) {
    cli::show_header("VIEW LEAGUES");

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

fn edit_league(db: &Database) {
    cli::show_header("EDIT LEAGUE");

    match League::get_all(db.get_connection()) {
        Ok(leagues) => {
            if leagues.is_empty() {
                cli::show_error("No league disponibile");
                return;
            }

            for (i, league) in leagues.iter().enumerate() {
                cli::show_list_item(i + 1, &league.name);
            }

            if let Some(choice) = cli::read_i64("\nSelect league to edit: ") {
                if choice < 1 || choice as usize > leagues.len() {
                    cli::show_error("Invalid selection");
                    return;
                }

                let mut league = leagues[(choice - 1) as usize].clone();

                league.name = cli::read_string(&format!("Name [{}]: ", league.name));
                league.season = cli::read_optional_string("Stagione: ");
                league.description = cli::read_optional_string("Descrizione: ");

                match league.update(db.get_connection()) {
                    Ok(_) => cli::show_success("League updated!"),
                    Err(e) => cli::show_error(&format!("Error: {}", e)),
                }
            }
        }
        Err(e) => cli::show_error(&format!("Error: {}", e)),
    }
}

fn delete_league(db: &Database) {
    cli::show_header("DELETE LEAGUE");

    match League::get_all(db.get_connection()) {
        Ok(leagues) => {
            if leagues.is_empty() {
                cli::show_error("No league disponibile");
                return;
            }

            for (i, league) in leagues.iter().enumerate() {
                cli::show_list_item(i + 1, &league.name);
            }

            if let Some(choice) = cli::read_i64("\nSelect league to delete: ") {
                if choice < 1 || choice as usize > leagues.len() {
                    cli::show_error("Invalid selection");
                    return;
                }

                let league = &leagues[(choice - 1) as usize];

                if cli::confirm(&format!(
                    "Are you sure you want to delete '{}'?",
                    league.name
                )) && let Some(id) = league.id
                {
                    match League::delete(db.get_connection(), id) {
                        Ok(_) => cli::show_success("League deleted!"),
                        Err(e) => cli::show_error(&format!("Error: {}", e)),
                    }
                }
            }
        }
        Err(e) => cli::show_error(&format!("Error: {}", e)),
    }
}
