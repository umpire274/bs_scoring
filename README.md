# ⚾ Baseball Scorer CLI - Versione 2.0

Un'applicazione CLI professionale in Rust per il scoring di partite di baseball/softball con database SQLite integrato e interfaccia menu COBOL-style.

## 🆕 Novità Versione 2.0

- ✅ **Database SQLite** per persistenza dati
- ✅ **Menu principale stile COBOL** 
- ✅ **Gestione Leghe** completa (CRUD)
- ✅ **Gestione Squadre** con roster
- ✅ **Struttura modulare** migliorata
- 🚧 **Sistema di scoring** (in sviluppo)
- 🚧 **Statistiche avanzate** (in sviluppo)

## 📁 Struttura Progetto

```
baseball_scorer/
├── Cargo.toml              # Configurazione e dipendenze
├── main.rs                 # Entry point con menu principale
├── core/                   # Logica di business
│   ├── mod.rs
│   ├── parser.rs          # Parser comandi scoring
│   └── menu.rs            # Sistema menu navigazione
├── models/                 # Modelli dati e DB
│   ├── mod.rs
│   ├── types.rs           # Tipi scoring (Hit, Out, ecc.)
│   ├── database.rs        # Schema e init DB
│   ├── league.rs          # CRUD leghe
│   └── team.rs            # CRUD squadre e giocatori
└── baseball_scorer.db     # Database SQLite (auto-creato)
```

## 🚀 Installazione

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
