//! CLI command handlers for the Umpire Supervisor module.

use crate::db::game_queries::list_playable_games;
use crate::db::umpire::{Umpire, UmpireEvaluation, UmpirePosition};
use crate::models::session::PlayBallGameContext;
use crate::utils;
use crate::{Database, Menu, UmpireSupervisorMenuChoice};
use std::collections::HashMap;

use crate::cli::commands::export::{
    build_umpire_export_rows, export_umpire_reports_csv, export_umpire_reports_json,
};
use crate::cli::commands::game::{GameInfo, get_game_by_id};
use crate::utils::cli::prompt_export_directory;
use rusqlite::Connection;
use std::io::{self, Write};
// ─── Menu dispatcher ──────────────────────────────────────────────────────────

pub fn handle_umpire_supervisor_menu(db: &mut Database) {
    loop {
        match Menu::show_umpire_supervisor_menu() {
            UmpireSupervisorMenuChoice::ManageUmpires => handle_manage_umpires(db),
            UmpireSupervisorMenuChoice::AssignToGame => handle_assign_to_game(db),
            UmpireSupervisorMenuChoice::EvaluateGame => handle_evaluate_game(db),
            UmpireSupervisorMenuChoice::UmpireHistory => handle_umpire_history(db),
            UmpireSupervisorMenuChoice::ExportReports => handle_export_umpire_reports(db),
            UmpireSupervisorMenuChoice::Back => return,
        }
    }
}

// ─── 1. Manage Umpires (CRUD) ─────────────────────────────────────────────────

fn handle_manage_umpires(db: &mut Database) {
    loop {
        utils::cli::clear_screen();
        println!("╔════════════════════════════════════════════╗");
        println!("║          👤  MANAGE UMPIRES                ║");
        println!("╚════════════════════════════════════════════╝");
        println!();
        println!("  1. ➕ Add New Umpire");
        println!("  2. 📋 List All Umpires");
        println!("  3. ✏️ Edit Umpire");
        println!("  4. 🗑️ Delete Umpire");
        println!();
        println!("  0. 🔙 Back");
        println!();
        print!("Select an option (1-4 or 0): ");
        io::stdout().flush().unwrap();

        let choice = utils::cli::read_choice();
        match choice {
            1 => add_umpire(db),
            2 => list_umpires(db),
            3 => edit_umpire(db),
            4 => delete_umpire(db),
            0 => return,
            _ => {
                println!("\n❌ Invalid choice. Press ENTER to continue...");
                utils::cli::wait_for_enter();
            }
        }
    }
}

fn add_umpire(db: &mut Database) {
    utils::cli::clear_screen();
    println!("═══ Add New Umpire ═══\n");

    let first_name = utils::cli::read_string("First name: ");
    if first_name.is_empty() {
        println!("❌ First name cannot be empty.");
        utils::cli::wait_for_enter();
        return;
    }
    let last_name = utils::cli::read_string("Last name: ");
    if last_name.is_empty() {
        println!("❌ Last name cannot be empty.");
        utils::cli::wait_for_enter();
        return;
    }

    let license_number = utils::cli::read_optional_string("License number (or ENTER to skip): ");
    let level = utils::cli::read_optional_string("Level/Classification (or ENTER to skip): ");
    let email = utils::cli::read_optional_string("Email (or ENTER to skip): ");
    let phone = utils::cli::read_optional_string("Phone (or ENTER to skip): ");
    let notes = utils::cli::read_optional_string("Notes (or ENTER to skip): ");

    // ── League association ────────────────────────────────────────────────
    let conn = db.get_connection();
    let league_ids = select_leagues(conn);

    let mut umpire = Umpire::new(first_name, last_name);
    umpire.license_number = license_number;
    umpire.level = level;
    umpire.email = email;
    umpire.phone = phone;
    umpire.notes = notes;

    match umpire.create(conn) {
        Ok(id) => {
            println!("\n✅ Umpire {} created (ID: {id})", umpire.full_name());

            if !league_ids.is_empty() {
                match crate::db::umpire::set_umpire_leagues(conn, id, &league_ids) {
                    Ok(_) => println!("   Associated with {} league(s).", league_ids.len()),
                    Err(e) => println!("   ⚠️  Failed to associate leagues: {e}"),
                }
            }
        }
        Err(e) => println!("\n❌ Failed to create umpire: {e}"),
    }
    utils::cli::wait_for_enter();
}

