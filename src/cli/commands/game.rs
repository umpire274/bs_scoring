use crate::{Database, Team};
use crate::utils::cli;

pub fn handle_new_game(db: &Database) {
    cli::show_header("NEW GAME");

    let conn = db.get_connection();

    // List available teams
    match Team::get_all(conn) {
        Ok(teams) => {
            if teams.is_empty() {
                cli::show_error("No teams available. Create teams first!");
                return;
            }

            println!("Available teams:\n");
            for (i, team) in teams.iter().enumerate() {
                cli::show_list_item(
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
            if let Some(away_idx) = cli::read_i64("\nAway team (number): ") {
                if away_idx < 1 || away_idx as usize > teams.len() {
                    cli::show_error("Invalid selection");
                    return;
                }

                // Select home team
                if let Some(home_idx) = cli::read_i64("Home team (number): ") {
                    if home_idx < 1 || home_idx as usize > teams.len() {
                        cli::show_error("Invalid selection");
                        return;
                    }

                    if away_idx == home_idx {
                        cli::show_error("Teams must be different!");
                        return;
                    }

                    let venue = cli::read_string("Venue: ");

                    cli::show_success(&format!(
                        "Game created: {} @ {} - {}",
                        teams[(away_idx - 1) as usize].name,
                        teams[(home_idx - 1) as usize].name,
                        venue
                    ));

                    // TODO: Launch game scoring interface
                    println!("\n🚧 Scoring interface under development...");
                    cli::wait_for_enter();
                }
            }
        }
        Err(e) => {
            cli::show_error(&format!("Error loading teams: {}", e));
        }
    }
}

