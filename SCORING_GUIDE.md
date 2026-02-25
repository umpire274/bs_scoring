# ⚾ BS Scoring – Command Guide

This document describes all commands available in the **Play Ball** engine.

## Prompt format

The engine prompt format is:

↑ 1 (0 OUTS) AWY 0 - 0 HOM >

Where:

- ↑ / ↓ = Top / Bottom of the inning
- Number = current inning
- OUTS = number of outs in the half-inning
- AWY / HOM = team abbreviations
- Score displayed as AWY - HOM

Multiple commands can be entered separated by commas:

playball
regular
cancel

(As new scoring commands are added, you will be able to chain them the same way.)

---

## 1. Engine Control Commands

These commands control the scoring engine itself.

| Command          | Description                         |
|------------------|-------------------------------------|
| `exit` or `quit` | Exit engine and return to Main Menu |

---

## 2. Game Start Command

When you enter the Play Ball engine and **there are no previous events** for the selected game, you must start the game
with:

| Command    | Description                                                                |
|------------|----------------------------------------------------------------------------|
| `playball` | Starts the game and logs the first at-bat (Away lineup, batting order #1). |

### What `playball` does

- Appends a `game_started` event to `game_events`
- Appends an `at_bat_started` event to `game_events`
- Writes (and persists) a log line like:

At bat, for <TEAM ABBRV>, #<JERSEY_NUMBER> FirstName LastName

> Note: `playball` is only allowed when the game has **no previous events**.

---

## 3. Game Status Commands

These commands can be entered at any time during the game.

They update the game status in the database and exit the engine.

| Command   | New Status      | Description                    |
|-----------|-----------------|--------------------------------|
| `regular` | Regulation Game | Ends game as a regulation game |
| `post`    | Postponed Game  | Marks game as postponed        |
| `cancel`  | Cancelled Game  | Marks game as cancelled        |
| `susp`    | Suspended Game  | Suspends the game              |
| `forf`    | Forfeited Game  | Marks game as forfeited        |
| `protest` | Protested Game  | Marks game as protested        |

---

## 4. Command Format Rules

- Commands are case-insensitive
- Commands are separated by commas
- Spaces are ignored
- Unknown commands generate an error but do not stop execution

Examples:

PLAYBALL
playball
PlayBall

All valid.

---

## 5. Status Transition Rules

Game status transitions:

- Pregame → In Progress (when Play Ball starts)
- In Progress → Regulation
- In Progress → Postponed
- In Progress → Cancelled
- In Progress → Suspended
- In Progress → Forfeited
- In Progress → Protested

Status commands automatically exit the engine.

---

## 6. Current Scope (v0.6.1)

At the moment, the engine supports:

- Engine control: `exit`, `quit`
- Game start: `playball`
- Status commands: `regular`, `post`, `cancel`, `susp`, `forf`, `protest`

Scoring commands (pitching/batting/running/defense) will be documented here as they are implemented.
