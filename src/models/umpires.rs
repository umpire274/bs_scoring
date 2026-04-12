use serde::Serialize;

/// Represents one exported umpire evaluation row.
#[derive(Debug, Clone, Serialize)]
pub struct UmpireEvaluationExportRow {
    pub matchup: String,
    pub game_date: String,
    pub game_time: String,
    pub venue: String,
    pub position_evaluated: String,
    pub strike_zone_accuracy: Option<i32>,
    pub safe_out_accuracy: Option<i32>,
    pub positioning: Option<i32>,
    pub timing: Option<i32>,
    pub game_management: Option<i32>,
    pub professionalism: Option<i32>,
    pub communication: Option<i32>,
    pub hustle: Option<i32>,
    pub overall_score: Option<i32>,
    pub strengths: Option<String>,
    pub areas_to_improve: Option<String>,
    pub notes: Option<String>,
}
