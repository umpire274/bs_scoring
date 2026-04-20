use crate::cli::screens::game::GameInfo;
use crate::db::umpire::UmpireEvaluation;
use crate::models::umpires::UmpireEvaluationExportRow;
use crate::utils::normalize::slugify_filename_component;
use crate::utils::time::export_timestamp;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// Builds export rows for one umpire.
pub fn build_umpire_export_rows(
    evals: &[UmpireEvaluation],
    game_map: &HashMap<i64, GameInfo>,
) -> Vec<UmpireEvaluationExportRow> {
    evals
        .iter()
        .map(|ev| {
            let game = game_map.get(&ev.game_id);

            let (matchup, date, time, venue) = if let Some(g) = game {
                (
                    format!("{} @ {}", g.away_team, g.home_team),
                    g.game_date.clone(),
                    g.game_time.clone(),
                    g.venue.clone(),
                )
            } else {
                (
                    "-".to_string(),
                    "-".to_string(),
                    Some("-".to_string()),
                    "-".to_string(),
                )
            };

            UmpireEvaluationExportRow {
                matchup,
                game_date: date,
                game_time: time.unwrap_or("-".to_string()),
                venue,

                position_evaluated: ev.position_evaluated.clone(),
                strike_zone_accuracy: ev.strike_zone_accuracy,
                safe_out_accuracy: ev.safe_out_accuracy,
                positioning: ev.positioning,
                timing: ev.timing,
                game_management: ev.game_management,
                professionalism: ev.professionalism,
                communication: ev.communication,
                hustle: ev.hustle,
                overall_score: ev.overall_score,

                strengths: ev.strengths.clone(),
                areas_to_improve: ev.areas_to_improve.clone(),
                notes: ev.notes.clone(),
            }
        })
        .collect()
}

/// Writes umpire reports to a JSON file in the selected directory.
pub(crate) fn export_umpire_reports_json(
    rows: &[UmpireEvaluationExportRow],
    umpire_name: &str,
    output_dir: &std::path::Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let filename = format!(
        "{}-{}.json",
        slugify_filename_component(umpire_name),
        export_timestamp()
    );

    let path = output_dir.join(filename);
    let file = File::create(&path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, rows)?;

    Ok(path)
}

/// Writes umpire reports to a CSV file in the selected directory.
pub(crate) fn export_umpire_reports_csv(
    rows: &[UmpireEvaluationExportRow],
    umpire_name: &str,
    output_dir: &std::path::Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let filename = format!(
        "{}-{}.csv",
        slugify_filename_component(umpire_name),
        export_timestamp()
    );

    let path = output_dir.join(filename);
    let mut writer = csv::Writer::from_path(&path)?;

    for row in rows {
        writer.serialize(row)?;
    }

    writer.flush()?;

    Ok(path)
}
