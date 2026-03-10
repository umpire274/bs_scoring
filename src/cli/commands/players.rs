use crate::core::menu::PlayerMenuChoice;
use crate::db::player::Player;
use crate::models::player_traits::{BatSide, PitchHand};
use crate::models::types::Position;
use crate::utils::cli;
use crate::utils::cli::choose_enum;
use crate::{Database, Menu, Team};
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
        cli::show_header("IMPORT/EXPORT PLAYERS");
        println!("  1. 📥 Import from CSV");
        println!("  2. 📥 Import from JSON");
        println!("  3. 📤 Export to CSV");
        println!("  4. 📤 Export to JSON");
        println!();
        println!("  0. 🔙 Back");
        println!();
        print!("Select an option: ");
        io::stdout().flush().unwrap();

        match cli::read_choice() {
            1 => import_csv(db),
            2 => import_json(db),
            3 => export_csv(db),
            4 => export_json(db),
            0 => break,
            _ => {
                println!("\n❌ Invalid choice. Press ENTER to continue...");
                cli::wait_for_enter();
            }
        }
    }
}

fn import_csv(db: &Database) {
    cli::show_header("IMPORT PLAYERS FROM CSV");

    let filepath = cli::read_string("CSV file path: ");
    if filepath.is_empty() {
        cli::show_error("File path is required!");
        return;
    }

    let path = Path::new(&filepath);
    if !path.exists() {
        cli::show_error("File not found!");
        return;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            cli::show_error(&format!("Failed to read file: {}", e));
            return;
        }
    };

    let conn = db.get_connection();
    let mut imported = 0;
    let mut errors = 0;

    // CSV format supported:
    // old: team,number,first_name,last_name,position
    // new: team,number,first_name,last_name,position,pitch,bat
    for (line_num, line) in content.lines().enumerate() {
        if line_num == 0 && line.to_lowercase().contains("team") {
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() != 5 && parts.len() != 7 {
            println!(
                "⚠️  Line {}: Invalid format (expected 5 or 7 fields, got {})",
                line_num + 1,
                parts.len()
            );
            errors += 1;
            continue;
        }

        let team_name = parts[0];
        let number = match parts[1].parse::<i32>() {
            Ok(n) if n > 0 && n <= 99 => n,
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
        let first_name = parts[2].to_string();
        let last_name = parts[3].to_string();
        let position_num = match parts[4].parse::<u8>() {
            Ok(n) if (1..=9).contains(&n) => n,
            _ => {
                println!("⚠️  Line {}: Invalid position '{}'", line_num + 1, parts[4]);
                errors += 1;
                continue;
            }
        };

        let raw_pitch = parts.get(5).copied().unwrap_or("");
        let raw_bat = parts.get(6).copied().unwrap_or("");

        let pitch = if raw_pitch.is_empty() {
            None
        } else {
            match raw_pitch.parse::<PitchHand>() {
                Ok(v) => Some(v),
                Err(_) => {
                    println!(
                        "⚠️  Line {}: Invalid pitch value '{}'",
                        line_num + 1,
                        raw_pitch
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
        let mut player = Player::new(
            team_id,
            number,
            first_name.clone(),
            last_name.clone(),
            position,
            pitch,
            bat,
        );

        match player.create(conn) {
            Ok(_) => {
                let pitch_str = pitch.map(|p| p.as_str()).unwrap_or("-");
                let bat_str = bat.map(|b| b.as_str()).unwrap_or("-");

                println!(
                    "✓ Imported: #{} {} {} ({}) - {} [pitch: {}, bat: {}]",
                    number, first_name, last_name, team_name, position, pitch_str, bat_str
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
    cli::wait_for_enter();
}

fn import_json(db: &Database) {
    cli::show_header("IMPORT PLAYERS FROM JSON");

    let filepath = cli::read_string("JSON file path: ");
    if filepath.is_empty() {
        cli::show_error("File path is required!");
        return;
    }

    let path = Path::new(&filepath);
    if !path.exists() {
        cli::show_error("File not found!");
        return;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            cli::show_error(&format!("Failed to read file: {}", e));
            return;
        }
    };

    let players_data: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(data) => data,
        Err(e) => {
            cli::show_error(&format!("Invalid JSON format: {}", e));
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
            Some(n) if n > 0 && n <= 99 => n as i32,
            _ => {
                println!("⚠️  Player {}: Invalid 'number' field", idx + 1);
                errors += 1;
                continue;
            }
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

        let raw_pitch = player_data.get("pitch").and_then(|v| v.as_str());
        let raw_bat = player_data.get("bat").and_then(|v| v.as_str());

        let pitch = match raw_pitch {
            Some(s) if !s.trim().is_empty() => match s.parse::<PitchHand>() {
                Ok(v) => Some(v),
                Err(_) => {
                    println!("⚠️  Player {}: Invalid pitch value '{}'", idx + 1, s);
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
        let mut player = Player::new(
            team_id,
            number,
            first_name.clone(),
            last_name.clone(),
            position,
            pitch,
            bat,
        );

        match player.create(conn) {
            Ok(_) => {
                let pitch_str = pitch.map(|p| p.as_str()).unwrap_or("-");
                let bat_str = bat.map(|b| b.as_str()).unwrap_or("-");

                println!(
                    "✓ Imported: #{} {} {} ({}) - {} [pitch: {}, bat: {}]",
                    number, first_name, last_name, team_name, position, pitch_str, bat_str
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
    cli::wait_for_enter();
}

fn export_csv(db: &Database) {
    cli::show_header("EXPORT PLAYERS TO CSV");

    let conn = db.get_connection();
    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        cli::show_error("No players to export!");
        return;
    }

    let filepath = cli::read_string("Output CSV file path (e.g., players.csv): ");
    if filepath.is_empty() {
        cli::show_error("File path is required!");
        return;
    }

    let mut csv_content = String::from("team,number,first_name,last_name,position,pitch,bat\n");

    for (player, team_name) in &players {
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            team_name,
            player.number,
            player.first_name,
            player.last_name,
            player.position.to_number(),
            player.pitch.map(|p| p.as_str()).unwrap_or(""),
            player.bat.map(|b| b.as_str()).unwrap_or("")
        ));
    }

    match fs::write(&filepath, csv_content) {
        Ok(_) => {
            cli::show_success(&format!(
                "Exported {} players to '{}'\n\nFormat: team,number,first_name,last_name,position,pitch,bat",
                players.len(),
                filepath
            ));
        }
        Err(e) => {
            cli::show_error(&format!("Failed to write file: {}", e));
        }
    }
}

fn export_json(db: &Database) {
    cli::show_header("EXPORT PLAYERS TO JSON");

    let conn = db.get_connection();
    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        cli::show_error("No players to export!");
        return;
    }

    let filepath = cli::read_string("Output JSON file path (e.g., players.json): ");
    if filepath.is_empty() {
        cli::show_error("File path is required!");
        return;
    }

    let mut players_json = Vec::new();

    for (player, team_name) in &players {
        let player_obj = serde_json::json!({
            "team": team_name,
            "number": player.number,
            "first_name": player.first_name,
            "last_name": player.last_name,
            "position": player.position.to_number(),
            "pitch": player.pitch.map(|p| p.as_str()).unwrap_or(""),
            "bat": player.bat.map(|b| b.as_str()).unwrap_or("")
        });
        players_json.push(player_obj);
    }

    let json_content = match serde_json::to_string_pretty(&players_json) {
        Ok(json) => json,
        Err(e) => {
            cli::show_error(&format!("Failed to serialize JSON: {}", e));
            return;
        }
    };

    match fs::write(&filepath, json_content) {
        Ok(_) => {
            cli::show_success(&format!(
                "Exported {} players to '{}'",
                players.len(),
                filepath
            ));
        }
        Err(e) => {
            cli::show_error(&format!("Failed to write file: {}", e));
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
    cli::show_header("ADD NEW PLAYER");

    let conn = db.get_connection();

    // List available teams
    match Team::get_all(conn) {
        Ok(teams) => {
            if teams.is_empty() {
                cli::show_error("No teams available. Create a team first!");
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

            if let Some(team_choice) = cli::read_i64("Select team (0 to cancel): ") {
                if team_choice == 0 {
                    println!("\n❌ Operation cancelled");
                    cli::wait_for_enter();
                    return;
                }

                if team_choice < 1 || team_choice as usize > teams.len() {
                    cli::show_error("Invalid team selection");
                    return;
                }

                let team = &teams[(team_choice - 1) as usize];
                let team_id = team.id.unwrap();

                // Get player info
                let first_name = cli::read_string("First name: ");
                if first_name.is_empty() {
                    cli::show_error("First name is required!");
                    return;
                }

                let last_name = cli::read_string("Last name: ");

                let number = match cli::read_i32("Jersey number: ") {
                    Some(n) if n > 0 && n <= 99 => n,
                    _ => {
                        cli::show_error("Invalid jersey number (1-99)");
                        return;
                    }
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
                let position = match cli::read_choice() {
                    n if (1..=9).contains(&n) => Position::from_number(n as u8).unwrap(),
                    _ => {
                        cli::show_error("Invalid position");
                        return;
                    }
                };

                // Select pitch hand (optional)
                let pitch = cli::choose_enum_optional::<PitchHand>();

                // Select batting side (optional)
                let bat = cli::choose_enum_optional::<BatSide>();

                // Create player
                let mut player = Player::new(
                    team_id,
                    number,
                    first_name.clone(),
                    last_name.clone(),
                    position,
                    pitch,
                    bat,
                );

                match player.create(conn) {
                    Ok(id) => {
                        cli::show_success(&format!(
                            "Player created successfully!\n\n   {:<14} {}\n   {:<14} {} {}\n   {:<14} {}\n   {:<14} {}\n   {:<14} {:?}\n   {:<14} {}\n   {:<14} {}\n",
                            "ID:",
                            id,
                            "Name:",
                            first_name,
                            last_name,
                            "Number:",
                            number,
                            "Team:",
                            team.name,
                            "Position:",
                            position,
                            "Pitch hand:",
                            pitch.map(|p| p.as_str()).unwrap_or("None"),
                            "Batting side:",
                            bat.map(|b| b.as_str()).unwrap_or("None")
                        ));
                    }
                    Err(e) => {
                        cli::show_error(&format!("Failed to create player: {}", e));
                    }
                }
            }
        }
        Err(e) => {
            cli::show_error(&format!("Error loading teams: {}", e));
        }
    }
}

fn list_players(db: &Database) {
    cli::show_header("ALL PLAYERS");

    let conn = db.get_connection();

    // Option to filter by team
    println!("Filter options:");
    println!("  1. All players (all teams)");
    println!("  2. Filter by team");
    println!();

    print!("Select filter option: ");
    io::stdout().flush().unwrap();
    let filter_choice = cli::read_choice();
    println!();

    let players = if filter_choice == 2 {
        // List teams
        match Team::get_all(conn) {
            Ok(teams) if !teams.is_empty() => {
                println!("\nAvailable teams:\n");
                for (i, team) in teams.iter().enumerate() {
                    cli::show_list_item(i + 1, &team.name);
                }
                println!();

                if let Some(team_choice) = cli::read_i64("Select team (0 for all): ") {
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
        cli::show_separator(72);

        for (player, team_name) in players {
            println!(
                "  #{:<3} {:<25} {:<15} {:<12} (P:{:<3} B:{:<1})",
                player.number,
                player.full_name(),
                format!("({})", team_name),
                format!("{:?}", player.position),
                player.pitch.map(|p| p.as_str()).unwrap_or("-"),
                player.bat.map(|b| b.as_str()).unwrap_or("-")
            );
        }
        cli::show_separator(72);
    }

    cli::wait_for_enter();
}

fn get_all_players_with_teams(conn: &rusqlite::Connection) -> Vec<(Player, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.team_id, p.number, p.first_name, p.last_name, p.position, p.pitch, p.bat, p.is_active, t.name as team_name
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
    cli::show_header("UPDATE PLAYER");

    let conn = db.get_connection();

    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        cli::show_error("No players available");
        return;
    }

    println!("Players:\n");
    display_player_list(&players);
    println!();

    if let Some(choice) = cli::read_i64("Select player to update (0 to cancel): ") {
        if choice == 0 {
            println!("\n❌ Operation cancelled");
            cli::wait_for_enter();
            return;
        }

        if choice < 1 || choice as usize > players.len() {
            cli::show_error("Invalid selection");
            return;
        }

        let (mut player, _) = players[(choice - 1) as usize].clone();

        println!("\nCurrent values (press ENTER to keep):\n");

        // Update first name
        let new_first = cli::read_string(&format!("First name [{}]: ", player.first_name));
        if !new_first.is_empty() {
            player.first_name = new_first;
        }

        // Update last name
        let new_last = cli::read_string(&format!("Last name [{}]: ", player.last_name));
        if !new_last.is_empty() {
            player.last_name = new_last;
        }

        // Update number
        if let Some(new_number) = cli::read_i32(&format!("Number [{}]: ", player.number))
            && new_number > 0
            && new_number <= 99
        {
            player.number = new_number;
        }

        // Update position
        if let Some(pos_choice) = cli::read_i32(&format!(
            "Position [{}] (1-9, or 0 to keep): ",
            player.position.to_number()
        )) && pos_choice > 0
            && pos_choice <= 9
            && let Some(new_pos) = Position::from_number(pos_choice as u8)
        {
            player.position = new_pos;
        }

        // Update pitch hand
        player.pitch = choose_enum(player.pitch).or(player.pitch);

        // Update batting side
        player.bat = choose_enum(player.bat).or(player.bat);

        match player.update(conn) {
            Ok(_) => cli::show_success("Player updated successfully!"),
            Err(e) => cli::show_error(&format!("Failed to update player: {}", e)),
        }
    }
}

fn delete_player(db: &Database) {
    cli::show_header("DELETE PLAYER");

    let conn = db.get_connection();

    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        cli::show_error("No players available");
        return;
    }

    println!("Players:\n");
    display_player_list(&players);
    println!();

    if let Some(choice) = cli::read_i64("Select player to delete (0 to cancel): ") {
        if choice == 0 {
            println!("\n❌ Operation cancelled");
            cli::wait_for_enter();
            return;
        }

        if choice < 1 || choice as usize > players.len() {
            cli::show_error("Invalid selection");
            return;
        }

        let (player, team_name) = &players[(choice - 1) as usize];

        if cli::confirm(&format!(
            "Are you sure you want to delete '#{} {} ({})'?",
            player.number,
            player.full_name(),
            team_name
        )) {
            if let Some(id) = player.id {
                match Player::delete(conn, id) {
                    Ok(_) => cli::show_success("Player deleted successfully!"),
                    Err(e) => cli::show_error(&format!("Failed to delete player: {}", e)),
                }
            }
        } else {
            println!("\n❌ Deletion cancelled");
            cli::wait_for_enter();
        }
    }
}

fn change_team(db: &Database) {
    cli::show_header("CHANGE PLAYER TEAM");

    let conn = db.get_connection();

    let players = get_all_players_with_teams(conn);

    if players.is_empty() {
        cli::show_error("No players available");
        return;
    }

    println!("Players:\n");
    display_player_list(&players);
    println!();

    if let Some(player_choice) = cli::read_i64("Select player (0 to cancel): ") {
        if player_choice == 0 {
            println!("\n❌ Operation cancelled");
            cli::wait_for_enter();
            return;
        }

        if player_choice < 1 || player_choice as usize > players.len() {
            cli::show_error("Invalid selection");
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
                    cli::show_list_item(i + 1, &format!("{}{}", team.name, marker));
                }
                println!();

                if let Some(team_choice) = cli::read_i64("Select new team (0 to cancel): ") {
                    if team_choice == 0 {
                        println!("\n❌ Operation cancelled");
                        cli::wait_for_enter();
                        return;
                    }

                    if team_choice < 1 || team_choice as usize > teams.len() {
                        cli::show_error("Invalid selection");
                        return;
                    }

                    let new_team = &teams[(team_choice - 1) as usize];
                    let new_team_id = new_team.id.unwrap();

                    if new_team_id == player.team_id {
                        println!("\n⚠️  Player is already in this team!");
                        cli::wait_for_enter();
                        return;
                    }

                    player.team_id = new_team_id;

                    match player.update(conn) {
                        Ok(_) => {
                            cli::show_success(&format!(
                                "Player team changed!\n   {} → {}",
                                current_team, new_team.name
                            ));
                        }
                        Err(e) => {
                            cli::show_error(&format!("Failed to change team: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                cli::show_error(&format!("Error loading teams: {}", e));
            }
        }
    }
}

/// Helper function to display a list of players with team names
fn display_player_list(players: &[(Player, String)]) {
    for (i, (player, team_name)) in players.iter().enumerate() {
        cli::show_list_item(
            i + 1,
            &format!("#{} {} ({})", player.number, player.full_name(), team_name),
        );
    }
}
