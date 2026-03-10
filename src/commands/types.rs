use crate::Pitch;
use crate::models::field_zone::FieldZone;
use crate::models::types::GameStatus;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Exit,
    SetStatus(GameStatus),
    PlayBall,
    Pitch(Pitch),

    Single { zone: Option<FieldZone> },
    Double { zone: Option<FieldZone> },
    Triple { zone: Option<FieldZone> },
    HomeRun { zone: Option<FieldZone> },

    Unknown(String),
}
