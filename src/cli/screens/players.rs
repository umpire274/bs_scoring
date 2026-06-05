use crate::cli::menu::PlayerMenuChoice;
use crate::db::player::{NewPlayer, Player};
use crate::models::player_traits::{BatSide, ThrowHand};
use crate::models::types::Position;
use crate::utils::term;
use crate::utils::term::choose_enum;
use crate::{Database, League, Menu, Team};
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

pub fn handle_player_menu(db: &Database) {
    loop {
        match Menu::show_player_menu() {
            PlayerMenuChoice::AddPlayer => add_player(db),
            PlayerMenuChoice::ListPlayers => list_players(db),
            PlayerMenuChoice::UpdatePlayer => update_player(db),
            PlayerMenuChoice::DeletePlayer => delete_player(db),
            PlayerMenuChoice::ChangeTeam => change_team(db),
            PlayerMenuChoice::ImportExport => import_export_menu(db),
            PlayerMenuChoice::Back => break,
        }
    }
}

fn import_export_menu(db: &Database) {
    loop {
        term::show_header("IMPORT/EXPORT PLAYERS");
        println!("  1. 📥 Import from CSV");
        println!("  2. 📥 Import from JSON");
        println!("  3. 📤 Export to CSV");
        println!("  4. 📤 Export to JSON");
        println!("  5. 📥 Download CSV template");
        println!("  6. 📥 Download JSON template");
        println!();
        println!("  0. 🔙 Back");
        println!();
        print!("Select an option: ");
        io::stdout().flush().unwrap();

        match term::read_choice() {
            1 => import_csv(db),
            2 => import_json(db),
            3 => export_csv(db),
            4 => export_json(db),
            5 => {
                if let Err(e) = download_csv_template() {
                    term::show_error(&format!("{e}"));
                }
            }
            6 => {
                if let Err(e) = download_json_template() {
                    term::show_error(&format!("{e}"));
                }
            }
            0 => break,
            _ => {
                println!("\n❌ Invalid choice. Press ENTER to continue...");
                term::wait_for_enter();
            }
        }
    }
}

