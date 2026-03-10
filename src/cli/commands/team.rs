use crate::utils::cli;
use crate::{Database, League, Menu, Team, TeamMenuChoice};
use rusqlite::{Connection, OptionalExtension, params};
use std::fs;

#[derive(Debug, Clone, serde::Deserialize)]
struct ImportTeamRow {
    name: String,
    #[serde(default)]
    abbreviation: Option<String>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    founded_year: Option<i32>,
    #[serde(default)]
    league: Option<String>, // league name
}

pub fn handle_team_menu(db: &mut Database) {
    loop {
        match Menu::show_team_menu() {
            TeamMenuChoice::CreateTeam => create_team(db),
            TeamMenuChoice::ViewTeams => view_teams(db),
            TeamMenuChoice::EditTeam => edit_team(db),
            TeamMenuChoice::ImportTeam => import_teams(db),
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
                    cli::show_separator(40);
                }
            }
            cli::wait_for_enter();
        }
        Err(e) => {
            cli::show_error(&format!("Error loading: {}", e));
        }
    }
}

fn edit_team(db: &Database) {
    cli::show_header("EDIT TEAM");

    let conn = db.get_connection();

    match Team::get_all(conn) {
        Ok(teams) => {
            if teams.is_empty() {
                cli::show_error("No teams available");
                return;
            }

            for (i, team) in teams.iter().enumerate() {
                cli::show_list_item(i + 1, &team.name);
            }

            let choice = match cli::read_i64("\nSelect team to edit: ") {
                Some(v) => v,
                None => return,
            };

            if choice < 1 || choice as usize > teams.len() {
                cli::show_error("Invalid selection");
                return;
            }

            // ✅ CARICA IL RECORD COMPLETO (non usare clone() della lista)
            let team_id = match teams[(choice - 1) as usize].id {
                Some(id) => id,
                None => {
                    cli::show_error("Selected team has no id");
                    return;
                }
            };

            let mut team = match Team::get_by_id(conn, team_id) {
                Ok(t) => t,
                Err(e) => {
                    cli::show_error(&format!("Error loading team: {}", e));
                    return;
                }
            };

            // helper: input pulito
            let read = |p: String| cli::read_string(&p).trim().to_string();

            // Name
            let name = read(format!("Team name [{}]: ", team.name));
            if !name.is_empty() {
                team.name = name;
            }

            // City
            let city_current = team.city.clone().unwrap_or_else(|| "None".to_string());
            let city = read(format!(
                "City [optional] [{}] (type 'none' to clear): ",
                city_current
            ));
            if !city.is_empty() {
                team.city = if city.eq_ignore_ascii_case("none") {
                    None
                } else {
                    Some(city)
                };
            }

            // Abbreviation
            let abbr_current = team
                .abbreviation
                .clone()
                .unwrap_or_else(|| "None".to_string());
            let abbreviation = read(format!(
                "Abbreviation (e.g. BOS) [optional] [{}] (type 'none' to clear): ",
                abbr_current
            ));
            if !abbreviation.is_empty() {
                team.abbreviation = if abbreviation.eq_ignore_ascii_case("none") {
                    None
                } else {
                    Some(abbreviation)
                };
            }

            // Founded year
            let year_current = team
                .founded_year
                .map(|y| y.to_string())
                .unwrap_or_else(|| "None".to_string());

            let founded_year_input = read(format!(
                "Founded year [optional] [{}] (type 'none' to clear): ",
                year_current
            ));
            if !founded_year_input.is_empty() {
                if founded_year_input.eq_ignore_ascii_case("none") {
                    team.founded_year = None;
                } else {
                    match founded_year_input.parse::<i32>() {
                        Ok(year) => team.founded_year = Some(year),
                        Err(_) => {
                            cli::show_error("Invalid founded year");
                            return;
                        }
                    }
                }
            }

            // League
            let leagues = match League::get_all(conn) {
                Ok(l) => l,
                Err(e) => {
                    cli::show_error(&format!("Error loading leagues: {}", e));
                    return;
                }
            };

            if !leagues.is_empty() {
                println!("\nAvailable leagues:");
                for (i, league) in leagues.iter().enumerate() {
                    cli::show_list_item(i + 1, &league.name);
                }
                cli::show_list_item(0, "No league");

                let current_league_label = team
                    .league_id
                    .and_then(|id| {
                        leagues
                            .iter()
                            .find(|league| league.id == Some(id))
                            .map(|league| league.name.clone())
                    })
                    .unwrap_or_else(|| "None".to_string());

                match cli::read_i64(&format!(
                    "\nLeague [{}] (ENTER to keep, 0 for none): ",
                    current_league_label
                )) {
                    None => {} // keep current
                    Some(0) => team.league_id = None,
                    Some(c) if c > 0 && (c as usize) <= leagues.len() => {
                        team.league_id = leagues[(c - 1) as usize].id;
                    }
                    _ => {
                        cli::show_error("Invalid league selection");
                        return;
                    }
                }
            }

            match team.update(conn) {
                Ok(_) => cli::show_success("Team updated!"),
                Err(e) => cli::show_error(&format!("Error: {}", e)),
            }
        }
        Err(e) => cli::show_error(&format!("Error: {}", e)),
    }
}

