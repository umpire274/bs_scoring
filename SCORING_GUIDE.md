# ⚾ BS Scoring — Scoring Command Guide (v0.11.0-alpha2)

This document describes every command the **Play Ball** engine accepts as
of v0.11.0-alpha2, the grammar rules that govern them, and the diagnostic
messages the parser produces when an input is not valid.

The command grammar changed substantially in v0.11.0-alpha2. If you are
coming from v0.10.x, read section 1 and section 2 first — they summarize
the new model.

---

## 1) Prompt and line format

The engine prompt is:

> ↑ 1 (0 OUTS) AWY 0 - 0 HOM >

Where:

- `↑` / `↓` — top / bottom of the inning
- number — current inning
- `OUTS` — outs in the current half-inning
- `AWY` / `HOM` — team abbreviations
- score shown as `AWY – HOM`

One input line can carry multiple **segments**, separated by commas:

```
b, b, f, k
5 h, 3 sc
k, 6 st 2b
5 l6, 3 64, 4 43
```

Everything in the line is case-insensitive. Commas are the only segment
separators; whitespace is ignored inside a segment.

If the line has no errors, the engine applies every segment in the order
that makes physical sense — defensive plays first, then pitches and
steals in the order typed, then the hit (if any). If the line has one or
more errors, **nothing is applied** and every error is reported at once.

---

## 2) The subject rule

An **action segment** describes something a specific batter or runner did.
Every action segment carries a **subject** — the batting-order slot
(`1`–`9`) of the player the segment is about.

> **The subject is mandatory on every action segment, with one exception:
> verbs whose shape cannot be confused with a lone digit may omit the
> subject, in which case it defaults to the current batter.**

The exception applies to:

- hit verbs (`h`, `2h`, `3h`, `hr`)
- multi-character batter-out verbs (fielding sequences like `63` / `6-3`,
  fly `f8`, foul-fly `ff3`, line-out `l6`, infield-fly `if4` / `iff4`)
- fielder's choice (`o6`) — the destination base is always required

It does **not** apply to:

- unassisted out by a single fielder (`5`) — always needs the subject,
  so you write `5 3` to mean "batter #5 unassisted by the first baseman"
- steals (`st`) — a steal is always about a runner, never the batter
- runner-advance overrides (`2b`, `sc`, …) — always need the subject

Pitches (`b`, `k`, `s`, `f`, `fl`) and control / status keywords
(`playball`, `regular`, `exit`, …) **never** take a subject — if you
write one you get an error.

Using an explicit subject on a batter-only verb is perfectly valid — it
just becomes a consistency check: the subject must equal the current
batter, otherwise the parser rejects the segment.

---

## 3) Engine control

| Command          | Description                         |
|------------------|-------------------------------------|
| `exit` or `quit` | Exit engine and return to Main Menu |

These are single-segment lines. Mixing `exit` with anything else in the
same line is an error.

---

## 4) Game start

