//! Umpire persistence layer — CRUD for umpires, game assignments, and evaluations.

use rusqlite::{Connection, Result, params};

// ─── Umpire registry ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Umpire {
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub license_number: Option<String>,
    pub level: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
}

impl Umpire {
    pub fn new(first_name: String, last_name: String) -> Self {
        Self {
            id: None,
            first_name,
            last_name,
            license_number: None,
            level: None,
            email: None,
            phone: None,
            notes: None,
            is_active: true,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Umpire {
            id: Some(row.get(0)?),
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            license_number: row.get(3)?,
            level: row.get(4)?,
            email: row.get(5)?,
            phone: row.get(6)?,
            notes: row.get(7)?,
            is_active: row.get(8)?,
        })
    }

    pub fn create(&mut self, conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT INTO umpires (first_name, last_name, license_number, level, email, phone, notes, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                self.first_name,
                self.last_name,
                self.license_number,
                self.level,
                self.email,
                self.phone,
                self.notes,
                self.is_active,
            ],
        )?;
        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Umpire> {
        let mut stmt = conn.prepare(
            "SELECT id, first_name, last_name, license_number, level, email, phone, notes, is_active
             FROM umpires WHERE id = ?1",
        )?;
        stmt.query_row(params![id], Self::from_row)
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Umpire>> {
        let mut stmt = conn.prepare(
            "SELECT id, first_name, last_name, license_number, level, email, phone, notes, is_active
             FROM umpires
             ORDER BY last_name, first_name",
        )?;
        let rows = stmt.query_map([], Self::from_row)?;
        rows.collect()
    }

    pub fn get_active(conn: &Connection) -> Result<Vec<Umpire>> {
        let mut stmt = conn.prepare(
            "SELECT id, first_name, last_name, license_number, level, email, phone, notes, is_active
             FROM umpires
             WHERE is_active = 1
             ORDER BY last_name, first_name",
        )?;
        let rows = stmt.query_map([], Self::from_row)?;
        rows.collect()
    }

    pub fn update(&self, conn: &Connection) -> Result<()> {
        if let Some(id) = self.id {
            conn.execute(
                "UPDATE umpires SET first_name = ?1, last_name = ?2, license_number = ?3,
                 level = ?4, email = ?5, phone = ?6, notes = ?7, is_active = ?8
                 WHERE id = ?9",
                params![
                    self.first_name,
                    self.last_name,
                    self.license_number,
                    self.level,
                    self.email,
                    self.phone,
                    self.notes,
                    self.is_active,
                    id,
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            "DELETE FROM umpire_evaluations WHERE umpire_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM game_umpires WHERE umpire_id = ?1", params![id])?;
        conn.execute(
            "DELETE FROM umpire_leagues WHERE umpire_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM umpires WHERE id = ?1", params![id])?;
        Ok(())
    }
}

// ─── Umpire ↔ League association (N:N) ────────────────────────────────────────

/// Associate an umpire with a league. Ignores duplicates (OR IGNORE).
pub fn add_umpire_league(conn: &Connection, umpire_id: i64, league_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO umpire_leagues (umpire_id, league_id) VALUES (?1, ?2)",
        params![umpire_id, league_id],
    )?;
    Ok(())
}

/// Remove an umpire from a league.
pub fn remove_umpire_league(conn: &Connection, umpire_id: i64, league_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM umpire_leagues WHERE umpire_id = ?1 AND league_id = ?2",
        params![umpire_id, league_id],
    )?;
    Ok(())
}

/// Replace all league associations for an umpire.
pub fn set_umpire_leagues(conn: &Connection, umpire_id: i64, league_ids: &[i64]) -> Result<()> {
    conn.execute(
        "DELETE FROM umpire_leagues WHERE umpire_id = ?1",
        params![umpire_id],
    )?;
    for &lid in league_ids {
        conn.execute(
            "INSERT INTO umpire_leagues (umpire_id, league_id) VALUES (?1, ?2)",
            params![umpire_id, lid],
        )?;
    }
    Ok(())
}

/// Get all leagues associated with an umpire. Returns (league_id, league_name).
pub fn get_umpire_leagues(conn: &Connection, umpire_id: i64) -> Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT l.id, l.name
         FROM umpire_leagues ul
         JOIN leagues l ON ul.league_id = l.id
         WHERE ul.umpire_id = ?1
         ORDER BY l.name",
    )?;
    let rows = stmt.query_map(params![umpire_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

// ─── Umpire position in a game ────────────────────────────────────────────────

/// Valid umpire positions within a crew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UmpirePosition {
    HomePlate,
    FirstBase,
    SecondBase,
    ThirdBase,
    LeftField,
    RightField,
}

impl UmpirePosition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HomePlate => "HP",
            Self::FirstBase => "1B",
            Self::SecondBase => "2B",
            Self::ThirdBase => "3B",
            Self::LeftField => "LF",
            Self::RightField => "RF",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::HomePlate => "Home Plate",
            Self::FirstBase => "1st Base",
            Self::SecondBase => "2nd Base",
            Self::ThirdBase => "3rd Base",
            Self::LeftField => "Left Field",
            Self::RightField => "Right Field",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "HP" => Some(Self::HomePlate),
            "1B" => Some(Self::FirstBase),
            "2B" => Some(Self::SecondBase),
            "3B" => Some(Self::ThirdBase),
            "LF" => Some(Self::LeftField),
            "RF" => Some(Self::RightField),
            _ => None,
        }
    }

    /// Returns the positions for a given crew size.
    pub fn crew(size: u8) -> Vec<Self> {
        match size {
            2 => vec![Self::HomePlate, Self::FirstBase],
            3 => vec![Self::HomePlate, Self::FirstBase, Self::ThirdBase],
            4 => vec![
                Self::HomePlate,
                Self::FirstBase,
                Self::SecondBase,
                Self::ThirdBase,
            ],
            6 => vec![
                Self::HomePlate,
                Self::FirstBase,
                Self::SecondBase,
                Self::ThirdBase,
                Self::LeftField,
                Self::RightField,
            ],
            _ => vec![
                Self::HomePlate,
                Self::FirstBase,
                Self::SecondBase,
                Self::ThirdBase,
            ],
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::HomePlate,
            Self::FirstBase,
            Self::SecondBase,
            Self::ThirdBase,
            Self::LeftField,
            Self::RightField,
        ]
    }
}

