use crate::core::menu::GameMenuChoice;
use crate::utils::cli;
use crate::{Database, Menu, Team};
use chrono::Local;
use std::io;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub enum EditGameMenuChoice {
    EditTeams,
    EditLineups,
    EditInningsScore,
    Back,
}

pub fn handle_game_menu(db: &Database) {
    loop {
        match Menu::show_game_menu() {
            GameMenuChoice::NewGame => create_new_game(db),
            GameMenuChoice::ListGames => list_games(db),
            GameMenuChoice::EditGame => handle_edit_game_menu(db),
            GameMenuChoice::PlayBall => play_ball(db),
            GameMenuChoice::Back => break,
        }
    }
}

pub fn handle_edit_game_menu(db: &Database) {
    loop {
        match show_edit_game_menu() {
            EditGameMenuChoice::EditTeams => edit_teams(db),
            EditGameMenuChoice::EditLineups => edit_lineups(db),
            EditGameMenuChoice::EditInningsScore => edit_innings_score(db),
            EditGameMenuChoice::Back => break,
        }
    }
}
pub fn show_edit_game_menu() -> EditGameMenuChoice {
    loop {
        cli::clear_screen();
        println!("╔════════════════════════════════════════════╗");
        println!("║           🎮  EDIT GAME MENU               ║");
        println!("╚════════════════════════════════════════════╝");
        println!();
        println!("  1. ⚾ Edit Teams");
        println!("  2. 📋 Edit Lineups");
        println!("  3. ✏️ Edit Innings/Score");
        println!();
        println!("  0. 🔙 Back to Main Menu");
        println!();
        print!("Select an option (1-3 or 0): ");
        io::stdout().flush().unwrap();

        let choice = cli::read_choice();
        match choice {
            1 => return EditGameMenuChoice::EditTeams,
            2 => return EditGameMenuChoice::EditLineups,
            3 => return EditGameMenuChoice::EditInningsScore,
            0 => return EditGameMenuChoice::Back,
            _ => {
                println!("\n❌ Invalid choice. Press ENTER to continue...");
                cli::wait_for_enter();
            }
        }
    }
}

