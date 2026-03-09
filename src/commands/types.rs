use crate::Pitch;
use crate::models::types::GameStatus;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Exit,
    SetStatus(GameStatus),
    PlayBall,
    Pitch(Pitch),

    Single,
    Double,
    Triple,
    HomeRun,

    Unknown(String),
}
