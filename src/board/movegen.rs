use num_enum::FromPrimitive;

use crate::{
    bitboard::{
        BitBoard, BISHOP_MOVE_PATTERNS, BORDER, KING_MOVE_PATTERNS, KNIGHT_MOVE_PATTERNS,
        RANK_BITBOARDS,
    },
    moves::{Move16, Move16Builder, Move32},
    types::{CastlePerm, Color, File, Piece, Rank, Square64},
};

use super::Board;

impl Board {
    pub fn generate_all_moves(&self, list: &mut Vec<Move32>) {
        self.generate_pawn_attacks(list);
        self.generate_en_passant(list);
        self.generate_pawn_pushes(list);
        self.generate_knight_moves(list);
        self.generate_bishop_queen_moves(list);
        self.generate_rook_queen_moves(list);
        self.generate_king_moves(list);
        self.generate_castling_moves(list);
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
            let capture = self.pieces[end];

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
            let capture = self.pieces[end];

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
            let capture = self.pieces[end];

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
            let capture = self.pieces[end];

            if end.rank().unwrap() == Rank::R1 {
                insert_promotions(list, m16, Color::Black, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }
    }

    fn generate_en_passant(&self, list: &mut Vec<Move32>) {
        let Some(en_pas_sq) = self.en_passant else {
            return;
        };

        if self.color == Color::Both {
            return;
        }

        let attacker = match self.color {
            Color::White => Piece::WhitePawn,
            Color::Black => Piece::BlackPawn,
            _ => unreachable!(),
        };

        let captured_piece = match self.color {
            Color::White => Piece::BlackPawn,
            Color::Black => Piece::WhitePawn,
            _ => unreachable!(),
        };

        let mut bb_en_pas = BitBoard::EMPTY;
        bb_en_pas.set(en_pas_sq);

        let mut attacker_east = match self.color {
            Color::White => bb_en_pas.shifted_southeast(),
            Color::Black => bb_en_pas.shifted_northeast(),
            _ => unreachable!(),
        };

        let mut attacker_west = match self.color {
            Color::White => bb_en_pas.shifted_southwest(),
            Color::Black => bb_en_pas.shifted_northwest(),
            _ => unreachable!(),
        };

        if !attacker_west
            .intersection(self.bitboards[attacker])
            .is_empty()
        {
            list.push(Move32::new(
                Move16::build()
                    .start(attacker_west.pop())
                    .end(en_pas_sq)
                    .en_passant()
                    .finish(),
                Some(captured_piece),
            ))
        }

        if !attacker_east
            .intersection(self.bitboards[attacker])
            .is_empty()
        {
            list.push(Move32::new(
                Move16::build()
                    .start(attacker_east.pop())
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
                let capture = self.pieces[end];

                let m = Move16::build().start(start).end(end);
                let m = if capture.is_some() { m.capture() } else { m };

                list.push(Move32::new(m.finish(), capture));
            }
        }
    }

    fn generate_king_moves(&self, list: &mut Vec<Move32>) {
        let start = self.king_square[self.color];
        let targets = KING_MOVE_PATTERNS[start].without(self.bb_all_pieces[self.color]);

        for end in targets.iter_bit_indices() {
            let capture = self.pieces[end];

            let m = Move16::build().start(start).end(end);
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
                self.bitboards[Piece::BlackRook].union(self.bitboards[Piece::BlackQueen])
            }
        };

        let blockers = self.bb_all_pieces[Color::Both];

        for start in rooks_and_queens.iter_bit_indices() {
            let attack_pattern = magic_rook_moves(start, blockers);
            let quiet_moves = attack_pattern.without(self.bb_all_pieces[Color::Both]);
            let captures = attack_pattern.intersection(self.bb_all_pieces[self.color.flipped()]);

            for end in quiet_moves.iter_bit_indices() {
                list.push(Move32::new(
                    Move16::build().start(start).end(end).finish(),
                    None,
                ));
            }

            for end in captures.iter_bit_indices() {
                let capture = self.pieces[end];
                list.push(Move32::new(
                    Move16::build().start(start).end(end).capture().finish(),
                    capture,
                ));
            }
        }
    }

    fn generate_bishop_queen_moves(&self, list: &mut Vec<Move32>) {
        let bishops_and_qeens = match self.color {
            Color::Both => return,
            Color::White => {
                self.bitboards[Piece::WhiteBishop].union(self.bitboards[Piece::WhiteQueen])
            }
            Color::Black => {
                self.bitboards[Piece::BlackBishop].union(self.bitboards[Piece::BlackQueen])
            }
        };

        let blockers = self.bb_all_pieces[Color::Both];

        for start in bishops_and_qeens.iter_bit_indices() {
            let attack_pattern = magic_bishop_moves(start, blockers);
            let quiet_moves = attack_pattern.without(self.bb_all_pieces[Color::Both]);
            let captures = attack_pattern.intersection(self.bb_all_pieces[self.color.flipped()]);

            for end in quiet_moves.iter_bit_indices() {
                list.push(Move32::new(
                    Move16::build().start(start).end(end).finish(),
                    None,
                ));
            }

            for end in captures.iter_bit_indices() {
                let capture = self.pieces[end];
                list.push(Move32::new(
                    Move16::build().start(start).end(end).capture().finish(),
                    capture,
                ));
            }
        }
    }

    fn generate_castling_moves(&self, list: &mut Vec<Move32>) {
        if self.color == Color::White
            && self.castle_perms.get(CastlePerm::WhiteKingside)
            && self.pieces[Square64::F1].is_none()
            && self.pieces[Square64::G1].is_none()
            && !self.is_square_attacked(Square64::E1, Color::Black)
            && !self.is_square_attacked(Square64::F1, Color::Black)
        {
            list.push(Move32::new(
                Move16::build()
                    .start(Square64::E1)
                    .end(Square64::G1)
                    .castle(true)
                    .finish(),
                None,
            ));
        }

        if self.color == Color::White
            && self.castle_perms.get(CastlePerm::WhiteQueenside)
            && self.pieces[Square64::D1].is_none()
            && self.pieces[Square64::C1].is_none()
            && self.pieces[Square64::B1].is_none()
            && !self.is_square_attacked(Square64::E1, Color::Black)
            && !self.is_square_attacked(Square64::D1, Color::Black)
        {
            list.push(Move32::new(
                Move16::build()
                    .start(Square64::E1)
                    .end(Square64::C1)
                    .castle(false)
                    .finish(),
                None,
            ));
        }

        if self.color == Color::Black
            && self.castle_perms.get(CastlePerm::BlackKingside)
            && self.pieces[Square64::F8].is_none()
            && self.pieces[Square64::G8].is_none()
            && !self.is_square_attacked(Square64::E8, Color::White)
            && !self.is_square_attacked(Square64::F8, Color::White)
        {
            list.push(Move32::new(
                Move16::build()
                    .start(Square64::E8)
                    .end(Square64::G8)
                    .castle(true)
                    .finish(),
                None,
            ));
        }

        if self.color == Color::Black
            && self.castle_perms.get(CastlePerm::BlackQueenside)
            && self.pieces[Square64::D8].is_none()
            && self.pieces[Square64::C8].is_none()
            && self.pieces[Square64::B8].is_none()
            && !self.is_square_attacked(Square64::E8, Color::White)
            && !self.is_square_attacked(Square64::D8, Color::White)
        {
            list.push(Move32::new(
                Move16::build()
                    .start(Square64::E8)
                    .end(Square64::C8)
                    .castle(false)
                    .finish(),
                None,
            ));
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// MAGIC TABLES --------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

pub fn magic_bishop_moves(square: Square64, blockers: BitBoard) -> BitBoard {
    let blockers = blockers.intersection(BISHOP_MAGIC_MASKS[square]);
    let key = blockers.to_u64().wrapping_mul(BISHOP_MAGICS[square]);
    let key = key >> (64 - BISHOP_MAGIC_BIT_COUNT[square]);
    BISHOP_ATTACK_TABLE[square][key as usize]
}

pub fn magic_rook_moves(square: Square64, blockers: BitBoard) -> BitBoard {
    let blockers = blockers.intersection(ROOK_MAGIC_MASKS[square]);
    let key = blockers.to_u64().wrapping_mul(ROOK_MAGICS[square]);
    let key = key >> (64 - ROOK_MAGIC_BIT_COUNT[square]);
    ROOK_ATTACK_TABLE[square][key as usize]
}

pub const BISHOP_MAGICS: [u64; 64] =
    unsafe { std::mem::transmute(*include_bytes!("../../bishop_magics")) };

pub const ROOK_MAGICS: [u64; 64] =
    unsafe { std::mem::transmute(*include_bytes!("../../rook_magics")) };

#[rustfmt::skip]
pub const ROOK_MAGIC_BIT_COUNT: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

#[rustfmt::skip]
pub const BISHOP_MAGIC_BIT_COUNT: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

lazy_static::lazy_static! {
    pub static ref ROOK_MAGIC_MASKS: [BitBoard; 64] = {
        let mut boards = [BitBoard::EMPTY; 64];

        for (i, m) in boards.iter_mut().enumerate() {
            let mut result = BitBoard::EMPTY;
            let square = Square64::from_primitive(i);
            let rank = square.rank().unwrap();
            let file = square.file().unwrap();

            if let Some(r) = rank.up() {
                for r in Rank::range_inclusive(r, Rank::R7) {
                    result.set(Square64::from_file_rank(file, r));
                }
            }

            if let Some(r) = rank.down() {
                for r in Rank::range_inclusive(Rank::R2, r) {
                    result.set(Square64::from_file_rank(file, r));
                }
            }

            if let Some(f) = file.up() {
                for f in File::range_inclusive(f, File::G) {
                    result.set(Square64::from_file_rank(f, rank));
                }
            }

            if let Some(f) = file.down() {
                for f in File::range_inclusive(File::B, f) {
                    result.set(Square64::from_file_rank(f, rank));
                }
            }

            *m = result;
        }

        boards
    };

    pub static ref BISHOP_MAGIC_MASKS: [BitBoard; 64] = {
        let mut masks = *BISHOP_MOVE_PATTERNS;

        for m in masks.iter_mut() {
            *m = m.without(*BORDER);
        }

        masks
    };

   pub  static ref ROOK_ATTACK_TABLE: Vec<Vec<BitBoard>> = {
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


    pub static ref BISHOP_ATTACK_TABLE: Vec<Vec<BitBoard>> = {
        let mut table = vec![vec![]; 64];

        for square_num in 0..64 {
            let square = Square64::from_primitive(square_num);
            let mask = BISHOP_MAGIC_MASKS[square_num];
            let permutations = 1 << mask.bit_count();
            let file = square.file().unwrap();
            let rank = square.rank().unwrap();
            table[square].resize(1 << BISHOP_MAGIC_BIT_COUNT[square] as usize, BitBoard::EMPTY);

            for i in 0..permutations {
                let blockers = blocker_permutation(i, mask);
                let mut attack = BitBoard::EMPTY;

                if let Some((r, f)) = rank.up().zip(file.up()) {
                    for (r, f) in std::iter::zip(Rank::range_inclusive(r, Rank::R8), File::range_inclusive(f, File::H)) {
                        attack.set(Square64::from_file_rank(f, r));
                        if blockers.get(Square64::from_file_rank(f, r)) { break; }
                    }
                }

                if let Some((r, f)) = rank.up().zip(file.down()) {
                    for (r, f) in std::iter::zip(Rank::range_inclusive(r, Rank::R8), File::range_inclusive(File::A, f).rev()) {
                        attack.set(Square64::from_file_rank(f, r));
                        if blockers.get(Square64::from_file_rank(f, r)) { break; }
                    }
                }

                if let Some((r, f)) = rank.down().zip(file.up()) {
                    for (r, f) in std::iter::zip(Rank::range_inclusive(Rank::R1, r).rev(), File::range_inclusive(f, File::H)) {
                        attack.set(Square64::from_file_rank(f, r));
                        if blockers.get(Square64::from_file_rank(f, r)) { break; }
                    }
                }

                if let Some((r, f)) = rank.down().zip(file.down()) {
                    for (r, f) in std::iter::zip(Rank::range_inclusive(Rank::R1, r).rev(), File::range_inclusive(File::A, f).rev()) {
                        attack.set(Square64::from_file_rank(f, r));
                        if blockers.get(Square64::from_file_rank(f, r)) { break; }
                    }
                }

                let key = blockers.to_u64().wrapping_mul(BISHOP_MAGICS[square]) >> (64 - BISHOP_MAGIC_BIT_COUNT[square]);
                table[square][key as usize] = attack;
            }
        }

        table
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
            let idx = Square64::from_primitive(mask.to_u64().trailing_zeros() as usize);
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