fn create_new_game(db: &Database) {
    cli::show_header("CREATE NEW GAME");

    let conn = db.get_connection();

    // List available teams
    let teams = match Team::get_all(conn) {
        Ok(teams) => {
            if teams.is_empty() {
                cli::show_error("No teams available. Create teams first!");
                return;
            }

            if teams.len() < 2 {
                cli::show_error("Need at least 2 teams to create a game!");
                return;
            }
            teams
        }
        Err(e) => {
            cli::show_error(&format!("Error loading teams: {}", e));
            return;
        }
    };

    // STEP 1: Select teams and basic metadata
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

    // Get team references
    let away_team = teams.iter().find(|t| t.id == Some(away_team_id)).unwrap();
    let home_team = teams.iter().find(|t| t.id == Some(home_team_id)).unwrap();

    // STEP 2: Game ID (default or custom)
    let default_game_id = format!(
        "GAME_{}_{}_vs_{}",
        Local::now().format("%Y%m%d_%H%M%S"),
        away_team
            .abbreviation
            .as_ref()
            .unwrap_or(&"AWAY".to_string()),
        home_team
            .abbreviation
            .as_ref()
            .unwrap_or(&"HOME".to_string())
    );

    println!("\nDefault Game ID: {}", default_game_id);
    let game_id = cli::read_optional_string("Custom Game ID (press ENTER for default): ")
        .unwrap_or(default_game_id);

    // STEP 3: Date and Time
    let default_date = Local::now().format("%Y-%m-%d").to_string();
    let game_date =
        cli::read_optional_string(&format!("Game date (YYYY-MM-DD) [{}]: ", default_date))
            .unwrap_or(default_date);

    let default_time = Local::now().format("%H:%M").to_string();
    let game_time = cli::read_optional_string(&format!("Game time (HH:MM) [{}]: ", default_time))
        .unwrap_or(default_time);

    // STEP 4: Venue
    let venue = cli::read_string("Venue: ");
    if venue.is_empty() {
        cli::show_error("Venue is required!");
        return;
    }

    // STEP 5: Insert lineup for AWAY team
    println!("\n═══════════════════════════════════════");
    println!("    AWAY TEAM LINEUP: {}", away_team.name);
    println!("═══════════════════════════════════════\n");

    let away_lineup = match insert_team_lineup(conn, away_team_id, &away_team.name) {
        Some(lineup) => lineup,
        None => {
            println!("\n❌ Away team lineup cancelled. Game creation aborted.");
            cli::wait_for_enter();
            return;
        }
    };

    // STEP 6: Insert lineup for HOME team
    println!("\n═══════════════════════════════════════");
    println!("    HOME TEAM LINEUP: {}", home_team.name);
    println!("═══════════════════════════════════════\n");

    let home_lineup = match insert_team_lineup(conn, home_team_id, &home_team.name) {
        Some(lineup) => lineup,
        None => {
            println!("\n❌ Home team lineup cancelled. Game creation aborted.");
            cli::wait_for_enter();
            return;
        }
    };

    // STEP 7: Save game to database
    let at_uses_dh = away_lineup.iter().any(|p| p.2 == "DH");
    let ht_uses_dh = home_lineup.iter().any(|p| p.2 == "DH");

    match conn.execute(
        "INSERT INTO games (game_id, home_team_id, away_team_id, venue, game_date, game_time,
                            at_uses_dh, ht_uses_dh, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
        rusqlite::params![
            game_id,
            home_team_id,
            away_team_id,
            venue,
            game_date,
            game_time,
            at_uses_dh,
            ht_uses_dh
        ],
    ) {
        Ok(_) => {
            // Save away team lineup
            if let Err(e) = save_lineup(conn, &game_id, away_team_id, &away_lineup) {
                cli::show_error(&format!("Failed to save away team lineup: {}", e));
                return;
            }

            // Save home team lineup
            if let Err(e) = save_lineup(conn, &game_id, home_team_id, &home_lineup) {
                cli::show_error(&format!("Failed to save home team lineup: {}", e));
                return;
            }

            cli::show_success(&format!(
                "Game created successfully!\n\n\
                 Game ID: {}\n\
                 Date: {} at {}\n\
                 Away: {} {}\n\
                 Home: {} {}\n\
                 Venue: {}\n\n\
                 Use 'Play Ball!' to start scoring.",
                game_id,
                game_date,
                game_time,
                away_team.name,
                if at_uses_dh { "(DH)" } else { "" },
                home_team.name,
                if ht_uses_dh { "(DH)" } else { "" },
                venue
            ));
        }
        Err(e) => {
            cli::show_error(&format!("Failed to create game: {}", e));
        }
    }
}

