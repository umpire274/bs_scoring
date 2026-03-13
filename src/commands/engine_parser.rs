use crate::Pitch;
use crate::commands::types::EngineCommand;
use crate::models::field_zone::FieldZone;
use crate::models::game_state::BatterOrder;
use crate::models::runner::{RunnerDest, RunnerOverride};
use crate::models::types::GameStatus;

/// Parse a raw input line into a list of engine commands.
///
/// Input format (comma-separated tokens):
///
/// ```text
/// h                    → single, automatic runner advancement
/// h lf                 → single to left field
/// 6 h                  → batter #6 hits single  (batting-order prefix)
/// 6 h, 5 2b            → batter #6 single; runner #5 stays on 2B
/// 6 h lf, 5 2b, 3 sc   → batter #6 single LF; runner #5 → 2B; runner #3 scores
/// ```
///
/// Token grammar for hits:
/// ```text
/// [<order>] <hit_cmd> [<zone>]          → batter token (first token in PA)
/// <order> <dest>                         → runner-override token (subsequent tokens)
/// ```
pub fn parse_engine_commands(line: &str) -> Vec<EngineCommand> {
    let tokens: Vec<&str> = line
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if tokens.is_empty() {
        return vec![];
    }

    // Try to parse the first token as a hit command (possibly with batting-order prefix).
    // If it matches, gather runner overrides from the remaining tokens.
    if let Some((hit_cmd_no_overrides, batter_order)) = parse_batter_token(tokens[0]) {
        // All subsequent tokens must be valid runner overrides.
        // If any token fails to parse, reject the whole command — silently
        // dropping an invalid token (e.g. "6 h, 5 xx") would cause incorrect scoring.
        let mut runner_overrides: Vec<RunnerOverride> = Vec::new();
        for token in &tokens[1..] {
            match parse_runner_override_token(token) {
                Some(ro) => runner_overrides.push(ro),
                None => return vec![EngineCommand::Unknown(line.to_string())],
            }
        }

        // Rebuild the hit command with overrides.
        let hit_cmd = attach_overrides(hit_cmd_no_overrides, runner_overrides);

        // Validate: the batter's own batting-order must not appear in runner overrides.
        // (The batter is placed automatically on their target base; they can't also be
        //  a runner override in the same PA.)
        if let Some(order) = batter_order {
            let hit_cmd = match &hit_cmd {
                EngineCommand::Single {
                    runner_overrides, ..
                }
                | EngineCommand::Double {
                    runner_overrides, ..
                }
                | EngineCommand::Triple {
                    runner_overrides, ..
                }
                | EngineCommand::HomeRun {
                    runner_overrides, ..
                } => {
                    if runner_overrides.iter().any(|r| r.order == order) {
                        return vec![EngineCommand::Unknown(line.to_string())];
                    }
                    hit_cmd
                }
                _ => hit_cmd,
            };
            return vec![hit_cmd];
        }

        return vec![hit_cmd];
    }

    // Not a hit command — parse each token independently as a non-hit command.
    tokens.iter().map(|t| parse_non_hit_command(t)).collect()
}

// ─── Batter token ─────────────────────────────────────────────────────────────

/// Try to parse a token as a batter hit command.
///
/// Accepted forms:
/// - `h`            → single
/// - `h lf`         → single with zone
/// - `6 h`          → single, batter order = 6
/// - `6 h lf`       → single to LF, batter order = 6
///
/// Returns `(command_without_overrides, Option<batter_order>)` or `None`.
fn parse_batter_token(raw: &str) -> Option<(EngineCommand, Option<BatterOrder>)> {
    let parts: Vec<&str> = raw.split_whitespace().collect();

    // Extract optional leading batting-order number
    let (order, rest): (Option<BatterOrder>, &[&str]) = {
        if let Some(&first) = parts.first() {
            if let Ok(n) = first.parse::<u8>() {
                if (1..=9).contains(&n) && parts.len() >= 2 {
                    (Some(n), &parts[1..])
                } else {
                    (None, &parts[..])
                }
            } else {
                (None, &parts[..])
            }
        } else {
            return None;
        }
    };

    if rest.is_empty() {
        return None;
    }

    let cmd_str = rest[0].to_ascii_lowercase();
    let zone_str = rest.get(1).copied();

    // Zone is optional; if present it must be a valid FieldZone
    let zone: Option<FieldZone> = match zone_str {
        Some(z) => match FieldZone::parse(z) {
            Some(fz) => Some(fz),
            None => return None, // unrecognised zone → not a valid batter token
        },
        None => None,
    };

    // More than 2 tokens after order? Not a valid batter token.
    if rest.len() > 2 {
        return None;
    }

    let cmd = match cmd_str.as_str() {
        "h" => EngineCommand::Single {
            zone,
            runner_overrides: vec![],
        },
        "2h" => EngineCommand::Double {
            zone,
            runner_overrides: vec![],
        },
        "3h" => EngineCommand::Triple {
            zone,
            runner_overrides: vec![],
        },
        "hr" => EngineCommand::HomeRun {
            zone,
            runner_overrides: vec![],
        },
        _ => return None,
    };

    Some((cmd, order))
}

