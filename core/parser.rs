use crate::models::types::*;
use std::error::Error;
use crate::models::types;

pub struct CommandParser;

impl CommandParser {
    /// Parse a scoring command from user input
    /// Examples:
    /// "1B" -> Single
    /// "K" -> Strikeout swinging
    /// "Kꓘ" or "KL" -> Strikeout looking
    /// "6-3" -> Groundout shortstop to first
    /// "F8" -> Flyout to center
    /// "HR" -> Home run
    /// "BB" -> Base on balls
    /// "HBP" -> Hit by pitch
    /// "E6" -> Error by shortstop
    /// "SB2" -> Stolen base to second
    /// "CS3" -> Caught stealing at third
    /// "6-4-3 DP" -> Double play
    /// "SF8" -> Sacrifice fly to center
    pub fn parse_command(input: &str) -> Result<PlateAppearanceResult, Box<dyn Error>> {
        let input = input.trim().to_uppercase();

        // Hit types
        if input == "1B" || input == "SINGLE" {
            return Ok(PlateAppearanceResult::Hit {
                hit_type: HitType::Single,
                location: None,
                rbis: 0,
            });
        }

        if input == "2B" || input == "DOUBLE" {
            return Ok(PlateAppearanceResult::Hit {
                hit_type: HitType::Double,
                location: None,
                rbis: 0,
            });
        }

        if input == "3B" || input == "TRIPLE" {
            return Ok(PlateAppearanceResult::Hit {
                hit_type: HitType::Triple,
                location: None,
                rbis: 0,
            });
        }

        if input == "HR" || input == "HOMERUN" {
            return Ok(PlateAppearanceResult::Hit {
                hit_type: HitType::HomeRun,
                location: None,
                rbis: 0,
            });
        }

        if input == "GRD" {
            return Ok(PlateAppearanceResult::Hit {
                hit_type: HitType::GroundRule,
                location: None,
                rbis: 0,
            });
        }

        // Strikeouts
        if input == "K" {
            return Ok(PlateAppearanceResult::Out {
                out_type: OutType::Strikeout {
                    swinging: true,
                    looking: false,
                },
                rbi: false,
            });
        }

        if input == "KL" || input == "K-L" || input.contains("ꓚ") {
            return Ok(PlateAppearanceResult::Out {
                out_type: OutType::Strikeout {
                    swinging: false,
                    looking: true,
                },
                rbi: false,
            });
        }

        // Walks
        if input == "BB" {
            return Ok(PlateAppearanceResult::Walk(Walk::BaseOnBalls));
        }

        if input == "IBB" {
            return Ok(PlateAppearanceResult::Walk(Walk::IntentionalWalk));
        }

        if input == "HBP" {
            return Ok(PlateAppearanceResult::Walk(Walk::HitByPitch));
        }

        // Flyouts (F + number)
        if input.starts_with('F') && input.len() >= 2 {
            let pos_str = &input[1..];
            if let Ok(pos_num) = pos_str.parse::<u8>() {
                if let Some(position) = Position::from_number(pos_num) {
                    return Ok(PlateAppearanceResult::Out {
                        out_type: OutType::Flyout {
                            positions: vec![position],
                        },
                        rbi: false,
                    });
                }
            }
        }

        // Lineouts (L + number)
        if input.starts_with('L') && input.len() >= 2 {
            let pos_str = &input[1..];
            if let Ok(pos_num) = pos_str.parse::<u8>() {
                if let Some(position) = Position::from_number(pos_num) {
                    return Ok(PlateAppearanceResult::Out {
                        out_type: OutType::Lineout {
                            positions: vec![position],
                        },
                        rbi: false,
                    });
                }
            }
        }

        // Popouts (P + number)
        if input.starts_with('P') && input.len() >= 2 {
            let pos_str = &input[1..];
            if let Ok(pos_num) = pos_str.parse::<u8>() {
                if let Some(position) = Position::from_number(pos_num) {
                    return Ok(PlateAppearanceResult::Out {
                        out_type: OutType::Popup {
                            positions: vec![position],
                        },
                        rbi: false,
                    });
                }
            }
        }

        // Groundouts and plays (6-3, 4-3, etc.)
        if input.contains('-') {
            let parts: Vec<&str> = input.split('-').collect();
            let mut positions = Vec::new();

            for part in parts {
                let part = part.trim();
                // Check if this part contains DP or TP
                let pos_str = part.split_whitespace().next().unwrap_or(part);
                
                if let Ok(pos_num) = pos_str.parse::<u8>() {
                    if let Some(position) = Position::from_number(pos_num) {
                        positions.push(position);
                    }
                }
            }

            if !positions.is_empty() {
                // Check for double play
                if input.contains("DP") {
                    return Ok(PlateAppearanceResult::Out {
                        out_type: OutType::DoublePlay { positions },
                        rbi: false,
                    });
                }

                // Check for triple play
                if input.contains("TP") {
                    return Ok(PlateAppearanceResult::Out {
                        out_type: OutType::TriplePlay { positions },
                        rbi: false,
                    });
                }

                // Regular groundout
                return Ok(PlateAppearanceResult::Out {
                    out_type: OutType::Groundout { positions },
                    rbi: false,
                });
            }
        }

        // Sacrifice fly (SF + number)
        if input.starts_with("SF") && input.len() >= 3 {
            let pos_str = &input[2..];
            if let Ok(pos_num) = pos_str.parse::<u8>() {
                if let Some(position) = Position::from_number(pos_num) {
                    return Ok(PlateAppearanceResult::Out {
                        out_type: OutType::Flyout {
                            positions: vec![position],
                        },
                        rbi: true,
                    });
                }
            }
        }

        // Errors (E + number)
        if input.starts_with('E') && input.len() >= 2 {
            let pos_str = &input[1..];
            if let Ok(pos_num) = pos_str.parse::<u8>() {
                if let Some(position) = Position::from_number(pos_num) {
                    return Ok(PlateAppearanceResult::Error {
                        error: types::Error {
                            position,
                            description: input.clone(),
                        },
                        reached_base: Base::First,
                    });
                }
            }
        }

        // Fielder's choice
        if input == "FC" {
            return Ok(PlateAppearanceResult::FieldersChoice {
                positions: vec![],
                out_at: None,
            });
        }

        if let Ok(advanced) = Self::parse_advanced_play(&input) {
            return Ok(PlateAppearanceResult::AdvancedPlay(advanced));
        }

        Err(format!("Comando non riconosciuto: {}", input).into())
    }

