use crate::Pitch;
use crate::commands::types::EngineCommand;
use crate::core::scoring::batter_outs::{BatterOutType, FieldingSequence};
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
/// 6 h                  → batter #6 hits single
/// 6 h, 5 2b            → batter #6 single; runner #5 stays on 2B
/// 6 h lf, 5 2b, 3 sc   → batter #6 single LF; runner #5 → 2B; runner #3 scores
///
/// 6 63                 → batter #6 ground out 6-3
/// 6 6-3                → batter #6 ground out 6-3
/// 4 862                → batter #4 ground out 8-6-2
/// 4 8-6-2              → batter #4 ground out 8-6-2
/// 7 F8                 → batter #7 fly out to CF
/// 7 FF3                → batter #7 foul fly out to 1B
/// 7 L6                 → batter #7 line out to SS
/// 7 IF4                → batter #7 infield fly to 2B
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

    // First try to parse the first token as a hit command (possibly with batting-order prefix).
    // If it matches, gather runner overrides from the remaining tokens.
    if let Some((hit_cmd_no_overrides, batter_order)) = parse_batter_token(tokens[0]) {
        let mut runner_overrides: Vec<RunnerOverride> = Vec::new();
        for token in &tokens[1..] {
            match parse_runner_override_token(token) {
                Some(ro) => runner_overrides.push(ro),
                None => return vec![EngineCommand::Unknown(line.to_string())],
            }
        }

        let hit_cmd = attach_overrides(hit_cmd_no_overrides, runner_overrides);

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

    let zone: Option<FieldZone> = match zone_str {
        Some(z) => match FieldZone::parse(z) {
            Some(fz) => Some(fz),
            None => return None,
        },
        None => None,
    };

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
    let parts: Vec<&str> = raw.split_whitespace().collect();

    let (order_str, dest_str): (&str, &str) = match parts.as_slice() {
        [order, dest] => (order, dest),
        [compact] => {
            if compact.is_empty() {
                return None;
            }
            let mut chars = compact.char_indices();
            let (_, first_char) = chars.next()?;
            if !first_char.is_ascii_digit() {
                return None;
            }
            let split_at = first_char.len_utf8();
            (&compact[..split_at], &compact[split_at..])
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

fn parse_zone_arg(arg: Option<&str>, raw: &str) -> Result<Option<FieldZone>, EngineCommand> {
    match arg {
        Some(z) => match FieldZone::parse(z) {
            Some(zone) => Ok(Some(zone)),
            None => Err(EngineCommand::Unknown(raw.to_string())),
        },
        None => Ok(None),
    }
}

fn parse_batter_out_token(raw: &str) -> Option<EngineCommand> {
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let order = parse_batter_order(parts[0])?;
    let token = parts[1];

    let out_type = parse_batter_out_type(token)?;
    Some(EngineCommand::BatterOut { order, out_type })
}

fn parse_batter_order(raw: &str) -> Option<BatterOrder> {
    let order = raw.parse::<u8>().ok()?;
    if (1..=9).contains(&order) {
        Some(order)
    } else {
        None
    }
}

fn parse_batter_out_type(token: &str) -> Option<BatterOutType> {
    let normalized = token.to_ascii_uppercase();

    if let Some(rest) = normalized.strip_prefix("IFF") {
        let fielder = parse_single_fielder(rest)?;
        return Some(BatterOutType::InfieldFly { fielder });
    }

    if let Some(rest) = normalized.strip_prefix("FF") {
        let fielder = parse_single_fielder(rest)?;
        return Some(BatterOutType::FlyOut {
            fielder,
            in_foul_territory: true,
        });
    }

    if let Some(rest) = normalized.strip_prefix('F') {
        let fielder = parse_single_fielder(rest)?;
        return Some(BatterOutType::FlyOut {
            fielder,
            in_foul_territory: false,
        });
    }

    if let Some(rest) = normalized.strip_prefix('L') {
        let fielder = parse_single_fielder(rest)?;
        return Some(BatterOutType::LineOut { fielder });
    }

    if is_fielding_sequence_token(normalized.as_str()) {
        let sequence = parse_fielding_sequence(token)?;
        return Some(BatterOutType::GroundOut { sequence });
    }

    None
}

fn parse_single_fielder(raw: &str) -> Option<u8> {
    if raw.is_empty() {
        return None;
    }

    let value = raw.parse::<u8>().ok()?;
    if (1..=9).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn is_fielding_sequence_token(token: &str) -> bool {
    !token.is_empty() && token.chars().all(|ch| ch.is_ascii_digit() || ch == '-')
}

fn parse_fielding_sequence(token: &str) -> Option<FieldingSequence> {
    let fielders = if token.contains('-') {
        parse_hyphenated_fielding_sequence(token)?
    } else {
        parse_compact_fielding_sequence(token)?
    };

    FieldingSequence::new(fielders).ok()
}

fn parse_compact_fielding_sequence(token: &str) -> Option<Vec<u8>> {
    let mut fielders = Vec::with_capacity(token.len());

    for ch in token.chars() {
        let digit = ch.to_digit(10)?;
        let value = u8::try_from(digit).ok()?;
        if !(1..=9).contains(&value) {
            return None;
        }
        fielders.push(value);
    }

    if fielders.len() < 2 {
        return None;
    }

    Some(fielders)
}

fn parse_hyphenated_fielding_sequence(token: &str) -> Option<Vec<u8>> {
    let mut fielders = Vec::new();

    for part in token.split('-') {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }

        let value = part.parse::<u8>().ok()?;
        if !(1..=9).contains(&value) {
            return None;
        }

        fielders.push(value);
    }

    if fielders.len() < 2 {
        return None;
    }

    Some(fielders)
}

// ─── Non-hit commands ─────────────────────────────────────────────────────────

fn parse_non_hit_command(raw: &str) -> EngineCommand {
    if let Some(cmd) = parse_batter_out_token(raw) {
        return cmd;
    }

    let tokens: Vec<&str> = raw.split_whitespace().collect();

    // ── Steal: `<order> st <base>` ────────────────────────────────────────────
    if tokens.len() == 3 {
        if let (Ok(order), true, Some(dest)) = (
            tokens[0].parse::<u8>(),
            tokens[1].eq_ignore_ascii_case("st"),
            RunnerDest::parse(tokens[2]),
        ) && (1..=9).contains(&order)
        {
            return EngineCommand::StealBase { order, dest };
        }
        return EngineCommand::Unknown(raw.to_string());
    }

    let mut parts = raw.split_whitespace();
    let Some(cmd) = parts.next() else {
        return EngineCommand::Unknown(raw.to_string());
    };
    let arg = parts.next();

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

        // Hit commands without batting-order prefix
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
        let cmd = single(parse_engine_commands("6 h, 7 sc, 5 sc, 3 3b"));
        match cmd {
            EngineCommand::Single {
                runner_overrides, ..
            } => {
                assert_eq!(runner_overrides.len(), 3);
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
        let cmds = parse_engine_commands("6 h, 5 xx");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EngineCommand::Unknown(_)));
    }

    #[test]
    fn test_invalid_override_typo_rejected() {
        let cmds = parse_engine_commands("h, b");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EngineCommand::Unknown(_)));
    }

    #[test]
    fn test_steal_second() {
        let cmds = parse_engine_commands("6 st 2b");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            EngineCommand::StealBase {
                order: 6,
                dest: RunnerDest::Second
            }
        ));
    }

    #[test]
    fn test_steal_third() {
        let cmds = parse_engine_commands("3 st 3b");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            EngineCommand::StealBase {
                order: 3,
                dest: RunnerDest::Third
            }
        ));
    }

    #[test]
    fn test_steal_home() {
        let cmds = parse_engine_commands("7 st sc");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(
            cmds[0],
            EngineCommand::StealBase {
                order: 7,
                dest: RunnerDest::Score
            }
        ));
    }

    #[test]
    fn test_steal_combined_with_pitch() {
        let cmds = parse_engine_commands("k, 6 st 2b");
        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], EngineCommand::Pitch(Pitch::CalledStrike)));
        assert!(matches!(
            cmds[1],
            EngineCommand::StealBase {
                order: 6,
                dest: RunnerDest::Second
            }
        ));
    }

    #[test]
    fn test_steal_invalid_dest() {
        let cmds = parse_engine_commands("6 st 1b");
        assert!(matches!(
            cmds[0],
            EngineCommand::StealBase {
                order: 6,
                dest: RunnerDest::First
            }
        ));
    }

    #[test]
    fn test_steal_bad_order() {
        let cmds = parse_engine_commands("0 st 2b");
        assert!(matches!(cmds[0], EngineCommand::Unknown(_)));
    }

    #[test]
    fn test_steal_bad_dest_token() {
        let cmds = parse_engine_commands("6 st xx");
        assert!(matches!(cmds[0], EngineCommand::Unknown(_)));
    }

    #[test]
    fn test_compact_override_unicode_no_panic() {
        let cmds = parse_engine_commands("h, é2b");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EngineCommand::Unknown(_)));
    }

    // ─── Batter out tests ─────────────────────────────────────────────────────

    #[test]
    fn test_ground_out_compact() {
        let cmd = single(parse_engine_commands("6 63"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 6);
                match out_type {
                    BatterOutType::GroundOut { sequence } => {
                        assert_eq!(sequence.fielders(), &[6, 3]);
                    }
                    _ => panic!("expected GroundOut"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_ground_out_hyphenated() {
        let cmd = single(parse_engine_commands("6 6-3"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 6);
                match out_type {
                    BatterOutType::GroundOut { sequence } => {
                        assert_eq!(sequence.fielders(), &[6, 3]);
                    }
                    _ => panic!("expected GroundOut"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_ground_out_multi_assist_compact() {
        let cmd = single(parse_engine_commands("4 862"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 4);
                match out_type {
                    BatterOutType::GroundOut { sequence } => {
                        assert_eq!(sequence.fielders(), &[8, 6, 2]);
                    }
                    _ => panic!("expected GroundOut"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_ground_out_multi_assist_hyphenated() {
        let cmd = single(parse_engine_commands("4 8-6-2"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 4);
                match out_type {
                    BatterOutType::GroundOut { sequence } => {
                        assert_eq!(sequence.fielders(), &[8, 6, 2]);
                    }
                    _ => panic!("expected GroundOut"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_fly_out_fair() {
        let cmd = single(parse_engine_commands("7 F8"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 7);
                match out_type {
                    BatterOutType::FlyOut {
                        fielder,
                        in_foul_territory,
                    } => {
                        assert_eq!(fielder, 8);
                        assert!(!in_foul_territory);
                    }
                    _ => panic!("expected FlyOut"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_fly_out_foul() {
        let cmd = single(parse_engine_commands("7 FF3"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 7);
                match out_type {
                    BatterOutType::FlyOut {
                        fielder,
                        in_foul_territory,
                    } => {
                        assert_eq!(fielder, 3);
                        assert!(in_foul_territory);
                    }
                    _ => panic!("expected FlyOut"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_line_out() {
        let cmd = single(parse_engine_commands("8 L6"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 8);
                match out_type {
                    BatterOutType::LineOut { fielder } => {
                        assert_eq!(fielder, 6);
                    }
                    _ => panic!("expected LineOut"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_infield_fly() {
        let cmd = single(parse_engine_commands("2 IFF4"));
        match cmd {
            EngineCommand::BatterOut { order, out_type } => {
                assert_eq!(order, 2);
                match out_type {
                    BatterOutType::InfieldFly { fielder } => {
                        assert_eq!(fielder, 4);
                    }
                    _ => panic!("expected InfieldFly"),
                }
            }
            _ => panic!("expected BatterOut"),
        }
    }

    #[test]
    fn test_invalid_ground_out_sequence_rejected() {
        let cmd = single(parse_engine_commands("6 6--3"));
        assert!(matches!(cmd, EngineCommand::Unknown(_)));
    }

    #[test]
    fn test_invalid_single_fielder_ground_out_rejected() {
        let cmd = single(parse_engine_commands("6 3"));
        assert!(matches!(cmd, EngineCommand::Unknown(_)));
    }

    #[test]
    fn test_invalid_batter_out_fielder_rejected() {
        let cmd = single(parse_engine_commands("7 F10"));
        assert!(matches!(cmd, EngineCommand::Unknown(_)));
    }
}
