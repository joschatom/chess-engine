#[allow(unused)]

pub fn print_bitboard(board: BitBoard) {
    for rank in (0..8).rev() {
        // Iterate from 7 to 0 for standard chessboard representation
        for file in 0..8 {
            // Correctly calculate the bit position
            let bit_position = (rank * 8 + file) as u64;
            print!(
                "{} ",
                if board.0 & (1 << bit_position) != 0 {
                    '%'
                } else {
                    '.'
                }
            );
        }
        println!(); // Print a newline after each rank
    }
}

pub fn build_king_moves_lookup() -> [BitBoard; Square::NUM] {
    let mut out = [BitBoard::EMPTY; Square::NUM];

    for square in Square::ALL {
        let bitboard = square.bitboard().0;

        // Calculate potential king moves with boundary checks
        let mut moves = 0;

        // Up (8)
        if ((square.rank() as usize) as usize) < 7 {
            moves |= bitboard.checked_shl(8).unwrap_or(0);
        }
        // Down (-8)
        if (square.rank() as usize) > 0 {
            moves |= bitboard.checked_shr(8).unwrap_or(0);
        }
        // Right (1)
        if (square.file() as usize) < 7 {
            moves |= bitboard.checked_shl(1).unwrap_or(0);
        }
        // Left (-1)
        if (square.file() as usize) > 0 {
            moves |= bitboard.checked_shr(1).unwrap_or(0);
        }
        // Up-Right (9)
        if (square.rank() as usize) < 7 && (square.file() as usize) < 7 {
            moves |= bitboard.checked_shl(9).unwrap_or(0);
        }
        // Up-Left (7)
        if (square.rank() as usize) < 7 && (square.file() as usize) > 0 {
            moves |= bitboard.checked_shl(7).unwrap_or(0);
        }
        // Down-Right (-7)
        if (square.rank() as usize) > 0 && ((square.file() as usize) as usize) < 7 {
            moves |= bitboard.checked_shr(7).unwrap_or(0);
        }
        // Down-Left (-9)
        if (square.rank() as usize) > 0 && ((square.file() as usize) as usize) > 0 {
            moves |= bitboard.checked_shr(9).unwrap_or(0);
        }

        out[square as usize] = BitBoard(moves);
    }

    out
}

fn generate_bishop_moves() -> [BitBoard; Square::NUM] {
    let mut moves: [BitBoard; Square::NUM] = [BitBoard::EMPTY; Square::NUM];

    for square in 0..64 {
        let mut bitboard = 0;
        let file = square % 8; // Calculate the file (column)
        let rank = square / 8; // Calculate the rank (row)

        // Top-right diagonal
        let mut pos = square;
        while pos % 8 < 7 && pos / 8 < 7 {
            // Ensure within bounds
            pos += 9; // Move to the next top-right square
            bitboard |= 1 << pos; // Set the bit for the target square
        }

        // Top-left diagonal
        pos = square;
        while pos % 8 > 0 && pos / 8 < 7 {
            // Ensure within bounds
            pos += 7; // Move to the next top-left square
            bitboard |= 1 << pos; // Set the bit for the target square
        }

        // Bottom-right diagonal
        pos = square;
        while pos % 8 < 7 && pos / 8 > 0 {
            // Ensure within bounds
            pos -= 7; // Move to the next bottom-right square
            bitboard |= 1 << pos; // Set the bit for the target square
        }

        // Bottom-left diagonal
        pos = square;
        while pos % 8 > 0 && pos / 8 > 0 {
            // Ensure within bounds
            pos -= 9; // Move to the next bottom-left square
            bitboard |= 1 << pos; // Set the bit for the target square
        }

        moves[square] = BitBoard(bitboard); // Store the bitboard for the current square
    }

    moves
}

macro_rules! timed_block {
    {[$name:literal]$blk:expr} => {
        {
            let __time_start = std::time::Instant::now();
            $blk;
            println!("\nTime spend \"{}\": {:?}", $name, std::time::Instant::now() - __time_start);
        }
    };
}

use crate::{bitboard::BitBoard, square::Square};

pub fn generate_knight_move_magics() -> [BitBoard; Square::NUM] {
    let mut out: [BitBoard; Square::NUM] = [BitBoard::EMPTY; Square::NUM];

    const DELTAS: &[(isize, isize)] = &[
        (2, 1),
        (2, -1),
        (-2, 1),
        (-2, -1),
        (-1, 2),
        (1, 2),
        (-1, -2),
        (1, -2),
    ];

    for (idx, square) in Square::ALL.iter().enumerate() {
        let mut moves = BitBoard::EMPTY;

        for (f_off, r_off) in DELTAS {
            let file = ((square.file() as isize) + f_off);
            let rank = (((square.rank() as usize) as isize) + r_off);

            if file < 0 || rank < 0 || file > 7 || rank > 7 {
                continue;
            }

            moves |= BitBoard(1 << ((rank * 8) + file));
        }

        out[idx] = moves;
    }

    out
}
