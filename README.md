# ⚾ Baseball Scorer CLI

Un'applicazione CLI professionale in Rust per il scoring di partite di baseball/softball, utilizzando i simboli ufficiali degli scorer.

## 🚀 Installazione

### Prerequisiti
- Rust 1.75 o superiore (installa da [rustup.rs](https://rustup.rs/))

### Compilazione

```bash
cd baseball_scorer
cargo build --release
```

L'eseguibile sarà disponibile in `target/release/baseball_scorer`

## 📖 Utilizzo

```bash
cargo run
# oppure
./target/release/baseball_scorer
```

## 🎯 Simboli di Scoring

### Basi (Hits)
- `1B` o `SINGLE` - Singolo
- `2B` o `DOUBLE` - Doppio
- `3B` o `TRIPLE` - Triplo
- `HR` o `HOMERUN` - Fuoricampo (Home Run)
- `GRD` - Ground Rule Double

### Out
- `K` - Strikeout al volo (swinging)
- `KL` o `K-L` - Strikeout guardato (looking) (può anche usare ꓘ)
- `6-3` - Groundout interbase → prima base
- `4-3` - Groundout seconda base → prima base
- `5-3` - Groundout terza base → prima base
- `F7` - Flyout all'esterno sinistro
- `F8` - Flyout all'esterno centro
- `F9` - Flyout all'esterno destro
- `L6` - Lineout all'interbase
- `P5` - Popup alla terza base
- `6-4-3 DP` - Doppio gioco (double play)
- `SF8` - Sacrifice Fly all'esterno centro

### Basi su Ball (Walks)
- `BB` - Base on Balls (base su ball)
- `IBB` - Intentional Walk (base intenzionale)
- `HBP` - Hit By Pitch (colpito dal lancio)

### Errori e Scelte del Difensore
- `E6` - Errore dell'interbase
- `E4` - Errore della seconda base
- `FC` - Fielder's Choice (scelta del difensore)

### Giochi Avanzati
- `SB2` - Stolen Base alla seconda
- `SB3` - Stolen Base alla terza
- `SBH` - Stolen Base a casa base
- `WP` - Wild Pitch
- `PB` - Passed Ball
- `BK` - Balk
- `SH` o `SAC` - Sacrifice Hit (bunt di sacrificio)

## 🔢 Posizioni Difensive

Il sistema di numbering standard per le posizioni:

1. **Pitcher** (Lanciatore)
2. **Catcher** (Ricevitore)
3. **First Base** (Prima base)
4. **Second Base** (Seconda base)
5. **Third Base** (Terza base)
6. **Shortstop** (Interbase)
7. **Left Field** (Esterno sinistro)
8. **Center Field** (Esterno centro)
9. **Right Field** (Esterno destro)

## 📝 Comandi dell'Applicazione

### Comandi di Base
- `help` o `h` - Mostra l'aiuto
- `save` o `s` - Salva la partita in formato JSON
- `show` o `view` - Mostra le statistiche attuali
- `next` o `n` - Passa al prossimo battitore
- `inning` - Avanza al prossimo inning (o metà inning)
- `exit`, `quit`, o `q` - Esci dall'applicazione

## 💡 Esempi di Utilizzo

### Esempio 1: Sequenza di Gioco Semplice

```
> 1B
✅ Play registrato: Single

> 6-3
✅ Play registrato: Groundout [Shortstop, FirstBase]

> K
✅ Play registrato: Strikeout (swinging)

> HR
✅ Play registrato: Home Run

> save
💾 Partita salvata in: GAME_20260201_143022.json
```

### Esempio 2: Situazioni Complesse

```
> 6-4-3 DP
✅ Play registrato: Double Play [Shortstop, SecondBase, FirstBase]

> SF8
✅ Play registrato: Sacrifice Fly to Center Field (RBI)

> E6
✅ Play registrato: Error by Shortstop

> BB
✅ Play registrato: Base on Balls
```

### Esempio 3: Giochi Avanzati

```
> SB2
✅ Stolen Base to Second

> WP
✅ Wild Pitch

> BK
✅ Balk
```

## 📊 Formato JSON Output

Il file JSON salvato contiene tutti i dettagli della partita:

```json
{
  "game_id": "GAME_20260201_143022",
  "date": "2026-02-01",
  "home_team": {
    "name": "Red Sox",
    "lineup": [],
    "runs": 3,
    "hits": 5,
    "errors": 1
  },
  "away_team": {
    "name": "Yankees",
    "lineup": [],
    "runs": 2,
    "hits": 4,
    "errors": 0
  },
  "venue": "Fenway Park",
  "plate_appearances": [
    {
      "inning": 1,
      "half_inning": "Top",
      "batter_number": 1,
      "batter_name": "Player Name",
      "pitcher_name": "Pitcher Name",
      "result": {
        "Hit": {
          "hit_type": "Single",
          "location": null,
          "rbis": 0
        }
      },
      "pitch_count": null,
      "runners": [],
      "outs_before": 0,
      "outs_after": 0,
      "runs_scored": 0,
      "notes": null
    }
  ],
  "current_inning": 1,
  "current_half": "Top"
}
```

## 🎓 Guida al Scoring Professionale

### Simboli Comuni Combinati

- `1B-E7` - Singolo con errore dell'esterno sinistro
- `FC-6` - Fielder's choice all'interbase
- `K+WP` - Strikeout con wild pitch (batter raggiunge la base)
- `K+PB` - Strikeout con passed ball
- `4-6-3 DP` - Doppio gioco seconda → interbase → prima

### Notazione per Runners

Quando i corridori avanzano:
- Usa le note per indicare avanzamenti specifici
- `SB` per stolen base
- `WP` per wild pitch advancement
- `E` seguita dal numero per errori che permettono avanzamenti

### Best Practices

1. **Registra ogni plate appearance** immediatamente
2. **Usa 'save' frequentemente** per non perdere dati
3. **Verifica le statistiche** con 'show' dopo ogni inning
4. **Segna i dettagli** nelle note quando necessario

## 🔧 Sviluppi Futuri

- [ ] Interfaccia grafica per il diamante
- [ ] Tracking completo dei corridori
- [ ] Statistiche avanzate (ERA, WHIP, OPS, etc.)
- [ ] Import/Export in altri formati (CSV, Excel)
- [ ] Visualizzazione della scorecard ASCII
- [ ] Replay delle azioni
- [ ] Multi-game support e statistiche di stagione

## 📚 Riferimenti

- [Official MLB Scoring Rules](https://www.mlb.com/official-information/official-rules)
- [Project Scoresheet](http://www.projectscoresheet.org/)
- [Baseball Scorekeeping Handbook](https://www.littleleague.org/university/scorekeeping/)

## 🤝 Contributi

Contributi, issues e feature requests sono benvenuti!

## 📄 Licenza

MIT License - sentiti libero di usare questo progetto per le tue partite! ⚾

---

**Buon Scoring! Play Ball! ⚾**
