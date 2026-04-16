# ⚾ BS Scoring – Scoring Command Guide (v0.10.6)

This document describes all commands available in the **Play Ball** engine (v0.10.6).

---

## Prompt format

The engine prompt is:

> ↑ 1 (0 OUTS) AWY 0 - 0 HOM >

Where:

* `↑` / `↓` = Top / Bottom of the inning
* Number = current inning
* `OUTS` = outs in the current half-inning
* `AWY` / `HOM` = team abbreviations
* Score displayed as `AWY - HOM`

Multiple commands can be entered on the same line, comma-separated:

```
b, b, f, k
6 h, 5 2b
k, 6 st 2b
9 64, 1 o6 1b
```

Commands are case-insensitive.

---

## 1) Engine control

| Command          | Description                         |
| ---------------- | ----------------------------------- |
| `exit` or `quit` | Exit engine and return to Main Menu |

---

## 2) Game start

| Command    | Description                                                                 |
| ---------- | --------------------------------------------------------------------------- |
| `playball` | Starts the game and loads the first at-bat (Away lineup, batting order #1). |

`playball` is only allowed when the game has no previous events.

---

## 3) Pitching commands

Recorded for the **current plate appearance** (current batter vs current pitcher).

| Command | Meaning                 |
| ------- | ----------------------- |
| `b`     | Ball                    |
| `k`     | Called strike (looking) |
| `s`     | Swinging strike         |
| `f`     | Foul ball               |
| `fl`    | Foul bunt               |

Rules:

* `k` and `s` always increment the strike count.
* `f` increments strikes only if strikes < 2.
* `fl` increments strikes even if strikes = 2 (foul bunt can be strike 3).
* 4 balls → walk (BB); 3 strikes → strikeout (K).
* After either outcome, the count resets and the engine advances to the next batter.

---

## 4) Hit commands

### 4.1 Basic syntax

| Command | Meaning     |
| ------- | ----------- |
| `h`     | Single (1B) |
| `2h`    | Double (2B) |
| `3h`    | Triple (3B) |
| `hr`    | Home Run    |

An optional field zone can follow the hit command:

```
h lf       → single to left field
2h rc      → double to right-center
hr cf      → home run to center field
```

### 4.2 Field zones

| Code  | Area                       |
| ----- | -------------------------- |
| `ll`  | Left line (down the line)  |
| `lf`  | Left field                 |
| `lc`  | Left-center                |
| `cf`  | Center field               |
| `rc`  | Right-center               |
| `rf`  | Right field                |
| `rl`  | Right line (down the line) |
| `gll` | Ground ball — left line    |
| `ls`  | Left side infield          |
| `mi`  | Middle infield             |
| `rs`  | Right side infield         |
| `grl` | Ground ball — right line   |

Zone codes are case-insensitive.

### 4.3 Batting-order prefix (optional)

```
6 h        → batter #6 singles
4 2h lf    → batter #4 doubles to left field
```

### 4.4 Runner overrides

Syntax:

```
<hit>, <order> <destination>
```

Valid destinations:

| Destination             | Meaning          |
| ----------------------- | ---------------- |
| `1b`                    | Goes to 1st base |
| `2b`                    | Goes to 2nd base |
| `3b`                    | Goes to 3rd base |
| `sc` / `score` / `home` | Runner scores    |

Examples:

```
h
6 h, 5 2b
6 h, 5 2b, 3 sc
4 2h, 2 sc
hr
```

---

## 5) Steal commands

Syntax:

```
<order> st <destination>
```

Examples:

```
6 st 2b
3 st 3b
7 st sc
```

Rules:

* Runner must be on the correct base
* `st sc` scores 1 run
* Does not affect pitch count

---

## 6) Out commands (v0.10.6)

### 6.1 Legacy batter-out syntax

```
<order> <out_token>
```

Examples:

```
6 63
8 5
7 f8
7 ff2
7 l6
7 if4
```

### 6.2 Implicit batter syntax

```
63
5
f9
l6
if4
```

### 6.3 Fielder's choice

```
<order> o<fielder> <base>
```

Examples:

```
1 o6 1b
1 o5 2b
```

Notes:

* Base is mandatory

### 6.4 Multi-command defensive plays

Commands can be combined:

```
9 64, 1 o6 1b
8 5, 9 54, 1 o5 1b
l6, 1 64, 2 43
```

Rules:

* Must include batter result
* Cannot mix batter out and batter safe

### 6.5 Infield fly rule

Valid only when:

* fewer than 2 outs
* runner on 1B
* runner on 2B

---

## 7) Game status commands

| Command   | Description |
| --------- | ----------- |
| `regular` | End game    |
| `post`    | Postponed   |
| `cancel`  | Cancelled   |
| `susp`    | Suspended   |
| `forf`    | Forfeit     |
| `protest` | Protest     |

---

## 8) Command format rules

* Commands are case-insensitive
* Commands are comma-separated
* Unknown commands produce an error

---

## 9) Resume / replay behavior

* Game reconstructed from `plate_appearances`
* Runner movements replayed from DB
* Scoreboard restored deterministically

---

## 10) Current scope (v0.10.6)

Supported:

* Pitch commands
* Hits + overrides
* Steals
* Batter outs
* Unassisted outs
* Fly/line/infield outs
* Fielder's choice
* Defensive multi-commands

Implemented:

* Deterministic resume
* Scoreboard (totals + innings)
* Runner identity tracking

Planned next steps:

* Grammar refactor (v0.11.0)
* Regex-based parser
* Caught stealing
* Errors
* Sacrifice plays