fn import_csv(db: &Database) {
    term::show_header("IMPORT PLAYERS FROM CSV");

    let filepath = term::read_string("CSV file path: ");
    if filepath.is_empty() {
        term::show_error("File path is required!");
        return;
    }

    let path = Path::new(&filepath);
    if !path.exists() {
        term::show_error("File not found!");
        return;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            term::show_error(&format!("Failed to read file: {}", e));
            return;
        }
    };

    let conn = db.get_connection();
    let mut imported = 0;
    let mut errors = 0;

    // CSV format supported:
    // old: team,number,first_name,last_name,position
    // current: team,number,away_number,first_name,last_name,position,throw,bat
    // If away_number is omitted, it defaults to number.
    for (line_num, line) in content.lines().enumerate() {
        if line_num == 0 && line.to_lowercase().contains("team") {
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if !matches!(parts.len(), 5 | 7 | 8) {
            println!(
                "⚠️  Line {}: Invalid format (expected 5, 7, or 8 fields, got {})",
                line_num + 1,
                parts.len()
            );
            errors += 1;
            continue;
        }

        let team_name = parts[0];
        let number = match parts[1].parse::<i32>() {
            Ok(n) if (0..=99).contains(&n) => n,
            _ => {
                println!(
                    "⚠️  Line {}: Invalid jersey number '{}'",
                    line_num + 1,
                    parts[1]
                );
                errors += 1;
                continue;
            }
        };
        let has_away_number = parts.len() == 8;
        let away_number = if has_away_number {
            if parts[2].is_empty() {
                number
            } else {
                match parts[2].parse::<i32>() {
                    Ok(n) if (0..=99).contains(&n) => n,
                    _ => {
                        println!(
                            "⚠️  Line {}: Invalid away jersey number '{}'",
                            line_num + 1,
                            parts[2]
                        );
                        errors += 1;
                        continue;
                    }
                }
            }
        } else {
            number
        };

        let data_offset = if has_away_number { 1 } else { 0 };
        let first_name = parts[2 + data_offset].to_string();
        let last_name = parts[3 + data_offset].to_string();
        let position_num = match parts[4 + data_offset].parse::<u8>() {
            Ok(n) if (1..=9).contains(&n) => n,
            _ => {
                println!(
                    "⚠️  Line {}: Invalid position '{}'",
                    line_num + 1,
                    parts[4 + data_offset]
                );
                errors += 1;
                continue;
            }
        };

        let raw_throw = parts.get(5 + data_offset).copied().unwrap_or("");
        let raw_bat = parts.get(6 + data_offset).copied().unwrap_or("");

        let throw = if raw_throw.is_empty() {
            None
        } else {
            match raw_throw.parse::<ThrowHand>() {
                Ok(v) => Some(v),
                Err(_) => {
                    println!(
                        "⚠️  Line {}: Invalid throw value '{}'",
                        line_num + 1,
                        raw_throw
                    );
                    errors += 1;
                    continue;
                }
            }
        };

        let bat = if raw_bat.is_empty() {
            None
        } else {
            match raw_bat.parse::<BatSide>() {
                Ok(v) => Some(v),
                Err(_) => {
                    println!("⚠️  Line {}: Invalid bat value '{}'", line_num + 1, raw_bat);
                    errors += 1;
                    continue;
                }
            }
        };

        let team_id = match get_or_create_team(conn, team_name) {
            Ok(id) => id,
            Err(e) => {
                println!(
                    "⚠️  Line {}: Failed to get/create team '{}': {}",
                    line_num + 1,
                    team_name,
                    e
                );
                errors += 1;
                continue;
            }
        };

        let position = Position::from_number(position_num).unwrap();
        let mut player = Player::new(NewPlayer {
            team_id,
            number,
            away_number,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            position,
            throw,
            bat,
        });

        match player.create(conn) {
            Ok(_) => {
                let throw_str = throw.map(|p| p.as_str()).unwrap_or("-");
                let bat_str = bat.map(|b| b.as_str()).unwrap_or("-");

                println!(
                    "✓ Imported: #{} {} {} ({}) - {} [throw: {}, bat: {}]",
                    number, first_name, last_name, team_name, position, throw_str, bat_str
                );
                imported += 1;
            }
            Err(e) => {
                println!("⚠️  Line {}: Failed to create player: {}", line_num + 1, e);
                errors += 1;
            }
        }
    }

    println!("\n═══════════════════════════════════════");
    println!("✅ Import complete!");
    println!("   Imported: {}", imported);
    if errors > 0 {
        println!("   Errors:   {}", errors);
    }
    println!("═══════════════════════════════════════\n");
    term::wait_for_enter();
}

