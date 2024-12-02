use std::str::FromStr;

use crate::{square::Square, Piece};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UciMove {
    pub starting_square: Square,
    pub target_square: Square,
    pub promotion: Option<Piece>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UciFen(pub(self) String);

impl UciFen {
    pub const FEN_PART_COUNT: usize= 6;
    pub const FEN_PREALLOC_SIZE: usize = 14;

    pub fn new(s: &str) -> Self {
        Self(s.to_owned())
    }

    pub fn from_cmdline<'c>(i: &mut impl core::iter::Iterator<Item = &'c str>) -> Option<Self> {
   

        let mut buf = String::from_str(i.next()?).ok()?;
        
        buf.reserve(Self::FEN_PREALLOC_SIZE);

        for _ in 0..Self::FEN_PART_COUNT-1 {
            buf.push(' ');
            buf.push_str(i.next()?);

        }

        Some(Self(buf))
    }

    pub fn inner(&self) -> String {
        self.0.clone()
    }
}



#[derive(Debug, Clone, PartialEq, Eq, ::strum_macros::EnumString, ::strum_macros::Display)]

pub enum UciCommand {
    Debug(bool),
    Perft(u32),
    Position {
        fen: Option<UciFen>,
        moves: Vec<UciMove>,
    },
    Stop,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ::strum_macros::EnumString)]
pub enum UciRawCommand {
    #[strum(ascii_case_insensitive)]
    Debug,
    #[strum(ascii_case_insensitive)]
    Perft,
    #[strum(ascii_case_insensitive)]
    Position,
    #[strum(ascii_case_insensitive)]
    Stop,
    #[strum(ascii_case_insensitive)]
    Go,
    #[strum(ascii_case_insensitive)]
    Quit,
}

impl UciRawCommand {
    pub fn parse<T: AsRef<str>>(cmdline: &mut impl Iterator<Item = T>) -> Option<Self> {
        for cmd in cmdline {
            if let Ok(parsed) = Self::from_str(cmd.as_ref()) {
                return Some(parsed);
            }
        }

        None
    }
}

impl UciCommand {
    pub fn try_parse(s: String) -> Option<UciCommand> {
        let mut parts = s.split_ascii_whitespace();

        let raw = UciRawCommand::parse(&mut parts)?;

        match raw {
            UciRawCommand::Debug => todo!(),
            UciRawCommand::Perft => Some(UciCommand::Perft(parts.next()?.parse::<u32>().ok()?)),
            UciRawCommand::Position => {
                let fen = match parts.next()? {
                    "fen" => Some(UciFen::from_cmdline(&mut parts)?),
                    "startpos" => None,
                    _ => None?,
                };

                let moves = Self::position_get_moves_helper(&mut parts);

                Some(UciCommand::Position { fen, moves })
            }
            UciRawCommand::Stop => Some(UciCommand::Stop),
            UciRawCommand::Go => todo!(),
            UciRawCommand::Quit => Some(UciCommand::Quit),
        }
    }

    pub fn position_get_moves_helper<'a>(i: &mut impl Iterator<Item = &'a str>) -> Vec<UciMove> {
        if i.next() != Some("moves") {
            return vec![];
        }

        i.filter_map(|mv| UciMove::parse(mv)).collect()
    }
}

impl UciMove {
    pub fn parse(mv: &str) -> Option<Self> {
        if mv.len() < 4 {
            None?
        }

        let start = Square::from_str(&mv[0..=1]).ok()?;
        let target = Square::from_str(&mv[2..=3]).ok()?;

        let promotion = match mv.len() {
            ..=4 => None,
            _ => Piece::from_notation(mv.chars().nth(4)?),
        };

        if promotion.map(|p| !p.is_sliding()).unwrap_or(false) {
            None? // Cannot promote to that piece.
        }

        Some(Self {
            promotion,
            starting_square: start,
            target_square: target,
        })
    }
}

#[cfg(test)]
pub(super) mod move_parser_tests {
    use super::*;

    #[test]
    pub fn simple() {
        assert_eq!(
            UciMove::parse("a2a4"),
            Some(UciMove {
                promotion: None,
                starting_square: Square::A2,
                target_square: Square::A4,
            })
        );
    }

    #[test]
    pub fn pawn_promotion() {
        assert_eq!(
            UciMove::parse("a2a4Q"),
            Some(UciMove {
                promotion: Some(crate::Piece::Queen),
                starting_square: Square::A2,
                target_square: Square::A4,
            })
        );
    }

    #[test]
    pub fn invalid_pawn_promotion() {
        assert_eq!(
            UciMove::parse("a2a4K"), // Promote to a new King...
            None
        );
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use crate::uci::{UciCommand, UciMove};

    #[test]
    pub fn basic_startpos() {
        assert_eq!(
            UciCommand::try_parse("position startpos".to_owned()),
            Some(UciCommand::Position {
                fen: None,
                moves: vec![],
            })
        )
    }

    #[test]
    pub fn startpos_kingspawn() {
        assert_eq!(
            UciCommand::try_parse("position startpos moves e2e4 e7e5".to_owned()),
            Some(UciCommand::Position {
                fen: None,
                moves: vec![
                    UciMove {
                        starting_square: Square::E2,
                        target_square: Square::E4,
                        promotion: None
                    },
                    UciMove {
                        starting_square: Square::E7,
                        target_square: Square::E5,
                        promotion: None
                    }
                ],
            })
        )
    }

    #[test]
    pub fn fen_startpos() {
        assert_eq!(
            UciCommand::try_parse(
                "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned()
            ),
            Some(UciCommand::Position {
                fen: Some(UciFen::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")),
                moves: vec![],
            })
        )
    }


    #[test]
    pub fn position_invalid_fen() {
        assert_eq!(
            UciCommand::try_parse(
                "position fen 0000/A0A0A.? @NOT_A_FEN_STRING".to_owned()
            ),
            None
        )
    }


}
