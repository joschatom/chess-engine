// A Simple chess engine.

use std::{io::Read, num::NonZero, time::Instant};

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

use utils::{perft, print_bitboard};

const STARTING_FEN: &'static str =
    //"8/3k4/8/5n2/6q1/1r1NK3/8/8 w - - 0 1"
    // "8/R6p/6pk/p2pP3/P1b2P2/2P3P1/1rB5/6K1 w - - 5 37";
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//"5k2/1P6/8/4Pp2/8/1p6/P7/4K2R w K f6 0 1";
//  "rnbqk1nr/pppppppp/8/2b5/8/8/PPP1P1PP/R3K2R w KQkq - 0 1";
//"4k3/8/8/2b5/8/8/8/R3K2R w KQ - 0 1"; // NO PAWNS(O-O-O + BLOCKED O-O)

fn main() {
    let mut board = Board::load_fen(/*""*/ STARTING_FEN.to_owned()).unwrap();

    /*  println!("==== BOARD INFO ====");
        println!("Material: {:?}", board.count_material());
        println!("Turn: {:?}", board.turn);
        println!("FEN: {:?}", STARTING_FEN);
        println!("SYNTAX: . = EMPTY, % = ACTIVE");

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

            board.do_move(*m);
            print_bitboard(board.bitboards.all_pieces(None));
            board.undo_move(*m);

            println!()
        }

        let mut board = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned()).unwrap();
    */

    // board.prepare();

    /*board.do_move(Move {
            starting_square: Square::E2,
            target_square: Square::E4,
            flag: r#move::MoveFlag::None,
        });

        board.do_move(Move {
            starting_square: Square::B8,
            target_square: Square::A6,
            flag: r#move::MoveFlag::None,
        });

        board.do_move(Move {
            starting_square: Square::F1,
            target_square: Square::A6,
            flag: r#move::MoveFlag::Capture(Piece::Knight),
        });
    */
    //print_bitboard(board.bitboards.all_pieces(None));

    println!();

    let mv = Move {
        starting_square: Square::C2,
        target_square: Square::C4,
        flag: r#move::MoveFlag::None,
    };

    board.do_move(mv);

    print_bitboard(board.bitboards.all_pieces(None));
    println!();

    board.undo_move(mv);

    // board = Board::load_fen("8/8/2k5/8/8/8/1K1P1r2/8 w - - 0 1".to_owned()).unwrap();

    print_bitboard(board.bitboards.all_pieces(None));

    board.do_move(Move {
        starting_square: Square::C2,
        target_square: Square::C3,
        flag: r#move::MoveFlag::None,
    });

    board.do_move(Move {
        starting_square: Square::A7,
        target_square: Square::A5,
        flag: r#move::MoveFlag::None,
    });

    board.do_move(Move {
        starting_square: Square::D1,
        target_square: Square::A5,
        flag: r#move::MoveFlag::None,
    });

    dbg!(board.pinned_pieces(Color::Black));

    print_bitboard(board.pieces(Color::White));

    let mut start = Instant::now();
    let v = perft(&mut board, 1, 1);
    let time = Instant::now() - start;

    start = Instant::now();
    println!(
        "[Nodes: {}]\n[Time]\n  Total: {:?}\n  Avg. Per Node: {:?}\n",
        v,
        time,
        time.div_f32(v as _)
    );

    println!();
}
