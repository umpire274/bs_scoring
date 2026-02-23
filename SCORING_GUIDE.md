# ⚾ BS Scoring – Command Guide

This document describes all commands available in the Play Ball engine.

The engine prompt format is:

↑ 1 (0 OUTS) AWY 0 - 0 HOM >

Where:

- ↑ / ↓ = Top / Bottom of the inning
- Number = current inning
- OUTS = number of outs in the half-inning
- AWY / HOM = team abbreviations
- Score displayed as AWY - HOM

Multiple commands can be entered separated by commas:

out,out
1b,sb2
bb,wp

---

# 🔹 1. Engine Control Commands

These commands control the scoring engine itself.

| Command          | Description                         |
|------------------|-------------------------------------|
| `exit` or `quit` | Exit engine and return to Main Menu |

---

# 🔹 2. Game Status Commands

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

# 🔹 3. Engine Philosophy

The Play Ball engine is designed to:

- Be fast to use
- Accept compact baseball notation
- Allow multiple commands per line
- Update game state dynamically

Example:

↑ 3 (1 OUTS) MOD 2 - 1 BOL > out,out

Result:

- 3rd out recorded
- Half-inning changes automatically

---

# 🔹 4. Command Format Rules

- Commands are case-insensitive
- Commands are separated by commas
- Spaces are ignored
- Unknown commands generate an error but do not stop execution

Example:

OUT , out , Out

is valid.

---

# 🔹 5. Status Transition Rules

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
