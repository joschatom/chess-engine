use int_enum::IntEnum;

use crate::hardcoded_moves::{BISHOP_MOVES, KING_MOVES, KNIGHT_MOVES};
use crate::sliders_gen;
use crate::{bitboard::BitBoard, square::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slider {
    pub left: u8,
    pub right: u8,
}

impl Slider {
    pub const UP: Slider = Self { left: 8, right: 0 };
    pub const DOWN: Slider = Self { left: 0, right: 8 };
    pub const LEFT: Slider = Self { left: 1, right: 0 };
    pub const RIGHT: Slider = Self { left: 0, right: 1 };
    pub const LEFTUP: Slider = Self { left: 9, right: 0 };
    pub const RIGHTUP: Slider = Self { left: 7, right: 0 };
    pub const LEFTDOWN: Slider = Self { left: 0, right: 9 };
    pub const RIGHTDOWN: Slider = Self { left: 0, right: 7 };

    pub const fn new(left: u8, right: u8) -> Self {
        assert!(left <= 64);
        assert!(right <= 64);

        Self { left, right }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum, ::strum_macros::EnumString, ::strum_macros::Display)]
#[repr(u8)]
pub enum Piece {
    King = 2,
    Pawn = 3,
    Bishop = 4,
    Knight = 5,
    Rook = 6,
    Queen = 7,
}

impl Piece {
    pub const ALL: [Piece; 6] = [
        Self::King,
        Self::Bishop,
        Self::Knight,
        Self::Pawn,
        Self::Rook,
        Self::Queen,
    ];
    pub const SLIDING: [Piece; 3] = [Self::Bishop, Self::Rook, Self::Queen];
    pub const PROMOTIONS: [Piece; 4] = [Self::Bishop, Self::Rook, Self::Knight, Self::Queen];

    pub fn notation(&self) -> char {
        match self {
            Self::King => 'K',
            Self::Queen => 'Q',
            Self::Knight => 'N',
            Self::Bishop => 'B',
            Self::Pawn => 'P',
            Self::Rook => 'R',
        }
    }

    pub fn from_notation(c: char) -> Option<Self> {
        Some(match c {
            'K' => Self::King,
            'Q' => Self::Queen,
            'N' => Self::Knight,
            'B' => Self::Bishop,
            'P' => Self::Pawn,
            'R' => Self::Rook,
            _ => None?,
        })
    }

    pub fn possible_moves(&self, square: Square) -> BitBoard {
        match self {
            Self::Bishop => BISHOP_MOVES[square as usize],
            Self::Knight => KNIGHT_MOVES[square as usize],
            Self::Rook => square.file().bitboard() | square.rank().bitboard(),
            Self::Queen => {
                BISHOP_MOVES[square as usize] | square.file().bitboard() | square.rank().bitboard()
            }
            Self::King => KING_MOVES[square as usize],
            _ => BitBoard::EMPTY,
        }
    }

    pub fn sliders(&self) -> Option<&'static [Slider]> {
        match self {
            Self::Rook => Some(sliders_gen!(
                @rookSliders
                8:0,
                0:8,
                1:0,
                0:1,
            )),
            Self::Bishop => Some(&sliders_gen!(
                @bishopSliders
                9:0,
                7:0,
                0:9,
                0:7,
            )),
            Self::Queen => Some(&sliders_gen!(
                @queenSliders
                8:0,
                0:8,
                1:0,
                0:1,
                9:0,
                7:0,
                0:9,
                0:7,
            )),
            _ => None,
        }
    }

    pub fn material_value(&self) -> u32 {
        match self {
            Self::King => 0,
            Self::Pawn => 1,
            Self::Bishop => 3,
            Self::Knight => 3,
            Self::Rook => 5,
            Self::Queen => 9,
        }
    }

    pub fn is_sliding(&self) -> bool {
        match self {
            Self::Bishop | Self::Rook | Self::Queen => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn opponent(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }

    pub fn promotion_rank(&self) -> Rank {
        match self {
            Color::Black => Rank::First,
            Color::White => Rank::Eighth,
        }
    }
}
