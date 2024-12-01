use std::{cell::LazyCell, collections::HashMap, str::FromStr};

use crate::{
    bitboard::{self, BitBoard},
    hardcoded_moves::KNIGHT_MOVES,
    piece::{self, Color, Piece},
    r#move::{CastlingMethod, Move, MoveFlag},
    square::*,
    utils::{self, print_bitboard},
    Slider,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BitBoards(pub [BitBoard; 10]);

impl BitBoards {
    pub fn all_pieces(&self, color: Option<Color>) -> BitBoard {
        match color {
            Some(color) => self.0[(color as u8) as usize],
            None => self.0[Color::White as usize] | self.0[Color::Black as usize],
        }
    }

    /// Attacked/Defended BItboard for a [Color].
    pub fn ad_bitboard(color: Color) -> usize {
        match color {
            Color::White => 8,
            Color::Black => 9,
        }
    }

    pub fn get_piece_set<'a>(&'a self, piece: Piece, color: Option<Color>) -> BitBoard {
        match color {
            Some(color) => self.0[(color as u8) as usize] & self.0[(piece as u8) as usize],
            _ => self.0[(piece as u8) as usize],
        }
    }

    pub fn add_attackerd_bitboard(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.0[8],
            Color::Black => self.0[9],
        }
    }

    pub fn sliding_pieces(&self, color: Color) -> BitBoard {
        self.get_piece_set(Piece::Bishop, Some(color))
            | self.get_piece_set(Piece::Queen, Some(color))
            | self.get_piece_set(Piece::Rook, Some(color))
    }

    pub fn remove_piece(&mut self, piece: Piece, color: Color, square: Square) {
        self.0[piece as usize] = self.0[piece as usize] & !square.bitboard();
        self.0[color as usize] = self.0[color as usize] & !square.bitboard();
    }

    pub fn r#move(&mut self, color: Color, piece: Piece, mv: Move) {
        self.0[color as usize] = self.0[color as usize] & !mv.starting_square.bitboard();
        self.0[piece as usize] = self.0[piece as usize] & !mv.starting_square.bitboard();

        self.0[color as usize] |= mv.target_square.bitboard();
        self.0[piece as usize] |= mv.target_square.bitboard();
    }

    pub fn undo_move(&mut self, color: Color, piece: Piece, mv: Move) {
        self.0[color as usize] = self.0[color as usize] & !mv.target_square.bitboard();
        self.0[piece as usize] = self.0[piece as usize] & !mv.target_square.bitboard();

        self.0[color as usize] |= mv.starting_square.bitboard();
        self.0[piece as usize] |= mv.starting_square.bitboard();
    }

    pub fn insert_piece(&mut self, square: Square, piece: Piece, color: Color) {
        self.0[(color as u8) as usize] |= square.bitboard();

        self.0[(piece as u8) as usize] |= square.bitboard();
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub bitboards: BitBoards,
    pub turn: Color,
    pub castling_availability: [(bool, bool); 2],
    pub en_passant: BitBoard,
    pub en_passant_prev: BitBoard,
    pub halfmove_count: usize,
    pub move_count: usize,
    pub squares: [Option<(Color, Piece)>; 64],
    pub move_filters: [BitBoard; 2], // used for checks...
}

impl Board {
    pub fn new() -> Self {
        Self {
            bitboards: BitBoards([BitBoard(0b0u64); 10]),
            squares: [None; 64],
            turn: Color::White,
            castling_availability: [(false, false); 2],
            en_passant: BitBoard::EMPTY,
            en_passant_prev: BitBoard::EMPTY,
            halfmove_count: 0,
            move_count: 1,
            move_filters: [BitBoard::EMPTY; 2],
        }
    }

    pub fn prepare(&mut self) {
        _ = self.generate_moves(self.turn.opponent());
    }

