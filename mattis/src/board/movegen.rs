use super::Board;
use crate::{
    chess_move::{ChessMove, ChessMoveBuilder},
    tables::{
        BISHOP_MAGICS, BISHOP_MAGIC_BIT_COUNT, BISHOP_MAGIC_MASKS, KING_MOVE_PATTERNS, KNIGHT_MOVE_PATTERNS,
        RANK_BITBOARDS, ROOK_MAGICS, ROOK_MAGIC_BIT_COUNT, ROOK_MAGIC_MASKS,
    },
};
use ctor::ctor;
use mattis_bitboard::BitBoard;
use mattis_types::{CastlePerm, Color, File, Piece, PieceType, Rank, Square, TryFromPrimitive};

pub type MoveList = smallvec::SmallVec<[ChessMove; 128]>;

impl Board {
    pub fn generate_capture_moves(&self, list: &mut MoveList) {
        self.generate_pawn_attacks(list);
        self.generate_en_passant(list);

        self.generate_knight_moves(list, true);
        self.generate_bishop_queen_moves(list, true);
        self.generate_rook_queen_moves(list, true);
        self.generate_king_moves(list, true);
    }

    pub fn generate_all_moves(&self, list: &mut MoveList) {
        self.generate_pawn_attacks(list);
        self.generate_en_passant(list);
        self.generate_pawn_pushes(list);
        self.generate_knight_moves(list, false);
        self.generate_bishop_queen_moves(list, false);
        self.generate_rook_queen_moves(list, false);
        self.generate_king_moves(list, false);
        self.generate_castling_moves(list);
    }

    fn generate_pawn_pushes(&self, list: &mut MoveList) {
        match self.color {
            Color::White => self.generate_white_pawn_pushes(list),
            Color::Black => self.generate_black_pawn_pushes(list),
        }
    }

    fn generate_white_pawn_pushes(&self, list: &mut MoveList) {
        let target_squares_single = self.bitboards[Piece::WhitePawn].shifted_north().without(self.bb_all);

        let target_squares_double = target_squares_single
            .shifted_north()
            .without(self.bb_all)
            .intersection(RANK_BITBOARDS[Rank::R4]);

        for end in target_squares_single.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(-8) };
            let m16 = ChessMove::build().start(start).end(end);

