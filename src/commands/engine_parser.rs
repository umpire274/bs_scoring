use crate::Pitch;
use crate::commands::types::EngineCommand;
use crate::models::field_zone::FieldZone;
use crate::models::types::GameStatus;

/// Parse a raw input line into a list of engine commands.
///
/// - Commands are comma-separated.
/// - Commands are case-insensitive.
pub fn parse_engine_commands(line: &str) -> Vec<EngineCommand> {
    line.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(parse_one)
        .collect()
}

fn parse_one(raw: &str) -> EngineCommand {
    let mut parts = raw.split_whitespace();
    let Some(cmd) = parts.next() else {
        return EngineCommand::Unknown(raw.to_string());
    };
    let arg = parts.next();
    let extra = parts.next();

    if extra.is_some() {
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

        // ---- Pitch commands (0.6.7) ----
        "b" => EngineCommand::Pitch(Pitch::Ball),
        "k" => EngineCommand::Pitch(Pitch::CalledStrike),
        "s" => EngineCommand::Pitch(Pitch::SwingingStrike),
        "f" => EngineCommand::Pitch(Pitch::Foul),
        "fl" => EngineCommand::Pitch(Pitch::FoulBunt),

        // ---- Hit commands (0.7.4) ----
        "h" => {
            let zone = match arg {
                Some(z) => match FieldZone::parse(z) {
                    Some(zone) => Some(zone),
                    None => return EngineCommand::Unknown(raw.to_string()),
                },
                None => None,
            };
            EngineCommand::Single { zone }
        }
        "2h" => {
            let zone = match arg {
                Some(z) => match FieldZone::parse(z) {
                    Some(zone) => Some(zone),
                    None => return EngineCommand::Unknown(raw.to_string()),
                },
                None => None,
            };
            EngineCommand::Double { zone }
        }
        "3h" => {
            let zone = match arg {
                Some(z) => match FieldZone::parse(z) {
                    Some(zone) => Some(zone),
                    None => return EngineCommand::Unknown(raw.to_string()),
                },
                None => None,
            };
            EngineCommand::Triple { zone }
        }
        "hr" => {
            let zone = match arg {
                Some(z) => match FieldZone::parse(z) {
                    Some(zone) => Some(zone),
                    None => return EngineCommand::Unknown(raw.to_string()),
                },
                None => None,
            };
            EngineCommand::HomeRun { zone }
        }

        _ => EngineCommand::Unknown(raw.to_string()),
    }
}