    pub fn count_material(&self) -> (u32, u32) {
        let mut white_material = 0;
        let mut black_material = 0;

        for piece in Piece::ALL {
            white_material += self
                .bitboards
                .get_piece_set(piece, Some(Color::White))
                .0
                .count_ones()
                * piece.material_value();
            black_material += self
                .bitboards
                .get_piece_set(piece, Some(Color::Black))
                .0
                .count_ones()
                * piece.material_value();
        }

        (white_material, black_material)
    }

    pub fn pieces(&self, color: Color) -> BitBoard {
        self.bitboards.all_pieces(Some(color))
    }

    pub fn next_turn(&mut self) {}

    fn castling_squares(color: Color, castling_method: CastlingMethod) -> (Square, Square) {
        let king_file = match castling_method {
            CastlingMethod::Short => File::G,
            CastlingMethod::Long => File::C,
        };

        let rook_file = match castling_method {
            CastlingMethod::Short => File::F,
            CastlingMethod::Long => File::D,
        };

        let rank = match color {
            Color::White => Rank::First,
            Color::Black => Rank::Eighth,
        };

        return (Square::new(king_file, rank), Square::new(rook_file, rank));
    }

    fn is_double_move(color: Color, mv: Move) -> bool {
        mv.starting_square
            .bitboard()
            .forward(color)
            .forward(color)
            .0
            & mv.target_square.bitboard().0
            != 0
    }

    pub fn do_simple_move(&mut self, piece: Piece, mv: Move) {
        self.bitboards.r#move(self.turn, piece, mv);
        self.squares[mv.target_square as usize] = Some((self.turn, piece));
        self.squares[mv.starting_square as usize] = None;
    }

    pub fn undo_simple_move(&mut self, piece: Piece, mv: Move) {
        self.bitboards.undo_move(self.turn, piece, mv);
        self.squares[mv.starting_square as usize] = Some((self.turn, piece));
        self.squares[mv.target_square as usize] = None;
    }

    pub fn get_piece_type(&self, sq: Square) -> Option<Piece> {
        if let Some((_, p)) = self.squares[sq as usize] {
            return Some(p);
        }

        for piece in Piece::ALL {
            if (self.bitboards.get_piece_set(piece, None) & sq.bitboard()).0 != 0 {
                return Some(piece);
            }
        }

        None
    }

    pub fn undo_move(&mut self, mv: Move) -> Option<()> {
        //println!("DEBUG: UndoMove(mv = {:?}), Turn: {:?}", mv, self.turn.opponent());

        _ = match self.get_piece_type(mv.target_square) {
            None => {
                print_bitboard(self.bitboards.all_pieces(None));
                println!("Turn: {:?}", self.turn);
                panic!();
            }
            Some(v) => v,
        };

        self.turn = self.turn.opponent();

        match mv.flag {
            MoveFlag::EnPassant(_t) => {
                let other_pawn = Square::index(
                    mv.target_square
                        .bitboard()
                        .backward(self.turn)
                        .0
                        .trailing_zeros() as usize,
                );

                assert!(_t == other_pawn);

                self.bitboards
                    .insert_piece(other_pawn, Piece::Pawn, self.turn.opponent());

                self.squares[other_pawn as usize] = Some((self.turn.opponent(), Piece::Pawn));

                self.undo_simple_move(Piece::Pawn, mv);
            }

            MoveFlag::Capture(target) => {
                self.undo_simple_move(self.get_piece_type(mv.target_square)?, mv);
                self.bitboards
                    .insert_piece(mv.target_square, target, self.turn.opponent());
                self.squares[mv.target_square as usize] = Some((self.turn.opponent(), target));
            }

            MoveFlag::Promotion(target) => {
                self.bitboards.0[target as usize].0 &= !mv.target_square.bitboard().0;
                self.bitboards.0[Piece::Pawn as usize] |= mv.starting_square.bitboard();
                self.squares[mv.starting_square as usize] = Some((self.turn, Piece::Pawn));
            }
            MoveFlag::Castle(method) => {
                let (king_target, rook_target) = Self::castling_squares(self.turn, method);

                // King
                self.undo_simple_move(
                    Piece::King,
                    Move {
                        starting_square: Self::CASTLING_SQUARES[self.turn as usize]
                            [method as usize]
                            .0,
                        target_square: king_target,
                        flag: mv.flag,
                    },
                );

                // Rook
                self.undo_simple_move(
                    Piece::Rook,
                    Move {
                        starting_square: Self::CASTLING_SQUARES[self.turn as usize]
                            [method as usize]
                            .1,
                        target_square: rook_target,
                        flag: mv.flag,
                    },
                );
            }
            MoveFlag::Untargeted => {}
            MoveFlag::NullMove => {}
            _ => self.undo_simple_move(self.get_piece_type(mv.target_square)?, mv),
        }

        self.move_filters = [BitBoard::EMPTY; 2];

        self.en_passant = self.en_passant_prev;
        self.en_passant_prev = BitBoard::EMPTY;

        self.move_count = self.move_count.saturating_sub(1);

        // halfmove clock!!

        //  self.turn = self.turn.opponent();

        Some(())
    }