            if end.rank() == Rank::R8 {
                insert_promotions(list, m16, Color::White);
            } else {
                list.push(m16.finish());
            }
        }

        for end in target_squares_double.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(-16) };
            list.push(ChessMove::build().start(start).end(end).double_pawn_push().finish());
        }
    }

    fn generate_black_pawn_pushes(&self, list: &mut MoveList) {
        let target_squares_single = self.bitboards[Piece::BlackPawn].shifted_south().without(self.bb_all);

        let target_squares_double = target_squares_single
            .shifted_south()
            .without(self.bb_all)
            .intersection(RANK_BITBOARDS[Rank::R5]);

        for end in target_squares_single.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(8) };
            let m16 = ChessMove::build().start(start).end(end);

            if end.rank() == Rank::R1 {
                insert_promotions(list, m16, Color::Black);
            } else {
                list.push(m16.finish());
            }
        }

        for end in target_squares_double.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(16) };
            list.push(ChessMove::build().start(start).end(end).double_pawn_push().finish());
        }
    }

    fn generate_pawn_attacks(&self, list: &mut MoveList) {
        match self.color {
            Color::White => self.generate_white_pawn_attacks(list),
            Color::Black => self.generate_black_pawn_attacks(list),
        }
    }

    fn generate_white_pawn_attacks(&self, list: &mut MoveList) {
        let targets_east = self.bitboards[Piece::WhitePawn]
            .shifted_northeast()
            .intersection(self.bb_all_per_color[Color::Black]);

        for end in targets_east.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(-9) };
            let m16 = ChessMove::build().start(start).end(end).capture();

            if end.rank() == Rank::R8 {
                insert_promotions(list, m16, Color::White);
            } else {
                list.push(m16.finish());
            }
        }

        let targets_west = self.bitboards[Piece::WhitePawn]
            .shifted_northwest()
            .intersection(self.bb_all_per_color[Color::Black]);

        for end in targets_west.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(-7) };
            let m16 = ChessMove::build().start(start).end(end).capture();

            if end.rank() == Rank::R8 {
                insert_promotions(list, m16, Color::White);
            } else {
                list.push(m16.finish());
            }
        }
    }

    fn generate_black_pawn_attacks(&self, list: &mut MoveList) {
        let targets_east = self.bitboards[Piece::BlackPawn]
            .shifted_southeast()
            .intersection(self.bb_all_per_color[Color::White]);

        for end in targets_east.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(7) };
            let m16 = ChessMove::build().start(start).end(end).capture();

            if end.rank() == Rank::R1 {
                insert_promotions(list, m16, Color::Black);
            } else {
                list.push(m16.finish());
            }
        }

        let targets_west = self.bitboards[Piece::BlackPawn]
            .shifted_southwest()
            .intersection(self.bb_all_per_color[Color::White]);

        for end in targets_west.iter_bit_indices() {
            // Safety: Always a valid square.
            let start = unsafe { end.add_unchecked(9) };
            let m16 = ChessMove::build().start(start).end(end).capture();

            if end.rank() == Rank::R1 {
                insert_promotions(list, m16, Color::Black);
            } else {
                list.push(m16.finish());
            }
        }
    }

    fn generate_en_passant(&self, list: &mut MoveList) {
        let Some(en_pas_sq) = self.en_passant else { return };

        let attacker = Piece::new(PieceType::Pawn, self.color);

        let mut bb_en_pas = BitBoard::EMPTY;
        bb_en_pas.set(en_pas_sq);

        let attacker_east = match self.color {
            Color::White => bb_en_pas.shifted_southeast(),
            Color::Black => bb_en_pas.shifted_northeast(),
        };

        let attacker_west = match self.color {
            Color::White => bb_en_pas.shifted_southwest(),
            Color::Black => bb_en_pas.shifted_northwest(),
        };

        if let Some(start) = attacker_west.intersection(self.bitboards[attacker]).pop() {
            list.push(ChessMove::build().start(start).end(en_pas_sq).en_passant().finish());
        }

        if let Some(start) = attacker_east.intersection(self.bitboards[attacker]).pop() {
            list.push(ChessMove::build().start(start).end(en_pas_sq).en_passant().finish());
        }
    }

    fn generate_knight_moves(&self, list: &mut MoveList, captures_only: bool) {
        let knights = match self.color {
            Color::White => self.bitboards[Piece::WhiteKnight],
            Color::Black => self.bitboards[Piece::BlackKnight],
        };

        for start in knights.iter_bit_indices() {
            let targets = KNIGHT_MOVE_PATTERNS[start].without(self.bb_all_per_color[self.color]);

            for end in targets.iter_bit_indices() {
                let capture = self.pieces[end];

                if capture.is_none() && captures_only {
                    continue;
                }

                let m = ChessMove::build().start(start).end(end);
                let m = if capture.is_some() { m.capture() } else { m };
                list.push(m.finish());
            }
        }
    }

    fn generate_king_moves(&self, list: &mut MoveList, captures_only: bool) {
        let start = self.king_square[self.color];
        let targets = KING_MOVE_PATTERNS[start].without(self.bb_all_per_color[self.color]);

        for end in targets.iter_bit_indices() {
            let capture = self.pieces[end];

            if capture.is_none() && captures_only {
                continue;
            }

            let m = ChessMove::build().start(start).end(end);
            let m = if capture.is_some() { m.capture() } else { m };

            list.push(m.finish());
        }
    }

    fn generate_rook_queen_moves(&self, list: &mut MoveList, captures_only: bool) {
        let rook_piece = Piece::new(PieceType::Rook, self.color);
        let queen_piece = Piece::new(PieceType::Queen, self.color);
        let rooks_and_queens = self.bitboards[rook_piece].union(self.bitboards[queen_piece]);

        for start in rooks_and_queens.iter_bit_indices() {
            let attack_pattern = magic_rook_moves(start, self.bb_all);
            let quiet_moves = attack_pattern.without(self.bb_all);
            let captures = attack_pattern.intersection(self.bb_all_per_color[self.color.flipped()]);

            for end in captures.iter_bit_indices() {
                list.push(ChessMove::build().start(start).end(end).capture().finish());
            }

            if captures_only {
                continue;
            }

            for end in quiet_moves.iter_bit_indices() {
                list.push(ChessMove::build().start(start).end(end).finish());
            }
        }
    }

    fn generate_bishop_queen_moves(&self, list: &mut MoveList, captures_only: bool) {
        let bishop_piece = Piece::new(PieceType::Bishop, self.color);
        let queen_piece = Piece::new(PieceType::Queen, self.color);
        let bishops_and_queens = self.bitboards[bishop_piece].union(self.bitboards[queen_piece]);

        for start in bishops_and_queens.iter_bit_indices() {
            let attack_pattern = magic_bishop_moves(start, self.bb_all);
            let quiet_moves = attack_pattern.without(self.bb_all);
            let captures = attack_pattern.intersection(self.bb_all_per_color[self.color.flipped()]);

            for end in captures.iter_bit_indices() {
                list.push(ChessMove::build().start(start).end(end).capture().finish());
            }

            if captures_only {
                continue;
            }

            for end in quiet_moves.iter_bit_indices() {
                list.push(ChessMove::build().start(start).end(end).finish());
            }
        }
    }

    fn generate_castling_moves(&self, list: &mut MoveList) {
        if self.color == Color::White
            && self.castle_perms.get(CastlePerm::WhiteKingside)
            && self.pieces[Square::F1].is_none()
            && self.pieces[Square::G1].is_none()
            && !self.is_square_attacked(Square::E1, Color::Black)
            && !self.is_square_attacked(Square::F1, Color::Black)
        {
            list.push(
                ChessMove::build()
                    .start(Square::E1)
                    .end(Square::G1)
                    .castle(true)
                    .finish(),
            );
        }

        if self.color == Color::White
            && self.castle_perms.get(CastlePerm::WhiteQueenside)
            && self.pieces[Square::D1].is_none()
            && self.pieces[Square::C1].is_none()
            && self.pieces[Square::B1].is_none()
            && !self.is_square_attacked(Square::E1, Color::Black)
            && !self.is_square_attacked(Square::D1, Color::Black)
        {
            list.push(
                ChessMove::build()
                    .start(Square::E1)
                    .end(Square::C1)
                    .castle(false)
                    .finish(),
            );
        }

        if self.color == Color::Black
            && self.castle_perms.get(CastlePerm::BlackKingside)
            && self.pieces[Square::F8].is_none()
            && self.pieces[Square::G8].is_none()
            && !self.is_square_attacked(Square::E8, Color::White)
            && !self.is_square_attacked(Square::F8, Color::White)
        {
            list.push(
                ChessMove::build()
                    .start(Square::E8)
                    .end(Square::G8)
                    .castle(true)
                    .finish(),
            );
        }

        if self.color == Color::Black
            && self.castle_perms.get(CastlePerm::BlackQueenside)
            && self.pieces[Square::D8].is_none()
            && self.pieces[Square::C8].is_none()
            && self.pieces[Square::B8].is_none()
            && !self.is_square_attacked(Square::E8, Color::White)
            && !self.is_square_attacked(Square::D8, Color::White)
        {
            list.push(
                ChessMove::build()
                    .start(Square::E8)
                    .end(Square::C8)
                    .castle(false)
                    .finish(),
            );
        }
    }
}

