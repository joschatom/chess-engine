use std::ops::{BitAnd, BitOr, BitOrAssign, Index, Not};

use crate::{
    piece::{self, Color},
    square::*,
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub struct BitBoard(pub u64);

impl Index<Square> for BitBoard {
    type Output = bool;

    fn index(&self, index: Square) -> &Self::Output {
        if (self.0 << (index.rank() as usize * 8) + (index.file() as usize)) != 0 {
            &true
        } else {
            &false
        }
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u64::MAX);
    pub const CORNERS: Self = Self(u64::from_ne_bytes([
        0b11111111, 0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b10000001,
        0b11111111,
    ]));

    pub const fn lshifted(v: Self, bits: u8) -> Self {
        Self(v.0 << bits)
    }

    pub fn get_rank(&self, idx: u8) -> Self {
        Self(dbg!(self.0 & ((u8::MAX as u64) << (idx - 1) * 8)))
    }

    pub fn shifted_up(&self, ranks: u8) -> Self {
        Self(self.0 << ranks * 8)
    }

    pub fn is_single(&self) -> bool {
        self.0.count_ones() == 1
    }

    pub fn active_squares(&self) -> Vec<Square> {
        let mut list = vec![];
        let mut copy = self.0;
        while copy != 0 {
            let e = copy & copy.wrapping_neg();
            list.push(Square::index(e.trailing_zeros() as usize));
            copy ^= e;
        }

        list
    }

    pub fn shl(self, bits: u8) -> Self {
        assert!(bits <= 64);
        // assert!(self.0.leading_zeros() + 1 >= bits as _, "Cannot shift value {:b} {} bits to the left.", self.0, bits);

        if self.0.checked_shl(bits as _).is_none() {
            eprintln!("DEBUG: Stopping Bitboard overflow!");
            return self;
        }

        Self(self.0 << bits)
    }

    pub fn shr(self, bits: u8) -> Self {
        assert!(bits <= 64);
        if self.0.checked_shr(bits as _).is_none() {
            eprintln!("DEBUG: Stopping Bitboard overflow!");
            return self;
        }
        Self(self.0 >> bits)
    }

    pub fn slide(self, s: &'_ piece::Slider) -> Self {
        Self((self.0 << s.left) >> s.right)
    }

    pub fn forward(self, color: Color) -> Self {
        match color {
            Color::White => self.shl(8),
            Color::Black => self.shr(8),
        }
    }

    pub fn backward(self, color: Color) -> Self {
        match color {
            Color::White => self.shr(8),
            Color::Black => self.shl(8),
        }
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.bitand(rhs.0))
    }
}
impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.bitor(rhs.0))
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0.bitor_assign(rhs.0);
    }
}