fn list_games(db: &Database) {
    cli::show_header("GAMES LIST");

    let conn = db.get_connection();

    let mut stmt = match conn.prepare(
        "SELECT g.id, g.game_id, g.game_date, g.venue, g.status,
                t1.name as away_team, t2.name as home_team,
                g.away_score, g.home_score
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
            row.get::<_, i64>(0)?,    // id
            row.get::<_, String>(1)?, // game_id
            row.get::<_, String>(2)?, // date
            row.get::<_, String>(3)?, // venue
            row.get::<_, i64>(4)?,    // status (now INTEGER)
            row.get::<_, String>(5)?, // away_team
            row.get::<_, String>(6)?, // home_team
            row.get::<_, i64>(7)?,    // away_score
            row.get::<_, i64>(8)?,    // home_score
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

                for (_id, game_id, date, venue, status_int, away, home, away_score, home_score) in
                    game_list
                {
                    use crate::models::types::GameStatus;

                    let status = GameStatus::from_i64(status_int).unwrap_or(GameStatus::Pregame);
                    let status_icon = match status {
                        GameStatus::Pregame => "🆕",
                        GameStatus::InProgress => "▶️",
                        GameStatus::Finished => "✅",
                    };

                    println!(
                        "  {} {} - {} @ {} ({}-{})",
                        status_icon, date, away, home, away_score, home_score
                    );
                    println!("     Venue: {} | Status: {}", venue, status);
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

/// Edit Game functions (placeholder)
fn edit_teams(_db: &Database) {
    cli::show_header("EDIT TEAMS");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

fn edit_lineups(db: &Database) {
    cli::show_header("EDIT LINEUPS");

    let conn = db.get_connection();

    // Query only pregame games
    let mut stmt = match conn.prepare(
        "SELECT g.id, g.game_id, g.game_date, g.venue,
                t1.name as away_team, t1.id as away_team_id,
                t2.name as home_team, t2.id as home_team_id
         FROM games g
         JOIN teams t1 ON g.away_team_id = t1.id
         JOIN teams t2 ON g.home_team_id = t2.id
         WHERE g.status = 1
         ORDER BY g.game_date DESC, g.id DESC",
    ) {
        Ok(stmt) => stmt,
        Err(e) => {
            cli::show_error(&format!("Error querying games: {}", e));
            return;
        }
    };

    let pregame_games: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,    // id
                row.get::<_, String>(1)?, // game_id
                row.get::<_, String>(2)?, // game_date
                row.get::<_, String>(3)?, // venue
                row.get::<_, String>(4)?, // away_team
                row.get::<_, i64>(5)?,    // away_team_id
                row.get::<_, String>(6)?, // home_team
                row.get::<_, i64>(7)?,    // home_team_id
            ))
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    if pregame_games.is_empty() {
        println!("📭 No pre-game games found.");
        println!("\nOnly games with status 'Pre-Game' can have their lineups edited.");
        println!("Once a game starts, lineup changes become substitutions.\n");
        cli::wait_for_enter();
        return;
    }

    // Display available games
    println!("\n📋 Pre-Game Games:\n");
    for (i, (_id, game_id, date, venue, away, _away_id, home, _home_id)) in
        pregame_games.iter().enumerate()
    {
        println!("  {}. {} - {} @ {}", i + 1, date, away, home);
        println!("     Venue: {} | ID: {}", venue, game_id);
        println!();
    }

    // Select game
    let game_choice = match cli::read_i64("Select game (number, 0 to cancel): ") {
        Some(0) | None => {
            println!("\n❌ Edit lineups cancelled");
            cli::wait_for_enter();
            return;
        }
        Some(choice) if choice > 0 && (choice as usize) <= pregame_games.len() => choice as usize,
        _ => {
            cli::show_error("Invalid selection");
            return;
        }
    };

    let selected_game = &pregame_games[game_choice - 1];
    let (_, game_id, _date, _venue, away_team, away_team_id, home_team, home_team_id) =
        selected_game;

    // Select which team's lineup to edit
    println!("\n═══════════════════════════════════════");
    println!("Select team to edit:");
    println!("  1. {} (Away)", away_team);
    println!("  2. {} (Home)", home_team);
    println!("  0. Cancel");
    println!();

    let team_choice = match cli::read_choice() {
        1 => (*away_team_id, away_team.clone(), "Away"),
        2 => (*home_team_id, home_team.clone(), "Home"),
        0 => {
            println!("\n❌ Cancelled");
            cli::wait_for_enter();
            return;
        }
        _ => {
            cli::show_error("Invalid selection");
            return;
        }
    };

    let (team_id, team_name, team_type) = team_choice;

    // Load current lineup
    let mut current_lineup_stmt = match conn.prepare(
        "SELECT gl.player_id, gl.batting_order, gl.defensive_position, p.number, p.name
         FROM game_lineups gl
         JOIN players p ON gl.player_id = p.id
         WHERE gl.game_id = ?1 AND gl.team_id = ?2 AND gl.is_starting = 1
         ORDER BY gl.batting_order",
    ) {
        Ok(stmt) => stmt,
        Err(e) => {
            cli::show_error(&format!("Error loading lineup: {}", e));
            return;
        }
    };

    let current_lineup: Vec<(i64, i32, String, i32, String)> = current_lineup_stmt
        .query_map(rusqlite::params![game_id, team_id], |row| {
            Ok((
                row.get(0)?, // player_id
                row.get(1)?, // batting_order
                row.get(2)?, // defensive_position
                row.get(3)?, // number
                row.get(4)?, // name
            ))
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    if current_lineup.is_empty() {
        cli::show_error("No lineup found for this team!");
        return;
    }

    // Display current lineup
    println!("\n╔═══════════════════════════════════════════════════╗");
    println!(
        "║ {: ^50}║",
        format!(
            "{} CURRENT LINEUP ({})",
            team_name.to_uppercase(),
            team_type
        )
    );
    println!("╚═══════════════════════════════════════════════════╝\n");

    let uses_dh = current_lineup.iter().any(|(_, _, pos, _, _)| pos == "DH");
    if uses_dh {
        println!("⚾ Designated Hitter: YES\n");
    }

    for (_player_id, batting_order, def_pos, number, name) in &current_lineup {
        let position_display = if def_pos == "DH" {
            "DH".to_string()
        } else if *batting_order == 10 {
            "P (does not bat)".to_string()
        } else {
            format!("Pos {}", def_pos)
        };

        println!(
            "  {:2}. #{:<3} {:<25} {}",
            batting_order, number, name, position_display
        );
    }
    println!();

    if !cli::confirm("Edit this lineup?") {
        println!("\n❌ Cancelled");
        cli::wait_for_enter();
        return;
    }

    // Re-enter lineup (using same function as create_new_game)
    println!("\n═══════════════════════════════════════");
    println!("    RE-ENTER {} LINEUP", team_name.to_uppercase());
    println!("═══════════════════════════════════════\n");

    let new_lineup = match insert_team_lineup(conn, team_id, &team_name) {
        Some(lineup) => lineup,
        None => {
            println!("\n❌ Lineup edit cancelled");
            cli::wait_for_enter();
            return;
        }
    };

    // Delete old lineup entries
    match conn.execute(
        "DELETE FROM game_lineups WHERE game_id = ?1 AND team_id = ?2",
        rusqlite::params![game_id, team_id],
    ) {
        Ok(_) => {}
        Err(e) => {
            cli::show_error(&format!("Failed to delete old lineup: {}", e));
            return;
        }
    }

    // Save new lineup
    if let Err(e) = save_lineup(conn, game_id, team_id, &new_lineup) {
        cli::show_error(&format!("Failed to save new lineup: {}", e));
        return;
    }

    cli::show_success(&format!(
        "Lineup updated successfully for {} ({})!\n\n\
         The lineup has been completely replaced.\n\
         Since the game is still in Pre-Game status, this is NOT a substitution.",
        team_name, team_type
    ));
}

fn edit_innings_score(_db: &Database) {
    cli::show_header("EDIT INNINGS/SCORE");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

/// Insert lineup for a team
/// Returns: Option<Vec<(player_id, batting_order, defensive_position)>>
fn insert_team_lineup(
    conn: &rusqlite::Connection,
    team_id: i64,
    team_name: &str,
) -> Option<Vec<(i64, i32, String)>> {
    use crate::db::player::Player;

    loop {
        // Get roster for this team
        let roster = match Player::get_by_team(conn, team_id) {
            Ok(players) => players,
            Err(e) => {
                cli::show_error(&format!("Error loading roster: {}", e));
                return None;
            }
        };

        if roster.len() < 12 {
            cli::show_error(&format!(
                "Team '{}' has only {} players. Need at least 12 players in roster!",
                team_name,
                roster.len()
            ));
            return None;
        }

        // Ask if using DH
        let uses_dh = cli::confirm("Use Designated Hitter (DH)?");

        println!("\n📋 Team Roster:\n");
        for player in &roster {
            println!(
                "  #{:<3} {} ({})",
                player.number, player.name, player.position
            );
        }
        println!();

        let mut lineup: Vec<(i64, i32, String)> = Vec::new();
        let mut used_positions: Vec<String> = Vec::new();
        let mut used_numbers: Vec<i32> = Vec::new();

        // Collect lineup positions 1-9 (ALWAYS 9, regardless of DH)
        for pos in 1..=9 {
            println!("\n─────────────────────────────────────");
            println!("Batting order position: {}", pos);

            // Read jersey number
            let jersey_number = loop {
                match cli::read_i32("Jersey number: ") {
                    Some(num) if roster.iter().any(|p| p.number == num) => {
                        if used_numbers.contains(&num) {
                            println!("❌ Player #{} already in lineup!", num);
                            continue;
                        }
                        break num;
                    }
                    Some(num) => {
                        println!("❌ Player #{} not found in roster!", num);
                    }
                    None => {
                        println!("❌ Invalid number!");
                    }
                }
            };

            let player = roster.iter().find(|p| p.number == jersey_number).unwrap();
            let player_id = player.id.unwrap();

            // Read defensive position
            let def_position = loop {
                print!("Defensive position (1-9");
                if uses_dh {
                    print!(" or DH");
                }
                print!("): ");
                io::stdout().flush().unwrap();

                let input = cli::read_string("");

                // Validate input
                let position = if input.to_uppercase() == "DH" {
                    if !uses_dh {
                        println!("❌ DH not being used for this team!");
                        continue;
                    }
                    if used_positions.contains(&"DH".to_string()) {
                        println!("❌ DH position already assigned!");
                        continue;
                    }
                    "DH".to_string()
                } else {
                    match input.parse::<u8>() {
                        Ok(n) if (1..=9).contains(&n) => {
                            let pos_str = n.to_string();
                            if used_positions.contains(&pos_str) {
                                println!("❌ Position {} already assigned!", n);
                                continue;
                            }
                            pos_str
                        }
                        _ => {
                            println!("❌ Invalid position! Enter 1-9 or DH");
                            continue;
                        }
                    }
                };

                break position;
            };

            used_positions.push(def_position.clone());
            used_numbers.push(jersey_number);
            lineup.push((player_id, pos, def_position.clone()));

            let position_display = if def_position == "DH" {
                "DH".to_string()
            } else {
                format!("Pos {}", def_position)
            };

            println!(
                "✓ Position {}: #{} {} - {}",
                pos, jersey_number, player.name, position_display
            );
        }

        // If DH used, ask for pitcher (informational only, position 10)
        if uses_dh {
            println!("\n─────────────────────────────────────");
            println!("PITCHER INFO (does not bat, informational only)");

            let pitcher_number = loop {
                match cli::read_i32("Pitcher jersey number: ") {
                    Some(num) if roster.iter().any(|p| p.number == num) => {
                        // Pitcher CAN be in the batting lineup if they're also playing a position
                        // This is rare but legal
                        break num;
                    }
                    Some(num) => {
                        println!("❌ Player #{} not found in roster!", num);
                    }
                    None => {
                        println!("❌ Invalid number!");
                    }
                }
            };

            let pitcher = roster.iter().find(|p| p.number == pitcher_number).unwrap();
            let pitcher_id = pitcher.id.unwrap();

            lineup.push((pitcher_id, 10, "1".to_string())); // Position 1 = Pitcher
            println!(
                "✓ Position 10: #{} {} - Pitcher (P)",
                pitcher_number, pitcher.name
            );
        }

        // Display complete lineup and ask for confirmation
        display_lineup(conn, &lineup, team_name, uses_dh);

        if cli::confirm("\nConfirm this lineup?") {
            return Some(lineup);
        } else {
            println!("\n🔄 Lineup cancelled. Restarting...\n");
            cli::wait_for_enter();
        }
    }
}

/// Display lineup for confirmation
fn display_lineup(
    conn: &rusqlite::Connection,
    lineup: &[(i64, i32, String)],
    team_name: &str,
    uses_dh: bool,
) {
    use crate::db::player::Player;

    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║ {: ^50}║", format!("{} LINEUP", team_name.to_uppercase()));
    println!("╚═══════════════════════════════════════════════════╝\n");

    if uses_dh {
        println!("⚾ Designated Hitter: YES\n");
    }

    for (player_id, batting_order, def_pos) in lineup {
        if let Ok(player) = Player::get_by_id(conn, *player_id) {
            let position_display = if def_pos == "DH" {
                "DH".to_string()
            } else if *batting_order == 10 {
                "P (does not bat)".to_string()
            } else {
                format!("Pos {}", def_pos)
            };

            println!(
                "  {:2}. #{:<3} {:<25} {}",
                batting_order, player.number, player.name, position_display
            );
        }
    }
    println!();
}

/// Save lineup to database
fn save_lineup(
    conn: &rusqlite::Connection,
    game_id: &str,
    team_id: i64,
    lineup: &[(i64, i32, String)],
) -> rusqlite::Result<()> {
    for (player_id, batting_order, def_pos) in lineup {
        conn.execute(
            "INSERT INTO game_lineups (game_id, team_id, player_id, batting_order, defensive_position)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![game_id, team_id, player_id, batting_order, def_pos],
        )?;
    }
    Ok(())
}