pub fn magic_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let blockers = blockers.intersection(BISHOP_MAGIC_MASKS[square]);
    let key = blockers.to_u64().wrapping_mul(BISHOP_MAGICS[square]);
    let key = key >> (64 - BISHOP_MAGIC_BIT_COUNT[square]);

    // Safety: `square` is always in a valid range (0-64)
    let table_row = unsafe { BISHOP_ATTACK_TABLE.get_unchecked(square as u8 as usize) };

    // Safety: `key` is always in a valid range
    unsafe { *table_row.get_unchecked(key as usize) }
}

pub fn magic_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let blockers = blockers.intersection(ROOK_MAGIC_MASKS[square]);
    let key = blockers.to_u64().wrapping_mul(ROOK_MAGICS[square]);
    let key = key >> (64 - ROOK_MAGIC_BIT_COUNT[square]);

    // Safety: `square` is always in a valid range (0-64)
    let table_row = unsafe { ROOK_ATTACK_TABLE.get_unchecked(square as u8 as usize) };

    // Safety: `key` is always in a valid range
    unsafe { *table_row.get_unchecked(key as usize) }
}

#[ctor]
static ROOK_ATTACK_TABLE: Vec<Vec<BitBoard>> = {
    let mut table = vec![vec![]; 64];

    for (square, square_entry) in table.iter_mut().enumerate() {
        let square = Square::try_from_primitive(square as u8).unwrap();
        let mask = ROOK_MAGIC_MASKS[square];
        let permutations = 1 << mask.bit_count();
        let file = square.file();
        let rank = square.rank();
        square_entry.resize(1 << ROOK_MAGIC_BIT_COUNT[square] as usize, BitBoard::EMPTY);

        for i in 0..permutations {
            let blockers = blocker_permutation(i, mask);
            let mut attack = BitBoard::EMPTY;

            if let Some(r) = rank.up() {
                for r in Rank::range_inclusive(r, Rank::R8) {
                    attack.set(Square::from_file_rank(file, r));
                    if blockers.get(Square::from_file_rank(file, r)) {
                        break;
                    }
                }
            }

            if let Some(r) = rank.down() {
                for r in Rank::range_inclusive(Rank::R1, r).rev() {
                    attack.set(Square::from_file_rank(file, r));
                    if blockers.get(Square::from_file_rank(file, r)) {
                        break;
                    }
                }
            }

            if let Some(f) = file.up() {
                for f in File::range_inclusive(f, File::H) {
                    attack.set(Square::from_file_rank(f, rank));
                    if blockers.get(Square::from_file_rank(f, rank)) {
                        break;
                    }
                }
            }

            if let Some(f) = file.down() {
                for f in File::range_inclusive(File::A, f).rev() {
                    attack.set(Square::from_file_rank(f, rank));
                    if blockers.get(Square::from_file_rank(f, rank)) {
                        break;
                    }
                }
            }

            let key = blockers.to_u64().wrapping_mul(ROOK_MAGICS[square]) >> (64 - ROOK_MAGIC_BIT_COUNT[square]);
            square_entry[key as usize] = attack;
        }
    }

    table
};