fn import_teams(db: &mut Database) {
    cli::show_header("IMPORT TEAMS");

    let conn = db.get_connection_mut();

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

    let rows = match parse_teams_file(&path, &content) {
        Ok(r) => r,
        Err(msg) => {
            cli::show_error(&msg);
            return;
        }
    };

    if rows.is_empty() {
        cli::show_error("File is empty");
        return;
    }

    // carica leagues una volta (name -> id)
    let league_map = match load_league_name_map(conn) {
        Ok(m) => m,
        Err(e) => {
            cli::show_error(&format!("Error loading leagues: {e}"));
            return;
        }
    };

    let tx = match conn.transaction() {
        Ok(t) => t,
        Err(e) => {
            cli::show_error(&format!("Transaction error: {e}"));
            return;
        }
    };

    let mut inserted = 0usize;
    let mut updated = 0usize;

    for r in rows {
        let name = r.name.trim().to_string();
        if name.is_empty() {
            tx.rollback().ok();
            cli::show_error("Invalid row: empty team name");
            return;
        }

        let abbr = r
            .abbreviation
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let city = r
            .city
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let league_id = match r
            .league
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            None => None,
            Some(league_name) => match league_map.get(&league_name.to_lowercase()).copied() {
                Some(id) => Some(id),
                None => {
                    tx.rollback().ok();
                    cli::show_error(&format!("Unknown league: {league_name}"));
                    return;
                }
            },
        };

        // UPSERT: criterio (1) abbreviation se presente, altrimenti (2) name
        let existing_id = match find_existing_team_id(&tx, &name, abbr.as_deref()) {
            Ok(v) => v,
            Err(e) => {
                tx.rollback().ok();
                cli::show_error(&format!("DB error while searching existing team: {e}"));
                return;
            }
        };

        if let Some(id) = existing_id {
            if let Err(e) = tx.execute(
                "UPDATE teams
         SET name = ?2,
             abbreviation = ?3,
             city = ?4,
             founded_year = ?5,
             league_id = ?6
         WHERE id = ?1",
                params![id, name, abbr, city, r.founded_year, league_id],
            ) {
                tx.rollback().ok();
                cli::show_error(&format!("DB error while updating team: {e}"));
                return;
            }
            updated += 1;
        } else {
            if let Err(e) = tx.execute(
                "INSERT INTO teams (name, abbreviation, city, founded_year, league_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
                params![name, abbr, city, r.founded_year, league_id],
            ) {
                tx.rollback().ok();
                cli::show_error(&format!("DB error while inserting team: {e}"));
                return;
            }
            inserted += 1;
        }
    }

    if let Err(e) = tx.commit() {
        cli::show_error(&format!("Commit error: {e}"));
        return;
    }

    cli::show_success(&format!(
        "Teams import completed: {inserted} inserted, {updated} updated."
    ));
}

// -------- parsing --------

fn parse_teams_file(path: &str, content: &str) -> Result<Vec<ImportTeamRow>, String> {
    let lower = path.to_lowercase();
    if lower.ends_with(".json") {
        serde_json::from_str::<Vec<ImportTeamRow>>(content)
            .map_err(|e| format!("JSON parse error: {e}"))
    } else if lower.ends_with(".csv") {
        let mut rdr = csv::Reader::from_reader(content.as_bytes());
        let mut out = Vec::new();
        for rec in rdr.deserialize::<ImportTeamRow>() {
            out.push(rec.map_err(|e| format!("CSV parse error: {e}"))?);
        }
        Ok(out)
    } else {
        Err("Unsupported format: use .csv or .json".to_string())
    }
}

// -------- helpers DB --------

fn load_league_name_map(
    conn: &Connection,
) -> rusqlite::Result<std::collections::HashMap<String, i64>> {
    let mut stmt = conn.prepare("SELECT id, name FROM leagues")?;
    let mut rows = stmt.query([])?;
    let mut map = std::collections::HashMap::new();

    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let name: String = r.get(1)?;
        map.insert(name.to_lowercase(), id);
    }

    Ok(map)
}

fn find_existing_team_id(
    tx: &rusqlite::Transaction<'_>,
    name: &str,
    abbr: Option<&str>,
) -> rusqlite::Result<Option<i64>> {
    if let Some(a) = abbr {
        // prova prima per abbreviation
        let mut stmt =
            tx.prepare("SELECT id FROM teams WHERE LOWER(abbreviation) = LOWER(?1) LIMIT 1")?;
        let id_opt: Option<i64> = stmt.query_row([a], |r| r.get(0)).optional()?;
        if id_opt.is_some() {
            return Ok(id_opt);
        }
    }

    // fallback per name
    let mut stmt = tx.prepare("SELECT id FROM teams WHERE LOWER(name) = LOWER(?1) LIMIT 1")?;
    let id_opt: Option<i64> = stmt.query_row([name], |r| r.get(0)).optional()?;
    Ok(id_opt)
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