/// Show all available leagues and let the user pick one or more.
/// Returns a Vec of selected league IDs (may be empty).
fn select_leagues(conn: &rusqlite::Connection) -> Vec<i64> {
    let leagues = match crate::db::league::League::get_all(conn) {
        Ok(l) => l,
        Err(e) => {
            println!("  ⚠️  Cannot load leagues: {e}");
            return vec![];
        }
    };

    if leagues.is_empty() {
        println!("\n  No leagues registered. Skipping league association.");
        return vec![];
    }

    println!("\n  Available leagues:");
    for lg in &leagues {
        let season = lg.season.as_deref().unwrap_or("");
        println!(
            "    {:>3}. {} {}",
            lg.id.unwrap_or(0),
            lg.name,
            if season.is_empty() {
                String::new()
            } else {
                format!("({})", season)
            }
        );
    }

    println!("\n  Enter league IDs separated by comma (e.g. 1,3) or ENTER to skip:");
    print!("  Leagues: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    let input = input.trim();

    if input.is_empty() {
        return vec![];
    }

    let valid_ids: std::collections::HashSet<i64> = leagues.iter().filter_map(|l| l.id).collect();

    let mut selected = Vec::new();
    for token in input.split(',') {
        let token = token.trim();
        if let Ok(id) = token.parse::<i64>() {
            if valid_ids.contains(&id) {
                selected.push(id);
            } else {
                println!("    ⚠️  League ID {id} not found — skipped.");
            }
        }
    }

    selected
}

fn list_umpires(db: &mut Database) {
    utils::cli::clear_screen();
    println!("═══ All Umpires ═══\n");

    let conn = db.get_connection();
    match Umpire::get_all(conn) {
        Ok(umpires) => {
            if umpires.is_empty() {
                println!("  No umpires registered yet.");
            } else {
                println!(
                    "  {:>3}  {:<25} {:<12} {:<8} {:<6}  Leagues",
                    "ID", "Name", "License", "Level", "Active"
                );
                println!("  {}", "─".repeat(80));
                for u in &umpires {
                    let leagues_str =
                        match crate::db::umpire::get_umpire_leagues(conn, u.id.unwrap_or(0)) {
                            Ok(leagues) if !leagues.is_empty() => leagues
                                .iter()
                                .map(|(_, name)| name.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            _ => "-".to_string(),
                        };

                    println!(
                        "  {:>3}  {:<25} {:<12} {:<8} {:<6}  {}",
                        u.id.unwrap_or(0),
                        u.full_name(),
                        u.license_number.as_deref().unwrap_or("-"),
                        u.level.as_deref().unwrap_or("-"),
                        if u.is_active { "Yes" } else { "No" },
                        leagues_str,
                    );
                }
                println!("\n  Total: {} umpire(s)", umpires.len());
            }
        }
        Err(e) => println!("❌ Failed to load umpires: {e}"),
    }
    utils::cli::wait_for_enter();
}

fn edit_umpire(db: &mut Database) {
    utils::cli::clear_screen();
    println!("═══ Edit Umpire ═══\n");

    let conn = db.get_connection();

    // Show available umpires
    match Umpire::get_all(conn) {
        Ok(umpires) if !umpires.is_empty() => {
            println!(
                "  {:>3}  {:<25} {:<12} {:<8}",
                "ID", "Name", "License", "Level"
            );
            println!("  {}", "─".repeat(52));
            for u in &umpires {
                println!(
                    "  {:>3}  {:<25} {:<12} {:<8}",
                    u.id.unwrap_or(0),
                    u.full_name(),
                    u.license_number.as_deref().unwrap_or("-"),
                    u.level.as_deref().unwrap_or("-"),
                );
            }
            println!();
        }
        Ok(_) => {
            println!("  No umpires registered yet.");
            utils::cli::wait_for_enter();
            return;
        }
        Err(e) => {
            println!("❌ Failed to load umpires: {e}");
            utils::cli::wait_for_enter();
            return;
        }
    }

    let id = utils::cli::read_i64_required("Umpire ID to edit: ");

    let mut umpire = match Umpire::get_by_id(conn, id) {
        Ok(u) => u,
        Err(_) => {
            println!("❌ Umpire not found.");
            utils::cli::wait_for_enter();
            return;
        }
    };

    println!("\nEditing: {} (ID: {id})", umpire.full_name());
    println!("(Press ENTER to keep current value)\n");

    let new_first = utils::cli::read_string_with_default(
        &format!("First name [{}]: ", umpire.first_name),
        &umpire.first_name,
    );
    let new_last = utils::cli::read_string_with_default(
        &format!("Last name [{}]: ", umpire.last_name),
        &umpire.last_name,
    );
    let new_license = utils::cli::read_optional_string_with_default(
        &format!(
            "License [{}]: ",
            umpire.license_number.as_deref().unwrap_or("-")
        ),
        umpire.license_number.as_deref(),
    );
    let new_level = utils::cli::read_optional_string_with_default(
        &format!("Level [{}]: ", umpire.level.as_deref().unwrap_or("-")),
        umpire.level.as_deref(),
    );
    let new_email = utils::cli::read_optional_string_with_default(
        &format!("Email [{}]: ", umpire.email.as_deref().unwrap_or("-")),
        umpire.email.as_deref(),
    );
    let new_phone = utils::cli::read_optional_string_with_default(
        &format!("Phone [{}]: ", umpire.phone.as_deref().unwrap_or("-")),
        umpire.phone.as_deref(),
    );

    umpire.first_name = new_first;
    umpire.last_name = new_last;
    umpire.license_number = new_license;
    umpire.level = new_level;
    umpire.email = new_email;
    umpire.phone = new_phone;

    match umpire.update(conn) {
        Ok(_) => println!("\n✅ Umpire updated."),
        Err(e) => println!("\n❌ Failed to update: {e}"),
    }

    // ── League association ────────────────────────────────────────────────
    let current_leagues = crate::db::umpire::get_umpire_leagues(conn, id).unwrap_or_default();

    if current_leagues.is_empty() {
        println!("\n  Current leagues: (none)");
    } else {
        let names: Vec<&str> = current_leagues.iter().map(|(_, n)| n.as_str()).collect();
        println!("\n  Current leagues: {}", names.join(", "));
    }

    println!("  Update leagues? (ENTER to keep, or enter new selection)");
    let new_league_ids = select_leagues(conn);

    if !new_league_ids.is_empty() {
        match crate::db::umpire::set_umpire_leagues(conn, id, &new_league_ids) {
            Ok(_) => println!("  ✅ Leagues updated ({} league(s)).", new_league_ids.len()),
            Err(e) => println!("  ⚠️  Failed to update leagues: {e}"),
        }
    } else if !current_leagues.is_empty() {
        println!("  (Leagues unchanged)");
    }

    utils::cli::wait_for_enter();
}

/// Fetch an umpire by id, printing an error and pausing if not found.
/// Returns None if the umpire does not exist.
pub fn fetch_umpire_or_notify(conn: &Connection, id: i64) -> Option<Umpire> {
    match Umpire::get_by_id(conn, id) {
        Ok(u) => Some(u),
        Err(_) => {
            println!("❌ Umpire not found.");
            utils::cli::wait_for_enter();
            None
        }
    }
}

fn delete_umpire(db: &mut Database) {
    utils::cli::clear_screen();
    println!("═══ Delete Umpire ═══\n");

    let id = utils::cli::read_i64_required("Umpire ID to delete: ");
    let conn = db.get_connection();

    let Some(umpire) = fetch_umpire_or_notify(conn, id) else {
        return;
    };

    println!(
        "\n⚠️  Delete {} (ID: {id})? This will also remove all assignments and evaluations.",
        umpire.full_name()
    );
    let confirm = utils::cli::read_string("Type 'yes' to confirm: ");
    if confirm.eq_ignore_ascii_case("yes") {
        match Umpire::delete(conn, id) {
            Ok(_) => println!("\n✅ Umpire deleted."),
            Err(e) => println!("\n❌ Failed to delete: {e}"),
        }
    } else {
        println!("\n↩️  Cancelled.");
    }
    utils::cli::wait_for_enter();
}

// ─── Shared: game picker (same criteria as Play Ball) ─────────────────────────

/// Display available games and let the user pick one.
/// Returns `None` if canceled or no games available.
fn select_game(db: &mut Database, header: &str) -> Option<PlayBallGameContext> {
    let conn = db.get_connection_mut();

    let games = match list_playable_games(conn) {
        Ok(v) => v,
        Err(e) => {
            utils::cli::show_error(&format!("Error querying games: {e}"));
            return None;
        }
    };

    if games.is_empty() {
        println!("📭 No available games found.");
        utils::cli::wait_for_enter();
        return None;
    }

    println!("\n📋 {header}:\n");
    for (i, g) in games.iter().enumerate() {
        let away = g.away_team_abbr.as_deref().unwrap_or(&g.away_team_name);
        let home = g.home_team_abbr.as_deref().unwrap_or(&g.home_team_name);

        println!("  {}. {} - {} @ {}", i + 1, g.game_date, away, home);
        println!(
            "     Status: {} | Venue: {} | ID: {}",
            g.status, g.venue, g.game_id
        );
        println!();
    }

    let choice = match utils::cli::read_i64("Select game (number, 0 to cancel): ") {
        Some(0) | None => return None,
        Some(c) if c > 0 && (c as usize) <= games.len() => c as usize,
        _ => {
            utils::cli::show_error("Invalid selection");
            return None;
        }
    };

    Some(games[choice - 1].clone())
}

// ─── 2. Assign Umpires to Game ────────────────────────────────────────────────

fn handle_assign_to_game(db: &mut Database) {
    utils::cli::clear_screen();
    println!("═══ Assign Umpires to Game ═══\n");

    let game = match select_game(db, "Available Games") {
        Some(g) => g,
        None => return,
    };

    let game_pk = game.id;
    let conn = db.get_connection();

    // Show current assignments
    match crate::db::umpire::list_game_umpires(conn, game_pk) {
        Ok(assignments) => {
            if assignments.is_empty() {
                println!("  No umpires assigned to this game yet.");
            } else {
                println!("  Current crew:");
                for a in &assignments {
                    println!(
                        "    {:<4} {}",
                        a.position,
                        a.umpire_name.as_deref().unwrap_or("?")
                    );
                }
            }
        }
        Err(e) => println!("❌ Failed to load assignments: {e}"),
    }

    // Crew size
    let crew_size = utils::cli::read_i64_required("\nSelect crew size (2,3,4,6): ") as u8;
    let positions = UmpirePosition::crew(crew_size);

    if positions.is_empty() {
        println!("❌ Invalid crew size.");
        utils::cli::wait_for_enter();
        return;
    }

    // Show available umpires
    let umpires = match Umpire::get_active(conn) {
        Ok(u) => u,
        Err(e) => {
            println!("❌ Failed to load umpires: {e}");
            utils::cli::wait_for_enter();
            return;
        }
    };

    if umpires.is_empty() {
        println!("\n❌ No active umpires registered. Add umpires first.");
        utils::cli::wait_for_enter();
        return;
    }

    println!("\n  Available umpires:");
    for u in &umpires {
        println!(
            "    {:>3}. {} {}",
            u.id.unwrap_or(0),
            u.full_name(),
            u.level
                .as_ref()
                .map(|l| format!("[{l}]"))
                .unwrap_or_default()
        );
    }

    println!();
    for pos in &positions {
        let prompt = format!("  Umpire ID for {} (or 0 to skip): ", pos);
        let ump_id = utils::cli::read_i64_required(&prompt);
        if ump_id == 0 {
            continue;
        }
        match crate::db::umpire::assign_umpire(conn, game_pk, ump_id, *pos) {
            Ok(_) => println!("    ✅ {} assigned", pos),
            Err(e) => println!("    ❌ Failed: {e}"),
        }
    }

    println!("\n✅ Crew assignment complete.");
    utils::cli::wait_for_enter();
}

// ─── 3. Evaluate Game (Report Card) ──────────────────────────────────────────

fn handle_evaluate_game(db: &mut Database) {
    utils::cli::clear_screen();
    println!("═══ Evaluate Game — Umpire Report Card ═══\n");

    let game = match select_game(db, "Select Game to Evaluate") {
        Some(g) => g,
        None => return,
    };

    let game_pk = game.id;
    let conn = db.get_connection();

    // Load assigned umpires
    let assignments = match crate::db::umpire::list_game_umpires(conn, game_pk) {
        Ok(a) => a,
        Err(e) => {
            println!("❌ Failed to load crew: {e}");
            utils::cli::wait_for_enter();
            return;
        }
    };

    if assignments.is_empty() {
        println!("❌ No umpires assigned to this game. Assign umpires first.");
        utils::cli::wait_for_enter();
        return;
    }

    let evaluator = utils::cli::read_optional_string("Evaluator name (or ENTER to skip): ");

    for a in &assignments {
        let pos = UmpirePosition::parse(&a.position).unwrap_or(UmpirePosition::HomePlate);
        let name = a.umpire_name.as_deref().unwrap_or("Unknown");

        let content = format!("  Evaluating: {name} ({pos})  ");
        let width = content.chars().count();
        println!("\n╔{}╗", "═".repeat(width));
        println!("║{content}║");
        println!("╚{}╝", "═".repeat(width));
        println!("  (Enter score 1-10, or ENTER to skip each category)\n");

        let mut eval = UmpireEvaluation::new(game_pk, a.umpire_id, pos);
        eval.evaluator_name = evaluator.clone();

        // Only show strike zone for HP umpire
        if pos == UmpirePosition::HomePlate {
            eval.strike_zone_accuracy = read_score("  Strike zone accuracy (1-10): ");
        }

        eval.safe_out_accuracy = read_score("  Safe/Out accuracy (1-10): ");
        eval.positioning = read_score("  Positioning (1-10): ");
        eval.timing = read_score("  Timing (1-10): ");
        eval.game_management = read_score("  Game management (1-10): ");
        eval.professionalism = read_score("  Professionalism (1-10): ");
        eval.communication = read_score("  Communication (1-10): ");
        eval.hustle = read_score("  Hustle (1-10): ");

        // Show computed average and let supervisor override
        if let Some(avg) = eval.calculated_average() {
            println!("\n  Calculated average: {avg:.1}");
        }
        eval.overall_score = read_score("  Overall score (1-10, or ENTER to use average): ");
        if eval.overall_score.is_none() {
            eval.overall_score = eval.calculated_average().map(|a| a.round() as i32);
        }

        eval.strengths = utils::cli::read_optional_string("  Strengths: ");
        eval.areas_to_improve = utils::cli::read_optional_string("  Areas to improve: ");
        eval.notes = utils::cli::read_optional_string("  Notes: ");

        match eval.save(conn) {
            Ok(_) => println!("  ✅ Evaluation saved for {name}."),
            Err(e) => println!("  ❌ Failed to save evaluation: {e}"),
        }
    }

    println!("\n✅ Game evaluation complete.");
    utils::cli::wait_for_enter();
}

fn read_score(prompt: &str) -> Option<i32> {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<i32>() {
        Ok(n) if (1..=10).contains(&n) => Some(n),
        _ => {
            println!("    (Invalid — skipping)");
            None
        }
    }
}

fn extract_game_summary_info(game_map: &HashMap<i64, GameInfo>, game_id: i64) -> (String, &str) {
    match game_map.get(&game_id) {
        Some(g) => (
            format!("{} @ {}", g.away_team, g.home_team),
            g.game_date.as_str(),
        ),
        None => ("-".to_string(), "-"),
    }
}

fn extract_game_info(
    game_map: &HashMap<i64, GameInfo>,
    game_id: i64,
) -> (String, &str, &str, &str) {
    match game_map.get(&game_id) {
        Some(g) => (
            format!("{} @ {}", g.away_team, g.home_team),
            g.game_date.as_str(),
            g.game_time.as_deref().unwrap_or("-"),
            g.venue.as_str(),
        ),
        None => ("-".to_string(), "-", "-", "-"),
    }
}

// ─── 4. Umpire History / Statistics ──────────────────────────────────────────

fn handle_umpire_history(db: &mut Database) {
    use std::collections::HashMap;
    use std::io::{self, Write};

    utils::cli::clear_screen();
    println!("═══ Umpire History / Statistics ═══\n");

    let conn = db.get_connection();

    // Optional league filter
    let all_umpires = Umpire::get_all(conn).unwrap_or_default();
    let league_ids = select_leagues(conn);

    let filtered_umpires: Vec<Umpire> = if league_ids.is_empty() {
        all_umpires
    } else {
        all_umpires
            .into_iter()
            .filter(|u| {
                let Some(uid) = u.id else {
                    return false;
                };

                crate::db::umpire::get_umpire_leagues(conn, uid)
                    .unwrap_or_default()
                    .iter()
                    .any(|(lid, _)| league_ids.contains(lid))
            })
            .collect()
    };

    if filtered_umpires.is_empty() {
        println!("\n  No umpires found for the selected filter.");
        utils::cli::wait_for_enter();
        return;
    }

    println!("\n  Available umpires:");
    println!("    {:>3}  {:<25} {:<12}", "ID", "Name", "Level");
    println!("    {}", "─".repeat(42));
    for u in &filtered_umpires {
        println!(
            "    {:>3}  {:<25} {:<12}",
            u.id.unwrap_or(0),
            u.full_name(),
            u.level.as_deref().unwrap_or("-"),
        );
    }
    println!();

    let umpire_id = utils::cli::read_i64_required("Umpire ID: ");

    let Some(umpire) = filtered_umpires
        .iter()
        .find(|u| u.id == Some(umpire_id))
        .cloned()
    else {
        println!("❌ Umpire not found in the selected list.");
        utils::cli::wait_for_enter();
        return;
    };

    let evals = match UmpireEvaluation::list_by_umpire(conn, umpire_id) {
        Ok(e) => e,
        Err(e) => {
            println!("\n❌ Failed to load evaluations: {e}");
            utils::cli::wait_for_enter();
            return;
        }
    };

    if evals.is_empty() {
        print_umpire_header(&umpire);
        println!("\n  No evaluations recorded yet.");
        utils::cli::wait_for_enter();
        return;
    }

    let mut game_map: HashMap<i64, GameInfo> = HashMap::new();

    for ev in &evals {
        if let std::collections::hash_map::Entry::Vacant(entry) = game_map.entry(ev.game_id)
            && let Ok(Some(game_info)) = get_game_by_id(conn, ev.game_id)
        {
            entry.insert(game_info);
        }
    }

    loop {
        utils::cli::clear_screen();
        println!("═══ Umpire History / Statistics ═══\n");

        print_umpire_header(&umpire);
        print_umpire_evaluation_summary(&evals, &game_map);

        println!("\n  Options:");
        println!("    [V] View detailed report by Game ID");
        println!("    [E] Exit");
        print!("\n  Choice: ");
        let _ = io::stdout().flush();

        let mut choice = String::new();
        if io::stdin().read_line(&mut choice).is_err() {
            println!("\n❌ Failed to read input.");
            utils::cli::wait_for_enter();
            continue;
        }

        match choice.trim().to_uppercase().as_str() {
            "" | "E" | "X" => break,

            "V" => {
                let game_id = utils::cli::read_i64_required("\n  Game ID: ");

                let Some(report) = evals.iter().find(|ev| ev.game_id == game_id) else {
                    println!("❌ No report found for the selected Game ID.");
                    utils::cli::wait_for_enter();
                    continue;
                };

                utils::cli::clear_screen();
                print_umpire_evaluation_detail(&umpire, report, &game_map);
                utils::cli::wait_for_enter();
            }

            _ => {
                println!("❌ Invalid choice.");
                utils::cli::wait_for_enter();
            }
        }
    }
}

fn print_umpire_header(umpire: &Umpire) {
    println!("  Umpire: {}", umpire.full_name());
    if let Some(ref lic) = umpire.license_number {
        println!("  License: {lic}");
    }
    if let Some(ref level) = umpire.level {
        println!("  Level: {level}");
    }
}

fn print_umpire_evaluation_summary(evals: &[UmpireEvaluation], game_map: &HashMap<i64, GameInfo>) {
    println!(
        "\n  {:>5}  {:<28} {:<10} {:<4}  {:>5}  {:>5}  {:>7}",
        "Game", "Matchup", "Date", "Pos", "SZ", "S/O", "Overall"
    );
    println!("  {}", "─".repeat(78));

    let mut total_overall: f64 = 0.0;
    let mut count_overall: u32 = 0;

    for ev in evals {
        let overall_str = ev
            .overall_score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());

        if let Some(s) = ev.overall_score {
            total_overall += s as f64;
            count_overall += 1;
        }

        let (matchup, game_date) = extract_game_summary_info(game_map, ev.game_id);

        println!(
            "  {:>5}  {:<28} {:<10} {:<4}  {:>5}  {:>5}  {:>7}",
            ev.game_id,
            matchup,
            game_date,
            ev.position_evaluated,
            ev.strike_zone_accuracy
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string()),
            ev.safe_out_accuracy
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string()),
            overall_str,
        );
    }

    println!("  {}", "─".repeat(78));
    println!("  Games evaluated: {}", evals.len());

    if count_overall > 0 {
        let avg = total_overall / count_overall as f64;
        println!("  Career average overall: {avg:.1}");
    }
}

