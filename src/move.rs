use crate::{piece::Piece, square::Square};

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub starting_square: Square,
    pub target_square: Square,
    pub flag: MoveFlag,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    None,
    Castle(CastlingMethod),
    Promotion(Piece),
    NullMove,
    Capture(Piece),
    Untargeted,
    EnPassant(Square),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingMethod {
    Short = 0,
    Long = 1,
}

impl core::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.flag {
            MoveFlag::Castle(CastlingMethod::Long) => f.write_str("O-O-O"),
            MoveFlag::Castle(CastlingMethod::Short) => f.write_str("O-O"),
            MoveFlag::EnPassant(sq) => f.write_fmt(format_args!(
                "[EP {:?}]{:?}{:?}",
                sq,self.starting_square, self.target_square
            )),
            MoveFlag::Capture(_) => f.write_fmt(format_args!(
                "{:?}x{:?}",
                self.starting_square, self.target_square
            )),
            MoveFlag::Promotion(p) => f.write_fmt(format_args!(
                "{:?}{:?}={:?}",
                self.starting_square, self.target_square, p
            )),
            MoveFlag::Untargeted => f.write_fmt(format_args!("{:?}<???>", self.starting_square)),
            MoveFlag::NullMove => f.write_str("<null>"),
            _ => f.write_fmt(format_args!(
                "{:?}{:?}",
                self.starting_square, self.target_square
            )),
        }
    }
}