fn import_json(db: &Database) {
    term::show_header("IMPORT PLAYERS FROM JSON");

    let filepath = term::read_string("JSON file path: ");
    if filepath.is_empty() {
        term::show_error("File path is required!");
        return;
    }

    let path = Path::new(&filepath);
    if !path.exists() {
        term::show_error("File not found!");
        return;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            term::show_error(&format!("Failed to read file: {}", e));
            return;
        }
    };

    let players_data: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(data) => data,
        Err(e) => {
            term::show_error(&format!("Invalid JSON format: {}", e));
            return;
        }
    };

    let conn = db.get_connection();
    let mut imported = 0;
    let mut errors = 0;

    for (idx, player_data) in players_data.iter().enumerate() {
        let team_name = match player_data.get("team").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => {
                println!("⚠️  Player {}: Missing 'team' field", idx + 1);
                errors += 1;
                continue;
            }
        };

        let number = match player_data.get("number").and_then(|v| v.as_i64()) {
            Some(n) if (0..=99).contains(&n) => n as i32,
            _ => {
                println!("⚠️  Player {}: Invalid 'number' field", idx + 1);
                errors += 1;
                continue;
            }
        };

        let away_number = match player_data.get("away_number") {
            None => number,

            Some(value) => match value.as_i64() {
                Some(n) if (0..=99).contains(&n) => n as i32,
                Some(_) => {
                    println!("⚠️  Player {}: Invalid 'away_number' field", idx + 1);
                    errors += 1;
                    continue;
                }
                None => {
                    println!(
                        "⚠️  Player {}: Invalid 'away_number' field: must be an integer",
                        idx + 1
                    );
                    errors += 1;
                    continue;
                }
            },
        };

        let first_name = match player_data.get("first_name").and_then(|v| v.as_str()) {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => {
                println!(
                    "⚠️  Player {}: Missing or empty 'first_name' field",
                    idx + 1
                );
                errors += 1;
                continue;
            }
        };

        let last_name = match player_data.get("last_name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => String::new(),
        };

        let position_num = match player_data.get("position").and_then(|v| v.as_i64()) {
            Some(n) if (1..=9).contains(&n) => n as u8,
            _ => {
                println!("⚠️  Player {}: Invalid 'position' field", idx + 1);
                errors += 1;
                continue;
            }
        };

        let raw_throw = player_data
            .get("throw")
            .or_else(|| player_data.get("pitch"))
            .and_then(|v| v.as_str());
        let raw_bat = player_data.get("bat").and_then(|v| v.as_str());

        let throw = match raw_throw {
            Some(s) if !s.trim().is_empty() => match s.parse::<ThrowHand>() {
                Ok(v) => Some(v),
                Err(_) => {
                    println!("⚠️  Player {}: Invalid throw value '{}'", idx + 1, s);
                    errors += 1;
                    continue;
                }
            },
            _ => None,
        };

        let bat = match raw_bat {
            Some(s) if !s.trim().is_empty() => match s.parse::<BatSide>() {
                Ok(v) => Some(v),
                Err(_) => {
                    println!("⚠️  Player {}: Invalid bat value '{}'", idx + 1, s);
                    errors += 1;
                    continue;
                }
            },
            _ => None,
        };

        let team_id = match get_or_create_team(conn, team_name) {
            Ok(id) => id,
            Err(e) => {
                println!(
                    "⚠️  Player {}: Failed to get/create team '{}': {}",
                    idx + 1,
                    team_name,
                    e
                );
                errors += 1;
                continue;
            }
        };

        let position = Position::from_number(position_num).unwrap();
        let mut player = Player::new(NewPlayer {
            team_id,
            number,
            away_number,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            position,
            throw,
            bat,
        });

        match player.create(conn) {
            Ok(_) => {
                let throw_str = throw.map(|p| p.as_str()).unwrap_or("-");
                let bat_str = bat.map(|b| b.as_str()).unwrap_or("-");

                println!(
                    "✓ Imported: #{} {} {} ({}) - {} [throw: {}, bat: {}]",
                    number, first_name, last_name, team_name, position, throw_str, bat_str
                );
                imported += 1;
            }
            Err(e) => {
                println!("⚠️  Player {}: Failed to create player: {}", idx + 1, e);
                errors += 1;
            }
        }
    }

    println!("\n═══════════════════════════════════════");
    println!("✅ Import complete!");
    println!("   Imported: {}", imported);
    if errors > 0 {
        println!("   Errors:   {}", errors);
    }
    println!("═══════════════════════════════════════\n");
    term::wait_for_enter();
}