    /// Parse advanced plays like stolen bases, wild pitches, etc.
    pub fn parse_advanced_play(input: &str) -> Result<AdvancedPlay, Box<dyn Error>> {
        let input = input.trim().to_uppercase();

        if input.starts_with("SB") {
            // SB2 = stolen base to second, SB3 = to third, SBH = to home
            let base = match input.chars().nth(2) {
                Some('2') => Base::Second,
                Some('3') => Base::Third,
                Some('H') => Base::Home,
                _ => return Err("Base non valida per stolen base".into()),
            };

            let from = match base {
                Base::Second => Base::First,
                Base::Third => Base::Second,
                Base::Home => Base::Third,
                _ => return Err("Base non valida".into()),
            };

            return Ok(AdvancedPlay::StolenBase { from, to: base });
        }

        if input == "BK" || input == "BALK" {
            return Ok(AdvancedPlay::Balk);
        }

        if input == "WP" {
            return Ok(AdvancedPlay::WildPitch);
        }

        if input == "PB" {
            return Ok(AdvancedPlay::PassedBall);
        }

        if input == "SH" || input == "SAC" {
            return Ok(AdvancedPlay::SacrificeHit);
        }

        Err(format!("Advanced play non riconosciuto: {}", input).into())
    }

    /// Parse pitch sequence
    /// Example: "BCCFBFX" = Ball, Called Strike, Called Strike, Foul, Ball, Foul, In Play
    #[allow(dead_code)]
    pub fn parse_pitch_sequence(input: &str) -> Result<Vec<Pitch>, Box<dyn Error>> {
        let mut pitches = Vec::new();

        for c in input.trim().to_uppercase().chars() {
            let pitch = match c {
                'B' => Pitch::Ball,
                'C' => Pitch::CalledStrike,
                'S' => Pitch::SwingingStrike,
                'F' => Pitch::Foul,
                'L' => Pitch::FoulBunt,
                'X' => Pitch::InPlay,
                'H' => Pitch::HitByPitch,
                _ => return Err(format!("Pitch symbol non valido: {}", c).into()),
            };
            pitches.push(pitch);
        }

        Ok(pitches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hits() {
        assert!(matches!(
            CommandParser::parse_command("1B"),
            Ok(PlateAppearanceResult::Hit { .. })
        ));
        assert!(matches!(
            CommandParser::parse_command("HR"),
            Ok(PlateAppearanceResult::Hit { .. })
        ));
    }

    #[test]
    fn test_parse_strikeouts() {
        assert!(matches!(
            CommandParser::parse_command("K"),
            Ok(PlateAppearanceResult::Out { .. })
        ));
        assert!(matches!(
            CommandParser::parse_command("KL"),
            Ok(PlateAppearanceResult::Out { .. })
        ));
    }

    #[test]
    fn test_parse_groundouts() {
        assert!(matches!(
            CommandParser::parse_command("6-3"),
            Ok(PlateAppearanceResult::Out {
                out_type: OutType::Groundout { .. },
                ..
            })
        ));
    }

    #[test]
    fn test_parse_walks() {
        assert!(matches!(
            CommandParser::parse_command("BB"),
            Ok(PlateAppearanceResult::Walk(Walk::BaseOnBalls))
        ));
    }
}
