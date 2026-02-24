use crate::commands::types::EngineCommand;
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

fn parse_one(cmd: &str) -> EngineCommand {
    match cmd.to_ascii_lowercase().as_str() {
        "exit" | "quit" => EngineCommand::Exit,

        "regular" => EngineCommand::SetStatus(GameStatus::Regulation),
        "post" => EngineCommand::SetStatus(GameStatus::Postponed),
        "cancel" => EngineCommand::SetStatus(GameStatus::Cancelled),
        "susp" => EngineCommand::SetStatus(GameStatus::Suspended),
        "forf" => EngineCommand::SetStatus(GameStatus::Forfeited),
        "protest" => EngineCommand::SetStatus(GameStatus::Protested),

        "out" => EngineCommand::Out,

        _ => EngineCommand::Unknown(cmd.to_string()),
    }
}
