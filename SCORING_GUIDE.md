# ⚾ BS Scoring – Scoring Command Guide

This document describes all commands available in the **Play Ball** engine.

---

## Prompt format

The engine prompt format is:

> ↑ 1 (0 OUTS) AWY 0 - 0 HOM >

Where:

- `↑` / `↓` = Top / Bottom of the inning
- Number = current inning
- `OUTS` = outs in the current half-inning
- `AWY` / `HOM` = team abbreviations
- Score displayed as `AWY - HOM`

Multiple commands can be entered separated by commas:

> playball  
> b,b,f,k  
> susp

Commands are case-insensitive.

---

## 1) Engine control commands

These commands control the engine itself.

| Command          | Description                         |
|------------------|-------------------------------------|
| `exit` or `quit` | Exit engine and return to Main Menu |

---

## 2) Game start command

When you enter the Play Ball engine and **there are no previous events** for the selected game, start the game with:

| Command    | Description                                                                |
|------------|----------------------------------------------------------------------------|
| `playball` | Starts the game and logs the first at-bat (Away lineup, batting order #1). |

### What `playball` does

- Appends a `game_started` event to `game_events`
- Appends an `at_bat_started` event to `game_events`
- Writes (and persists) a log line like:

> At bat: `<TEAM ABBRV>` `#<JERSEY_NUMBER>` `<FirstName> <LastName>`

> Note: `playball` is only allowed when the game has **no previous events**.

---

## 3) Pitching commands (current batter)

Pitching commands are recorded for the **current plate appearance** (current batter vs current pitcher).

### 3.1 Ball

| Command | Meaning |
|---------|---------|
| `b`     | Ball    |

Rules:

- Each `b` increments the **ball count**.
- When **balls reach 4** (and strikes are less than 3), the batter is awarded **first base** (BB).
- After a walk:
    - the count resets to `0-0`
    - the engine will advance to the next batter (when the “next batter” logic is enabled)

### 3.2 Strikes

| Command | Meaning                 |
|---------|-------------------------|
| `k`     | Called strike (looking) |
| `s`     | Swinging strike         |
| `f`     | Foul                    |
| `fl`    | Foul bunt               |

Rules:

- `k` and `s` always increment **strike count**.
- `f` increments strikes **only if strikes < 2** (foul does not make strike 3).
- `fl` increments strikes **even if strikes = 2** (foul bunt CAN be strike 3).
- When **strikes reach 3** before 4 balls:
    - the batter is out (strikeout)
    - outs increment by 1
    - the count resets to `0-0`
    - the engine will advance to the next batter (when enabled)

> Commands `x` (in play) and `h` are reserved for future versions.

---

## 4) Game status commands

These commands can be entered at any time during the game.
They update the game status in the database and **exit the engine**.

| Command   | New Status      | Description                    |
|-----------|-----------------|--------------------------------|
| `regular` | Regulation Game | Ends game as a regulation game |
| `post`    | Postponed Game  | Marks game as postponed        |
| `cancel`  | Cancelled Game  | Marks game as cancelled        |
| `susp`    | Suspended Game  | Suspends the game              |
| `forf`    | Forfeited Game  | Marks game as forfeited        |
| `protest` | Protested Game  | Marks game as protested        |

---

## 5) Command format rules

- Commands are case-insensitive
- Commands are comma-separated
- Spaces are ignored
- Unknown commands generate an error but do not stop execution

Valid examples:

- `PLAYBALL`
- `playball`
- `PlayBall`
- `b,b,f,k`
- `FL`

---

## 6) Status transition rules

- Pregame → In Progress (when Play Ball starts)
- In Progress → Regulation
- In Progress → Postponed
- In Progress → Cancelled
- In Progress → Suspended
- In Progress → Forfeited
- In Progress → Protested

Status commands automatically exit the engine.

---

## 7) Current scope (v0.6.7)

Supported commands:

- Engine: `exit`, `quit`
- Start: `playball`
- Status: `regular`, `post`, `cancel`, `susp`, `forf`, `protest`
- Pitching (count): `b`, `k`, `s`, `f`, `fl`

Next steps will add:

- automatic next batter selection
- full runner/base state changes (forced advances, etc.)
- in-play commands (`x`) and hit-by-pitch (`h`)
- official scoring outcomes (hits/outs/PA results)