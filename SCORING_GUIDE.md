# ⚾ BS Scoring – Scoring Command Guide

This document describes all commands available in the **Play Ball** engine (v0.8.0).

---

## Prompt format

The engine prompt is:

> ↑ 1 (0 OUTS) AWY 0 - 0 HOM >

Where:

- `↑` / `↓` = Top / Bottom of the inning
- Number = current inning
- `OUTS` = outs in the current half-inning
- `AWY` / `HOM` = team abbreviations
- Score displayed as `AWY - HOM`

Multiple commands can be entered on the same line, comma-separated:

```
b, b, f, k
6 h, 5 2b
```

Commands are case-insensitive.

---

## 1) Engine control

| Command          | Description                         |
|------------------|-------------------------------------|
| `exit` or `quit` | Exit engine and return to Main Menu |

---

## 2) Game start

| Command    | Description                                                                  |
|------------|------------------------------------------------------------------------------|
| `playball` | Starts the game and loads the first at-bat (Away lineup, batting order #1). |

`playball` is only allowed when the game has no previous events.

---

## 3) Pitching commands

Recorded for the **current plate appearance** (current batter vs current pitcher).

| Command | Meaning                 |
|---------|-------------------------|
| `b`     | Ball                    |
| `k`     | Called strike (looking) |
| `s`     | Swinging strike         |
| `f`     | Foul ball               |
| `fl`    | Foul bunt               |

Rules:

- `k` and `s` always increment the strike count.
- `f` increments strikes only if strikes < 2.
- `fl` increments strikes even if strikes = 2 (foul bunt can be strike 3).
- 4 balls → walk (BB); 3 strikes → strikeout (K).
- After either outcome, the count resets and the engine advances to the next batter.

---

## 4) Hit commands

### 4.1 Basic syntax

| Command | Meaning      |
|---------|--------------|
| `h`     | Single (1B)  |
| `2h`    | Double (2B)  |
| `3h`    | Triple (3B)  |
| `hr`    | Home Run     |

An optional field zone can follow the hit command:

```
h lf       → single to left field
2h rc      → double to right-center
hr cf      → home run to center field
```

### 4.2 Field zones

| Code  | Area                        |
|-------|-----------------------------|
| `ll`  | Left line (down the line)   |
| `lf`  | Left field                  |
| `lc`  | Left-center                 |
| `cf`  | Center field                |
| `rc`  | Right-center                |
| `rf`  | Right field                 |
| `rl`  | Right line (down the line)  |
| `gll` | Ground ball — left line     |
| `ls`  | Left side infield           |
| `mi`  | Middle infield              |
| `rs`  | Right side infield          |
| `grl` | Ground ball — right line    |

Zone codes are case-insensitive (`LF`, `lf`, `Lf` are all valid).

These commands also count as the **final pitch of the plate appearance** (pitch count +1).

### 4.3 Batting-order prefix (optional)

A batting-order number (1–9) can precede the hit command:

```
6 h        → batter #6 hits a single
4 2h lf    → batter #4 doubles to left field
```

The prefix is informational context for the scorer; the engine identifies the current batter
from the game state regardless.

### 4.4 Runner overrides (v0.8.0)

By default, runners advance automatically based on the number of bases hit:

| Hit | Automatic runner advancement     |
|-----|----------------------------------|
| `h` | All runners advance +1 base      |
| `2h`| All runners advance +2 bases     |
| `3h`| All runners advance +3 bases     |
| `hr`| All runners score                |

To override where a specific runner ends up, add **runner tokens** after the hit command,
comma-separated. Runner tokens are identified by **batting order** (the order slot they
occupied when they got on base).

Syntax: `<batting_order> <destination>` oppure `<batting_order><destination>` (senza spazio)

Valid destinations:

| Destination     | Meaning                  |
|-----------------|--------------------------|
| `1b`            | Stays / goes to 1st base |
| `2b`            | Stays / goes to 2nd base |
| `3b`            | Advances to 3rd base     |
| `sc` / `score`  | Runner scores (run++)    |

#### Examples

```
h                          → single, all runners advance +1 automatically
6 h, 5 2b                  → batter #6 singles; runner #5 stays on 2nd
6 h, 5 2b, 3 sc            → batter #6 singles; runner #5 stays on 2nd; runner #3 scores
6 h, 5 sc                  → batter #6 singles; runner #5 scores (aggressive read)
4 2h, 2 sc                 → batter #4 doubles; runner #2 scores instead of stopping at 4th
3h, 7 sc, 5 sc             → triple; both runners on base score
9 h, 8 2b, 7sc, 6sc        → bases loaded single; #8→2B, #7 scores, #6 scores (compact format)
hr                         → home run; all runners and batter score automatically (no override needed)
```

**Notes:**

- Any runner **not** mentioned in a runner token uses automatic advancement.
- The batter cannot be listed as a runner override in the same command.
- Overrides are persisted in the plate appearance record and are used faithfully
  when replaying the game state on resume.

---

## 5) Game status commands

These commands update the game status and **exit the engine**.

| Command   | New Status       | Description                    |
|-----------|------------------|--------------------------------|
| `regular` | Regulation Game  | Ends game as a regulation game |
| `post`    | Postponed Game   | Marks game as postponed        |
| `cancel`  | Cancelled Game   | Marks game as cancelled        |
| `susp`    | Suspended Game   | Suspends the game              |
| `forf`    | Forfeited Game   | Marks game as forfeited        |
| `protest` | Protested Game   | Marks game as protested        |

---

## 6) Command format rules

- Commands are case-insensitive
- Commands are comma-separated
- Leading/trailing spaces are ignored
- Unknown commands generate an error but do not stop execution

---

## 7) Resume / replay behavior

Game reconstruction is based on persisted **compact plate appearances**:

- The game state is rebuilt deterministically from `plate_appearances_compact`
- Pitch sequences and outcomes are replayed from persisted data
- Runner overrides entered during live scoring are stored in the plate appearance
  record and faithfully applied on replay
- Batting order and pitcher pitch counts are restored deterministically
- An in-progress plate appearance is restored via the draft mechanism

---

## 8) Current scope (v0.8.0)

Supported commands:

- Engine: `exit`, `quit`
- Start: `playball`
- Status: `regular`, `post`, `cancel`, `susp`, `forf`, `protest`
- Pitching/count: `b`, `k`, `s`, `f`, `fl`
- Hits: `h`, `2h`, `3h`, `hr` (with optional zone and runner overrides)

Implemented:

- Deterministic resume from compact plate appearances
- Pitch sequence persistence
- Automatic runner advancement for hits
- **Explicit runner overrides by batting order** (v0.8.0)
- Runner identity on bases (`Option<BatterOrder>`)
- Scoreboard totals and inning-by-inning runs
- Hit totals in the scoreboard

Planned next steps:

- In-play result modeling (`x`)
- Hit-by-pitch (`hbp`)
- Errors (`E`)
- Fielder's choice
- Sacrifice plays
- Out on base (runner thrown out attempting to advance)