    pub fn do_move(&mut self, mv: Move) -> Option<()> {
        let piece = match self.get_piece_type(mv.starting_square) {
            None => {
                print_bitboard(self.bitboards.all_pieces(None));
                println!("Turn: {:?}", self.turn);
                panic!();
            }
            Some(v) => v,
        };

        if piece == Piece::Pawn && Self::is_double_move(self.turn, mv) {
            self.en_passant = mv.starting_square.bitboard().forward(self.turn);
        } else {
            self.en_passant_prev = self.en_passant;
            self.en_passant = BitBoard::EMPTY;
        }

        match mv.flag {
            MoveFlag::EnPassant(_t) => {
                assert!(piece == Piece::Pawn, "only pawns can do en passant!");

                let other_pawn = Square::index(
                    mv.target_square
                        .bitboard()
                        .backward(self.turn)
                        .0
                        .trailing_zeros() as usize,
                );

                assert!(other_pawn == _t);

                self.bitboards
                    .remove_piece(Piece::Pawn, self.turn.opponent(), other_pawn);

                self.squares[other_pawn as usize] = None;

                self.do_simple_move(piece, mv);
            }

            MoveFlag::Capture(target) => {
                self.bitboards
                    .remove_piece(target, self.turn.opponent(), mv.target_square);
                self.squares[mv.target_square as usize] = None;
                self.do_simple_move(piece, mv);
            }

            MoveFlag::Promotion(target) => {
                self.bitboards.0[piece as usize].0 &= !mv.starting_square.bitboard().0;
                self.bitboards.0[target as usize].0 |= mv.target_square.bitboard().0;
                self.squares[mv.target_square as usize] = Some((self.turn, target));
            }
            MoveFlag::Castle(method) => {
                let (king_target, rook_target) = Self::castling_squares(self.turn, method);

                // King
                self.do_simple_move(
                    Piece::King,
                    Move {
                        starting_square: Self::CASTLING_SQUARES[self.turn as usize]
                            [method as usize]
                            .0,
                        target_square: king_target,
                        flag: mv.flag,
                    },
                );

                // Rook
                self.do_simple_move(
                    Piece::Rook,
                    Move {
                        starting_square: Self::CASTLING_SQUARES[self.turn as usize]
                            [method as usize]
                            .1,
                        target_square: rook_target,
                        flag: mv.flag,
                    },
                );
            }
            MoveFlag::Untargeted => {}
            MoveFlag::NullMove => {}
            _ => self.do_simple_move(piece, mv),
        }

        self.move_filters = [BitBoard::EMPTY; 2];
        self.bitboards.0[BitBoards::ad_bitboard(self.turn.opponent())] = BitBoard::EMPTY;

        self.move_count += 1;

        // halfmove clock!!

        //    println!("Move  {}:", mv);
        //       print_bitboard(self.bitboards.all_pieces(None));

        self.turn = self.turn.opponent();

        Some(())
    }

