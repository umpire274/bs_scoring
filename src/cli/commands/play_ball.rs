use crate::core::play_ball::{gate_check_lineups, list_playable_games, set_game_status};
use crate::models::play_ball::{LineupSide, PlayBallGameContext, PlayBallGate};
use rusqlite::params;

use crate::Database;
use crate::cli::commands::game::{insert_team_lineup, save_lineup};
use crate::engine::play_ball::run_play_ball_engine;
use crate::models::types::GameStatus;
use crate::ui::factory::create_ui;
use crate::utils::cli;

pub fn play_ball(db: &mut Database) {
    cli::show_header("PLAY BALL");

    let conn = db.get_connection_mut();

    let games = match list_playable_games(conn) {
        Ok(v) => v,
        Err(e) => {
            cli::show_error(&format!("Error querying games: {e}"));
            return;
        }
    };

    if games.is_empty() {
        println!("📭 No pre-game games found.");
        cli::wait_for_enter();
        return;
    }

    println!("\n📋 Available Games:\n");
    for (i, g) in games.iter().enumerate() {
        let away_display = g.away_team_abbr.as_deref().unwrap_or(&g.away_team_name);
        let home_display = g.home_team_abbr.as_deref().unwrap_or(&g.home_team_name);

        println!(
            "  {}. {} - {} @ {}",
            i + 1,
            g.game_date,
            away_display,
            home_display
        );
        println!(
            "     Status: {} | Venue: {} | ID: {}",
            g.status, g.venue, g.game_id
        );
        println!();
    }

    let game_choice = match cli::read_i64("Select game (number, 0 to cancel): ") {
        Some(0) | None => return,
        Some(c) if c > 0 && (c as usize) <= games.len() => c as usize,
        _ => {
            cli::show_error("Invalid selection");
            return;
        }
    };

    let g = &games[game_choice - 1];

    // Se la partita NON è in Pregame, si entra direttamente nell'engine (resume),
    // senza bloccare su lineup gate-check.
    if g.status != GameStatus::Pregame {
        let away_display = g.away_team_abbr.as_deref().unwrap_or(&g.away_team_name);
        let home_display = g.home_team_abbr.as_deref().unwrap_or(&g.home_team_name);

        let mut ui = create_ui();

        run_play_ball_engine(
            conn,
            &mut *ui,
            g.id,
            &g.game_id,
            g.away_team_id,
            away_display,
            home_display,
        );

        return;
    }

    // Qui sotto: solo Pregame -> gate check obbligatorio
    match gate_check_lineups(conn, &g.game_id, g.away_team_id, g.home_team_id) {
        Ok(PlayBallGate::Ready) => {
            // Se la gara è in Pregame, passiamo automaticamente a InProgress
            if g.status == GameStatus::Pregame {
                match set_game_status(conn, &g.game_id, GameStatus::InProgress) {
                    Ok(true) => {}
                    Ok(false) => {
                        cli::show_error(
                            "Game status was not updated (game not in Pre-Game status?)",
                        );
                        cli::wait_for_enter();
                        return;
                    }
                    Err(e) => {
                        cli::show_error(&format!("Failed to set game LIVE: {e}"));
                        cli::wait_for_enter();
                        return;
                    }
                }
            }

            let away_display = g.away_team_abbr.as_deref().unwrap_or(&g.away_team_name);
            let home_display = g.home_team_abbr.as_deref().unwrap_or(&g.home_team_name);

            let mut ui = create_ui();

            run_play_ball_engine(
                conn,
                &mut *ui,
                g.id,
                &g.game_id,
                g.away_team_id,
                away_display,
                home_display,
            );
        }

        Ok(PlayBallGate::InvalidLineup {
            side,
            required,
            found,
        }) => {
            handle_invalid_lineup(db, g, side, required, found);
            // handle_invalid_lineup immagino faccia già wait_for_enter; se no aggiungilo qui.
        }

        Err(e) => {
            cli::show_error(&format!("Error checking lineups: {e}"));
            cli::wait_for_enter();
        }
    }
}

fn handle_invalid_lineup(
    db: &mut Database,
    g: &PlayBallGameContext,
    side: LineupSide,
    required: i64,
    found: i64,
) {
    let conn = db.get_connection_mut();

    let (team_id, team_name) = match side {
        LineupSide::Away => (g.away_team_id, g.away_team_name.clone()),
        LineupSide::Home => (g.home_team_id, g.home_team_name.clone()),
    };

    cli::show_error(&format!(
        "Invalid lineup for {} team: {} (found {}, required {}).",
        side.label(),
        team_name,
        found,
        required
    ));

    // Se found < required: proponiamo inserimento/rimpiazzo lineup
    if found < required {
        if !cli::confirm(&format!(
            "Do you want to insert/replace the lineup for {} now?",
            team_name
        )) {
            println!("\n❌ Cannot start the game without valid lineups.");
            cli::wait_for_enter();
            return;
        }

        println!(
            "\nLineup required for {}: {} players ({} rule).",
            team_name,
            required,
            if required == 10 { "DH" } else { "No DH" }
        );

        let new_lineup = match insert_team_lineup(conn, team_id, &team_name, required as usize) {
            Some(lineup) => lineup,
            None => {
                println!("\n❌ Lineup insertion cancelled");
                cli::wait_for_enter();
                return;
            }
        };

        // Rimpiazzo totale delle righe lineup per game/team
        if let Err(e) = conn.execute(
            "DELETE FROM game_lineups WHERE game_id = ?1 AND team_id = ?2",
            params![&g.game_id, team_id],
        ) {
            cli::show_error(&format!("Failed to cleanup old lineup: {e}"));
            return;
        }

        if let Err(e) = save_lineup(conn, &g.game_id, team_id, &new_lineup) {
            cli::show_error(&format!("Failed to save lineup: {e}"));
            return;
        }

        cli::show_success(&format!("Lineup saved for {}.", team_name));
    } else {
        // found > required: caso “troppi starter” rispetto alla regola DH
        println!("\nThe lineup has too many starters for the current DH setting.");
        println!("Use 'Edit Lineups' or 'Import Lineup' to fix it.");
        cli::wait_for_enter();
        return;
    }

    // Ricontrollo dopo l’azione correttiva
    match gate_check_lineups(conn, &g.game_id, g.away_team_id, g.home_team_id) {
        Ok(PlayBallGate::Ready) => {
            cli::show_success_no_wait_for_enter(
                "Both lineups are valid now. You can start the game.",
            );
        }
        Ok(PlayBallGate::InvalidLineup {
            side,
            required,
            found,
        }) => {
            let other_name = match side {
                LineupSide::Away => &g.away_team_name,
                LineupSide::Home => &g.home_team_name,
            };
            cli::show_error(&format!(
                "Still invalid lineup for {} team: {} (found {}, required {}).",
                side.label(),
                other_name,
                found,
                required
            ));
        }
        Err(e) => cli::show_error(&format!("Error checking lineups: {e}")),
    }

    cli::wait_for_enter();
}
