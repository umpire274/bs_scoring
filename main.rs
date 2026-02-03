mod models;
mod core;

use models::types::*;
use core::parser::CommandParser;
use std::io::{self, Write};

fn main() {
    println!("⚾ Baseball Scorer CLI");
    println!("====================\n");

    // Initialize game
    let mut game = setup_game();

    println!("\n🎮 Comandi disponibili:");
    print_help();

    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "help" | "h" => print_help(),
            "save" | "s" => save_game(&game),
            "show" | "view" => show_game(&game),
            "exit" | "quit" | "q" => {
                println!("👋 Arrivederci!");
                break;
            }
            "next" | "n" => next_batter(&mut game),
            "inning" => next_inning(&mut game),
            _ => process_play(&mut game, input),
        }
    }
}

fn setup_game() -> Game {
    println!("📝 Setup partita");
    println!("================\n");

    print!("Squadra ospite: ");
    io::stdout().flush().unwrap();
    let mut away_name = String::new();
    io::stdin().read_line(&mut away_name).unwrap();

    print!("Squadra casa: ");
    io::stdout().flush().unwrap();
    let mut home_name = String::new();
    io::stdin().read_line(&mut home_name).unwrap();

    print!("Stadio: ");
    io::stdout().flush().unwrap();
    let mut venue = String::new();
    io::stdin().read_line(&mut venue).unwrap();

    let away_team = Team {
        name: away_name.trim().to_string(),
        lineup: Vec::new(),
        runs: 0,
        hits: 0,
        errors: 0,
    };

    let home_team = Team {
        name: home_name.trim().to_string(),
        lineup: Vec::new(),
        runs: 0,
        hits: 0,
        errors: 0,
    };

    let game_id = format!("GAME_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    Game::new(
        game_id,
        date,
        home_team,
        away_team,
        venue.trim().to_string(),
    )
}

fn print_help() {
    println!("\n📋 SIMBOLI DI SCORING:");
    println!("  Basi:");
    println!("    1B, 2B, 3B, HR    - Singolo, Doppio, Triplo, Fuoricampo");
    println!("    GRD               - Ground Rule Double");
    println!("\n  Out:");
    println!("    K                 - Strikeout al volo");
    println!("    KL                - Strikeout guardato");
    println!("    6-3               - Groundout (es: interbase-prima base)");
    println!("    F8                - Flyout (es: al centro)");
    println!("    L9                - Lineout (es: al destro)");
    println!("    P5                - Popup (es: alla terza base)");
    println!("    6-4-3 DP          - Doppio gioco");
    println!("    SF8               - Sacrifice Fly");
    println!("\n  Basi su ball:");
    println!("    BB                - Base on Balls");
    println!("    IBB               - Intenzionale");
    println!("    HBP               - Colpito dal lancio");
    println!("\n  Errori e altro:");
    println!("    E6                - Errore (numero posizione)");
    println!("    FC                - Fielder's Choice");
    println!("\n  Giochi avanzati:");
    println!("    SB2, SB3, SBH     - Stolen Base");
    println!("    WP                - Wild Pitch");
    println!("    PB                - Passed Ball");
    println!("    BK                - Balk");
    println!("\n  Posizioni difensive:");
    println!("    1=Lanciatore  2=Ricevitore  3=Prima base");
    println!("    4=Seconda base  5=Terza base  6=Interbase");
    println!("    7=Esterno sinistro  8=Esterno centro  9=Esterno destro");
    println!("\n📌 COMANDI:");
    println!("  help, h           - Mostra questo aiuto");
    println!("  save, s           - Salva la partita in JSON");
    println!("  show, view        - Mostra statistiche attuali");
    println!("  next, n           - Prossimo battitore");
    println!("  inning            - Prossimo inning");
    println!("  exit, quit, q     - Esci");
}

fn process_play(game: &mut Game, input: &str) {
    // Parse the command
    match CommandParser::parse_command(input) {
        Ok(result) => {
            println!("✅ Play registrato: {:?}", result);
            
            // Create a placeholder plate appearance
            // In a full implementation, you'd collect more details
            let pa = PlateAppearance {
                inning: game.current_inning,
                half_inning: game.current_half,
                batter_number: 1, // Would track actual batter
                batter_name: "Batter".to_string(),
                pitcher_name: "Pitcher".to_string(),
                result,
                pitch_count: None,
                runners: Vec::new(),
                outs_before: 0,
                outs_after: 0,
                runs_scored: 0,
                notes: None,
            };

            game.add_plate_appearance(pa);

            // Update team stats based on result
            update_team_stats(game);
        }
        Err(e) => {
            println!("❌ Errore: {}", e);
            println!("💡 Usa 'help' per vedere i comandi disponibili");
        }
    }
}

fn update_team_stats(game: &mut Game) {
    // Count hits and errors from last play
    if let Some(last_pa) = game.plate_appearances.last() {
        let team = match last_pa.half_inning {
            HalfInning::Top => &mut game.away_team,
            HalfInning::Bottom => &mut game.home_team,
        };

        match &last_pa.result {
            PlateAppearanceResult::Hit { .. } => {
                team.hits += 1;
            }
            PlateAppearanceResult::Error { .. } => {
                team.errors += 1;
            }
            _ => {}
        }

        if last_pa.runs_scored > 0 {
            team.runs += last_pa.runs_scored;
        }
    }
}

fn next_batter(_game: &mut Game) {
    println!("🏃 Prossimo battitore all'at-bat");
    // In full implementation, would cycle through lineup
}

fn next_inning(game: &mut Game) {
    match game.current_half {
        HalfInning::Top => {
            game.current_half = HalfInning::Bottom;
            println!("⬇️  Cambio metà inning - Bottom of {}", game.current_inning);
        }
        HalfInning::Bottom => {
            game.current_inning += 1;
            game.current_half = HalfInning::Top;
            println!("⬆️  Inning {} - Top", game.current_inning);
        }
    }
}

fn save_game(game: &Game) {
    let filename = format!("{}.json", game.game_id);
    match game.save_to_file(&filename) {
        Ok(_) => println!("💾 Partita salvata in: {}", filename),
        Err(e) => println!("❌ Errore nel salvataggio: {}", e),
    }
}

fn show_game(game: &Game) {
    println!("\n📊 STATO PARTITA");
    println!("================");
    println!("🏟️  {}", game.venue);
    println!("📅 {}", game.date);
    println!("\n⚾ Inning: {} {}", 
        game.current_inning,
        match game.current_half {
            HalfInning::Top => "(Top)",
            HalfInning::Bottom => "(Bottom)",
        }
    );
    
    println!("\n📋 PUNTEGGIO:");
    println!("  {} (Ospiti): {} runs, {} hits, {} errors",
        game.away_team.name,
        game.away_team.runs,
        game.away_team.hits,
        game.away_team.errors
    );
    println!("  {} (Casa): {} runs, {} hits, {} errors",
        game.home_team.name,
        game.home_team.runs,
        game.home_team.hits,
        game.home_team.errors
    );
    
    println!("\n📝 Plate Appearances registrati: {}", game.plate_appearances.len());
}
