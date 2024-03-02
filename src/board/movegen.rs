use num_enum::FromPrimitive;

use crate::{
    bitboard::{
        BitBoard, KING_MOVE_PATTERNS, KNIGHT_MOVE_PATTERNS, RANK_BITBOARDS, ROOK_MAGICS,
        ROOK_MAGIC_BIT_COUNT, ROOK_MAGIC_MASKS,
    },
    moves::{Move16, Move16Builder, Move32},
    types::{Color, File, Piece, Rank, Square120, Square64},
};

use super::Board;

impl Board {
    pub fn generate_all_moves(&self) -> Vec<Move32> {
        let mut list = vec![];

        self.generate_pawn_pushes(&mut list);
        self.generate_pawn_attacks(&mut list);
        self.generate_en_passant(&mut list);
        self.generate_knight_moves(&mut list);
        self.generate_king_moves(&mut list);
        self.generate_rook_queen_moves(&mut list);

        list
    }

    fn generate_pawn_pushes(&self, list: &mut Vec<Move32>) {
        match self.color {
            Color::White => self.generate_white_pawn_pushes(list),
            Color::Black => self.generate_black_pawn_pushes(list),
            _ => (),
        }
    }

    fn generate_white_pawn_pushes(&self, list: &mut Vec<Move32>) {
        let target_squares_single = self.bitboards[Piece::WhitePawn]
            .shifted_north()
            .without(self.bb_all_pieces[Color::Both]);

        let target_squares_double = target_squares_single
            .shifted_north()
            .without(self.bb_all_pieces[Color::Both])
            .intersection(RANK_BITBOARDS[Rank::R4]);

        for end in target_squares_single.iter_bit_indices() {
            let m16 = Move16::build().start(end - 8usize).end(end);

            if end.rank().unwrap() == Rank::R8 {
                insert_promotions(list, m16, Color::White, None);
            } else {
                list.push(Move32::new(m16.finish(), None));
            }
        }

        for end in target_squares_double.iter_bit_indices() {
            list.push(Move32::new(
                Move16::build()
                    .start(end - 16usize)
                    .end(end)
                    .double_pawn_push()
                    .finish(),
                None,
            ));
        }
    }

    fn generate_black_pawn_pushes(&self, list: &mut Vec<Move32>) {
        let target_squares_single = self.bitboards[Piece::BlackPawn]
            .shifted_south()
            .without(self.bb_all_pieces[Color::Both]);

        let target_squares_double = target_squares_single
            .shifted_south()
            .without(self.bb_all_pieces[Color::Both])
            .intersection(RANK_BITBOARDS[Rank::R5]);

        for end in target_squares_single.iter_bit_indices() {
            let m16 = Move16::build().start(end + 8usize).end(end);

            if end.rank().unwrap() == Rank::R1 {
                insert_promotions(list, m16, Color::Black, None);
            } else {
                list.push(Move32::new(m16.finish(), None));
            }
        }

        for end in target_squares_double.iter_bit_indices() {
            list.push(Move32::new(
                Move16::build()
                    .start(end + 16usize)
                    .end(end)
                    .double_pawn_push()
                    .finish(),
                None,
            ));
        }
    }

    fn generate_pawn_attacks(&self, list: &mut Vec<Move32>) {
        match self.color {
            Color::White => self.generate_white_pawn_attacks(list),
            Color::Black => self.generate_black_pawn_attacks(list),
            _ => (),
        }
    }

