use crate::models::types::GameStatus;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Exit,
    SetStatus(GameStatus),
    /// Temporary/debug command used to validate inning/outs prompt dynamics.
    Out,
    Unknown(String),
}