// ─── Runner-override token ─────────────────────────────────────────────────────

/// Parse a runner-override token: `<batting_order> <dest>`.
///
/// Examples: `"5 2b"`, `"3 sc"`, `"7 home"`, `"2 3b"`
fn parse_runner_override_token(raw: &str) -> Option<RunnerOverride> {
    // Accepted formats: "7 sc", "7sc", "5 2b", "52b"
    // Split on whitespace first; if only one token, try to split the leading digits
    // from the trailing destination string.
    let parts: Vec<&str> = raw.split_whitespace().collect();

    let (order_str, dest_str): (&str, &str) = match parts.as_slice() {
        [order, dest] => (order, dest),
        [compact] => {
            // Compact format: first character is the batting order (1-9),
            // the rest is the destination string (e.g. "7sc" → "7" + "sc", "52b" → "5" + "2b")
            if compact.is_empty() {
                return None;
            }
            (&compact[..1], &compact[1..])
        }
        _ => return None,
    };

    let order: u8 = order_str.parse().ok()?;
    if !(1..=9).contains(&order) {
        return None;
    }
    let dest = RunnerDest::parse(dest_str)?;
    Some(RunnerOverride { order, dest })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn attach_overrides(cmd: EngineCommand, overrides: Vec<RunnerOverride>) -> EngineCommand {
    match cmd {
        EngineCommand::Single { zone, .. } => EngineCommand::Single {
            zone,
            runner_overrides: overrides,
        },
        EngineCommand::Double { zone, .. } => EngineCommand::Double {
            zone,
            runner_overrides: overrides,
        },
        EngineCommand::Triple { zone, .. } => EngineCommand::Triple {
            zone,
            runner_overrides: overrides,
        },
        EngineCommand::HomeRun { zone, .. } => EngineCommand::HomeRun {
            zone,
            runner_overrides: overrides,
        },
        other => other,
    }
}

// ─── Non-hit commands ─────────────────────────────────────────────────────────

fn parse_non_hit_command(raw: &str) -> EngineCommand {
    let mut parts = raw.split_whitespace();
    let Some(cmd) = parts.next() else {
        return EngineCommand::Unknown(raw.to_string());
    };
    let arg = parts.next();

    // Reject unexpected extra tokens
    if parts.next().is_some() {
        return EngineCommand::Unknown(raw.to_string());
    }

    match cmd.to_ascii_lowercase().as_str() {
        "exit" | "quit" => EngineCommand::Exit,

        "regular" => EngineCommand::SetStatus(GameStatus::Regulation),
        "post" => EngineCommand::SetStatus(GameStatus::Postponed),
        "cancel" => EngineCommand::SetStatus(GameStatus::Cancelled),
        "susp" => EngineCommand::SetStatus(GameStatus::Suspended),
        "forf" => EngineCommand::SetStatus(GameStatus::Forfeited),
        "protest" => EngineCommand::SetStatus(GameStatus::Protested),

        "playball" => EngineCommand::PlayBall,

        // Pitch commands
        "b" => EngineCommand::Pitch(Pitch::Ball),
        "k" => EngineCommand::Pitch(Pitch::CalledStrike),
        "s" => EngineCommand::Pitch(Pitch::SwingingStrike),
        "f" => EngineCommand::Pitch(Pitch::Foul),
        "fl" => EngineCommand::Pitch(Pitch::FoulBunt),

        // Hit commands without batting-order prefix (no runner overrides possible
        // when going through the non-hit path, since we only reach here if
        // parse_batter_token returned None for the first token)
        "h" => {
            let zone = parse_zone_arg(arg, raw);
            match zone {
                Ok(z) => EngineCommand::Single {
                    zone: z,
                    runner_overrides: vec![],
                },
                Err(e) => e,
            }
        }
        "2h" => {
            let zone = parse_zone_arg(arg, raw);
            match zone {
                Ok(z) => EngineCommand::Double {
                    zone: z,
                    runner_overrides: vec![],
                },
                Err(e) => e,
            }
        }
        "3h" => {
            let zone = parse_zone_arg(arg, raw);
            match zone {
                Ok(z) => EngineCommand::Triple {
                    zone: z,
                    runner_overrides: vec![],
                },
                Err(e) => e,
            }
        }
        "hr" => {
            let zone = parse_zone_arg(arg, raw);
            match zone {
                Ok(z) => EngineCommand::HomeRun {
                    zone: z,
                    runner_overrides: vec![],
                },
                Err(e) => e,
            }
        }

        _ => EngineCommand::Unknown(raw.to_string()),
    }
}

fn parse_zone_arg(arg: Option<&str>, raw: &str) -> Result<Option<FieldZone>, EngineCommand> {
    match arg {
        Some(z) => match FieldZone::parse(z) {
            Some(zone) => Ok(Some(zone)),
            None => Err(EngineCommand::Unknown(raw.to_string())),
        },
        None => Ok(None),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn single(cmds: Vec<EngineCommand>) -> EngineCommand {
        assert_eq!(cmds.len(), 1);
        cmds.into_iter().next().unwrap()
    }

    #[test]
    fn test_bare_single() {
        let cmd = single(parse_engine_commands("h"));
        assert!(
            matches!(cmd, EngineCommand::Single { zone: None, runner_overrides } if runner_overrides.is_empty())
        );
    }

    #[test]
    fn test_single_with_zone() {
        let cmd = single(parse_engine_commands("h lf"));
        assert!(
            matches!(cmd, EngineCommand::Single { zone: Some(_), runner_overrides } if runner_overrides.is_empty())
        );
    }

    #[test]
    fn test_order_prefix_single() {
        let cmd = single(parse_engine_commands("6 h"));
        assert!(
            matches!(cmd, EngineCommand::Single { zone: None, runner_overrides } if runner_overrides.is_empty())
        );
    }

    #[test]
    fn test_single_with_runner_override() {
        let cmd = single(parse_engine_commands("6 h, 5 2b"));
        match cmd {
            EngineCommand::Single {
                runner_overrides, ..
            } => {
                assert_eq!(runner_overrides.len(), 1);
                assert_eq!(runner_overrides[0].order, 5);
                assert_eq!(runner_overrides[0].dest, RunnerDest::Second);
            }
            _ => panic!("expected Single"),
        }
    }

    #[test]
    fn test_single_runner_scores() {
        let cmd = single(parse_engine_commands("6 h, 3 sc"));
        match cmd {
            EngineCommand::Single {
                runner_overrides, ..
            } => {
                assert_eq!(runner_overrides[0].dest, RunnerDest::Score);
            }
            _ => panic!("expected Single"),
        }
    }

    #[test]
    fn test_multiple_overrides() {
        let cmd = single(parse_engine_commands("6 h, 5 2b, 3 sc"));
        match cmd {
            EngineCommand::Single {
                runner_overrides, ..
            } => {
                assert_eq!(runner_overrides.len(), 2);
            }
            _ => panic!("expected Single"),
        }
    }

    #[test]
    fn test_double_with_override() {
        let cmd = single(parse_engine_commands("4 2h, 2 sc"));
        match cmd {
            EngineCommand::Double {
                runner_overrides, ..
            } => {
                assert_eq!(runner_overrides[0].order, 2);
                assert_eq!(runner_overrides[0].dest, RunnerDest::Score);
            }
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_full_bases_three_overrides() {
        // Basi piene: r1=order 3, r2=order 5, r3=order 7; batter=order 6
        // 6 h, 7 sc, 5 sc, 3 3b  → r7 e r5 segnano, r3 va in 3a, #6 in 1a
        let cmd = single(parse_engine_commands("6 h, 7 sc, 5 sc, 3 3b"));
        match cmd {
            EngineCommand::Single {
                runner_overrides, ..
            } => {
                assert_eq!(
                    runner_overrides.len(),
                    3,
                    "expected 3 overrides, got {}",
                    runner_overrides.len()
                );
                assert!(
                    runner_overrides
                        .iter()
                        .any(|r| r.order == 7 && r.dest == RunnerDest::Score)
                );
                assert!(
                    runner_overrides
                        .iter()
                        .any(|r| r.order == 5 && r.dest == RunnerDest::Score)
                );
                assert!(
                    runner_overrides
                        .iter()
                        .any(|r| r.order == 3 && r.dest == RunnerDest::Third)
                );
            }
            _ => panic!("expected Single"),
        }
    }

    #[test]
    fn test_pitch_command_passthrough() {
        let cmds = parse_engine_commands("b");
        assert!(matches!(single(cmds), EngineCommand::Pitch(Pitch::Ball)));
    }

    #[test]
    fn test_compact_format_no_space() {
        // "9 h, 8 2b, 7sc, 6sc" — caso reale con basi piene
        let cmd = single(parse_engine_commands("9 h, 8 2b, 7sc, 6sc"));
        match cmd {
            EngineCommand::Single {
                runner_overrides, ..
            } => {
                assert_eq!(runner_overrides.len(), 3);
                assert!(
                    runner_overrides
                        .iter()
                        .any(|r| r.order == 8 && r.dest == RunnerDest::Second)
                );
                assert!(
                    runner_overrides
                        .iter()
                        .any(|r| r.order == 7 && r.dest == RunnerDest::Score)
                );
                assert!(
                    runner_overrides
                        .iter()
                        .any(|r| r.order == 6 && r.dest == RunnerDest::Score)
                );
            }
            _ => panic!("expected Single"),
        }
    }

    #[test]
    fn test_compact_format_variants() {
        assert_eq!(
            parse_runner_override_token("7sc"),
            Some(RunnerOverride {
                order: 7,
                dest: RunnerDest::Score
            })
        );
        assert_eq!(
            parse_runner_override_token("7 sc"),
            Some(RunnerOverride {
                order: 7,
                dest: RunnerDest::Score
            })
        );
        assert_eq!(
            parse_runner_override_token("52b"),
            Some(RunnerOverride {
                order: 5,
                dest: RunnerDest::Second
            })
        );
        assert_eq!(
            parse_runner_override_token("5 2b"),
            Some(RunnerOverride {
                order: 5,
                dest: RunnerDest::Second
            })
        );
        assert_eq!(
            parse_runner_override_token("33b"),
            Some(RunnerOverride {
                order: 3,
                dest: RunnerDest::Third
            })
        );
        assert_eq!(
            parse_runner_override_token("3 3b"),
            Some(RunnerOverride {
                order: 3,
                dest: RunnerDest::Third
            })
        );
    }

    #[test]
    fn test_runner_dest_parse() {
        assert_eq!(RunnerDest::parse("2b"), Some(RunnerDest::Second));
        assert_eq!(RunnerDest::parse("sc"), Some(RunnerDest::Score));
        assert_eq!(RunnerDest::parse("score"), Some(RunnerDest::Score));
        assert_eq!(RunnerDest::parse("home"), Some(RunnerDest::Score));
        assert_eq!(RunnerDest::parse("3b"), Some(RunnerDest::Third));
        assert_eq!(RunnerDest::parse("xyz"), None);
    }

    #[test]
    fn test_invalid_override_token_rejected() {
        // "5 xx" is not a valid runner override — whole command must become Unknown
        let cmds = parse_engine_commands("6 h, 5 xx");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EngineCommand::Unknown(_)));
    }

    #[test]
    fn test_invalid_override_typo_rejected() {
        // "h, b" — "b" looks like a ball command but is not a valid runner override
        let cmds = parse_engine_commands("h, b");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EngineCommand::Unknown(_)));
    }
}
