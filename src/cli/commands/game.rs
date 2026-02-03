use crate::core::menu::GameMenuChoice;
use crate::utils::cli;
use crate::{Database, Menu, Team};
use chrono::Local;

pub fn handle_game_menu(db: &Database) {
    loop {
        match Menu::show_game_menu() {
            GameMenuChoice::NewGame => create_new_game(db),
            GameMenuChoice::ListGames => list_games(db),
            GameMenuChoice::EditGame => edit_game(db),
            GameMenuChoice::PlayBall => play_ball(db),
            GameMenuChoice::Back => break,
        }
    }
}

fn create_new_game(db: &Database) {
    cli::show_header("CREATE NEW GAME");

    let conn = db.get_connection();

    // List available teams
    match Team::get_all(conn) {
        Ok(teams) => {
            if teams.is_empty() {
                cli::show_error("No teams available. Create teams first!");
                return;
            }

            if teams.len() < 2 {
                cli::show_error("Need at least 2 teams to create a game!");
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
            let away_team_id = match cli::read_i64("Away team (number, 0 to cancel): ") {
                Some(0) | None => {
                    println!("\n❌ Game creation cancelled");
                    cli::wait_for_enter();
                    return;
                }
                Some(choice) if choice > 0 && (choice as usize) <= teams.len() => {
                    teams[(choice - 1) as usize].id.unwrap()
                }
                _ => {
                    cli::show_error("Invalid selection");
                    return;
                }
            };

            // Select home team
            let home_team_id = match cli::read_i64("Home team (number, 0 to cancel): ") {
                Some(0) | None => {
                    println!("\n❌ Game creation cancelled");
                    cli::wait_for_enter();
                    return;
                }
                Some(choice) if choice > 0 && (choice as usize) <= teams.len() => {
                    teams[(choice - 1) as usize].id.unwrap()
                }
                _ => {
                    cli::show_error("Invalid selection");
                    return;
                }
            };

            if away_team_id == home_team_id {
                cli::show_error("Away and Home teams must be different!");
                return;
            }

            // Get venue
            let venue = cli::read_string("Venue: ");
            if venue.is_empty() {
                cli::show_error("Venue is required!");
                return;
            }

            // Get game date (default today)
            let default_date = Local::now().format("%Y-%m-%d").to_string();
            let game_date_str =
                cli::read_optional_string(&format!("Game date (YYYY-MM-DD) [{}]: ", default_date))
                    .unwrap_or(default_date);

            // Generate game_id
            let game_id = format!(
                "GAME_{}_{}_vs_{}",
                Local::now().format("%Y%m%d_%H%M%S"),
                teams
                    .iter()
                    .find(|t| t.id == Some(away_team_id))
                    .unwrap()
                    .abbreviation
                    .as_ref()
                    .unwrap_or(&"AWAY".to_string()),
                teams
                    .iter()
                    .find(|t| t.id == Some(home_team_id))
                    .unwrap()
                    .abbreviation
                    .as_ref()
                    .unwrap_or(&"HOME".to_string())
            );

            // Insert game into database
            match conn.execute(
                "INSERT INTO games (game_id, home_team_id, away_team_id, venue, game_date,
                                    current_inning, current_half, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, 'Top', 'not_started')",
                rusqlite::params![game_id, home_team_id, away_team_id, venue, game_date_str],
            ) {
                Ok(_) => {
                    let away_team_name = &teams
                        .iter()
                        .find(|t| t.id == Some(away_team_id))
                        .unwrap()
                        .name;
                    let home_team_name = &teams
                        .iter()
                        .find(|t| t.id == Some(home_team_id))
                        .unwrap()
                        .name;

                    cli::show_success(&format!(
                        "Game created successfully!\n\n\
                         Game ID: {}\n\
                         Away: {}\n\
                         Home: {}\n\
                         Venue: {}\n\
                         Date: {}\n\n\
                         Use 'Play Ball!' to start scoring.",
                        game_id, away_team_name, home_team_name, venue, game_date_str
                    ));
                }
                Err(e) => {
                    cli::show_error(&format!("Failed to create game: {}", e));
                }
            }
        }
        Err(e) => {
            cli::show_error(&format!("Error loading teams: {}", e));
        }
    }
}

fn list_games(db: &Database) {
    cli::show_header("GAMES LIST");

    let conn = db.get_connection();

    let mut stmt = match conn.prepare(
        "SELECT g.id, g.game_id, g.game_date, g.venue, g.status,
                t1.name as away_team, t2.name as home_team,
                g.away_score, g.home_score, g.current_inning, g.current_half
         FROM games g
         JOIN teams t1 ON g.away_team_id = t1.id
         JOIN teams t2 ON g.home_team_id = t2.id
         ORDER BY g.game_date DESC, g.id DESC",
    ) {
        Ok(stmt) => stmt,
        Err(e) => {
            cli::show_error(&format!("Error querying games: {}", e));
            return;
        }
    };

    let games = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,     // id
            row.get::<_, String>(1)?,  // game_id
            row.get::<_, String>(2)?,  // date
            row.get::<_, String>(3)?,  // venue
            row.get::<_, String>(4)?,  // status
            row.get::<_, String>(5)?,  // away_team
            row.get::<_, String>(6)?,  // home_team
            row.get::<_, i64>(7)?,     // away_score
            row.get::<_, i64>(8)?,     // home_score
            row.get::<_, i64>(9)?,     // inning
            row.get::<_, String>(10)?, // half
        ))
    });

    match games {
        Ok(results) => {
            let game_list: Vec<_> = results.filter_map(Result::ok).collect();

            if game_list.is_empty() {
                println!("📭 No games found.\n");
            } else {
                println!("\n📋 Games ({} total):\n", game_list.len());
                cli::show_separator();

                for (
                    _id,
                    game_id,
                    date,
                    venue,
                    status,
                    away,
                    home,
                    away_score,
                    home_score,
                    inning,
                    half,
                ) in game_list
                {
                    let status_icon = match status.as_str() {
                        "not_started" => "🆕",
                        "in_progress" => "▶️",
                        "completed" => "✅",
                        "suspended" => "⏸️",
                        _ => "❓",
                    };

                    println!(
                        "  {} {} - {} @ {} ({}-{})",
                        status_icon, date, away, home, away_score, home_score
                    );
                    println!(
                        "     Venue: {} | Status: {} | Inning: {} {}",
                        venue, status, inning, half
                    );
                    println!("     ID: {}", game_id);
                    cli::show_separator();
                }
            }
        }
        Err(e) => {
            cli::show_error(&format!("Error loading games: {}", e));
        }
    }

    cli::wait_for_enter();
}

fn edit_game(_db: &Database) {
    cli::show_header("EDIT GAME");
    println!("🚧 Feature under development...\n");
    println!("Future capabilities:");
    println!("  - Edit game date");
    println!("  - Change venue");
    println!("  - Update team assignments");
    println!("  - Modify game metadata\n");
    cli::wait_for_enter();
}

fn play_ball(_db: &Database) {
    cli::show_header("PLAY BALL!");
    println!("⚾ Game Scoring Interface\n");
    println!("🚧 This is where the magic happens...\n");
    println!("Coming in next version:");
    println!("  - Select game to score");
    println!("  - Pitch-by-pitch input");
    println!("  - Real-time score display");
    println!("  - Base runner tracking");
    println!("  - Live statistics\n");
    cli::wait_for_enter();
}