fn export_csv(db: &Database) {
    term::show_header("EXPORT PLAYERS TO CSV");

    let conn = db.get_connection();
    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        term::show_error("No players to export!");
        return;
    }

    let filepath = term::read_string("Output CSV file path (e.g., players.csv): ");
    if filepath.is_empty() {
        term::show_error("File path is required!");
        return;
    }

    let mut csv_content =
        String::from("team,number,away_number,first_name,last_name,position,throw,bat\n");

    for (player, team_name) in &players {
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            team_name,
            player.number,
            player.away_number,
            player.first_name,
            player.last_name,
            player.position.to_number(),
            player.throw.map(|p| p.as_str()).unwrap_or(""),
            player.bat.map(|b| b.as_str()).unwrap_or("")
        ));
    }

    match fs::write(&filepath, csv_content) {
        Ok(_) => {
            term::show_success(&format!(
                "Exported {} players to '{}'\n\nFormat: team,number,away_number,first_name,last_name,position,throw,bat",
                players.len(),
                filepath
            ));
        }
        Err(e) => {
            term::show_error(&format!("Failed to write file: {}", e));
        }
    }
}

fn export_json(db: &Database) {
    term::show_header("EXPORT PLAYERS TO JSON");

    let conn = db.get_connection();
    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        term::show_error("No players to export!");
        return;
    }

    let filepath = term::read_string("Output JSON file path (e.g., players.json): ");
    if filepath.is_empty() {
        term::show_error("File path is required!");
        return;
    }

    let mut players_json = Vec::new();

    for (player, team_name) in &players {
        let player_obj = serde_json::json!({
            "team": team_name,
            "number": player.number,
            "away_number": player.away_number,
            "first_name": player.first_name,
            "last_name": player.last_name,
            "position": player.position.to_number(),
            "throw": player.throw.map(|p| p.as_str()).unwrap_or(""),
            "bat": player.bat.map(|b| b.as_str()).unwrap_or("")
        });
        players_json.push(player_obj);
    }

    let json_content = match serde_json::to_string_pretty(&players_json) {
        Ok(json) => json,
        Err(e) => {
            term::show_error(&format!("Failed to serialize JSON: {}", e));
            return;
        }
    };

    match fs::write(&filepath, json_content) {
        Ok(_) => {
            term::show_success(&format!(
                "Exported {} players to '{}'",
                players.len(),
                filepath
            ));
        }
        Err(e) => {
            term::show_error(&format!("Failed to write file: {}", e));
        }
    }
}

fn get_or_create_team(conn: &rusqlite::Connection, team_name: &str) -> rusqlite::Result<i64> {
    // Try to find existing team
    let mut stmt = conn.prepare("SELECT id FROM teams WHERE name = ?1")?;

    match stmt.query_row([team_name], |row| row.get(0)) {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Team doesn't exist, create it
            let mut team = Team::new(team_name.to_string(), None, None, None, None);
            team.create(conn)
        }
        Err(e) => Err(e),
    }
}