    fn generate_white_pawn_attacks(&self, list: &mut Vec<Move32>) {
        let targets_east = self.bitboards[Piece::WhitePawn]
            .shifted_northeast()
            .intersection(self.bb_all_pieces[Color::Black]);

        for end in targets_east.iter_bit_indices() {
            let m16 = Move16::build().start(end - 9usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R8 {
                insert_promotions(list, m16, Color::White, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }

        let targets_west = self.bitboards[Piece::WhitePawn]
            .shifted_northwest()
            .intersection(self.bb_all_pieces[Color::Black]);

        for end in targets_west.iter_bit_indices() {
            let m16 = Move16::build().start(end - 7usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R8 {
                insert_promotions(list, m16, Color::White, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }
    }

    fn generate_black_pawn_attacks(&self, list: &mut Vec<Move32>) {
        let targets_east = self.bitboards[Piece::BlackPawn]
            .shifted_southeast()
            .intersection(self.bb_all_pieces[Color::White]);

        for end in targets_east.iter_bit_indices() {
            let m16 = Move16::build().start(end + 7usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R1 {
                insert_promotions(list, m16, Color::Black, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }

        let targets_west = self.bitboards[Piece::BlackPawn]
            .shifted_southwest()
            .intersection(self.bb_all_pieces[Color::White]);

        for end in targets_west.iter_bit_indices() {
            let m16 = Move16::build().start(end + 9usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R1 {
                insert_promotions(list, m16, Color::Black, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }
    }

    fn generate_en_passant(&self, list: &mut Vec<Move32>) {
        let Some(en_pas_sq) = self.en_passant else { return };

        if self.color == Color::Both {
            return;
        }

        let candidates = match self.color {
            Color::White => [en_pas_sq - 9usize, en_pas_sq - 11usize],
            Color::Black => [en_pas_sq + 9usize, en_pas_sq + 11usize],
            _ => unreachable!(),
        };

        let moving_piece = match self.color {
            Color::White => Piece::WhitePawn,
            Color::Black => Piece::BlackPawn,
            _ => unreachable!(),
        };

        let captured_piece = match self.color {
            Color::White => Piece::BlackPawn,
            Color::Black => Piece::WhitePawn,
            _ => unreachable!(),
        };

        for candidate in candidates {
            if self.pieces[candidate] != Some(moving_piece) {
                continue;
            }

            let candidate = Square64::try_from(candidate).unwrap();
            let en_pas_sq = Square64::try_from(en_pas_sq).unwrap();

            list.push(Move32::new(
                Move16::build()
                    .start(candidate)
                    .end(en_pas_sq)
                    .en_passant()
                    .finish(),
                Some(captured_piece),
            ))
        }
    }

    fn generate_knight_moves(&self, list: &mut Vec<Move32>) {
        let knights = match self.color {
            Color::Both => return,
            Color::White => self.bitboards[Piece::WhiteKnight],
            Color::Black => self.bitboards[Piece::BlackKnight],
        };

        for start in knights.iter_bit_indices() {
            let targets = KNIGHT_MOVE_PATTERNS[start].without(self.bb_all_pieces[self.color]);

            for end in targets.iter_bit_indices() {
                let capture = self.pieces[Square120::try_from(end).unwrap()];

                let m = Move16::build().start(start).end(end);
                let m = if capture.is_some() { m.capture() } else { m };

                list.push(Move32::new(m.finish(), capture));
            }
        }
    }

    fn generate_king_moves(&self, list: &mut Vec<Move32>) {
        let start120 = self.king_square[self.color];
        let Ok(start64) = Square64::try_from(start120) else { return };
        let targets = KING_MOVE_PATTERNS[start64].without(self.bb_all_pieces[self.color]);

        for end in targets.iter_bit_indices() {
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            let m = Move16::build().start(start64).end(end);
            let m = if capture.is_some() { m.capture() } else { m };

            list.push(Move32::new(m.finish(), capture));
        }
    }

    fn generate_rook_queen_moves(&self, list: &mut Vec<Move32>) {
        let rooks_and_queens = match self.color {
            Color::Both => return,
            Color::White => {
                self.bitboards[Piece::WhiteRook].union(self.bitboards[Piece::WhiteQueen])
            }
            Color::Black => {
                self.bitboards[Piece::BlackRook].union(self.bitboards[Piece::WhiteQueen])
            }
        };

        let blockers = self.bb_all_pieces[Color::Both];

        for start in rooks_and_queens.iter_bit_indices() {
            let blockers = blockers.intersection(ROOK_MAGIC_MASKS[start]);
            let key = blockers.to_u64().wrapping_mul(ROOK_MAGICS[start]);
            let key = key >> (64 - ROOK_MAGIC_BIT_COUNT[start]);
            let attack_pattern = ROOK_ATTACK_TABLE[start][key as usize];
            let quiet_moves = attack_pattern.without(self.bb_all_pieces[Color::Both]);
            let captures = attack_pattern.intersection(self.bb_all_pieces[self.color.flipped()]);

            for end in quiet_moves.iter_bit_indices() {
                list.push(Move32::new(
                    Move16::build().start(start).end(end).finish(),
                    None,
                ));
            }

            for end in captures.iter_bit_indices() {
                let capture = self.pieces[Square120::try_from(end).unwrap()];
                list.push(Move32::new(
                    Move16::build().start(start).end(end).capture().finish(),
                    capture,
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ATTACK TABLES -------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

lazy_static::lazy_static! {
    static ref ROOK_ATTACK_TABLE: Vec<Vec<BitBoard>> = {
        let mut table = vec![vec![]; 64];

        for square_num in 0..64 {
            let square = Square64::from_primitive(square_num);
            let mask = ROOK_MAGIC_MASKS[square_num];
            let permutations = 1 << mask.bit_count();
            let file = square.file().unwrap();
            let rank = square.rank().unwrap();
            table[square].resize(1 << ROOK_MAGIC_BIT_COUNT[square] as usize, BitBoard::EMPTY);

            for i in 0..permutations {
                let blockers = blocker_permutation(i, mask);
                let mut attack = BitBoard::EMPTY;

                if let Some(r) = rank.up() {
                    for r in Rank::range_inclusive(r, Rank::R8) {
                        attack.set(Square64::from_file_rank(file, r));
                        if blockers.get(Square64::from_file_rank(file, r)) { break; }
                    }
                }

                if let Some(r) = rank.down() {
                    for r in Rank::range_inclusive(Rank::R1, r).rev() {
                        attack.set(Square64::from_file_rank(file, r));
                        if blockers.get(Square64::from_file_rank(file, r)) { break; }
                    }
                }

                if let Some(f) = file.up() {
                    for f in File::range_inclusive(f, File::H) {
                        attack.set(Square64::from_file_rank(f, rank));
                        if blockers.get(Square64::from_file_rank(f, rank)) { break; }
                    }
                }

                if let Some(f) = file.down() {
                    for f in File::range_inclusive(File::A, f).rev() {
                        attack.set(Square64::from_file_rank(f, rank));
                        if blockers.get(Square64::from_file_rank(f, rank)) { break; }
                    }
                }

                let key = blockers.to_u64().wrapping_mul(ROOK_MAGICS[square]) >> (64 - ROOK_MAGIC_BIT_COUNT[square]);
                table[square][key as usize] = attack;
            }
        }

        table
    };

    static ref BISHOP_ATTACK_TABLE: [[BitBoard; 1 << 9]; 64] = {
        todo!()
    };
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// UTILITY -------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

fn blocker_permutation(mut i: usize, mut mask: BitBoard) -> BitBoard {
    let mut blockers = BitBoard::EMPTY;

    while i != 0 {
        if (i & 1) != 0 {
            let idx = Square64::try_from(mask.to_u64().trailing_zeros() as usize).unwrap();
            blockers.set(idx);
        }

        i >>= 1;
        mask.silent_pop();
    }

    blockers
}

fn insert_promotions(
    list: &mut Vec<Move32>,
    builder: Move16Builder,
    color: Color,
    capture: Option<Piece>,
) {
    let pieces = if color == Color::White {
        [
            Piece::WhiteKnight,
            Piece::WhiteBishop,
            Piece::WhiteRook,
            Piece::WhiteQueen,
        ]
    } else {
        [
            Piece::BlackKnight,
            Piece::BlackBishop,
            Piece::BlackRook,
            Piece::BlackQueen,
        ]
    };

    for p in pieces {
        list.push(Move32::new(builder.promote(p).finish(), capture));
    }
}
