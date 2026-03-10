use rusqlite::{Connection, Result, params};

use crate::models::plate_appearance::{HitOutcomeData, PlateAppearance};

#[derive(Debug, Clone)]
pub struct PlateAppearanceRow {
    pub id: i64,
    pub game_id: i64,
    pub seq: i64,
    pub inning: i64,
    pub half_inning: String,
    pub batter_id: i64,
    pub pitcher_id: i64,
    pub pitches: i64,
    pub pitches_sequence: String,
    pub outcome_type: String,
    pub outcome_data: Option<String>,
    pub outs: i64,
}

fn serialize_hit_outcome_data(zone: &Option<crate::models::field_zone::FieldZone>) -> String {
    serde_json::to_string(&HitOutcomeData { zone: *zone })
        .unwrap_or_else(|_| r#"{"zone":null}"#.to_string())
}

pub fn append_plate_appearance(
    conn: &Connection,
    game_pk: i64,
    pa: &PlateAppearance,
) -> Result<()> {
    let (outcome_type, outcome_data) = match &pa.outcome {
        crate::models::plate_appearance::PlateAppearanceOutcome::Walk => ("walk".to_string(), None),
        crate::models::plate_appearance::PlateAppearanceOutcome::Out => ("out".to_string(), None),
        crate::models::plate_appearance::PlateAppearanceOutcome::Strikeout(kind) => (
            "strikeout".to_string(),
            Some(serde_json::to_string(kind).unwrap_or_else(|_| "null".to_string())),
        ),
        crate::models::plate_appearance::PlateAppearanceOutcome::Single { zone } => {
            ("single".to_string(), Some(serialize_hit_outcome_data(zone)))
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Double { zone } => {
            ("double".to_string(), Some(serialize_hit_outcome_data(zone)))
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::Triple { zone } => {
            ("triple".to_string(), Some(serialize_hit_outcome_data(zone)))
        }

        crate::models::plate_appearance::PlateAppearanceOutcome::HomeRun { zone } => (
            "home_run".to_string(),
            Some(serialize_hit_outcome_data(zone)),
        ),
    };

    // per-game sequence
    let seq: i64 = conn.query_row(
        "SELECT COALESCE(MAX(seq), 0) + 1 FROM plate_appearances_compact WHERE game_id = ?1",
        params![game_pk],
        |r| r.get(0),
    )?;

    let pitches_sequence =
        serde_json::to_string(&pa.pitches_sequence).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        r#"
        INSERT INTO plate_appearances_compact (
            game_id, seq, inning, half_inning,
            batter_id, pitcher_id,
            pitches,
            pitches_sequence,
            outcome_type, outcome_data,
            outs
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        params![
            game_pk,
            seq,
            pa.inning as i64,
            match pa.half {
                crate::models::types::HalfInning::Top => "Top",
                crate::models::types::HalfInning::Bottom => "Bottom",
            },
            pa.batter_id,
            pa.pitcher_id,
            pa.pitches as i64,
            pitches_sequence,
            outcome_type,
            outcome_data,
            pa.outs as i64,
        ],
    )?;

    Ok(())
}

pub fn list_plate_appearances(conn: &Connection, game_pk: i64) -> Result<Vec<PlateAppearanceRow>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, game_id, seq, inning, half_inning,
               batter_id, pitcher_id, pitches, pitches_sequence,
               outcome_type, outcome_data,
               outs
        FROM plate_appearances_compact
        WHERE game_id = ?1
        ORDER BY seq ASC
        "#,
    )?;

    let mut rows = stmt.query(params![game_pk])?;
    let mut out = Vec::new();
    while let Some(r) = rows.next()? {
        out.push(PlateAppearanceRow {
            id: r.get(0)?,
            game_id: r.get(1)?,
            seq: r.get(2)?,
            inning: r.get(3)?,
            half_inning: r.get(4)?,
            batter_id: r.get(5)?,
            pitcher_id: r.get(6)?,
            pitches: r.get(7)?,
            pitches_sequence: r.get(8)?,
            outcome_type: r.get(9)?,
            outcome_data: r.get(10)?,
            outs: r.get(11)?,
        });
    }
    Ok(out)
}
