# ⚾ BS Scoring – Scoring Command Guide

This document describes all commands currently available in the **Play Ball** engine.

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
> 1b  
> hr  
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
- Initializes the first at-bat
- Writes a log line like:

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
- When **balls reach 4** (and strikes are less than 3), the batter is awarded **first base** (`BB`).
- After a walk:
    - the count resets to `0-0`
    - the engine advances to the next batter

### 3.2 Strikes

| Command | Meaning                 |
|---------|-------------------------|
| `k`     | Called strike (looking) |
| `s`     | Swinging strike         |
| `f`     | Foul                    |
| `fl`    | Foul bunt               |

Rules:

- `k` and `s` always increment **strike count**.
- `f` increments strikes **only if strikes < 2**.
- `fl` increments strikes **even if strikes = 2** (foul bunt can be strike 3).
- When **strikes reach 3** before 4 balls:
    - the batter is out (`K`)
    - outs increment by 1
    - the count resets to `0-0`
    - the engine advances to the next batter

> Commands `x` (in play) and `h` (hit-by-pitch) are still reserved for future versions.

---

## 4) Hit commands

The following commands record a completed plate appearance resulting in a hit:

| Command | Meaning   |
|---------|-----------|
| `1b`    | Single    |
| `2b`    | Double    |
| `3b`    | Triple    |
| `hr`    | Home Run  |

### Current scoring rules for hits (v0.7.2)

This first implementation uses a **simplified automatic runner advancement model**:

- On `1b`, all existing runners advance **1 base**
- On `2b`, all existing runners advance **2 bases**
- On `3b`, all existing runners advance **3 bases**
- On `hr`, all existing runners score, and the batter also scores

### Important note

This is an intentionally simplified first implementation.

It does **not yet** model:

- runner-by-runner manual advancement
- fielder’s choice
- errors
- sacrifice plays
- realistic advancement on singles/doubles
- RBI attribution details

### Pitch count behavior for hit commands

Commands `1b`, `2b`, `3b`, and `hr` also count as the **final pitch of the plate appearance**.

This means:

- the current pitcher’s total pitch count is incremented by **1**
- the final step (`1B`, `2B`, `3B`, or `HR`) is appended to the persisted plate appearance sequence

Examples of persisted plate appearance sequences:

- `[B, K, F, 1B]`
- `[B, B, HR]`
- `[K, S, 2B]`

---

## 5) Game status commands

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

## 6) Command format rules

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
- `1b`
- `2B`
- `hr`

---

## 7) Resume / replay behavior

From the 0.7.x series onward, game reconstruction is based on persisted **compact plate appearances**.

This means:

- the game state is rebuilt deterministically from `plate_appearances_compact`
- pitch sequences are replayed from persisted data
- hit results such as `1B`, `2B`, `3B`, and `HR` are stored in the plate appearance sequence
- batting order and pitcher pitch counts are restored deterministically

An in-progress plate appearance is still restored through the draft mechanism when available.

---

## 8) Current scope (v0.7.2)

Supported commands:

- Engine: `exit`, `quit`
- Start: `playball`
- Status: `regular`, `post`, `cancel`, `susp`, `forf`, `protest`
- Pitching/count: `b`, `k`, `s`, `f`, `fl`
- Hits: `1b`, `2b`, `3b`, `hr`

Currently implemented:

- deterministic resume from compact plate appearances
- pitch sequence persistence
- simplified runner advancement for hits
- scoreboard totals and inning-by-inning runs
- hit totals (`H`) in the scoreboard

Planned next steps:

- realistic runner advancement
- in-play result modeling
- hit-by-pitch (`h`)
- errors (`E`)
- additional official scoring outcomes