| Command    | Description                                                                |
|------------|----------------------------------------------------------------------------|
| `playball` | Starts the game and loads the first at-bat (away lineup, batting order #1) |

`playball` is only allowed when the game has no previous events.

---

## 5) Pitching commands

Recorded for the **current plate appearance** (current batter vs current
pitcher). Never take a subject.

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
- After either outcome, the count resets and the engine advances to the
  next batter.

Pitches can be combined with steals on the same line — see section 7.2.

---

## 6) Hit commands

### 6.1 Basic syntax

| Command  | Meaning                               |
|----------|---------------------------------------|
| `h`      | Single (1B) — implicit current batter |
| `<n> h`  | Single by batter in slot `<n>`        |
| `<n> 2h` | Double                                |
| `<n> 3h` | Triple                                |
| `<n> hr` | Home run                              |

An optional field zone can follow the hit command:

```
5 h lf       → batter #5 singles to left field
5 2h rc      → batter #5 doubles to right-center
5 hr cf      → batter #5 home runs to center field
```

### 6.2 Field zones

| Code  | Area                       |
|-------|----------------------------|
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

Zone codes are case-insensitive. A zone is only ever the object of a hit
verb — it cannot appear on its own.

### 6.3 Runner overrides after a hit

After a hit, runners move by default according to the standard
advancement rules. If the default is not what actually happened on the
field, one or more runner-override segments can be added to the same
line:

| Destination             | Meaning          |
|-------------------------|------------------|
| `1b`                    | Goes to 1st base |
| `2b`                    | Goes to 2nd base |
| `3b`                    | Goes to 3rd base |
| `sc` / `score` / `home` | Runner scores    |

Examples:

```
5 h                     — batter #5 singles, runners advance by defaults
5 h lf, 3 sc            — batter #5 singles to LF; runner #3 scores
6 h, 5 2b, 3 sc         — batter #6 singles; runner #5 stays on 2B;
                          runner #3 scores
5 2h, 2 sc              — batter #5 doubles; runner #2 scores
5 hr                    — home run; overrides are unnecessary
```

Override segments can appear **in any order** relative to the hit — the
parser does not care whether the hit comes first or last. The following
three lines are equivalent:

```
5 h, 3 sc, 2 2b
3 sc, 5 h, 2 2b
2 2b, 3 sc, 5 h
```

---

## 7) Steal commands

### 7.1 Basic syntax

```
<n> st <base>
```

Examples:

```
5 st 2b       — runner #5 steals second
3 st 3b       — runner #3 steals third
7 st sc       — runner #7 steals home (scores 1 run)
```

Rules:

- The subject is **always** required.
- The destination base is **always** required.
- The runner must actually be on base, otherwise the segment is rejected.
- `st sc` scores 1 run.
- Does not affect the pitch count.

### 7.2 Steals alongside a pitch

A pitch and one or more steals can coexist on the same line — this is
how you record "the runner went on the pitch":

```
b, 5 st 2b              — ball; runner #5 steals second
k, 6 st 2b              — called strike; runner #6 steals second
b, 5 st 2b, 3 st 3b     — double steal on a ball
```

Steals cannot appear alongside a hit, an out, or a fielder's choice —
those events settle the play themselves. If you need that, the runner's
movement belongs in the hit's override list (section 6.3) or becomes an
implicit part of the defensive play.

---

## 8) Out commands

### 8.1 Batter-only outs

Multi-character shapes — subject optional (defaults to current batter):

| Command | Meaning                           |
|---------|-----------------------------------|
| `63`    | Ground out, 6-3 (SS→1B)           |
| `6-3`   | Ground out, 6-3 (hyphenated form) |
| `862`   | Ground out, 8-6-2                 |
| `8-6-2` | Ground out, 8-6-2                 |
| `f8`    | Fly out to CF                     |
| `ff3`   | Foul fly out to 1B                |
| `l6`    | Line out to SS                    |
| `if4`   | Infield fly, 2B                   |
| `iff4`  | Infield fly, 2B (legacy spelling) |

Single-digit shape — subject always required:

| Command | Meaning                        |
|---------|--------------------------------|
| `5 3`   | Batter #5 unassisted out by 1B |
| `5 5`   | Batter #5 unassisted out by 3B |

Adding an explicit subject to a multi-character shape is valid but must
match the current batter:

```
5 63                    — batter #5 ground out 6-3 (subject = current)
```

### 8.2 Runner-targeted outs

When a runner is retired on a play, write the segment with the runner's
batting-order subject first:

```
3 64                    — runner #3 out 6-4
4 43                    — runner #4 out 4-3
```

### 8.3 Composite defensive plays

Several outs on the same play are written as independent comma-separated
segments in any order:

```
5 l6, 3 64, 4 43        — triple play: batter #5 line out to SS;
                          runner #3 out 6-4; runner #4 out 4-3
3 64, 5 l6, 4 43        — same triple play, different segment order
4 43, 5 l6, 3 64        — same triple play, different segment order
```

The parser accepts up to 3 outs on a single play; a fourth triggers an
error.

### 8.4 Fielder's choice

The batter reaches safely because the defense chose to retire a runner.
The destination base is mandatory:

```
5 o6 1b                 — batter #5 safe at 1B on FC by SS
5 o4 2b                 — batter #5 safe at 2B on FC by 2B
o6 1b                   — implicit subject = current batter
```

An FC segment can be combined with runner-out segments:

```
4 46, 5 o4 1b           — runner #4 out 4-6, batter #5 safe at 1B on FC
5 o4 1b, 4 46           — same play, segments reordered
```

Combining an FC with a hit, with a batter-out on the same batter, or
with a runner-advance override is not supported and produces an error —
see section 11.

### 8.5 Infield-fly rule

An infield-fly call (`if4`, `iff4`, …) is only valid when:

- there are fewer than 2 outs, and
- there are runners on both 1B and 2B.

If either precondition fails the parser rejects the segment with
`infield-fly rule requires < 2 outs and runners on 1B and 2B`.

---

## 9) Runner-advance overrides

The standalone form `<n> <base>` records that runner `<n>` ended the
play on `<base>` — useful when the default advancement logic cannot
infer it from the triggering play.

An advance segment is only valid when it is part of a line that contains
either a hit (section 6) or a fielder's choice (section 8.4). A lone
advance with no triggering play produces an error.

Examples:

```
5 h, 3 sc               — runner #3 is pushed home by the single
5 2h, 3 sc, 2 3b        — double drives in #3 and moves #2 to 3B
```

---

## 10) Game status commands

| Command   | Description           |
|-----------|-----------------------|
| `regular` | End game (regulation) |
| `post`    | Postponed             |
| `cancel`  | Cancelled             |
| `susp`    | Suspended             |
| `forf`    | Forfeit               |
| `protest` | Protest               |

These are single-segment lines. Like control keywords, they never take a
subject and cannot be mixed with any other segment.

---

## 11) Diagnostic errors

Every syntactic and semantic problem is reported as:

```
error at segment <N>: '<text>': <reason>
```

where `<N>` is the 1-based position of the offending segment in the
line. When a line contains multiple errors, **every** error is reported
at once — the parser never stops at the first.

The tables below list the reasons the engine can produce, each paired
with a minimal input that triggers it and the exact diagnostic the
parser emits. Use these as a reference both when writing correct
commands and when teaching someone else the grammar.

### 11.1 Syntactic errors

These come from the grammar layer, before any semantic check.

| Input          | Error                                                                                       |
|----------------|---------------------------------------------------------------------------------------------|
| `5 h, , 3 sc`  | `error at segment 2: '': empty segment`                                                     |
| `st 2b`        | `error at segment 1: 'st 2b': verb 'st' requires a batting-order subject (1–9)`             |
| `2b`           | `error at segment 1: '2b': verb '2b' requires a batting-order subject (1–9)`                |
| `5 b`          | `error at segment 1: '5 b': verb 'b' does not accept a batting-order subject`               |
| `5 playball`   | `error at segment 1: '5 playball': verb 'playball' does not accept a batting-order subject` |
| `xyz`          | `error at segment 1: 'xyz': unknown verb 'xyz'`                                             |
| `5 xyz`        | `error at segment 1: '5 xyz': unknown verb 'xyz'`                                           |
| `5`            | `error at segment 1: '5': unknown verb '5'`                                                 |
| `o6`           | `error at segment 1: 'o6': verb 'o6' requires a destination base (1B / 2B / 3B / SC)`       |
| `5 o6`         | `error at segment 1: '5 o6': verb 'o6' requires a destination base (1B / 2B / 3B / SC)`     |
| `5 st`         | `error at segment 1: '5 st': verb 'st' requires a destination base (1B / 2B / 3B / SC)`     |
| `5 st xyz`     | `error at segment 1: '5 st xyz': invalid base 'xyz'`                                        |
| `5 h xyz`      | `error at segment 1: '5 h xyz': invalid field zone 'xyz'`                                   |
| `5 h lf extra` | `error at segment 1: '5 h lf extra': verb '5' does not accept extra tokens: 'extra'`        |
| `exit now`     | `error at segment 1: 'exit now': verb 'exit' does not accept extra tokens: 'now'`           |

### 11.2 Semantic errors

These come from the validator, which cross-checks the segment against
the current `GameState`.

Assume a game where the current batter is `#5` and runner `#3` is on 1B.

| Input                     | Error                                                                                                 |
|---------------------------|-------------------------------------------------------------------------------------------------------|
| `8 h`                     | `error at segment 1: '8 h': batter slot #8 does not match current batter #5`                          |
| `8 63`                    | `error at segment 1: '8 63': batter slot #8 does not match current batter #5`                         |
| `5 2b`                    | `error at segment 1: '5 2b': runner advance #5 has no triggering play (hit or FC) in this line`       |
| `h, 7 2b`                 | `error at segment 2: '7 2b': runner #7 is not on base`                                                |
| `7 st 2b`                 | `error at segment 1: '7 st 2b': runner #7 is not on base`                                             |
| `5 h, 5 2b`               | `error at segment 2: '5 2b': batting slot #5 appears both as batter and as a runner override`         |
| `5 if4` with 2 outs       | `error at segment 1: '5 if4': infield-fly rule requires < 2 outs and runners on 1B and 2B`            |
| `5 if4` with no one on 2B | `error at segment 1: '5 if4': infield-fly rule requires < 2 outs and runners on 1B and 2B`            |
| `5 l6, 3 64, 4 43, 2 32`  | `error at segment 1: '5 l6': play would record 4 outs (maximum is 3)`                                 |
| `5 h, exit`               | `error at segment 2: 'exit': 'exit' is a control command and cannot be combined with action segments` |
| `b, 5 h`                  | `error at segment 1: 'b': 'b' is a control command and cannot be combined with action segments`       |
| `5 h, 5 o6 1b`            | `error at segment 2: '5 o6 1b': 'FC cannot be combined with a hit' is a control command …`            |
| `5 h, 5 63`               | `error at segment 2: '5 63': 'an out cannot be combined with a hit' is a control command …`           |
| `5 o6 1b, 5 63`           | `error at segment 2: '5 63': 'batter cannot be both out and safe on FC' is a control command …`       |
| `5 o6 1b, 3 3b`           | `error at segment 2: '3 3b': 'runner advance alongside FC is not supported in this version' …`        |

> The "is a control command…" wording in the last five rows is a single
> catch-all diagnostic used for structural conflicts; the quoted prefix
> identifies the specific conflict.

### 11.3 Accumulated errors

When a line has more than one problem, every problem is reported. For
example, with current batter #5 and no runner on base:

Input:

```
8 h, 4 2b, xyz
```

Output:

```
error at segment 1: '8 h': batter slot #8 does not match current batter #5
error at segment 3: 'xyz': unknown verb 'xyz'
```

(Segment 2 is syntactically valid and is not flagged during the
syntactic pass; once the syntactic pass is clean, the validator adds a
semantic error for it too — `runner advance has no triggering play`.)

---

## 12) Examples at a glance

### A full inning

```
playball                        — start game, #1 up for AWY
k, b, k                         — first batter strikes out swinging
b, b                            — second batter walks after 4 pitches…
b, b
5 h                             — batter #5 singles (runner #2 advances
                                  to 2B by default advancement)
5 st 2b                         — INVALID: #5 is now the batter's slot
                                  — correct example below
```

(Once `5` singles, the batter's slot moves to `#6`. A subsequent steal
by the runner who just reached 1B is written with that runner's slot:
`6 st 2b` makes no sense either if #6 is now at the plate — real-world
timing depends on the engine state.)

### A forced double play

For a forced double play `4-6-3` (ball to 2B, throw to SS for the force
on the runner, relay to 1B for the out on the batter), the input is a
composite defensive play with a subject per out:

```
5 463, 3 46                     — batter #5 ground out 4-6-3; runner
                                  #3 out 4-6 on the force at 2B
```

### A triple play

```
5 l6, 3 64, 4 43                — line out to SS; runner #3 doubled off
                                  6-4; runner #4 thrown out 4-3
```

### A fielder's choice

```
4 46, 5 o4 1b                   — runner #4 out 4-6; batter #5 safe
                                  at 1B on fielder's choice by 4B
```

### A steal on a pitch

```
b, 5 st 2b                      — ball pitched; runner #5 steals 2B
b, 5 st 2b, 3 st 3b             — double steal on a ball
```

---

## 13) Resume / replay behavior

- The game is reconstructed from the `plate_appearances` table.
- Runner movements are replayed from the `runner_movements` table.
- The scoreboard is restored deterministically.
- New commands typed after a résumé obey the same grammar rules as a
  fresh game; the subject rule applies from the first input.

---

## 14) Current scope (v0.11.0-alpha2)

Supported:

- pitches
- hits with optional zone and any number of runner overrides
- steals, including double steal on a pitch
- batter outs (unassisted, ground, fly, foul fly, line, infield fly)
- fielder's choice with mandatory destination base
- composite defensive plays up to 3 outs
- game control (`exit`, `playball`) and status change (`regular` …)

Implemented end-to-end:

- deterministic resume from the database
- scoreboard with total and inning-by-inning scores
- runner identity tracking by batting order

The broader grammar and the diagnostic pipeline introduced in this
alpha are the foundation for the remaining v0.11.0 work. Refer to
`CHANGELOG.md` for the running release notes.
