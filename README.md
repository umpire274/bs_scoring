# ⚾ Baseball Scorer - v0.2.2

A comprehensive baseball and softball scoring application with SQLite persistence, official scoring symbols support, and cross-platform compatibility.

## 🆕 What's New in v0.2.2

- ✅ **Library Support**: Now usable as a Rust library in other projects
- ✅ **Standard Structure**: All code moved to `src/` directory
- ✅ **Enhanced Metadata**: Ready for crates.io publishing
- ✅ **Better Tooling**: Improved IDE support and documentation

## 📁 Project Structure

```
bs_scoring/
├── Cargo.toml           # Package configuration with lib + bin
├── README.md
├── CHANGELOG.md
├── SCORING_GUIDE.md
└── src/                 # All source code
    ├── lib.rs          # Library interface (NEW in v0.2.2)
    ├── main.rs         # CLI application entry point
    ├── core/           # Business logic
    │   ├── menu.rs     # COBOL-style menu system
    │   └── parser.rs   # Scoring notation parser
    ├── db/             # Database layer
    │   ├── config.rs   # Cross-platform path management
    │   ├── database.rs # SQLite schema and operations
    │   ├── league.rs   # League CRUD
    │   └── team.rs     # Team and Player CRUD
    └── models/         # Data types
        └── types.rs    # Game scoring types
```

## 🚀 Installation

### Prerequisiti
- Rust 1.75 o superiore (installa da [rustup.rs](https://rustup.rs/))

### Compilazione

```bash
cd baseball_scorer
cargo build --release
```

L'eseguibile sarà disponibile in `target/release/bs_scoring`

## 📖 Utilizzo

```bash
cargo run
# oppure
./target/release/bs_scoring
```

## 🎮 Menu Principale

All'avvio vedrai il menu principale:

```
╔════════════════════════════════════════════╗
║      ⚾ BASEBALL SCORER - MENU PRINCIPALE  ║
╚════════════════════════════════════════════╝

  1. 🆕 Nuova Partita
  2. 🏆 Gestione Leghe
  3. ⚾ Gestione Squadre
  4. 📊 Statistiche
  5. 🚪 Esci

Seleziona un'opzione (1-5):
```

## 🏆 Gestione Leghe

Crea e gestisci campionati:

- ➕ **Crea Nuova Lega**: Definisci nome, stagione, descrizione
- 📋 **Visualizza Leghe**: Vedi tutte le leghe esistenti
- ✏️ **Modifica Lega**: Aggiorna informazioni
- 🗑️ **Elimina Lega**: Rimuovi una lega (attenzione!)

**Esempio:**
```
Nome lega: Serie A Softball
Stagione: 2026
Descrizione: Campionato nazionale italiano
```

## ⚾ Gestione Squadre

Gestisci le tue squadre:

- ➕ **Crea Nuova Squadra**: Nome, città, abbreviazione, anno fondazione
- 📋 **Visualizza Squadre**: Lista di tutte le squadre
- ✏️ **Modifica Squadra**: Aggiorna dati squadra
- 👥 **Gestisci Roster**: Aggiungi/rimuovi giocatori (in sviluppo)
- 📥 **Importa Squadra**: Da JSON/CSV (in sviluppo)
- 🗑️ **Elimina Squadra**: Rimuovi squadra e roster

**Esempio:**
```
Nome squadra: Boston Red Sox
Città: Boston
Abbreviazione: BOS
Anno di fondazione: 1901
Lega: MLB (opzionale)
```

## 🗄️ Schema Database

### Tabelle Principali

#### leagues
- id, name (UNIQUE), season, description, created_at

#### teams
- id, name, league_id (FK), city, abbreviation, founded_year, created_at

#### players
- id, team_id (FK), number, name, position (1-9), batting_order, is_active, created_at

#### games
- id, game_id (UNIQUE), home/away_team_id (FK), venue, game_date, scores, hits, errors, current state

#### plate_appearances
- id, game_id (FK), inning, batter/pitcher_id (FK), result_type, pitch data, runs, rbis, notes

## 🎯 Simboli di Scoring

*Vedi [SCORING_GUIDE.md](SCORING_GUIDE.md) per la guida completa*

**Basi:** 1B, 2B, 3B, HR, GRD  
**Out:** K, KL, 6-3, F8, L9, P5, DP  
**Walks:** BB, IBB, HBP  
**Errori:** E6, E4, E9  
**Avanzati:** SB2, WP, PB, BK, SF  

## 📄 Licenza

MIT License ⚾

---

**Buon Scoring! Play Ball! ⚾**
