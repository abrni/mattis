use num_enum::FromPrimitive;

use crate::types::Square64;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u64::MAX);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(self) -> bool {
        self.0 == u64::MAX
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn complement(self) -> Self {
        Self(!self.0)
    }

    pub fn set(&mut self, idx: Square64) {
        let idx: usize = idx.into();

        if let Some(v) = 1u64.checked_shl(idx as u32) {
            self.0 |= v;
        }
    }

    pub fn set_to(&mut self, idx: Square64, value: bool) {
        if value {
            self.set(idx)
        } else {
            self.clear(idx)
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

    /// Clears the least significant 1-bit and returns its index
    pub fn pop(&mut self) -> Square64 {
        if self.is_empty() {
            return Square64::Invalid
        }

        #[rustfmt::skip]
        const MAGIC_TABLE: [usize ; 64] = [
            63, 30,  3, 32, 25, 41, 22, 33, 
            15, 50, 42, 13, 11, 53, 19, 34, 
            61, 29,  2, 51, 21, 43, 45, 10, 
            18, 47,  1, 54,  9, 57,  0, 35, 
            62, 31, 40,  4, 49,  5, 52, 26, 
            60,  6, 23, 44, 46, 27, 56, 16, 
             7, 39, 48, 24, 59, 14, 12, 55, 
            38, 28, 58, 20, 37, 17, 36,  8,
        ];

        let b = self.0 ^ (self.0 - 1);
        let fold: u32 = ((b & u64::MAX) ^ (b >> 32)) as u32;
        self.0 &= self.0 - 1;

        let idx = MAGIC_TABLE[(fold.wrapping_mul(0x783a9b23) >> 26)  as usize];
        Square64::from_primitive(idx)
    }
}

#[cfg(test)]
mod tests {
    use num_enum::FromPrimitive;
    use crate::types::Square64;
    use super::BitBoard;

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
}
