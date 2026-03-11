use crate::cli::commands::play_ball::play_ball;
use crate::core::menu::GameMenuChoice;
use crate::db::game_events::refactor_batter_order;
use crate::utils::cli;
use crate::{Database, Menu, Team};
use chrono::Local;
use rusqlite::{Connection, params};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::{fs, io};

#[derive(Debug, Clone, Copy)]
pub enum EditGameMenuChoice {
    EditTeams,
    EditLineups,
    ImportLineup,
    EditInningsScore,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub enum UtilitiesGameMenuChoice {
    RefactorBattersOrder,
    Back,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ImportLineupRow {
    batting_order: i32,
    defensive_position: String,
    player_number: i32,
}

pub fn handle_game_menu(db: &mut Database) {
    loop {
        match Menu::show_game_menu() {
            GameMenuChoice::NewGame => create_new_game(db),
            GameMenuChoice::ListGames => list_games(db),
            GameMenuChoice::EditGame => handle_edit_game_menu(db),
            GameMenuChoice::PlayBall => play_ball(db),
            GameMenuChoice::Utilities => handle_utilities_game_menu(db),
            GameMenuChoice::Back => break,
        }
    }
}

pub fn handle_edit_game_menu(db: &mut Database) {
    loop {
        match show_edit_game_menu() {
            EditGameMenuChoice::EditTeams => edit_teams(db),
            EditGameMenuChoice::EditLineups => edit_lineups(db),
            EditGameMenuChoice::ImportLineup => import_lineup(db),
            EditGameMenuChoice::EditInningsScore => edit_innings_score(db),
            EditGameMenuChoice::Back => break,
        }
    }
}

#[allow(clippy::while_let_loop)]
pub fn handle_utilities_game_menu(db: &mut Database) {
    loop {
        match show_utilities_game_menu() {
            UtilitiesGameMenuChoice::RefactorBattersOrder => {
                cli::show_header("REFACTOR BATTER ORDERS");
                println!(
                    "This utility will recalculate and update the 'batter_order' field in 'plate_appearances'\nbased on the current lineups and batting orders defined in 'game_lineups'."
                );
                println!(
                    "This is useful if you have made manual edits to lineups or\nif you want to ensure consistency after data imports."
                );
                println!();
                if cli::confirm("Proceed with refactoring batter orders? This cannot be undone!") {
                    match refactor_batter_order(db.get_connection_mut()) {
                        Ok(_) => cli::show_success_no_wait_for_enter(
                            "Batter orders refactored successfully!",
                        ),
                        Err(e) => {
                            cli::show_error(&format!("Error refactoring batter orders: {}", e))
                        }
                    }
                } else {
                    println!("\n❌ Refactoring cancelled.");
                }
                cli::wait_for_enter();
            }
            UtilitiesGameMenuChoice::Back => break,
        }
    }
}

pub fn show_utilities_game_menu() -> UtilitiesGameMenuChoice {
    loop {
        cli::clear_screen();
        println!("╔════════════════════════════════════════════╗");
        println!("║            🛠️  GAME UTILITIES              ║");
        println!("╚════════════════════════════════════════════╝");
        println!();
        println!("  1. 🔄 Refactor Batter Orders");
        println!();
        println!("  0. 🔙 Back to Game Menu");
        println!();
        print!("Select an option (1 or 0): ");
        io::stdout().flush().unwrap();

        let choice = cli::read_choice();
        match choice {
            1 => return UtilitiesGameMenuChoice::RefactorBattersOrder,
            0 => return UtilitiesGameMenuChoice::Back,
            _ => {
                println!("\n❌ Invalid choice. Press ENTER to continue...");
                cli::wait_for_enter();
            }
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
        println!("  3. 📥 Import Lineup (JSON/CSV)");
        println!("  4. ✏️ Edit Innings/Score");
        println!();
        println!("  0. 🔙 Back to Main Menu");
        println!();
        print!("Select an option (1-3 or 0): ");
        io::stdout().flush().unwrap();

        let choice = cli::read_choice();
        match choice {
            1 => return EditGameMenuChoice::EditTeams,
            2 => return EditGameMenuChoice::EditLineups,
            3 => return EditGameMenuChoice::ImportLineup,
            4 => return EditGameMenuChoice::EditInningsScore,
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

    let at_uses_dh = ask_team_dh("AWAY", &away_team.name);
    let away_required = if at_uses_dh { 10 } else { 9 };

    let away_lineup = match insert_team_lineup(conn, away_team_id, &away_team.name, away_required) {
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

    let ht_uses_dh = ask_team_dh("HOME", &home_team.name);
    let home_required = if ht_uses_dh { 10 } else { 9 };

    let home_lineup = match insert_team_lineup(conn, home_team_id, &home_team.name, home_required) {
        Some(lineup) => lineup,
        None => {
            println!("\n❌ Home team lineup cancelled. Game creation aborted.");
            cli::wait_for_enter();
            return;
        }
    };

    // STEP 7: Save game to database
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
                cli::show_separator(50);

                for (_id, game_id, date, venue, status_int, away, home, away_score, home_score) in
                    game_list
                {
                    use crate::models::types::GameStatus;

                    let status = GameStatus::from_i64(status_int).unwrap_or(GameStatus::Pregame);
                    let status_icon = status.icon();

                    println!(
                        "  {} {} - {} @ {} ({}-{})",
                        status_icon, date, away, home, away_score, home_score
                    );
                    println!("     Venue: {} | Status: {}", venue, status);
                    println!("     ID: {}", game_id);
                    cli::show_separator(50);
                }
            }
        }
        Err(e) => {
            cli::show_error(&format!("Error loading games: {}", e));
        }
    }

    cli::wait_for_enter();
}

/// Edit Game functions (placeholder)
fn edit_teams(_db: &Database) {
    cli::show_header("EDIT TEAMS");
    println!("🚧 Feature under development...\n");
    cli::wait_for_enter();
}

fn edit_lineups(db: &mut Database) {
    cli::show_header("EDIT LINEUPS");

    let conn = db.get_connection_mut();

    let (game_id, team_id, team_name, team_type) = match select_pregame_game_and_team(conn) {
        Some(v) => v,
        None => return,
    };

    let current_lineup: Vec<(i64, i32, String, i32, String, String)> = {
        let mut stmt = match conn.prepare(
            "SELECT gl.player_id, gl.batting_order, gl.defensive_position,
                    p.number, p.first_name, p.last_name
             FROM game_lineups gl
             JOIN players p ON gl.player_id = p.id
             WHERE gl.game_id = ?1 AND gl.team_id = ?2 AND gl.is_starting = 1
             ORDER BY gl.batting_order",
        ) {
            Ok(s) => s,
            Err(e) => {
                cli::show_error(&format!("Error loading lineup: {e}"));
                return;
            }
        };

        stmt.query_map(rusqlite::params![&game_id, team_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect()
    };

    if current_lineup.is_empty() {
        cli::show_error("No lineup found for this team!");
        return;
    }

    print_lineup(&team_name, team_type, &current_lineup);

    if !cli::confirm("Edit this lineup?") {
        println!("\n❌ Cancelled");
        cli::wait_for_enter();
        return;
    }

    println!("\n═══════════════════════════════════════");
    println!("    RE-ENTER {} LINEUP", team_name.to_uppercase());
    println!("═══════════════════════════════════════\n");

    edit_lineup_helper(conn, &game_id, team_id, &team_name, team_type);

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

fn ask_team_dh(team_label: &str, team_name: &str) -> bool {
    println!("\n═══════════════════════════════════════");
    println!("{} TEAM DH SETTING: {}", team_label, team_name);
    println!("═══════════════════════════════════════\n");

    cli::confirm("Use Designated Hitter (DH)?")
}

/// Insert lineup for a team
/// Returns: Option<Vec<(player_id, batting_order, defensive_position)>>
pub(crate) fn insert_team_lineup(
    conn: &rusqlite::Connection,
    team_id: i64,
    team_name: &str,
    required_players: usize, // 9 oppure 10
) -> Option<Vec<(i64, i32, String)>> {
    use crate::db::player::Player;
    use std::io::{self, Write};

    if required_players != 9 && required_players != 10 {
        cli::show_error("Internal error: required_players must be 9 or 10");
        return None;
    }

    let uses_dh = required_players == 10;

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

        println!(
            "\nLineup mode: {} players required ({})",
            required_players,
            if uses_dh { "DH = YES" } else { "DH = NO" }
        );

        println!("\n📋 Team Roster:\n");
        for player in &roster {
            println!(
                "  #{:<3} {} {} ({})",
                player.number, player.first_name, player.last_name, player.position
            );
        }
        println!();

        let mut lineup: Vec<(i64, i32, String)> = Vec::new();
        let mut used_positions: Vec<String> = Vec::new();
        let mut used_numbers: Vec<i32> = Vec::new();

        // Collect batting order positions 1..=9 (always)
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
                    Some(num) => println!("❌ Player #{} not found in roster!", num),
                    None => println!("❌ Invalid number!"),
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

                let input = cli::read_string("").trim().to_string();

                let position = if input.eq_ignore_ascii_case("DH") {
                    if !uses_dh {
                        println!("❌ DH is NOT allowed for this lineup (DH=NO).");
                        continue;
                    }
                    if used_positions.iter().any(|p| p == "DH") {
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
                            println!(
                                "❌ Invalid position! Enter 1-9{}",
                                if uses_dh { " or DH" } else { "" }
                            );
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
                "✓ Position {}: #{} {} {} - {}",
                pos, jersey_number, player.first_name, player.last_name, position_display
            );
        }

        // Enforce DH consistency:
        // - if uses_dh = true => must have exactly one DH among 1..9
        // - if uses_dh = false => must have zero DH (already enforced above)
        if uses_dh {
            let dh_count = used_positions.iter().filter(|p| p.as_str() == "DH").count();
            if dh_count != 1 {
                cli::show_error(
                    "DH lineup requires exactly ONE 'DH' assigned among batting spots 1-9.",
                );
                println!("🔄 Restarting lineup entry...\n");
                cli::wait_for_enter();
                continue;
            }
        }

        // If DH used, ask for pitcher info (position 10)
        if uses_dh {
            println!("\n─────────────────────────────────────");
            println!("PITCHER INFO (does not bat, required for DH lineup)");

            let pitcher_number = loop {
                match cli::read_i32("Pitcher jersey number: ") {
                    Some(num) if roster.iter().any(|p| p.number == num) => break num,
                    Some(num) => println!("❌ Player #{} not found in roster!", num),
                    None => println!("❌ Invalid number!"),
                }
            };

            let pitcher = roster.iter().find(|p| p.number == pitcher_number).unwrap();
            let pitcher_id = pitcher.id.unwrap();

            lineup.push((pitcher_id, 10, "1".to_string())); // Position 1 = Pitcher
            println!(
                "✓ Position 10: #{} {} {} - Pitcher (P)",
                pitcher_number, pitcher.first_name, pitcher.last_name
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
    conn: &Connection,
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
            } else {
                format!("Pos {}", def_pos)
            };

            if *batting_order == 10 {
                println!("{}", "═".repeat(53));
            }
            println!(
                "  {:2}. #{:<3} {:<20} {:<20} {}",
                batting_order, player.number, player.first_name, player.last_name, position_display
            );
        }
    }
    println!();
}

/// Save lineup to database
pub(crate) fn save_lineup(
    conn: &Connection,
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

fn load_starting_lineup(
    conn: &Connection,
    game_id: &str,
    team_id: i64,
) -> rusqlite::Result<Vec<LineupRow>> {
    let mut stmt = conn.prepare(
        "SELECT gl.player_id, gl.batting_order, gl.defensive_position,
                p.number, p.first_name, p.last_name
         FROM game_lineups gl
         JOIN players p ON gl.player_id = p.id
         WHERE gl.game_id = ?1 AND gl.team_id = ?2 AND gl.is_starting = 1
         ORDER BY gl.batting_order",
    )?;

    let v = stmt
        .query_map(rusqlite::params![game_id, team_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(v)
}

fn load_bench_from_roster(
    conn: &Connection,
    team_id: i64,
    current_lineup: &[(i64, i32, String, i32, String, String)],
) -> Result<Vec<(i64, i32, String, String)>, String> {
    use crate::db::player::Player;

    let starters: HashSet<i64> = current_lineup
        .iter()
        .map(|(pid, _, _, _, _, _)| *pid)
        .collect();

    let roster =
        Player::get_by_team(conn, team_id).map_err(|e| format!("Error loading roster: {e}"))?;

    // (player_id, number, first, last)
    let bench = roster
        .into_iter()
        .filter_map(|p| {
            let id = p.id?;
            if starters.contains(&id) {
                return None;
            }
            Some((id, p.number, p.first_name, p.last_name))
        })
        .collect::<Vec<_>>();

    Ok(bench)
}

fn swap_spots(
    conn: &mut Connection,
    game_id: &str,
    team_id: i64,
    a: i32,
    b: i32,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    let (a_player, a_pos): (i64, String) = tx.query_row(
        "SELECT player_id, defensive_position
         FROM game_lineups
         WHERE game_id = ?1 AND team_id = ?2 AND is_starting = 1 AND batting_order = ?3",
        rusqlite::params![game_id, team_id, a],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let (b_player, b_pos): (i64, String) = tx.query_row(
        "SELECT player_id, defensive_position
         FROM game_lineups
         WHERE game_id = ?1 AND team_id = ?2 AND is_starting = 1 AND batting_order = ?3",
        rusqlite::params![game_id, team_id, b],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    tx.execute(
        "UPDATE game_lineups
         SET player_id = ?4, defensive_position = ?5
         WHERE game_id = ?1 AND team_id = ?2 AND is_starting = 1 AND batting_order = ?3",
        rusqlite::params![game_id, team_id, a, b_player, b_pos],
    )?;

    tx.execute(
        "UPDATE game_lineups
         SET player_id = ?4, defensive_position = ?5
         WHERE game_id = ?1 AND team_id = ?2 AND is_starting = 1 AND batting_order = ?3",
        rusqlite::params![game_id, team_id, b, a_player, a_pos],
    )?;

    tx.commit()?;
    Ok(())
}

fn replace_with_roster_player(
    conn: &mut rusqlite::Connection,
    game_id: &str,
    team_id: i64,
    spot: i32,
    new_player_id: i64,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    // Evita duplicato: stesso giocatore già nello starting lineup
    let already_in_lineup: i64 = tx.query_row(
        "SELECT COUNT(1)
         FROM game_lineups
         WHERE game_id = ?1 AND team_id = ?2
           AND is_starting = 1
           AND player_id = ?3",
        rusqlite::params![game_id, team_id, new_player_id],
        |r| r.get(0),
    )?;

    if already_in_lineup > 0 {
        return Err(rusqlite::Error::InvalidQuery); // oppure errore custom
    }

    // Sostituisce direttamente il player nello spot
    tx.execute(
        "UPDATE game_lineups
         SET player_id = ?4
         WHERE game_id = ?1
           AND team_id = ?2
           AND is_starting = 1
           AND batting_order = ?3",
        rusqlite::params![game_id, team_id, spot, new_player_id],
    )?;

    tx.commit()?;
    Ok(())
}

type LineupRow = (i64, i32, String, i32, String, String);

fn print_lineup(team_name: &str, team_type: &str, lineup: &[LineupRow]) {
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

    let uses_dh = lineup.iter().any(|(_, _, pos, _, _, _)| pos == "DH");
    if uses_dh {
        println!("⚾ Designated Hitter: YES\n");
    }

    for (_player_id, batting_order, def_pos, number, first_name, last_name) in lineup {
        let position_display = if def_pos == "DH" {
            "DH".to_string()
        } else {
            format!("Pos {}", def_pos)
        };

        if *batting_order == 10 {
            println!("{}", "═".repeat(53));
        }

        println!(
            "  {:2}. #{:<3} {:<25} {}",
            batting_order,
            number,
            format!("{first_name} {last_name}"),
            position_display
        );
    }
    println!();
}

fn edit_lineup_helper(
    conn: &mut Connection,
    game_id: &str,
    team_id: i64,
    team_name: &str,
    team_type: &str,
) {
    loop {
        let lineup = match load_starting_lineup(conn, game_id, team_id) {
            Ok(v) => v,
            Err(e) => {
                cli::show_error(&format!("Error loading lineup: {e}"));
                return;
            }
        };

        // stampa lineup (puoi riusare il tuo blocco)
        print_lineup(team_name, team_type, &lineup);

        println!("\nActions:");
        println!("  1) Swap two spots");
        println!("  2) Replace a spot with bench player");
        println!("\n  0) Done\n");

        print!("Select an action: ");
        io::stdout().flush().unwrap();
        match cli::read_choice() {
            1 => {
                println!();
                let a = match cli::read_i64("Spot A (batting order): ") {
                    Some(x) => x as i32,
                    None => continue,
                };
                let b = match cli::read_i64("Spot B (batting order): ") {
                    Some(x) => x as i32,
                    None => continue,
                };
                if let Err(e) = swap_spots(conn, game_id, team_id, a, b) {
                    cli::show_error(&format!("Swap failed: {e}"));
                }
            }
            2 => {
                println!();
                let spot = match cli::read_i64("Spot to replace (batting order): ") {
                    Some(x) => x as i32,
                    None => continue,
                };

                let bench = match load_bench_from_roster(conn, team_id, &lineup) {
                    Ok(v) => v,
                    Err(msg) => {
                        cli::show_error(&msg);
                        continue;
                    }
                };

                if bench.is_empty() {
                    cli::show_error("Bench is empty");
                    continue;
                }

                // mostra panchina e fai scegliere
                for (i, (_pid, num, f, l)) in bench.iter().enumerate() {
                    println!("  {}. #{:<3} {} {}", i + 1, num, f, l);
                }

                let pick = match cli::read_i64("Select bench player (0 cancel): ") {
                    Some(0) | None => continue,
                    Some(x) if (x as usize) <= bench.len() => x as usize,
                    _ => {
                        cli::show_error("Invalid selection");
                        continue;
                    }
                };

                let bench_player_id = bench[pick - 1].0;

                if let Err(e) =
                    replace_with_roster_player(conn, game_id, team_id, spot, bench_player_id)
                {
                    cli::show_error(&format!("Replace failed: {e}"));
                }
            }
            0 => break,
            _ => cli::show_error("Invalid selection"),
        }
    }
}

fn import_lineup(db: &mut Database) {
    cli::show_header("IMPORT LINEUP");

    let conn = db.get_connection_mut();

    // 1) scegli game pregame + team (riusa la tua logica)
    // Qui assumo che tu abbia già ottenuto:
    // - game_id: String
    // - team_id: i64
    // - team_name: String
    // Se vuoi, posso adattarla al tuo codice esatto.
    let (game_id, team_id, team_name, team_type) = match select_pregame_game_and_team(conn) {
        Some(v) => v,
        None => return,
    };

    // 2) path file
    let path = cli::read_string("CSV/JSON file path: ").trim().to_string();
    if path.is_empty() {
        cli::show_error("No file path provided");
        return;
    }

    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            cli::show_error(&format!("Cannot read file: {e}"));
            return;
        }
    };

    // 3) parse
    let rows = match parse_lineup_file(&path, &content) {
        Ok(r) => r,
        Err(msg) => {
            cli::show_error(&msg);
            return;
        }
    };

    // 4) validate + resolve player_id tramite player_number nel roster team
    let resolved = match validate_and_resolve(conn, team_id, &rows) {
        Ok(v) => v,
        Err(msg) => {
            cli::show_error(&msg);
            return;
        }
    };

    // 5) save (replace)
    if let Err(e) = save_imported_lineup(conn, &game_id, team_id, &resolved) {
        cli::show_error(&format!("Import failed: {e}"));
        return;
    }

    cli::show_success(&format!(
        "Lineup imported for {} ({}) - game {}!",
        team_name, team_type, game_id
    ));
}

// ---------- parsing ----------

fn parse_lineup_file(path: &str, content: &str) -> Result<Vec<ImportLineupRow>, String> {
    let lower = path.to_lowercase();
    if lower.ends_with(".json") {
        serde_json::from_str::<Vec<ImportLineupRow>>(content)
            .map_err(|e| format!("JSON parse error: {e}"))
    } else if lower.ends_with(".csv") {
        let mut rdr = csv::Reader::from_reader(content.as_bytes());
        let mut out = Vec::new();
        for rec in rdr.deserialize::<ImportLineupRow>() {
            out.push(rec.map_err(|e| format!("CSV parse error: {e}"))?);
        }
        Ok(out)
    } else {
        Err("Unsupported format: use .csv or .json".to_string())
    }
}

// ---------- validation + resolve ----------
// ritorna: Vec<(batting_order, def_pos, player_id)>
fn validate_and_resolve(
    conn: &Connection,
    team_id: i64,
    rows: &[ImportLineupRow],
) -> Result<Vec<(i32, String, i64)>, String> {
    if rows.is_empty() {
        return Err("File is empty".to_string());
    }
    if rows.len() > 10 {
        return Err("Too many rows: max 10".to_string());
    }

    let mut seen_orders = HashSet::new();
    let mut seen_numbers = HashSet::new();

    for r in rows {
        if !(1..=10).contains(&r.batting_order) {
            return Err(format!(
                "Invalid batting_order {} (must be 1..10)",
                r.batting_order
            ));
        }
        if !seen_orders.insert(r.batting_order) {
            return Err(format!("Duplicate batting_order {}", r.batting_order));
        }
        if !seen_numbers.insert(r.player_number) {
            return Err(format!("Duplicate player_number {}", r.player_number));
        }
        if r.defensive_position.trim().is_empty() {
            return Err(format!(
                "Empty defensive_position for order {}",
                r.batting_order
            ));
        }
    }

    // carica roster team: mappa number -> player_id
    let roster_map =
        load_roster_number_map(conn, team_id).map_err(|e| format!("Error loading roster: {e}"))?;

    let mut out = Vec::new();
    for r in rows {
        let pid = roster_map.get(&r.player_number).copied().ok_or_else(|| {
            format!(
                "Player number {} not found in roster for team_id={}",
                r.player_number, team_id
            )
        })?;
        out.push((
            r.batting_order,
            r.defensive_position.trim().to_string(),
            pid,
        ));
    }

    // ordinamento per batting_order (così inserisci sempre ordinato)
    out.sort_by_key(|(bo, _, _)| *bo);
    Ok(out)
}

fn load_roster_number_map(conn: &Connection, team_id: i64) -> rusqlite::Result<HashMap<i32, i64>> {
    // ⚠️ Adatta la query in base al tuo schema roster.
    // Se hai players.team_id:
    let mut stmt = conn.prepare(
        "SELECT id, number
         FROM players
         WHERE team_id = ?1",
    )?;

    let mut map = HashMap::new();
    let mut rows = stmt.query(params![team_id])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let num: i32 = r.get(1)?;
        map.insert(num, id);
    }
    Ok(map)
}

// ---------- save (replace) ----------

fn save_imported_lineup(
    conn: &mut Connection,
    game_id: &str,
    team_id: i64,
    resolved: &[(i32, String, i64)],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    tx.execute(
        "DELETE FROM game_lineups
         WHERE game_id = ?1 AND team_id = ?2",
        params![game_id, team_id],
    )?;

    for (batting_order, def_pos, player_id) in resolved {
        tx.execute(
            "INSERT INTO game_lineups (game_id, team_id, player_id, batting_order, defensive_position, is_starting)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![game_id, team_id, player_id, batting_order, def_pos],
        )?;
    }

    tx.commit()?;
    Ok(())
}

fn select_pregame_game_and_team(
    conn: &mut rusqlite::Connection,
) -> Option<(String, i64, String, &'static str)> {
    let pregame_games: Vec<_> = {
        let mut stmt = conn
            .prepare(
                "SELECT g.id, g.game_id, g.game_date, g.venue,
                    t1.name as away_team, t1.id as away_team_id,
                    t2.name as home_team, t2.id as home_team_id
             FROM games g
             JOIN teams t1 ON g.away_team_id = t1.id
             JOIN teams t2 ON g.home_team_id = t2.id
             WHERE g.status = 1
             ORDER BY g.game_date DESC, g.id DESC",
            )
            .ok()?;

        stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })
        .ok()?
        .filter_map(Result::ok)
        .collect()
    };

    if pregame_games.is_empty() {
        println!("📭 No pre-game games found.");
        cli::wait_for_enter();
        return None;
    }

    println!("\n📋 Pre-Game Games:\n");
    for (i, (_id, game_id, date, venue, away, _away_id, home, _home_id)) in
        pregame_games.iter().enumerate()
    {
        println!("  {}. {} - {} @ {}", i + 1, date, away, home);
        println!("     Venue: {} | ID: {}", venue, game_id);
        println!();
    }

    let game_choice = match cli::read_i64("Select game (number, 0 to cancel): ") {
        Some(0) | None => return None,
        Some(choice) if choice > 0 && (choice as usize) <= pregame_games.len() => choice as usize,
        _ => {
            cli::show_error("Invalid selection");
            return None;
        }
    };

    let selected_game = &pregame_games[game_choice - 1];
    let (_id, game_id, _date, _venue, away_team, away_team_id, home_team, home_team_id) =
        selected_game;

    println!("\n═══════════════════════════════════════");
    println!("Select team:");
    println!("  1. {} (Away)", away_team);
    println!("  2. {} (Home)", home_team);
    println!("  0. Cancel");
    println!();

    match cli::read_choice() {
        1 => Some((game_id.clone(), *away_team_id, away_team.clone(), "Away")),
        2 => Some((game_id.clone(), *home_team_id, home_team.clone(), "Home")),
        0 => None,
        _ => {
            cli::show_error("Invalid selection");
            None
        }
    }
}
