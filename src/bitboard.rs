use std::fmt::{Debug, Display};

use crate::types::{File, Rank, Square64};
use num_enum::{FromPrimitive, UnsafeFromPrimitive};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u64::MAX);

    pub fn from_u64(v: u64) -> Self {
        Self(v)
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(self) -> bool {
        self.0 == u64::MAX
    }

    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn complement(self) -> Self {
        Self(!self.0)
    }

    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub fn set(&mut self, idx: Square64) {
        let idx: usize = idx.into();

        if let Some(v) = 1u64.checked_shl(idx as u32) {
            self.0 |= v;
        }
    }

    pub fn set_to(&mut self, idx: Square64, value: bool) {
        if value {
            self.set(idx);
        } else {
            self.clear(idx);
        }
    }

    pub fn clear(&mut self, idx: Square64) {
        let idx: usize = idx.into();

        if let Some(v) = 1u64.checked_shl(idx as u32) {
            self.0 &= !v;
        }
    }

    pub fn get(&self, idx: Square64) -> bool {
        let idx: usize = idx.into();

        if let Some(v) = 1u64.checked_shl(idx as u32) {
            (self.0 & v) > 0
        } else {
            false
        }
    }

    pub fn silent_pop(&mut self) {
        self.0 &= self.0 - 1;
    }

    /// Clears the least significant 1-bit and returns its index

    pub fn pop(&mut self) -> Square64 {
        #[rustfmt::skip]
        const POP_MAGIC_TABLE: [usize ; 64] = [
            63, 30,  3, 32, 25, 41, 22, 33,
            15, 50, 42, 13, 11, 53, 19, 34,
            61, 29,  2, 51, 21, 43, 45, 10,
            18, 47,  1, 54,  9, 57,  0, 35,
            62, 31, 40,  4, 49,  5, 52, 26,
            60,  6, 23, 44, 46, 27, 56, 16,
             7, 39, 48, 24, 59, 14, 12, 55,
            38, 28, 58, 20, 37, 17, 36,  8,
        ];

        if self.is_empty() {
            return Square64::Invalid;
        }

        let b = self.0 ^ (self.0 - 1);
        let fold: u32 = ((b & u64::MAX) ^ (b >> 32)) as u32;
        self.0 &= self.0 - 1;

        let idx = POP_MAGIC_TABLE[(fold.wrapping_mul(0x783a9b23) >> 26) as usize];
        Square64::from_primitive(idx)
    }

    pub fn iter_bit_indices(self) -> impl Iterator<Item = Square64> {
        let mut b = self;
        std::iter::from_fn(move || {
            let sq = b.pop();

            if sq == Square64::Invalid {
                None
            } else {
                Some(sq)
            }
        })
    }

    pub fn bit_count(self) -> u32 {
        self.0.count_ones()
    }

    #[must_use]
    pub fn shifted_north(self) -> Self {
        Self(self.0 << 8)
    }

    #[must_use]
    pub fn shifted_south(self) -> Self {
        Self(self.0 >> 8)
    }

    #[must_use]
    pub fn shifted_east(self) -> Self {
        Self((self.0 << 1) & NOT_FILE_BITBOARDS[File::A].to_u64())
    }

    #[must_use]
    pub fn shifted_west(self) -> Self {
        Self((self.0 >> 1) & NOT_FILE_BITBOARDS[File::H].to_u64())
    }

    #[must_use]
    pub fn shifted_northeast(self) -> Self {
        Self((self.0 << 9) & NOT_FILE_BITBOARDS[File::A].to_u64())
    }

    #[must_use]
    pub fn shifted_southeast(self) -> Self {
        Self((self.0 >> 7) & NOT_FILE_BITBOARDS[File::A].to_u64())
    }

    #[must_use]
    pub fn shifted_southwest(self) -> Self {
        Self((self.0 >> 9) & NOT_FILE_BITBOARDS[File::H].to_u64())
    }

    #[must_use]
    pub fn shifted_northwest(self) -> Self {
        Self((self.0 << 7) & NOT_FILE_BITBOARDS[File::H].to_u64())
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            let rank = unsafe { Rank::unchecked_transmute_from(rank) };
            for file in 0..8 {
                let file = unsafe { File::unchecked_transmute_from(file) };
                let sq = Square64::from_file_rank(file, rank);

                write!(f, "{} ", if self.get(sq) { "X" } else { "." })?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// # CONST BITBOARD TABLES ---------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

lazy_static::lazy_static! {
    pub static ref FILE_BITBOARDS: [BitBoard; 8] = {
        let mut boards = [BitBoard::EMPTY; 8];

        for f in File::iter_all() {
            for r in Rank::iter_all() {
                boards[f].set(Square64::from_file_rank(f, r));
            }
        }

        boards
    };

    pub static ref NOT_FILE_BITBOARDS: [BitBoard; 8] = {
        let mut boards = *FILE_BITBOARDS;

        for m in &mut boards {
            *m = m.complement();
        }

        boards
    };

    pub static ref RANK_BITBOARDS: [BitBoard; 8] = {
        let mut boards = [BitBoard::EMPTY; 8];

        for r in Rank::iter_all() {
            for f in File::iter_all() {
                boards[r].set(Square64::from_file_rank(f, r));
            }
        }

        boards
    };

    pub static ref NOT_RANK_BITBOARDS: [BitBoard; 8] = {
        let mut boards = *RANK_BITBOARDS;

        for m in &mut boards {
            *m = m.complement();
        }

        boards
    };


    pub static ref BORDER: BitBoard = {
        FILE_BITBOARDS[File::A]
            .union(FILE_BITBOARDS[File::H])
            .union(RANK_BITBOARDS[Rank::R1])
            .union(RANK_BITBOARDS[Rank::R8])
    };

    pub static ref WHITE_PAWN_PASSED_MASKS: [BitBoard; 64] =  {
        let mut bitboards = [BitBoard::EMPTY; 64];

        for (i, board) in bitboards.iter_mut().enumerate() {
            let square = Square64::from_primitive(i);
            let (file, rank) = (square.file().unwrap(), square.rank().unwrap());

            for r in rank.iter_up().skip(1) {
                let sq = Square64::from_file_rank(file, r);
                board.set(sq);
            }

            if let Some(file) = file.up() {
                for r in rank.iter_up().skip(1) {
                    let sq = Square64::from_file_rank(file, r);
                    board.set(sq);
                }
            }

            if let Some(file) = file.down() {
                for r in rank.iter_up().skip(1) {
                    let sq = Square64::from_file_rank(file, r);
                    board.set(sq);
                }
            }
        }

        bitboards
    };

    pub static ref BLACK_PAWN_PASSED_MASKS: [BitBoard; 64] = {
        let mut bitboards = [BitBoard::EMPTY; 64];

        for (i, board) in bitboards.iter_mut().enumerate() {
            let square = Square64::from_primitive(i);
            let (file, rank) = (square.file().unwrap(), square.rank().unwrap());

            for r in rank.iter_down().skip(1) {
                let sq = Square64::from_file_rank(file, r);
                board.set(sq);
            }

            if let Some(file) = file.up() {
                for r in rank.iter_down().skip(1) {
                    let sq = Square64::from_file_rank(file, r);
                    board.set(sq);
                }
            }

            if let Some(file) = file.down() {
                for r in rank.iter_down().skip(1) {
                    let sq = Square64::from_file_rank(file, r);
                    board.set(sq);
                }
            }
        }

        bitboards
    };


    pub static ref ISOLATED_PAWN_MASKS: [BitBoard; 64] = {
        let mut bitboards = [BitBoard::EMPTY; 64];

        for (i, board) in bitboards.iter_mut().enumerate() {
            let square = Square64::from_primitive(i);
            let file = square.file().unwrap();

            if let Some(f) = file.up() {
                *board = board.union(FILE_BITBOARDS[f]);
            }

            if let Some(f) = file.down() {
                *board = board.union(FILE_BITBOARDS[f]);
            }
        }

        bitboards
    };

    pub static ref KNIGHT_MOVE_PATTERNS: [BitBoard; 64] = {
        const DIRS: [(isize, Rank, Rank, File, File); 8] = [
            (  6, Rank::R1, Rank::R7, File::C, File::H),
            ( 15, Rank::R1, Rank::R6, File::B, File::H),
            ( 17, Rank::R1, Rank::R6, File::A, File::G),
            ( 10, Rank::R1, Rank::R7, File::A, File::F),
            ( -6, Rank::R2, Rank::R8, File::A, File::F),
            (-15, Rank::R3, Rank::R8, File::A, File::G),
            (-17, Rank::R3, Rank::R8, File::B, File::H),
            (-10, Rank::R2, Rank::R8, File::C, File::H),
        ];

        let mut boards = [BitBoard::EMPTY; 64];

        for (i, m) in boards.iter_mut().enumerate() {
            let mut result = BitBoard::EMPTY;
            let square = Square64::from_primitive(i);
            let rank = square.rank().unwrap();
            let file = square.file().unwrap();

            for (dir, min_rank, max_rank, min_file, max_file) in DIRS {
                if file < min_file || file > max_file || rank <  min_rank || rank > max_rank {
                    continue;
                }

                let target = square + dir;
                result.set(target);
            }

            *m = result;
        }

        boards
    };

    pub static ref KING_MOVE_PATTERNS: [BitBoard; 64] = {
        const DIRS: [(isize, Rank, Rank, File, File); 8] = [
            ( 7, Rank::R1, Rank::R7, File::B, File::H),
            ( 8, Rank::R1, Rank::R7, File::A, File::H),
            ( 9, Rank::R1, Rank::R7, File::A, File::G),
            ( 1, Rank::R1, Rank::R8, File::A, File::G),
            (-7, Rank::R2, Rank::R8, File::A, File::G),
            (-8, Rank::R2, Rank::R8, File::A, File::H),
            (-9, Rank::R2, Rank::R8, File::B, File::H),
            (-1, Rank::R1, Rank::R8, File::B, File::H)
        ];

        let mut boards = [BitBoard::EMPTY; 64];

        for (i, m) in boards.iter_mut().enumerate() {
            let mut result = BitBoard::EMPTY;
            let square = Square64::from_primitive(i);
            let rank = square.rank().unwrap();
            let file = square.file().unwrap();

            for (dir, min_rank, max_rank, min_file, max_file) in DIRS {
                if file < min_file || file > max_file || rank <  min_rank || rank > max_rank {
                    continue;
                }

                let target = square + dir;
                result.set(target);

            }

            *m = result;
        }

        boards
    };

    pub static ref ROOK_MOVE_PATTERNS: [BitBoard; 64] = {
        let mut boards = [BitBoard::EMPTY; 64];

        for (i, m) in boards.iter_mut().enumerate() {
            let mut result = BitBoard::EMPTY;
            let square = Square64::from_primitive(i);
            let rank = square.rank().unwrap();
            let file = square.file().unwrap();

            if let Some(r) = rank.up() {
                for r in Rank::range_inclusive(r, Rank::R8) {
                    result.set(Square64::from_file_rank(file, r));
                }
            }

            if let Some(r) = rank.down() {
                for r in Rank::range_inclusive(Rank::R1, r) {
                    result.set(Square64::from_file_rank(file, r));
                }
            }

            if let Some(f) = file.up() {
                for f in File::range_inclusive(f, File::H) {
                    result.set(Square64::from_file_rank(f, rank));
                }
            }

            if let Some(f) = file.down() {
                for f in File::range_inclusive(File::A, f) {
                    result.set(Square64::from_file_rank(f, rank));
                }
            }

            *m = result;
        }

        boards
    };

    pub static ref BISHOP_MOVE_PATTERNS: [BitBoard; 64] = {
        let mut boards = [BitBoard::EMPTY; 64];

        for (i, m) in boards.iter_mut().enumerate() {
            let mut result = BitBoard::EMPTY;
            let square = Square64::from_primitive(i);
            let rank = square.rank().unwrap();
            let file = square.file().unwrap();

            if let Some((r, f)) = rank.up().zip(file.up()) {
                for (r, f) in std::iter::zip(Rank::range_inclusive(r, Rank::R8), File::range_inclusive(f, File::H)) {
                    result.set(Square64::from_file_rank(f, r));
                }
            }

            if let Some((r, f)) = rank.up().zip(file.down()) {
                for (r, f) in std::iter::zip(Rank::range_inclusive(r, Rank::R8), File::range_inclusive(File::A, f).rev()) {
                    result.set(Square64::from_file_rank(f, r));
                }
            }

            if let Some((r, f)) = rank.down().zip(file.up()) {
                for (r, f) in std::iter::zip(Rank::range_inclusive(Rank::R1, r).rev(), File::range_inclusive(f, File::H)) {
                    result.set(Square64::from_file_rank(f, r));
                }
            }

            if let Some((r, f)) = rank.down().zip(file.down()) {
                for (r, f) in std::iter::zip(Rank::range_inclusive(Rank::R1, r).rev(), File::range_inclusive(File::A, f).rev()) {
                    result.set(Square64::from_file_rank(f, r));
                }
            }

            *m = result;
        }

        boards
    };
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// # TESTS -------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::BitBoard;
    use crate::types::Square64;
    use num_enum::FromPrimitive;

    #[test]
    fn invalid_index() {
        let mut bitboard = BitBoard::EMPTY;
        assert!(!bitboard.get(Square64::Invalid));

        bitboard.set(Square64::Invalid);
        assert!(!bitboard.get(Square64::Invalid));
        assert!(bitboard.is_empty());

        let mut bitboard = BitBoard::FULL;
        assert!(!bitboard.get(Square64::Invalid));

        bitboard.set(Square64::Invalid);
        assert!(!bitboard.get(Square64::Invalid));
        assert!(bitboard.is_full());
    }

    #[test]
    fn set_and_clear() {
        let mut bitboard = BitBoard::EMPTY;

        for sq in 0..64 {
            let sq = Square64::from_primitive(sq);
            assert!(!bitboard.get(sq));
            bitboard.set(sq);
            assert!(bitboard.get(sq));
            bitboard.clear(sq);
            assert!(!bitboard.get(sq));
            assert!(bitboard.is_empty());
        }
    }

    #[test]
    fn fill_and_clear() {
        let mut bitboard = BitBoard::EMPTY;

        for i in 0..64 {
            let sq = Square64::from_primitive(i);
            bitboard.set(sq);
        }

        assert_eq!(BitBoard::FULL, bitboard);

        for i in 0..64 {
            let sq = Square64::from_primitive(i);
            bitboard.clear(sq);
        }

        assert_eq!(BitBoard::EMPTY, bitboard);
    }

    #[test]
    fn pop_all_bits() {
        let mut bitboard = BitBoard::FULL;

        for i in 0..64 {
            let sq = bitboard.pop();
            let sq: usize = sq.into();
            assert_eq!(sq, i);
        }
    }

    #[test]
    fn pop_empty() {
        let mut bitboard = BitBoard::EMPTY;
        let sq = bitboard.pop();
        assert_eq!(sq, Square64::Invalid);
    }

    #[test]
    fn iter_all_indices() {
        let bb = BitBoard::FULL;
        let mut iter = bb.iter_bit_indices();

        for sq in 0..64 {
            let sq = Square64::from_primitive(sq);
            assert_eq!(iter.next(), Some(sq));
        }

        assert_eq!(iter.next(), None);
    }
}