fn print_umpire_evaluation_detail(
    umpire: &Umpire,
    report: &UmpireEvaluation,
    game_map: &HashMap<i64, GameInfo>,
) {
    let (matchup, game_date, game_time, venue) = extract_game_info(game_map, report.game_id);

    println!("═══ Detailed Umpire Evaluation Report ═══\n");

    println!("  Umpire         : {}", umpire.full_name());
    println!("  Game ID        : {}", report.game_id);
    println!("  Matchup        : {}", matchup);
    println!("  Date and Venue : {} {} - {}", game_date, game_time, venue);
    println!("  Position       : {}", report.position_evaluated);

    println!("\n  Numeric scores:");
    println!(
        "    Strike zone accuracy : {}",
        report
            .strike_zone_accuracy
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Safe/Out accuracy    : {}",
        report
            .safe_out_accuracy
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Positioning          : {}",
        report
            .positioning
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Timing               : {}",
        report
            .timing
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Game management      : {}",
        report
            .game_management
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Professionalism      : {}",
        report
            .professionalism
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Communication        : {}",
        report
            .communication
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Hustle               : {}",
        report
            .hustle
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "    Overall              : {}",
        report
            .overall_score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string())
    );

    println!("\n  Strengths:");
    println!("    {}", report.strengths.as_deref().unwrap_or("-"));

    println!("\n  Areas to improve:");
    println!("    {}", report.areas_to_improve.as_deref().unwrap_or("-"));

    println!("\n  Notes:");
    println!("    {}", report.notes.as_deref().unwrap_or("-"));
}