    pub fn generate_moves(&mut self, color: Color) -> Vec<Move> {
        let mut move_bitboards: HashMap<Square, BitBoard> = HashMap::new();
        let mut out = vec![];

        let pawns = self.bitboards.get_piece_set(Piece::Pawn, Some(color));

        let king_moves = self.king_moves(color) & !self.pieces(color);

        self.bitboards.0[BitBoards::ad_bitboard(color)] |= king_moves;
        move_bitboards.insert(self.king_square(color), king_moves);

        let (pinned, checkers);

        (pinned, checkers) = self.pinned_pieces(color);

        for pawn in Self::isolate_pieces(pawns) {
            let square = Square::index(pawn.0.trailing_zeros() as _);

            let (ad, moves) = self.pawn_moves(pawn, color);
            move_bitboards.insert(square, moves & !self.pieces(color));
            self.bitboards.0[BitBoards::ad_bitboard(color)] |= ad;
        }

        for piece in Piece::SLIDING {
            for piece_board in
                Self::isolate_pieces(self.bitboards.get_piece_set(piece, Some(color)))
            {
                let square = Square::index(piece_board.0.trailing_zeros() as _);

                let relevant_blockers =
                    self.bitboards.all_pieces(None) & piece.possible_moves(square);

                let mut moves;

                /*if piece == Piece::Rook {
                    eprintln!("DEBUG: Generating Rook Moves!");
                    moves =
                        self.hacky_rook_fix_moves(color, square, piece_board.0, relevant_blockers)
                } else {*/
                moves = self.slider_moves(
                    piece
                        .sliders()
                        .expect("Tried to query sliders moves for a non-slider piece"),
                    color,
                    piece_board.0,
                    relevant_blockers,
                );
                //}

                moves = (moves & !self.pieces(color)) & piece.possible_moves(square);

                move_bitboards.insert(square, moves);

                self.bitboards.0[BitBoards::ad_bitboard(color)] |= moves; // Should this be a side-effect or not?
            }
        }

        for knight in Self::isolate_pieces(self.bitboards.get_piece_set(Piece::Knight, Some(color)))
        {
            let sq = Square::index(knight.0.trailing_zeros() as _);

            let moves = KNIGHT_MOVES[sq as usize];

            self.bitboards.0[BitBoards::ad_bitboard(color)] |= moves;

            move_bitboards.insert(sq, moves);
        }

        if self.can_castle_short(color) && !self.in_check(color) {
            out.push(Move {
                starting_square: self.king_square(color),
                target_square: Self::castling_squares(color, CastlingMethod::Short).0,
                flag: MoveFlag::Castle(CastlingMethod::Short),
            });
        }

        if self.can_castle_long(color) & !self.in_check(color) {
            out.push(Move {
                starting_square: self.king_square(color),
                target_square: Self::castling_squares(color, CastlingMethod::Long).1,
                flag: MoveFlag::Castle(CastlingMethod::Long),
            });
        }

        let mut move_bitboards_v = move_bitboards.into_iter().collect::<Vec<_>>();

        move_bitboards_v.sort_by_key(|f| f.0);

        'conv: for (sq, mut bitboard) in move_bitboards_v {
            //println!("moves for piece on {}: {}", sq,bitboard.0.count_ones());

            /*if sq == Square::A6 {
                print_bitboard(bitboard);
            }*/

            if sq.bitboard() & self.pieces(color.opponent()) != BitBoard::EMPTY {
                // eprintln!("BUG: Tried to to move an opponent's piece on {}", sq);
                continue;
            }

            if pinned.contains_key(&sq) {
                continue 'conv;
            }

            for target_sq in bitboard.active_squares() {
                if target_sq.bitboard() & self.pieces(color) != BitBoard::EMPTY {
                    //  eprintln!("BUG: Tried to capture a same-colored piece {}x{}.", sq, target_sq);
                    continue;
                }

                let piece = self.get_piece_type(sq).expect("failed to get piece type.");

                if (piece == Piece::Rook && sq.file() == File::H)
                    && self.castling_availability[color as usize].0
                {
                    self.castling_availability[color as usize].0 = false;
                }

                if (piece == Piece::Rook && sq.file() == File::A)
                    && self.castling_availability[color as usize].1
                {
                    self.castling_availability[color as usize].1 = false;
                }

                if piece == Piece::King && sq == Self::CASTLING_SQUARES[color as usize][0].0 {
                    self.castling_availability[color as usize] = (false, false);
                }

                if self.in_check(color) {
                    let (checks, rays) = checkers;
                    if checks == 1 && piece != Piece::King {
                        if (target_sq.bitboard() & rays).0 == 0 {
                            /*println!(
                                "Filtered Move: {}",
                                Move {
                                    starting_square: sq,
                                    target_square: target_sq,
                                    flag: MoveFlag::None,
                                }
                            );*/
                            continue;
                        }
                    } else if checks > 1 && piece != Piece::King {
                        /*println!(
                            "Filtered Move: {}",
                            Move {
                                starting_square: sq,
                                target_square: target_sq,
                                flag: MoveFlag::None,
                            }
                        );*/
                        continue 'conv;
                    }
                }

                if piece == Piece::Pawn && (target_sq.rank() == color.promotion_rank()) {
                    for promotion in Piece::PROMOTIONS {
                        out.push(Move {
                            starting_square: sq,
                            target_square: target_sq,
                            flag: MoveFlag::Promotion(promotion),
                        });
                    }

                    continue;
                }

                if piece == Piece::Pawn
                    && (target_sq.bitboard() & self.en_passant != BitBoard::EMPTY)
                {
                    out.push(Move {
                        starting_square: sq,
                        target_square: target_sq,
                        flag: MoveFlag::EnPassant(Square::index(
                            target_sq.bitboard().backward(self.turn).0.trailing_zeros() as usize,
                        )),
                    });

                    continue;
                }

                if (target_sq.bitboard() & self.bitboards.all_pieces(Some(color.opponent())))
                    != BitBoard::EMPTY
                {
                    if self.squares[target_sq as usize].is_none() {
                        continue;
                    }

                    out.push(Move {
                        starting_square: sq,
                        target_square: target_sq,
                        flag: MoveFlag::Capture(self.squares[target_sq as usize].unwrap().1),
                    });

                    continue;
                }

                out.push(Move {
                    starting_square: sq,
                    target_square: target_sq,
                    flag: MoveFlag::None,
                });
            }
        }