impl std::fmt::Display for UmpirePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.label(), self.as_str())
    }
}

// ─── Game ↔ Umpire assignment ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GameUmpireAssignment {
    pub id: i64,
    pub game_id: i64,
    pub umpire_id: i64,
    pub position: String,
    /// Populated by JOINed queries.
    pub umpire_name: Option<String>,
}

/// Assign an umpire to a game position.
pub fn assign_umpire(
    conn: &Connection,
    game_id: i64,
    umpire_id: i64,
    position: UmpirePosition,
) -> Result<i64> {
    conn.execute(
        "INSERT OR REPLACE INTO game_umpires (game_id, umpire_id, position)
         VALUES (?1, ?2, ?3)",
        params![game_id, umpire_id, position.as_str()],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Remove an umpire assignment from a game.
pub fn unassign_umpire(conn: &Connection, game_id: i64, position: UmpirePosition) -> Result<()> {
    conn.execute(
        "DELETE FROM game_umpires WHERE game_id = ?1 AND position = ?2",
        params![game_id, position.as_str()],
    )?;
    Ok(())
}

/// List all umpire assignments for a game, with umpire names.
pub fn list_game_umpires(conn: &Connection, game_id: i64) -> Result<Vec<GameUmpireAssignment>> {
    let mut stmt = conn.prepare(
        "SELECT gu.id, gu.game_id, gu.umpire_id, gu.position,
                u.first_name || ' ' || u.last_name as umpire_name
         FROM game_umpires gu
         JOIN umpires u ON gu.umpire_id = u.id
         WHERE gu.game_id = ?1
         ORDER BY CASE gu.position
             WHEN 'HP' THEN 1
             WHEN '1B' THEN 2
             WHEN '2B' THEN 3
             WHEN '3B' THEN 4
             WHEN 'LF' THEN 5
             WHEN 'RF' THEN 6
         END",
    )?;

    let rows = stmt.query_map(params![game_id], |r| {
        Ok(GameUmpireAssignment {
            id: r.get(0)?,
            game_id: r.get(1)?,
            umpire_id: r.get(2)?,
            position: r.get(3)?,
            umpire_name: r.get(4)?,
        })
    })?;

    rows.collect()
}

// ─── Umpire evaluation (report card) ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UmpireEvaluation {
    pub id: Option<i64>,
    pub game_id: i64,
    pub umpire_id: i64,
    pub evaluator_name: Option<String>,
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

impl UmpireEvaluation {
    pub fn new(game_id: i64, umpire_id: i64, position: UmpirePosition) -> Self {
        Self {
            id: None,
            game_id,
            umpire_id,
            evaluator_name: None,
            position_evaluated: position.as_str().to_string(),
            strike_zone_accuracy: None,
            safe_out_accuracy: None,
            positioning: None,
            timing: None,
            game_management: None,
            professionalism: None,
            communication: None,
            hustle: None,
            overall_score: None,
            strengths: None,
            areas_to_improve: None,
            notes: None,
        }
    }

    /// Calculate average of all non-null numeric scores.
    pub fn calculated_average(&self) -> Option<f64> {
        let scores: Vec<i32> = [
            self.strike_zone_accuracy,
            self.safe_out_accuracy,
            self.positioning,
            self.timing,
            self.game_management,
            self.professionalism,
            self.communication,
            self.hustle,
        ]
        .iter()
        .filter_map(|s| *s)
        .collect();

        if scores.is_empty() {
            return None;
        }
        let sum: i32 = scores.iter().sum();
        Some(sum as f64 / scores.len() as f64)
    }

    pub fn save(&mut self, conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT OR REPLACE INTO umpire_evaluations (
                game_id, umpire_id, evaluator_name, position_evaluated,
                strike_zone_accuracy, safe_out_accuracy, positioning, timing,
                game_management, professionalism, communication, hustle,
                overall_score, strengths, areas_to_improve, notes
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                self.game_id,
                self.umpire_id,
                self.evaluator_name,
                self.position_evaluated,
                self.strike_zone_accuracy,
                self.safe_out_accuracy,
                self.positioning,
                self.timing,
                self.game_management,
                self.professionalism,
                self.communication,
                self.hustle,
                self.overall_score,
                self.strengths,
                self.areas_to_improve,
                self.notes,
            ],
        )?;
        let id = conn.last_insert_rowid();
        self.id = Some(id);
        Ok(id)
    }

    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        Ok(Self {
            id: Some(row.get(0)?),
            game_id: row.get(1)?,
            umpire_id: row.get(2)?,
            evaluator_name: row.get(3)?,
            position_evaluated: row.get(4)?,
            strike_zone_accuracy: row.get(5)?,
            safe_out_accuracy: row.get(6)?,
            positioning: row.get(7)?,
            timing: row.get(8)?,
            game_management: row.get(9)?,
            professionalism: row.get(10)?,
            communication: row.get(11)?,
            hustle: row.get(12)?,
            overall_score: row.get(13)?,
            strengths: row.get(14)?,
            areas_to_improve: row.get(15)?,
            notes: row.get(16)?,
        })
    }

    /// Load all evaluations for a game.
    pub fn list_by_game(conn: &Connection, game_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, umpire_id, evaluator_name, position_evaluated,
                    strike_zone_accuracy, safe_out_accuracy, positioning, timing,
                    game_management, professionalism, communication, hustle,
                    overall_score, strengths, areas_to_improve, notes
             FROM umpire_evaluations
             WHERE game_id = ?1
             ORDER BY CASE position_evaluated
                 WHEN 'HP' THEN 1 WHEN '1B' THEN 2 WHEN '2B' THEN 3
                 WHEN '3B' THEN 4 WHEN 'LF' THEN 5 WHEN 'RF' THEN 6
             END",
        )?;
        let rows = stmt.query_map(params![game_id], Self::from_row)?;
        rows.collect()
    }

    /// Load all evaluations for an umpire (career history).
    pub fn list_by_umpire(conn: &Connection, umpire_id: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, umpire_id, evaluator_name, position_evaluated,
                    strike_zone_accuracy, safe_out_accuracy, positioning, timing,
                    game_management, professionalism, communication, hustle,
                    overall_score, strengths, areas_to_improve, notes
             FROM umpire_evaluations
             WHERE umpire_id = ?1
             ORDER BY evaluated_at DESC",
        )?;
        let rows = stmt.query_map(params![umpire_id], Self::from_row)?;
        rows.collect()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::database::Database;

    #[test]
    fn test_umpire_crud() {
        let db = Database::new(":memory:").unwrap();
        db.init_schema().unwrap();
        let conn = db.get_connection();

        let mut ump = Umpire::new("Mario".to_string(), "Rossi".to_string());
        ump.license_number = Some("IT-001".to_string());
        ump.level = Some("AA".to_string());
        let id = ump.create(conn).unwrap();
        assert!(id > 0);

        let retrieved = Umpire::get_by_id(conn, id).unwrap();
        assert_eq!(retrieved.full_name(), "Mario Rossi");
        assert_eq!(retrieved.license_number.as_deref(), Some("IT-001"));

        let all = Umpire::get_all(conn).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_crew_sizes() {
        assert_eq!(UmpirePosition::crew(2).len(), 2);
        assert_eq!(UmpirePosition::crew(3).len(), 3);
        assert_eq!(UmpirePosition::crew(4).len(), 4);
        assert_eq!(UmpirePosition::crew(6).len(), 6);
    }

    #[test]
    fn test_evaluation_average() {
        let mut eval = UmpireEvaluation::new(1, 1, UmpirePosition::HomePlate);
        assert!(eval.calculated_average().is_none());

        eval.strike_zone_accuracy = Some(8);
        eval.positioning = Some(6);
        let avg = eval.calculated_average().unwrap();
        assert!((avg - 7.0).abs() < 0.01);
    }
}