fn add_player(db: &Database) {
    term::show_header("ADD NEW PLAYER");

    let conn = db.get_connection();

    // List available teams
    match Team::get_all(conn) {
        Ok(teams) => {
            if teams.is_empty() {
                term::show_error("No teams available. Create a team first!");
                return;
            }

            println!("Available teams:\n");
            for (i, team) in teams.iter().enumerate() {
                term::show_list_item(
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

            if let Some(team_choice) = term::read_i64("Select team (0 to cancel): ") {
                if team_choice == 0 {
                    println!("\n❌ Operation cancelled");
                    term::wait_for_enter();
                    return;
                }

                if team_choice < 1 || team_choice as usize > teams.len() {
                    term::show_error("Invalid team selection");
                    return;
                }

                let team = &teams[(team_choice - 1) as usize];
                let team_id = team.id.unwrap();

                // Get player info
                let first_name = term::read_string("First name: ");
                if first_name.is_empty() {
                    term::show_error("First name is required!");
                    return;
                }

                let last_name = term::read_string("Last name: ");

                let number = match term::read_i32("Home jersey number: ") {
                    Some(n) if (0..=99).contains(&n) => n,
                    _ => {
                        term::show_error("Invalid home jersey number (0-99)");
                        return;
                    }
                };

                let away_number = match term::read_i32("Away jersey number [same as home]: ") {
                    Some(n) if (0..=99).contains(&n) => n,
                    Some(_) => {
                        term::show_error("Invalid away jersey number (0-99)");
                        return;
                    }
                    None => number,
                };

                // Select position
                println!("\nDefensive positions:");
                println!("  1. Pitcher");
                println!("  2. Catcher");
                println!("  3. First Base");
                println!("  4. Second Base");
                println!("  5. Third Base");
                println!("  6. Shortstop");
                println!("  7. Left Field");
                println!("  8. Center Field");
                println!("  9. Right Field");
                println!();

                print!("Select a defensive position (1-9): ");
                io::stdout().flush().unwrap();
                let position = match term::read_choice() {
                    n if (1..=9).contains(&n) => Position::from_number(n as u8).unwrap(),
                    _ => {
                        term::show_error("Invalid position");
                        return;
                    }
                };

                // Select throw hand (optional)
                let throw = term::choose_enum_optional::<ThrowHand>();

                // Select batting side (optional)
                let bat = term::choose_enum_optional::<BatSide>();

                // Create player
                let mut player = Player::new(NewPlayer {
                    team_id,
                    number,
                    away_number,
                    first_name: first_name.clone(),
                    last_name: last_name.clone(),
                    position,
                    throw,
                    bat,
                });

                match player.create(conn) {
                    Ok(id) => {
                        term::show_success(&format!(
                            "Player created successfully!\n\n   {:<14} {}\n   {:<14} {} {}\n   {:<14} {}\n   {:<14} {}\n   {:<14} {}\n   {:<14} {:?}\n   {:<14} {}\n   {:<14} {}\n",
                            "ID:",
                            id,
                            "Name:",
                            first_name,
                            last_name,
                            "Home number:",
                            number,
                            "Away number:",
                            away_number,
                            "Team:",
                            team.name,
                            "Position:",
                            position,
                            "Throw hand:",
                            throw.map(|p| p.as_str()).unwrap_or("None"),
                            "Batting side:",
                            bat.map(|b| b.as_str()).unwrap_or("None")
                        ));
                    }
                    Err(e) => {
                        term::show_error(&format!("Failed to create player: {}", e));
                    }
                }
            }
        }
        Err(e) => {
            term::show_error(&format!("Error loading teams: {}", e));
        }
    }
}

fn list_players(db: &Database) {
    term::show_header("ALL PLAYERS");

    let conn = db.get_connection();

    // Option to filter by team
    println!("Filter options:");
    println!("  1. All players (all teams)");
    println!("  2. Filter by team");
    println!();

    print!("Select filter option: ");
    io::stdout().flush().unwrap();
    let filter_choice = term::read_choice();
    println!();

    let players = if filter_choice == 2 {
        // List teams
        match Team::get_all(conn) {
            Ok(teams) if !teams.is_empty() => {
                println!("\nAvailable teams:\n");
                for (i, team) in teams.iter().enumerate() {
                    term::show_list_item(i + 1, &team.name);
                }
                println!();

                if let Some(team_choice) = term::read_i64("Select team (0 for all): ") {
                    if team_choice == 0 {
                        get_all_players_with_teams(conn)
                    } else if team_choice > 0 && (team_choice as usize) <= teams.len() {
                        let team_id = teams[(team_choice - 1) as usize].id.unwrap();
                        match Player::get_by_team(conn, team_id) {
                            Ok(players) => players
                                .into_iter()
                                .map(|p| (p, teams[(team_choice - 1) as usize].name.clone()))
                                .collect(),
                            Err(_) => Vec::new(),
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            _ => get_all_players_with_teams(conn),
        }
    } else {
        get_all_players_with_teams(conn)
    };

    if players.is_empty() {
        println!("📭 No players found.\n");
    } else {
        println!("\n📋 Players ({} total):\n", players.len());
        term::show_separator(72);

        for (player, team_name) in players {
            println!(
                "  H#{:<3} A#{:<3} {:<25} {:<15} {:<12} (P:{:<3} B:{:<1})",
                player.number,
                player.away_number,
                player.full_name(),
                format!("({})", team_name),
                format!("{:?}", player.position),
                player.throw.map(|p| p.as_str()).unwrap_or("-"),
                player.bat.map(|b| b.as_str()).unwrap_or("-")
            );
        }
        term::show_separator(72);
    }

    term::wait_for_enter();
}

fn get_all_players_with_teams(conn: &rusqlite::Connection) -> Vec<(Player, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.team_id, p.number, p.first_name, p.last_name, p.position, p.pitch, p.bat, p.is_active,
                    COALESCE(p.away_number, p.number) AS away_number,
                    t.name as team_name
             FROM players p
             JOIN teams t ON p.team_id = t.id
             WHERE p.is_active = 1
             ORDER BY t.name, p.number",
        )
        .unwrap();

    let players = stmt.query_map([], Player::from_row_with_team).unwrap();

    players.flatten().collect()
}

fn update_player(db: &Database) {
    let conn = db.get_connection();

    let Some(team) = select_team_for_player_action(conn, "UPDATE PLAYER") else {
        return;
    };

    let Some(team_id) = team.id else {
        term::show_error("Selected team has no valid ID");
        return;
    };

    loop {
        term::show_header(&format!("UPDATE PLAYER - {}", team.name));

        let players = get_players_for_team_with_name(conn, team_id, &team.name);

        if players.is_empty() {
            println!("📭 No players available for this team.\n");
            term::wait_for_enter();
            break;
        }

        println!("Players for {}:\n", team.name);
        display_player_list(&players);
        println!("\n  0. 🔙 Back\n");

        let Some(choice) = term::read_i64("Select player to update (0 to go back): ") else {
            break;
        };

        if choice == 0 {
            break;
        }

        if choice < 1 || choice as usize > players.len() {
            term::show_error("Invalid selection");
            continue;
        }

        let (mut player, _) = players[(choice - 1) as usize].clone();

        println!("\nCurrent values (press ENTER to keep):\n");

        let new_first = term::read_string(&format!("First name [{}]: ", player.first_name));
        if !new_first.is_empty() {
            player.first_name = new_first;
        }

        let new_last = term::read_string(&format!("Last name [{}]: ", player.last_name));
        if !new_last.is_empty() {
            player.last_name = new_last;
        }

        if let Some(new_number) = term::read_i32(&format!("Home number [{}]: ", player.number)) {
            if (0..=99).contains(&new_number) {
                player.number = new_number;
            } else {
                println!("  ⚠️  Invalid home number ignored. Keeping current value.");
            }
        }

        if let Some(new_away_number) = term::read_i32(&format!(
            "Away number [{}] (ENTER to keep): ",
            player.away_number
        )) {
            if (0..=99).contains(&new_away_number) {
                player.away_number = new_away_number;
            } else {
                println!("  ⚠️  Invalid away number ignored. Keeping current value.");
            }
        }

        if let Some(pos_choice) = term::read_i32(&format!(
            "Position [{}] (1-9, or 0 to keep): ",
            player.position.to_number()
        )) {
            if pos_choice > 0 && pos_choice <= 9 {
                if let Some(new_pos) = Position::from_number(pos_choice as u8) {
                    player.position = new_pos;
                }
            } else if pos_choice != 0 {
                println!("  ⚠️  Invalid position ignored. Keeping current value.");
            }
        }

        player.throw = choose_enum(player.throw).or(player.throw);
        player.bat = choose_enum(player.bat).or(player.bat);

        match player.update(conn) {
            Ok(_) => term::show_success("Player updated successfully!"),
            Err(e) => term::show_error(&format!("Failed to update player: {}", e)),
        }
    }
}

fn delete_player(db: &Database) {
    let conn = db.get_connection();

    let Some(team) = select_team_for_player_action(conn, "DELETE PLAYER") else {
        return;
    };

    let Some(team_id) = team.id else {
        term::show_error("Selected team has no valid ID");
        return;
    };

    loop {
        term::show_header(&format!("DELETE PLAYER - {}", team.name));

        let players = get_players_for_team_with_name(conn, team_id, &team.name);

        if players.is_empty() {
            println!("📭 No players available for this team.\n");
            term::wait_for_enter();
            break;
        }

        println!("Players for {}:\n", team.name);
        display_player_list(&players);
        println!("\n  0. 🔙 Back\n");

        let Some(choice) = term::read_i64("Select player to delete (0 to go back): ") else {
            break;
        };

        if choice == 0 {
            break;
        }

        if choice < 1 || choice as usize > players.len() {
            term::show_error("Invalid selection");
            continue;
        }

        let (player, team_name) = &players[(choice - 1) as usize];

        if term::confirm(&format!(
            "Are you sure you want to delete '#{} {} ({})'?",
            player.number,
            player.full_name(),
            team_name
        )) {
            if let Some(id) = player.id {
                match Player::delete(conn, id) {
                    Ok(_) => term::show_success("Player deleted successfully!"),
                    Err(e) => term::show_error(&format!("Failed to delete player: {}", e)),
                }
            }
        } else {
            println!("\n❌ Deletion cancelled");
            term::wait_for_enter();
        }
    }
}

fn select_team_for_player_action(conn: &rusqlite::Connection, title: &str) -> Option<Team> {
    term::show_header(title);

    let leagues = match League::get_all(conn) {
        Ok(leagues) => leagues,
        Err(e) => {
            term::show_error(&format!("Error loading leagues: {}", e));
            return None;
        }
    };

    let unassigned_teams = match Team::get_without_league(conn) {
        Ok(teams) => teams,
        Err(e) => {
            term::show_error(&format!("Error loading unassigned teams: {}", e));
            return None;
        }
    };

    if leagues.is_empty() && unassigned_teams.is_empty() {
        term::show_error("No leagues or unassigned teams available");
        return None;
    }

    println!("Leagues:\n");
    for (i, league) in leagues.iter().enumerate() {
        let season = league.season.as_deref().unwrap_or("N/A");
        term::show_list_item(i + 1, &format!("{} ({})", league.name, season));
    }

    let no_league_choice = leagues.len() + 1;
    term::show_list_item(no_league_choice, "No league");
    println!("\n  0. 🔙 Back\n");

    let Some(league_choice) = term::read_i64("Select league (0 to cancel): ") else {
        println!("\n❌ Operation cancelled");
        term::wait_for_enter();
        return None;
    };

    if league_choice == 0 {
        println!("\n❌ Operation cancelled");
        term::wait_for_enter();
        return None;
    }

    if league_choice < 1 || league_choice as usize > no_league_choice {
        term::show_error("Invalid selection");
        return None;
    }

    let (teams, teams_label) = if league_choice as usize == no_league_choice {
        (unassigned_teams, "No league".to_string())
    } else {
        let league = &leagues[(league_choice - 1) as usize];
        let Some(league_id) = league.id else {
            term::show_error("Selected league has no valid ID");
            return None;
        };

        let teams = match Team::get_by_league(conn, league_id) {
            Ok(teams) => teams,
            Err(e) => {
                term::show_error(&format!("Error loading teams: {}", e));
                return None;
            }
        };

        (teams, league.name.clone())
    };

    if teams.is_empty() {
        term::show_error(&format!("No teams available for {}", teams_label));
        return None;
    }

    println!("\nTeams for {}:\n", teams_label);
    for (i, team) in teams.iter().enumerate() {
        term::show_list_item(i + 1, &team.name);
    }
    println!("\n  0. 🔙 Back\n");

    let Some(team_choice) = term::read_i64("Select team (0 to cancel): ") else {
        println!("\n❌ Operation cancelled");
        term::wait_for_enter();
        return None;
    };

    if team_choice == 0 {
        println!("\n❌ Operation cancelled");
        term::wait_for_enter();
        return None;
    }

    if team_choice < 1 || team_choice as usize > teams.len() {
        term::show_error("Invalid selection");
        return None;
    }

    Some(teams[(team_choice - 1) as usize].clone())
}

fn get_players_for_team_with_name(
    conn: &rusqlite::Connection,
    team_id: i64,
    team_name: &str,
) -> Vec<(Player, String)> {
    Player::get_by_team(conn, team_id)
        .unwrap_or_default()
        .into_iter()
        .map(|player| (player, team_name.to_string()))
        .collect()
}

fn change_team(db: &Database) {
    term::show_header("CHANGE PLAYER TEAM");

    let conn = db.get_connection();

    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        term::show_error("No players available");
        return;
    }

    println!("Players:\n");
    display_player_list(&players);
    println!();

    if let Some(player_choice) = term::read_i64("Select player (0 to cancel): ") {
        if player_choice == 0 {
            println!("\n❌ Operation cancelled");
            term::wait_for_enter();
            return;
        }

        if player_choice < 1 || player_choice as usize > players.len() {
            term::show_error("Invalid selection");
            return;
        }

        let (mut player, current_team) = players[(player_choice - 1) as usize].clone();

        // List available teams
        match Team::get_all(conn) {
            Ok(teams) => {
                println!("\nAvailable teams:\n");
                for (i, team) in teams.iter().enumerate() {
                    let marker = if Some(team.id.unwrap()) == Some(player.team_id) {
                        " (current)"
                    } else {
                        ""
                    };
                    term::show_list_item(i + 1, &format!("{}{}", team.name, marker));
                }
                println!();

                if let Some(team_choice) = term::read_i64("Select new team (0 to cancel): ") {
                    if team_choice == 0 {
                        println!("\n❌ Operation cancelled");
                        term::wait_for_enter();
                        return;
                    }

                    if team_choice < 1 || team_choice as usize > teams.len() {
                        term::show_error("Invalid selection");
                        return;
                    }

                    let new_team = &teams[(team_choice - 1) as usize];
                    let new_team_id = new_team.id.unwrap();

                    if new_team_id == player.team_id {
                        println!("\n⚠️  Player is already in this team!");
                        term::wait_for_enter();
                        return;
                    }

                    player.team_id = new_team_id;

                    match player.update(conn) {
                        Ok(_) => {
                            term::show_success(&format!(
                                "Player team changed!\n   {} → {}",
                                current_team, new_team.name
                            ));
                        }
                        Err(e) => {
                            term::show_error(&format!("Failed to change team: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                term::show_error(&format!("Error loading teams: {}", e));
            }
        }
    }
}

/// Helper function to display a list of players with team names
fn display_player_list(players: &[(Player, String)]) {
    for (i, (player, team_name)) in players.iter().enumerate() {
        term::show_list_item(
            i + 1,
            &format!(
                "H#{} A#{} {} ({})",
                player.number,
                player.away_number,
                player.full_name(),
                team_name
            ),
        );
    }
}

fn download_csv_template() -> anyhow::Result<()> {
    let path = term::read_string("Output CSV template path: ");
    let path = path.trim();

    if path.is_empty() {
        println!("Operation cancelled.");
        return Ok(());
    }

    fs::write(
        path,
        "team_name,number,away_number,first_name,last_name,position,throw,bat\n\
         \"Rimini Baseball\",12,9,\"Mario\",\"Rossi\",1,\"R\",\"R\"\n",
    )?;

    println!("✅ CSV template written to {}", path);
    Ok(())
}

fn download_json_template() -> anyhow::Result<()> {
    let path = term::read_string("Output JSON template path: ");
    let path = path.trim();

    if path.is_empty() {
        println!("Operation cancelled.");
        return Ok(());
    }

    let template = r#"[
  {
    "team": "Rimini Baseball",
    "number": 12,
    "away_number": 9,
    "first_name": "Mario",
    "last_name": "Rossi",
    "position": 1,
    "throw": "R",
    "bat": "R"
  }
]
"#;

    std::fs::write(path, template)?;

    println!("✅ JSON template written to {}", path);
    Ok(())
}