fn handle_export_umpire_reports(db: &mut Database) {
    use std::collections::HashMap;
    use std::io::{self, Write};

    utils::cli::clear_screen();
    println!("═══ Export Umpire Reports ═══\n");

    let conn = db.get_connection();

    // Optional league filter
    let all_umpires = Umpire::get_all(conn).unwrap_or_default();
    let league_ids = select_leagues(conn);

    let filtered_umpires: Vec<Umpire> = if league_ids.is_empty() {
        all_umpires
    } else {
        all_umpires
            .into_iter()
            .filter(|u| {
                let Some(uid) = u.id else {
                    return false;
                };

                crate::db::umpire::get_umpire_leagues(conn, uid)
                    .unwrap_or_default()
                    .iter()
                    .any(|(lid, _)| league_ids.contains(lid))
            })
            .collect()
    };

    if filtered_umpires.is_empty() {
        println!("  No umpires found for the selected filter.");
        utils::cli::wait_for_enter();
        return;
    }

    println!("  Available umpires:");
    println!("    {:>3}  {:<25} {:<12}", "ID", "Name", "Level");
    println!("    {}", "─".repeat(42));
    for u in &filtered_umpires {
        println!(
            "    {:>3}  {:<25} {:<12}",
            u.id.unwrap_or(0),
            u.full_name(),
            u.level.as_deref().unwrap_or("-"),
        );
    }
    println!();

    let umpire_id = utils::cli::read_i64_required("Umpire ID: ");

    let Some(umpire) = filtered_umpires
        .iter()
        .find(|u| u.id == Some(umpire_id))
        .cloned()
    else {
        println!("❌ Umpire not found in the selected list.");
        utils::cli::wait_for_enter();
        return;
    };

    let evals = match UmpireEvaluation::list_by_umpire(conn, umpire_id) {
        Ok(e) => e,
        Err(e) => {
            println!("\n❌ Failed to load evaluations: {e}");
            utils::cli::wait_for_enter();
            return;
        }
    };

    if evals.is_empty() {
        println!(
            "\n  No evaluations recorded yet for {}.",
            umpire.full_name()
        );
        utils::cli::wait_for_enter();
        return;
    }

    let mut game_map: HashMap<i64, GameInfo> = HashMap::new();
    for ev in &evals {
        game_map
            .entry(ev.game_id)
            .or_insert_with(|| match get_game_by_id(conn, ev.game_id) {
                Ok(Some(game)) => game,
                _ => GameInfo {
                    game_id: "-".to_string(),
                    away_team: "-".to_string(),
                    home_team: "-".to_string(),
                    game_date: "-".to_string(),
                    game_time: Some("-".to_string()),
                    venue: "-".to_string(),
                },
            });
    }

    let rows = build_umpire_export_rows(&evals, &game_map);

    println!("  Export format:");
    println!("    [1] CSV");
    println!("    [2] JSON");
    println!("    [Enter] Cancel");
    print!("\n  Choice: ");
    let _ = io::stdout().flush();

    let mut choice = String::new();
    if io::stdin().read_line(&mut choice).is_err() {
        println!("\n❌ Failed to read input.");
        utils::cli::wait_for_enter();
        return;
    }

    let choice = choice.trim();

    if choice.is_empty() {
        return;
    }

    let Some(output_dir) = prompt_export_directory() else {
        return;
    };

    let export_result = match choice {
        "1" => export_umpire_reports_csv(&rows, &umpire.full_name(), &output_dir),
        "2" => export_umpire_reports_json(&rows, &umpire.full_name(), &output_dir),
        _ => {
            println!("\n❌ Invalid choice.");
            utils::cli::wait_for_enter();
            return;
        }
    };

    match export_result {
        Ok(path) => {
            println!("\n✅ Export completed successfully.");
            println!("   File: {}", path.display());
        }
        Err(e) => {
            println!("\n❌ Export failed: {e}");
        }
    }

    utils::cli::wait_for_enter();
}
