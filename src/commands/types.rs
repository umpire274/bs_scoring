use crate::Pitch;
use crate::models::types::GameStatus;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Exit,
    SetStatus(GameStatus),
    PlayBall,
    Pitch(Pitch),
    Unknown(String),
}
