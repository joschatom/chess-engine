// A Simple chess engine.

use std::{
    io::{BufRead, Read},
    marker::PhantomData,
    num::NonZero,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use board::{BitBoards, Board};

pub mod bitboard;
pub mod board;
pub mod hardcoded_moves;
pub(crate) mod macros;
pub mod r#move;
pub mod piece;
pub mod square;
mod tests;
pub mod uci;
pub mod utils;

use piece::*;
use r#move::Move;
use square::Square;

use uci::{UciCommand, UciFen, UciMove};
use utils::{perft, print_bitboard};

const STARTING_FEN: &'static str =
    //"8/3k4/8/5n2/6q1/1r1NK3/8/8 w - - 0 1"
    // "8/R6p/6pk/p2pP3/P1b2P2/2P3P1/1rB5/6K1 w - - 5 37";
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//"5k2/1P6/8/4Pp2/8/1p6/P7/4K2R w K f6 0 1";
//  "rnbqk1nr/pppppppp/8/2b5/8/8/PPP1P1PP/R3K2R w KQkq - 0 1";
//"4k3/8/8/2b5/8/8/8/R3K2R w KQ - 0 1"; // NO PAWNS(O-O-O + BLOCKED O-O)

#[derive(Debug, Clone)]
pub enum EngineEvent {
    EndGame,
    Gameover,
    PerftResult {
        depth: u32,
        count: u64,
        root_nodes: Vec<(Move, u64)>,
    },
    Debug(String),
}

#[derive(Debug, Clone)]
pub enum EngineControl {
    UciCommand(UciCommand),
    Terminate,
    Perft(u32),
}

pub fn start_uci() {
    eprintln!("Starting UCI....");

    let stdin = std::io::stdin();

    let (ctl, ctl_rx) = channel::<EngineControl>();
    let (evt_tx, evt) = channel::<EngineEvent>();

    let io_ctl = ctl.clone();

    let engine_thread = thread::Builder::new()
        .name("engine".to_owned())
        .spawn(move || UciEngine::run_thread(ctl_rx, evt_tx))
        .expect("failed to start engine thread");

    thread::spawn(move || loop {
        let mut _lock = stdin.lock();

        let mut input = String::new();
        _lock.read_line(&mut input).expect("failed to read line");

        if let Some(cmd) = UciCommand::try_parse(input) {
            let exit = cmd.clone() == UciCommand::Quit;

            io_ctl.send(EngineControl::UciCommand(cmd)).unwrap();

            if exit {
                break;
            }
        }
        drop(_lock);
    });

    for event in evt.iter() {
        match event {
            EngineEvent::Debug(msg) => {
                println!("info string {}", msg);
            },
            EngineEvent::PerftResult { depth: _, count, root_nodes } 
                => {
                    for node in root_nodes {
                        println!("{}: {}", node.0.notation_long(), node.1);
                    }

                    println!();
                    println!("Count: {},", count);
                }
            _ => {}
        }
    }

    eprintln!("Waiting for Engine Thread to stop...");
    while !engine_thread.is_finished() {}
}

pub struct UciEngine<'a> {
    evt_tx: Sender<EngineEvent>,
    board: Board,
    stop: bool,
    _phantom: PhantomData<&'a ()>,
    is_position_set: bool,
}

impl<'a> UciEngine<'a> {
    pub(self) fn run_thread(ctl: Receiver<EngineControl>, evt: Sender<EngineEvent>) {
        let mut instance = Self {
            evt_tx: evt.clone(),
            board: Board::new(),
            stop: false,
            _phantom: PhantomData,
            is_position_set: false,
        };

        for control in ctl.iter() {
            match control {
                EngineControl::Terminate => instance.stop = true,
                EngineControl::UciCommand(c) => match instance.handle_command(c) {
                    Ok(()) => {}
                    Err(e) => evt
                        .send(EngineEvent::Debug(e.to_string()))
                        .expect("failed to send error event"),
                },
                _ => {}
            }

            if instance.stop == true {
                return;
            }
        }
    }

    pub fn print(&self, m: &'_ str) {
        self.evt_tx.send(EngineEvent::Debug(m.to_owned())).unwrap()
    }

    pub fn handle_command(&mut self, cmd: UciCommand) -> Result<(), &str> {
        match cmd {
            UciCommand::Perft(ply) => {
                if !self.is_position_set {
                    self.print("position not set");
                    return Ok(());
                }

                let mut nodes = None;

                let count = utils::perft(&mut self.board, ply, ply, &mut nodes);

                self.evt_tx.send(EngineEvent::PerftResult {
                    depth: ply,
                    count,
                    root_nodes: nodes.unwrap_or(vec![]),
                }).expect("failed to send perft() result");
            }
            UciCommand::Position { fen, moves } => {
                let fen = fen.unwrap_or(UciFen::new(&STARTING_FEN));

                self.board = Board::load_fen(fen.inner()).ok_or("error: invalid fen")?;

                self.is_position_set = true;

                for mv in moves {
                    let m = self
                        .board
                        .uci_to_board_move(self.board.turn, mv)
                        .ok_or("error: invalid moves")?;

                    self.board.do_move(m);
                }
            }
            UciCommand::Stop => todo!("UciCommand::Stop, searching is not yet implemented"),
            _ => {}
        }

        Ok(())
    }
}

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

    /*  board.do_move(Move {
        starting_square: Square::D2,
        target_square: Square::D3,
        flag: r#move::MoveFlag::None,
    });

    board.do_move(Move {
        starting_square: Square::A7,
        target_square: Square::A5,
        flag: r#move::MoveFlag::None,
    });

        board.do_move(Move {
            starting_square: Square::D1,
            target_square: Square::A4,
            flag: r#move::MoveFlag::None,
        });
    */
    print_bitboard(board.pieces(Color::White));

    let mut start = Instant::now();
    let v = perft(&mut board, 4, 4, &mut None);
    let time = Instant::now() - start;

    start = Instant::now();
    println!(
        "[Nodes: {}]\n[Time]\n  Total: {:?}\n  Avg. Per Node: {:?}\n",
        v,
        time,
        time.div_f32(v as _)
    );

    println!();

    start_uci();
}