        out
    }

    pub fn do_str_moves(&mut self, moves: &str) {
        for mv in moves.split_whitespace() {
            let start = Square::from_str(&mv[0..=1]).unwrap();
            let target = Square::from_str(&mv[2..=3]).unwrap();
        }
    }

    pub fn isolate_pieces(board: BitBoard) -> Vec<BitBoard> {
        let mut list = vec![];
        let mut board = board;
        while board.0 != 0 {
            let e = BitBoard(board.0 & board.0.wrapping_neg());
            list.push(e);
            board.0 ^= e.0;
        }

        list
    }

    pub fn blockers(&self) -> BitBoard {
        self.bitboards.all_pieces(None)
    }

    /// Possible moves for a isolated pawn with the given bitboard and color.
    pub fn pawn_moves(&self, pawn: BitBoard, color: Color) -> (BitBoard, BitBoard) {
        assert!(pawn.is_single());

        // println!("Generating pawn  moves"); // 100ns -> 1µs of avg. time per node in perft()....

        let square = Square::index(pawn.0.trailing_zeros() as _);

        let mut moves = pawn.forward(color) & !self.blockers();

        if (color == Color::White && square.rank() == Rank::Second)
            || (color == Color::Black && square.rank() == Rank::Seventh)
        {
            moves |= pawn.forward(color).forward(color)
                & !(self.blockers() | (self.blockers() & square.file().bitboard()).forward(color))
        }

        let capture_mask: BitBoard = match square.file() {
            File::A => pawn.forward(color).shl(1),
            File::H => pawn.forward(color).shr(1),
            _ => pawn.forward(color).shl(1) | pawn.forward(color).shr(1),
        };

        moves |= self.bitboards.all_pieces(Some(color.opponent())) & capture_mask;
        moves |= self.en_passant & capture_mask;

        (capture_mask, moves)
    }

    // slider_moves() doesn't seem to work with the rook for
    // some reason so for now we just have this hacky fix of reusing the old implementation.
    fn hacky_rook_fix_moves(
        &self,
        color: Color,
        square: Square,
        start_position: u64,
        relevant_blockers: BitBoard,
    ) -> BitBoard {
        let _ = color;
        let mut out = BitBoard::EMPTY;
        let rank_mask = square.rank().bitboard().0; // Create a mask for the rank
        let rank = square.rank();

        let mut position = start_position;
        let mut steps = 0;

        // LEFT
        while position != 0 {
            steps += 1;

            position = position >> 1;
            if position & relevant_blockers.0 != 0 {
                break;
            }

            out |= BitBoard(position);

            if position & BitBoard::CORNERS.0 != 0 {
                break;
            }
        }

        steps = 0;
        position = start_position;

        // RIGHT
        while position != 0 {
            steps += 1;

            position = position << 1;
            if position & relevant_blockers.0 != 0 {
                break;
            }

            out |= BitBoard(position);

            if position & BitBoard::CORNERS.0 != 0 {
                break;
            }
        }

        position = start_position;

        // UP
        while position != 0 {
            position = position << 8;
            if position & relevant_blockers.0 != 0 {
                break;
            }

            out |= BitBoard(position);

            if position & (Rank::Eighth.bitboard() & Rank::First.bitboard()).0 != 0 {
                break;
            }
        }

        position = start_position;

        // DOWN
        while position != 0 {
            position = position >> 8;
            if position & relevant_blockers.0 != 0 {
                break;
            }

            out |= BitBoard(position);

            if position & BitBoard::CORNERS.0 != 0 {
                break;
            }
        }

        out & BitBoard(!start_position)
    }

    pub fn slider_moves(
        &self,
        sliders: &[crate::piece::Slider],
        color: Color,
        start_position: u64,
        relevant_blockers: BitBoard,
    ) -> BitBoard {
        let mut out = BitBoard::EMPTY;

        for slider in sliders {
            let mut position = start_position;
            let mut hits: i32 = 0;

            for _ in 1..8 {
                position = utils::safe_shr(
                    utils::safe_shl(position, slider.left as _),
                    slider.right as _,
                );

                if position & relevant_blockers.0 != 0 {
                    if hits == 0 {
                        out |= BitBoard(position);
                    }

                    break;
                } else if hits == 0 {
                    out |= BitBoard(position);
                }

                /* if position & BitBoard::CORNERS.0 != 0 {
                    break;
                }*/
            }
        }

        out & !self.pieces(color)
    }

    pub fn king_square(&self, color: Color) -> Square {
        let board = self.bitboards.get_piece_set(Piece::King, Some(color)).0;

        assert!(board.count_ones() == 1);

        Square::index(board.trailing_zeros() as usize)
    }

    pub fn move_filter(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.move_filters[0],
            Color::Black => self.move_filters[1],
        }
    }

    // (enemy pieces that pin a piece, pinned pieces, checkers)
    pub fn pinned_pieces(&mut self, color: Color) -> (HashMap<Square, BitBoard>, (u32, BitBoard)) {
        let mut out = (0, BitBoard::EMPTY);
        let mut pinned_pieces = HashMap::new();

        let mut position;

        let checkers = (KNIGHT_MOVES[self.king_square(color) as usize]
            & self
                .bitboards
                .get_piece_set(Piece::Knight, Some(color.opponent())))
            | ((self.king_square(color).bitboard().forward(color).shr(1)
                | self.king_square(color).bitboard().forward(color).shl(1))
                & self
                    .bitboards
                    .get_piece_set(Piece::Pawn, Some(color.opponent())));

        out.0 += checkers.0.count_ones();
        out.1 |= checkers;

        for slider in Piece::Queen.sliders().unwrap() {
            position = self.king_square(color).bitboard();

            //let v = Piece::Queen.possible_moves(Square::index(position.0.trailing_zeros() as _)) & self.bitboards.sliding_pieces(color.opponent());

            //print_bitboard(v);

            let mut pinned = None;

            let mut ray = position;

            let cannot_pin = match *slider {
                Slider::DOWN | Slider::LEFT | Slider::RIGHT | Slider::UP => {
                    self.bitboards.get_piece_set(Piece::Bishop, None)
                }
                Slider::LEFTDOWN | Slider::LEFTUP | Slider::RIGHTDOWN | Slider::RIGHTUP => {
                    self.bitboards.get_piece_set(Piece::Rook, None)
                }
                s => unreachable!("Unhandled Slider: {:?}", s),
            };

            for _index in 1..8 {
                position = BitBoard(utils::safe_shr(
                    utils::safe_shl(position.0, slider.left as _),
                    slider.right as _,
                ));

                ray |= position;

                if position & self.pieces(color) != BitBoard::EMPTY {
                    if pinned.is_some() {
                        break;
                    }

                    pinned = Some(Square::index(position.0.trailing_zeros() as _));
                }

                let enemy_piece = position & (self.bitboards.sliding_pieces(color.opponent()));

                if enemy_piece != BitBoard::EMPTY {
                    if enemy_piece & cannot_pin != BitBoard::EMPTY {
                        break;  
                    }

                    // print_bitboard(enemy_piece);

                    // In Check
                    if pinned.is_none() {
                        out.1 |= ray;
                        out.0 += 1;

                        break;
                    }

                    pinned_pieces.insert(pinned.expect("unreachable"), ray);
                }

                if position & self.pieces(color.opponent()) != BitBoard::EMPTY {
                    break;
                }

                if position & BitBoard::CORNERS != BitBoard::EMPTY {
                    break;
                }
            }
        }

        (pinned_pieces, out)
    }

    const CASTLING_SQUARES: [[(Square, Square, u64); 2]; 2] = [
        [
            (
                Square::E1,
                Square::H1,
                Square::F1.bitboard().0 | Square::G1.bitboard().0,
            ),
            (
                Square::E1,
                Square::A1,
                Square::B1.bitboard().0 | Square::C1.bitboard().0 | Square::D1.bitboard().0,
            ),
        ],
        [
            (
                Square::E8,
                Square::H8,
                Square::F8.bitboard().0 | Square::G8.bitboard().0,
            ),
            (
                Square::E8,
                Square::A8,
                Square::B8.bitboard().0 | Square::C8.bitboard().0 | Square::D8.bitboard().0,
            ),
        ],
    ];

    pub fn can_castle_short(&self, color: Color) -> bool {
        self.castling_availability[color as usize].0
            && !self.in_check(color)
            // Double check if the placement is really correct.
            && self.king_square(color) == Self::CASTLING_SQUARES[color as usize][0].0
            && (self.bitboards.get_piece_set(Piece::Rook, Some(color)) & Self::CASTLING_SQUARES[color as usize][0].1.bitboard()
                != BitBoard::EMPTY)
            
            // Free of pieces & attacked squares?
            && (self.bitboards.all_pieces(Some(color))
                & BitBoard(Self::CASTLING_SQUARES[color as usize][0].2)
                == BitBoard::EMPTY)
            && (self.bitboards.0[BitBoards::ad_bitboard(color.opponent())]
                & BitBoard(Self::CASTLING_SQUARES[color as usize][0].2)
                == BitBoard::EMPTY)
    }

    pub fn can_castle_long(&self, color: Color) -> bool {
        self.castling_availability[color as usize].0
            && !self.in_check(color)

            // Double check if the placement is really correct.
            && self.king_square(color) == Self::CASTLING_SQUARES[color as usize][1].0
            && (self.bitboards.get_piece_set(Piece::Rook, Some(color)) & Self::CASTLING_SQUARES[color as usize][1].1.bitboard()
                != BitBoard::EMPTY)
            
            // Free of pieces & attacked squares?
            && (self.bitboards.all_pieces(Some(color))
                & BitBoard(Self::CASTLING_SQUARES[color as usize][1].2)
                == BitBoard::EMPTY)
            && (self.bitboards.0[BitBoards::ad_bitboard(color.opponent())]
                & BitBoard(Self::CASTLING_SQUARES[color as usize][1].2)
                == BitBoard::EMPTY)
    }

    pub fn king_moves(&self, color: Color) -> BitBoard {
        let board = self.bitboards.get_piece_set(Piece::King, Some(color)).0;

        assert_eq!(board.count_ones(), 1);

        let square = Square::index(board.trailing_zeros() as usize);

        Piece::King.possible_moves(square)
            & !self.bitboards.0[BitBoards::ad_bitboard(color.opponent())]
    }

    pub fn in_check(&self, color: Color) -> bool {
        self.bitboards.get_piece_set(Piece::King, Some(color))
            & self.bitboards.0[BitBoards::ad_bitboard(color.opponent())]
            != BitBoard::EMPTY
    }

    pub fn load_fen(fen: String) -> Option<Self> {
        let mut result = Self::new();

        let mut rank = 8u8;
        let mut file = 1u8;

        let mut parts = fen.split(' ');

        let placement = parts.next()?;

        for p in placement.chars() {
            if rank == 0 {
                break;
            }

            if let Some(skip) = p.to_digit(10) {
                assert!(skip <= 8);
                assert!(skip != 0);

                file = (file % 8) + (skip as u8);

                continue;
            }

            if p == '/' {
                rank -= 1;
                file = 1;

                continue;
            }

            let color = match p.is_uppercase() {
                true => Color::White,
                false => Color::Black,
            };

            let piece = match p.to_ascii_lowercase() {
                'p' => Piece::Pawn,
                'n' => Piece::Knight,
                'b' => Piece::Bishop,
                'r' => Piece::Rook,
                'q' => Piece::Queen,
                'k' => Piece::King,
                _ => panic!("Invalid FEN"),
            };

            result.squares[(((rank - 1) * 8) + (file - 1)) as usize] = Some((color, piece));

            result.bitboards.insert_piece(
                Square::new(File::index((file - 1) as _), Rank::index((rank - 1) as _)),
                piece,
                color,
            );

            file += 1;
        }

        match parts.next() {
            Some("w") => result.turn = Color::White,
            Some("b") => result.turn = Color::Black,
            _ => panic!("Invalid FEN."),
        }

        let castling = parts.next()?;

        if castling != "-" {
            for c in castling.chars() {
                match c {
                    'K' => result.castling_availability[0].0 = true,
                    'Q' => result.castling_availability[0].1 = true,
                    'k' => result.castling_availability[1].0 = true,
                    'q' => result.castling_availability[1].1 = true,
                    _ => {}
                }
            }
        }

        'en_passant: {
            let name = parts.next()?;

            if name.contains('-') {
                break 'en_passant;
            }

            match Square::from_str(name.trim().to_ascii_uppercase().as_str()) {
                Ok(sq) => result.en_passant = sq.bitboard(),
                Err(e) => eprintln!("Failed to load FEN: {}, {:?}", e.to_string(), name),
            }
        };

        result.halfmove_count = parts.next()?.parse().ok()?;
        result.move_count = parts.next()?.parse().ok()?;

        Some(result)
    }
}
