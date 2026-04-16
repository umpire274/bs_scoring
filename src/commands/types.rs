use crate::core::scoring::BatterOutType;
use crate::core::scoring::batter_outs::DefensivePlayCommand;
use crate::models::field_zone::FieldZone;
use crate::models::runner::RunnerOverride;
use crate::models::types::GameStatus;
use crate::{BatterOrder, Pitch};

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Exit,
    SetStatus(GameStatus),
    PlayBall,
    Pitch(Pitch),

    Single {
        zone: Option<FieldZone>,
        runner_overrides: Vec<RunnerOverride>,
    },
    Double {
        zone: Option<FieldZone>,
        runner_overrides: Vec<RunnerOverride>,
    },
    Triple {
        zone: Option<FieldZone>,
        runner_overrides: Vec<RunnerOverride>,
    },
    HomeRun {
        zone: Option<FieldZone>,
        runner_overrides: Vec<RunnerOverride>,
    },

    /// Runner steals a base: `<order> st <dest>`
    /// e.g. `6 st 2b` — runner in batting slot 6 steals second.
    StealBase {
        order: u8,
        dest: crate::models::runner::RunnerDest,
    },

    BatterOut {
        order: BatterOrder,
        out_type: BatterOutType,
    },

    DefensivePlay(DefensivePlayCommand),

    Unknown(String),
}
