use crate::core::menu::PlayerMenuChoice;
use crate::db::player::Player;
use crate::models::types::Position;
use crate::utils::cli;
use crate::{Database, Menu, Team};
use std::io;
use std::io::Write;

pub fn handle_player_menu(db: &Database) {
    loop {
        match Menu::show_player_menu() {
            PlayerMenuChoice::AddPlayer => add_player(db),
            PlayerMenuChoice::ListPlayers => list_players(db),
            PlayerMenuChoice::UpdatePlayer => update_player(db),
            PlayerMenuChoice::DeletePlayer => delete_player(db),
            PlayerMenuChoice::ChangeTeam => change_team(db),
            PlayerMenuChoice::Back => break,
        }
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
                let name = cli::read_string("Player name: ");
                if name.is_empty() {
                    cli::show_error("Name is required!");
                    return;
                }

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

                let batting_order = cli::read_i32("Batting order (optional, 0 to skip): ");
                let batting_order = if batting_order == Some(0) {
                    None
                } else {
                    batting_order
                };

                // Create player
                let mut player =
                    Player::new(team_id, number, name.clone(), position, batting_order);

                match player.create(conn) {
                    Ok(id) => {
                        cli::show_success(&format!(
                            "Player created successfully!\n\n   {:<14} {}\n   {:<14} {}\n   {:<14} {}\n   {:<14} {}\n   {:<14} {:?}\n   {:<14} {}",
                            "ID:",
                            id,
                            "Name:",
                            name,
                            "Number:",
                            number,
                            "Team:",
                            team.name,
                            "Position:",
                            position,
                            "Batting Order:",
                            batting_order
                                .map(|o| o.to_string())
                                .unwrap_or("-".to_string())
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
        cli::show_separator();

        for (player, team_name) in players {
            println!(
                "  #{:<3} {:<25} {:<15} {:<12} Order: {}",
                player.number,
                player.name,
                format!("({})", team_name),
                format!("{:?}", player.position),
                player
                    .batting_order
                    .map(|o| o.to_string())
                    .unwrap_or("-".to_string())
            );
        }
        cli::show_separator();
    }

    cli::wait_for_enter();
}

fn get_all_players_with_teams(conn: &rusqlite::Connection) -> Vec<(Player, String)> {
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.team_id, p.number, p.name, p.position, p.batting_order, p.is_active, t.name as team_name
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

        // Update name
        let new_name = cli::read_string(&format!("Name [{}]: ", player.name));
        if !new_name.is_empty() {
            player.name = new_name;
        }

        // Update number
        if let Some(new_number) = cli::read_i32(&format!("Number [{}]: ", player.number))
            && new_number > 0
            && new_number <= 99
        {
            player.number = new_number;
        }

        // Update position
        println!(
            "Position [{}] (1-9, or 0 to keep): ",
            player.position.to_number()
        );
        let pos = cli::read_choice();
        if pos > 0
            && pos <= 9
            && let Some(new_pos) = Position::from_number(pos as u8)
        {
            player.position = new_pos;
        }

        // Update batting order
        if let Some(new_order) = cli::read_i32(&format!(
            "Batting order [{}]: ",
            player
                .batting_order
                .map(|o| o.to_string())
                .unwrap_or("-".to_string())
        )) {
            player.batting_order = if new_order > 0 { Some(new_order) } else { None };
        }

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
            player.number, player.name, team_name
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
            &format!("#{} {} ({})", player.number, player.name, team_name),
        );
    }
}
