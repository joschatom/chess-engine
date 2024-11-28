use std::{cell::LazyCell, collections::HashMap};

use crate::{
    bitboard::BitBoard,
    hardcoded_moves::KNIGHT_MOVES,
    piece::{self, Color, Piece},
    r#move::{CastlingMethod, Move, MoveFlag},
    square::*,
    utils::print_bitboard,
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
        self.0[(color as u8) as usize]
            & (self.0[(Piece::Bishop as u8) as usize]
                | self.0[(Piece::Rook as u8) as usize]
                | self.0[(Piece::Queen as u8) as usize])
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

    pub fn insert_piece(&mut self, square: Square, piece: Piece, color: Color) {
        let mask = 1 << ((square.rank() as usize) * 8 + (square.file() as usize));

        self.0[(color as u8) as usize] |= BitBoard(mask);

        self.0[(piece as u8) as usize] |=
            BitBoard(1 << ((square.rank() as usize) * 8 + (square.file() as usize)));
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub bitboards: BitBoards,
    pub turn: Color,
    pub castling_availability: [(bool, bool); 2],
    pub en_passant: BitBoard,
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
            halfmove_count: 0,
            move_count: 1,
            move_filters: [BitBoard::EMPTY; 2],
        }
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

    pub fn frendly_pieces(&self, color: Color) -> BitBoard {
        self.bitboards.all_pieces(Some(color))
    }

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

    fn is_double_move(color: Color, mv: Move) -> bool{
        mv.starting_square.bitboard().forward(color).forward(color).0
            & mv.target_square.bitboard().0 != 0
    }

    pub fn do_move(&mut self, mv: Move) {
        let piece = self.squares[mv.starting_square as usize]
            .expect("Invalid Move")
            .1;

        if piece == Piece::Pawn && Self::is_double_move(self.turn, mv) {
            self.en_passant = mv.starting_square.bitboard().forward(self.turn);
        }else {
            self.en_passant = BitBoard::EMPTY;
        }

        match mv.flag {
            MoveFlag::EnPassant => {
                assert!(piece == Piece::Pawn, "only pawns can do en passant!");

                let other_pawn = Square::index(mv.target_square.bitboard().backward(self.turn).0.trailing_zeros() as usize);
            
                self.bitboards.remove_piece(Piece::Pawn, self.turn.opponent(), other_pawn);

                self.squares[other_pawn as usize] = None;

                self.bitboards.r#move(self.turn, piece, mv);
                self.squares[mv.target_square as usize] = Some((self.turn, piece));
            }

            MoveFlag::Capture(target) => {
                self.bitboards
                    .remove_piece(target, self.turn.opponent(), mv.target_square);
                self.bitboards.r#move(self.turn, piece, mv);
                self.squares[mv.target_square as usize] = Some((self.turn, piece));
            }

            MoveFlag::Promotion(target) => {
                self.bitboards.0[piece as usize].0 &= !mv.starting_square.bitboard().0;
                self.bitboards.0[target as usize].0 |= mv.target_square.bitboard().0;
                self.squares[mv.target_square as usize] = Some((self.turn, target));
            }
            MoveFlag::Castle(method) => {
                let (king_target, rook_target) = Self::castling_squares(self.turn, method);

                // King
                self.bitboards.r#move(
                    self.turn,
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
                self.bitboards.r#move(
                    self.turn,
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
            _ => {
                self.bitboards.r#move(self.turn, piece, mv);
                self.squares[mv.target_square as usize] = Some((self.turn, piece));
            }
        }

        if mv.flag != MoveFlag::NullMove {
            self.squares[mv.starting_square as usize] = None;
        }

        self.bitboards.0[BitBoards::ad_bitboard(self.turn.opponent())] = BitBoard::EMPTY;

        self.move_filters = [BitBoard::EMPTY; 2];

        self.move_count += 1;

        // halfmove clock!!

        self.turn = self.turn.opponent();
    }

    pub fn generate_moves(&mut self, color: Color) -> Vec<Move> {
        let mut move_bitboards: HashMap<Square, BitBoard> = HashMap::new();
        let mut out = vec![];

        let pawns = self.bitboards.get_piece_set(Piece::Pawn, Some(color));

        let king_moves = self.king_moves(color) & !self.frendly_pieces(color);

        self.bitboards.0[BitBoards::ad_bitboard(color)] |= king_moves;
        move_bitboards.insert(self.king_square(color), king_moves);

        let (_pinners, pinned, checkers);

        (_pinners, pinned, checkers) = self.pinned_pieces(color);

        for pawn in Self::isolate_pieces(pawns) {
            let square = Square::index(pawn.0.trailing_zeros() as _);

            let (ad, moves) = self.pawn_moves(pawn, color);
            move_bitboards.insert(square, moves & !self.frendly_pieces(color));
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

                moves = moves & !self.frendly_pieces(color);

                move_bitboards.insert(square, moves);

                self.bitboards.0[BitBoards::ad_bitboard(color)] |= moves; // Should this be a side-effect or not?
            }
        }

        println!();

        for knight in Self::isolate_pieces(self.bitboards.get_piece_set(Piece::Knight, Some(color)))
        {
            let sq = Square::index(knight.0.trailing_zeros() as _);

            let moves = KNIGHT_MOVES[sq as usize] & !self.frendly_pieces(color);

            self.bitboards.0[BitBoards::ad_bitboard(color)] |= moves;

            move_bitboards.insert(sq, moves);
        }

        if self.can_castle_short(color) && !self.in_check(color) {
            out.push(Move {
                starting_square: self.king_square(color),
                target_square: Square::A1,
                flag: MoveFlag::Castle(CastlingMethod::Short),
            });
        }

        if self.can_castle_long(color) & !self.in_check(color) {
            out.push(Move {
                starting_square: self.king_square(color),
                target_square: Square::A1,
                flag: MoveFlag::Castle(CastlingMethod::Long),
            });
        }

        'conv: for (sq, bitboard) in move_bitboards {
            if sq.bitboard() & pinned != BitBoard::EMPTY {
                println!(
                    "Filtered Move: {}",
                    Move {
                        starting_square: sq,
                        target_square: Square::A1,
                        flag: MoveFlag::Untargeted,
                    }
                );
                continue 'conv;
            }

            for target_sq in bitboard.active_squares() {

                let piece = self.squares[sq as usize]
                    .map(|(_, p)| p)
                    .expect("Piece exist in bitboard but not in simple board.");

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
                        if (target_sq.bitboard() & !rays).0 == 0 {
                            println!(
                                "Filtered Move: {}",
                                Move {
                                    starting_square: sq,
                                    target_square: target_sq,
                                    flag: MoveFlag::None,
                                }
                            );
                            continue;
                        }
                    } else if checks > 1 && piece != Piece::King {
                        println!(
                            "Filtered Move: {}",
                            Move {
                                starting_square: sq,
                                target_square: target_sq,
                                flag: MoveFlag::None,
                            }
                        );
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

                if target_sq.bitboard() & self.frendly_pieces(color.opponent()) != BitBoard::EMPTY {
                    out.push(Move {
                        starting_square: sq,
                        target_square: target_sq,
                        flag: MoveFlag::Capture(self.squares[target_sq as usize].unwrap().1),
                    });
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

        let square = Square::index(pawn.0.trailing_zeros() as _);

        let mut moves = pawn.forward(color) & !self.blockers();

        if (color == Color::White && square.rank() == Rank::Second)
            || (color == Color::Black && square.rank() == Rank::Seventh)
        {
            moves |= pawn.forward(color).forward(color)
                & !(self.blockers() | self.blockers().forward(color))
        }

        let capture_mask: BitBoard = match square.file() {
            File::A => pawn.forward(color).shl(1),
            File::H => pawn.forward(color).shr(1),
            _ => pawn.forward(color).shl(1) | pawn.forward(color).shr(1),
        };

        moves |= self.bitboards.all_pieces(Some(color.opponent())) & capture_mask;
        moves |= self.en_passant & capture_mask;

        // TODO: En-passant

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
        while position != 0 && square.file() != File::H {
            steps += 1;

            position = position >> 1;
            if position & relevant_blockers.0 != 0 {
                break;
            }

            let target_file = File::index((square.file() as usize) + steps);

            out |= BitBoard(1 << (Square::new(target_file, rank) as u64));

            if position & BitBoard::CORNERS.0 != 0 {
                break;
            }
        }

        steps = 0;
        position = start_position;

        // RIGHT
        while position != 0 && square.file() != File::A {
            steps += 1;

            position = position << 1;
            if position & relevant_blockers.0 != 0 {
                break;
            }

            let target_file = File::index((square.file() as usize) + steps);

            out |= BitBoard(1 << (Square::new(target_file, rank) as u64));

            if position & BitBoard::CORNERS.0 != 0 {
                break;
            }
        }

        // UP
        while position != 0 && rank as usize != 7 {
            steps += 1;

            if steps + (rank as usize) == 8 {
                break;
            }

            position = position >> 8;
            if position & relevant_blockers.0 != 0 {
                break;
            }

            let target_rank = Rank::index((rank as usize) + steps);

            out |= BitBoard(1 << (Square::new(square.file(), target_rank) as u64));
        }

        steps = 0;
        position = start_position;

        // DOWN
        while position != 0 && rank as usize != 0 {
            steps += 1;

            position = position << 8;

            if position & relevant_blockers.0 != 0 {
                break;
            }

            let target_rank = Rank::index((rank as usize).saturating_sub(steps));

            out |= BitBoard(1 << (Square::new(square.file(), target_rank) as u64));
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

            for _ in 1..8 {
                position = (position << slider.left) >> slider.right;

                out |= BitBoard(position);

                if position & relevant_blockers.0 != 0 {
                    break;
                }

                if position & BitBoard::CORNERS.0 != 0 {
                    break;
                }
            }
        }

        out & BitBoard(!self.bitboards.all_pieces(Some(color)).0)
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
    pub fn pinned_pieces(&mut self, color: Color) -> (BitBoard, BitBoard, (u32, BitBoard)) {
        let mut out = (BitBoard::EMPTY, BitBoard::EMPTY, (0, BitBoard::EMPTY));

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

        out.2 .0 += checkers.0.count_ones();
        out.2 .1 |= checkers;

        for slider in Piece::Queen.sliders().unwrap() {
            position = self.bitboards.get_piece_set(Piece::King, Some(color));

            let mut pinned = None;

            let mut ray = position;

            for _index in 1..8 {
                position = BitBoard((position.0 << slider.left) >> slider.right);

                ray |= position;

                if position & self.frendly_pieces(color) != BitBoard::EMPTY {
                    if pinned.is_some() {
                        break;
                    }

                    pinned = Some(Square::index(position.0.trailing_zeros() as _));
                }

                let enemy_piece = position & self.bitboards.sliding_pieces(color.opponent());

                if enemy_piece != BitBoard::EMPTY {
                    // In Check
                    if pinned.is_none() {
                        out.2 .1 |= ray;
                        out.2 .0 += 1;

                        break;
                    }

                    out.0 |= position;
                    out.1 |= pinned.unwrap().bitboard();
                }

                if position & BitBoard::CORNERS != BitBoard::EMPTY {
                    break;
                }
            }
        }

        out
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

        assert!(board.count_ones() == 1);

        let square = Square::index(board.trailing_zeros() as usize);

        Piece::King.possible_moves(square)
            & BitBoard(!self.bitboards.0[BitBoards::ad_bitboard(color.opponent())].0)
            & BitBoard(!board)
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

        // TODO: EN PASSANT

        _ = parts.next()?;

        result.halfmove_count = parts.next()?.parse().ok()?;
        result.move_count = parts.next()?.parse().ok()?;

        Some(result)
    }
}
