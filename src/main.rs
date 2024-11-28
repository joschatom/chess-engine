// A Simple chess engine.

use std::{io::Read, time::Instant};

use board::{BitBoards, Board};

pub mod bitboard;
pub mod board;
pub mod hardcoded_moves;
pub(crate) mod macros;
pub mod r#move;
pub mod piece;
pub mod square;
pub mod utils;

use piece::*;
use r#move::Move;
use square::Square;
use utils::print_bitboard;

const STARTING_FEN: &'static str =
    //"8/3k4/8/5n2/6q1/1r1NK3/8/8 w - - 0 1"
    // "8/R6p/6pk/p2pP3/P1b2P2/2P3P1/1rB5/6K1 w - - 5 37";
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//  "rnbqk1nr/pppppppp/8/2b5/8/8/PPP1P1PP/R3K2R w KQkq - 0 1";
//"4k3/8/8/2b5/8/8/8/R3K2R w KQ - 0 1"; // NO PAWNS(O-O-O + BLOCKED O-O)

fn main() {
    let mut board = Board::load_fen(/*""*/ STARTING_FEN.to_owned()).unwrap();

    println!("==== BOARD INFO ====");
    println!("Material: {:?}", board.count_material());
    println!("Turn: {:?}", board.turn);
    println!("FEN: {:?}", STARTING_FEN);

    println!("==== (PRE) MOVE GENERATION ====");
    let start = std::time::Instant::now();
    let moves = board.generate_moves(Color::Black);
    println!("[Color: {:?}]", Color::Black);
    println!("[Time: {:?}]\n", Instant::now() - start);
    println!("[Move: @PreScan]");
    for (i, m) in moves.iter().enumerate() {
        let piece = board.squares[m.starting_square as usize]
            .expect("Board state changed during move generation")
            .1;

        println!("{}. ({:?}){}", 1 + i, piece, m);
    }

    println!("==== MOVE GENERATION ====");


    dbg!(board.squares[Square::E4 as usize]);

    let start = std::time::Instant::now();
    let moves = board.generate_moves(Color::White);
    println!("[Color: {:?}]", Color::White);
    println!("[Time: {:?}]\n", Instant::now() - start);
    println!("[Move: {:?}]", board.move_count);

    for (i, m) in moves.iter().enumerate() {
        let piece = board.squares[m.starting_square as usize]
            .expect("Board state changed during move generation")
            .1;

        println!("{}. ({:?}){}", 1 + i, piece, m);
    }

    println!();

    print_bitboard(board.bitboards.0[BitBoards::ad_bitboard(Color::Black)]);
}