#[ctor]
static BISHOP_ATTACK_TABLE: Vec<Vec<BitBoard>> = {
    let mut table = vec![vec![]; 64];

    for (square, square_entry) in table.iter_mut().enumerate() {
        let square = Square::try_from_primitive(square as u8).unwrap();
        let mask = BISHOP_MAGIC_MASKS[square];
        let permutations = 1 << mask.bit_count();
        let file = square.file();
        let rank = square.rank();
        square_entry.resize(1 << BISHOP_MAGIC_BIT_COUNT[square] as usize, BitBoard::EMPTY);

        for i in 0..permutations {
            let blockers = blocker_permutation(i, mask);
            let mut attack = BitBoard::EMPTY;

            if let Some((r, f)) = rank.up().zip(file.up()) {
                for (r, f) in std::iter::zip(Rank::range_inclusive(r, Rank::R8), File::range_inclusive(f, File::H)) {
                    attack.set(Square::from_file_rank(f, r));
                    if blockers.get(Square::from_file_rank(f, r)) {
                        break;
                    }
                }
            }

            if let Some((r, f)) = rank.up().zip(file.down()) {
                for (r, f) in std::iter::zip(
                    Rank::range_inclusive(r, Rank::R8),
                    File::range_inclusive(File::A, f).rev(),
                ) {
                    attack.set(Square::from_file_rank(f, r));
                    if blockers.get(Square::from_file_rank(f, r)) {
                        break;
                    }
                }
            }

            if let Some((r, f)) = rank.down().zip(file.up()) {
                for (r, f) in std::iter::zip(
                    Rank::range_inclusive(Rank::R1, r).rev(),
                    File::range_inclusive(f, File::H),
                ) {
                    attack.set(Square::from_file_rank(f, r));
                    if blockers.get(Square::from_file_rank(f, r)) {
                        break;
                    }
                }
            }

            if let Some((r, f)) = rank.down().zip(file.down()) {
                for (r, f) in std::iter::zip(
                    Rank::range_inclusive(Rank::R1, r).rev(),
                    File::range_inclusive(File::A, f).rev(),
                ) {
                    attack.set(Square::from_file_rank(f, r));
                    if blockers.get(Square::from_file_rank(f, r)) {
                        break;
                    }
                }
            }

            let key = blockers.to_u64().wrapping_mul(BISHOP_MAGICS[square]) >> (64 - BISHOP_MAGIC_BIT_COUNT[square]);
            square_entry[key as usize] = attack;
        }
    }

    table
};

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// UTILITY -------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

fn blocker_permutation(mut i: usize, mut mask: BitBoard) -> BitBoard {
    let mut blockers = BitBoard::EMPTY;

    while i != 0 {
        if (i & 1) != 0 {
            let idx = Square::try_from_primitive(mask.to_u64().trailing_zeros() as u8).unwrap();
            blockers.set(idx);
        }

        i >>= 1;
        mask.silent_pop();
    }

    blockers
}

fn insert_promotions(list: &mut MoveList, builder: ChessMoveBuilder, color: Color) {
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
        list.push(builder.promote(p).finish());
    }
}
