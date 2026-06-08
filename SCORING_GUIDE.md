# ⚾ BS Scoring — Scoring Command Guide (v0.12.0)

This document describes the command language accepted by the **Play Ball** engine.

The v0.12.0 player-model changes do not change the Play Ball scoring grammar.
Player roster positions such as `P,C,IF` are separate from lineup defensive positions and scoring commands.

---

## 1. Prompt and Line Format

The engine prompt looks like this:

```text
↑ 1 (0 OUTS) AWY 0 - 0 HOM >
```

Where:

- `↑` / `↓` — top / bottom of the inning;
- inning number — current inning;
- `OUTS` — outs in the current half-inning;
- `AWY` / `HOM` — team abbreviations;
- score shown as away-home.

One input line can contain multiple comma-separated segments:

```text
b, b, f, k
5 h, 3 sc
k, 6 st 2b
5 l6, 3 64, 4 43
```

Commands are case-insensitive.

If the line is valid, the engine applies the command. If the line has errors, nothing is applied and all errors are reported.

---

## 2. Subject Rule

Most action segments refer to a player by **batting-order slot** (`1`–`9`).
This slot is the command subject.

The subject is mandatory on runner actions and single-digit unassisted outs.

Some batter-only verbs may omit the subject and default to the current batter:

- hits: `h`, `2h`, `3h`, `hr`;
- multi-character batter outs: `63`, `6-3`, `f8`, `ff3`, `l6`, `if4`;
- fielder's choice such as `o6 1b`.

Pitches and control commands never take a subject.

---

## 3. Control Commands

| Command | Description |
|---|---|
| `playball` | Start the game and load the first at-bat |
| `exit` | Exit Play Ball and return to menu |
| `quit` | Same as `exit` |

Control commands must be used alone.

---

## 4. Pitching Commands

Pitch commands are recorded for the current plate appearance. They never take a subject.

| Command | Meaning |
|---|---|
| `b` | Ball |
| `k` | Called strike |
| `s` | Swinging strike |
| `f` | Foul ball |
| `fl` | Foul bunt |

Rules:

- `k` and `s` increment strikes.
- `f` increments strikes only if strikes are fewer than 2.
- `fl` can produce strike 3.
- 4 balls produce a walk.
- 3 strikes produce a strikeout.

Examples:

```text
b
k
b, 5 st 2b
```

Pitches can be combined with steals on the same line.

---

## 5. Hits

| Command | Meaning |
|---|---|
| `h` | Single by current batter |
| `<n> h` | Single by batting-order slot `<n>` |
| `<n> 2h` | Double |
| `<n> 3h` | Triple |
| `<n> hr` | Home run |

Examples:

```text
h
5 h
5 2h
5 hr
```

A hit may include a field zone:

```text
5 h lf
5 2h rc
5 hr cf
```

### Field Zones

| Code | Area |
|---|---|
| `ll` | Left line |
| `lf` | Left field |
| `lc` | Left-center |
| `cf` | Center field |
| `rc` | Right-center |
| `rf` | Right field |
| `rl` | Right line |
| `gll` | Ground ball left line |
| `ls` | Left side infield |
| `mi` | Middle infield |
| `rs` | Right side infield |
| `grl` | Ground ball right line |

### Runner Overrides After a Hit

Use runner overrides when a runner does not follow the default advancement.

| Destination | Meaning |
|---|---|
| `1b` | Runner goes to first |
| `2b` | Runner goes to second |
| `3b` | Runner goes to third |
| `sc`, `score`, `home` | Runner scores |

Examples:

```text
5 h lf, 3 sc
6 h, 5 2b, 3 sc
5 2h, 2 sc
```

Overrides can appear in any order relative to the hit.

---

## 6. Steals

Syntax:

```text
<n> st <base>
```

Examples:

```text
5 st 2b
3 st 3b
7 st sc
```

Rules:

- the subject is required;
- the runner must be on base;
- `st sc` scores one run;
- steals do not affect pitch count.

Steals may be combined with pitches:

```text
b, 5 st 2b
k, 6 st 2b
b, 5 st 2b, 3 st 3b
```

Steals cannot be combined with a hit, out, fielder's choice, or standalone runner advance.

---

## 7. Outs

### Batter-only Outs

Multi-character forms may omit the subject and default to the current batter.

| Command | Meaning |
|---|---|
| `63` | Ground out, 6-3 |
| `6-3` | Ground out, 6-3 |
| `862` | Ground out, 8-6-2 |
| `8-6-2` | Ground out, 8-6-2 |
| `f8` | Fly out to CF |
| `ff3` | Foul fly out to 1B |
| `l6` | Line out to SS |
| `if4` | Infield fly, 2B |
| `iff4` | Legacy spelling for infield fly |

Single-fielder unassisted outs require a subject:

```text
5 3
5 5
```

### Runner-targeted Outs

```text
3 64
4 43
```

The subject is the runner's batting-order slot.

### Composite Defensive Plays

Several outs on the same play are written as independent comma-separated segments:

```text
5 l6, 3 64, 4 43
3 64, 5 l6, 4 43
4 43, 5 l6, 3 64
```

The engine accepts up to three outs on a single play.

---

## 8. Fielder's Choice

Syntax:

```text
<n> o<fielder> <base>
```

Examples:

```text
5 o6 1b
5 o4 2b
o6 1b
```

A fielder's choice may be combined with runner-out segments:

```text
4 46, 5 o4 1b
5 o4 1b, 4 46
```

It cannot be combined with a hit, batter-out on the same batter, or runner-advance override.

---

## 9. Infield Fly

Infield-fly commands:

```text
if4
iff4
```

They are valid only when:

- fewer than two outs;
- runners on first and second.

If preconditions are not met, the command is rejected.

---

## 10. Common Examples

```text
playball
b, b, k, f
5 h lf, 3 sc
5 2h, 2 sc
6 h, 5 2b, 3 sc
5 st 2b
b, 5 st 2b
63
5 63
5 3
5 l6, 3 64, 4 43
5 o6 1b, 3 64
exit
```

---

## 11. Player Model Notes

The roster player model is separate from Play Ball command notation.

### Roster positions

Player records may contain multiple roster positions:

```text
P,C,IF
IF,OF,DH
LF,CF,RF
```

These are not scoring commands and are not used as Play Ball defensive position inputs.

### Lineup defensive positions

Lineup defensive positions still use:

```text
1 = Pitcher
2 = Catcher
3 = First Base
4 = Second Base
5 = Third Base
6 = Shortstop
7 = Left Field
8 = Center Field
9 = Right Field
DH = Designated Hitter
```

### Bat/throw notation

Player import/export uses:

```text
BAT/THROW
```

Examples:

```text
R/R
L/R
S/L
```

This notation is used for player data, not for Play Ball commands.

---

## 12. Troubleshooting

### “subject does not match current batter”

You entered a batter-only command with an explicit subject that does not match the current batter.

### “runner not on base”

You tried to steal, score, or retire a runner who is not currently on base.

### “mixing not allowed”

You combined incompatible events, such as a steal with a hit or a hit with a defensive out.

### “too many outs”

The command would produce more than three outs in one play.
