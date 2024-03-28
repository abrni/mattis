use bytemuck::{Pod, Zeroable};
use mattis_types::{File, Rank, Square, UnsafeFromPrimitive};
use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Zeroable, Pod)]
#[repr(C)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u64::MAX);
    const NOT_FILE_A: Self = Self(0xfefefefefefefefe);
    const NOT_FILE_H: Self = Self(0x7f7f7f7f7f7f7f7f);

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

    pub fn set(&mut self, idx: Square) {
        let idx: usize = idx.into();

        if let Some(v) = 1u64.checked_shl(idx as u32) {
            self.0 |= v;
        }
    }

    pub fn set_to(&mut self, idx: Square, value: bool) {
        if value {
            self.set(idx);
        } else {
            self.clear(idx);
        }
    }

    pub fn clear(&mut self, idx: Square) {
        let idx: usize = idx.into();

        if let Some(v) = 1u64.checked_shl(idx as u32) {
            self.0 &= !v;
        }
    }

    pub fn get(&self, idx: Square) -> bool {
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
    pub fn pop(&mut self) -> Option<Square> {
        let sq = self.0.trailing_zeros();
        self.0 &= self.0 - 1;

        if sq == 64 {
            return None;
        };

        let sq = unsafe { Square::unchecked_transmute_from(sq as u8) };
        Some(sq)
    }

    pub fn iter_bit_indices(self) -> impl Iterator<Item = Square> {
        let mut b = self;
        std::iter::from_fn(move || b.pop())
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
        Self((self.0 << 1) & Self::NOT_FILE_A.to_u64())
    }

    #[must_use]
    pub fn shifted_west(self) -> Self {
        Self((self.0 >> 1) & Self::NOT_FILE_H.to_u64())
    }

    #[must_use]
    pub fn shifted_northeast(self) -> Self {
        Self((self.0 << 9) & Self::NOT_FILE_A.to_u64())
    }

    #[must_use]
    pub fn shifted_southeast(self) -> Self {
        Self((self.0 >> 7) & Self::NOT_FILE_A.to_u64())
    }

    #[must_use]
    pub fn shifted_southwest(self) -> Self {
        Self((self.0 >> 9) & Self::NOT_FILE_H.to_u64())
    }

    #[must_use]
    pub fn shifted_northwest(self) -> Self {
        Self((self.0 << 7) & Self::NOT_FILE_H.to_u64())
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in Rank::iter_all().rev() {
            for file in File::iter_all() {
                let sq = Square::from_file_rank(file, rank);
                write!(f, "{} ", if self.get(sq) { "X" } else { "." })?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// # TESTS -------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use mattis_types::{Square, TryFromPrimitive};

    use super::BitBoard;

    #[test]
    fn set_and_clear() {
        let mut bitboard = BitBoard::EMPTY;

        for sq in 0..64 {
            let sq = Square::try_from_primitive(sq).unwrap();
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
            let sq = Square::try_from_primitive(i).unwrap();
            bitboard.set(sq);
        }

        assert_eq!(BitBoard::FULL, bitboard);

        for i in 0..64 {
            let sq = Square::try_from_primitive(i).unwrap();
            bitboard.clear(sq);
        }

        assert_eq!(BitBoard::EMPTY, bitboard);
    }

    #[test]
    fn pop_all_bits() {
        let mut bitboard = BitBoard::FULL;

        for i in 0..64 {
            let sq = bitboard.pop().unwrap();
            let sq: usize = sq.into();
            assert_eq!(sq, i);
        }
    }

    #[test]
    fn pop_empty() {
        let mut bitboard = BitBoard::EMPTY;
        let sq = bitboard.pop();
        assert_eq!(sq, None);
    }

    #[test]
    fn iter_all_indices() {
        let bb = BitBoard::FULL;
        let mut iter = bb.iter_bit_indices();

        for sq in 0..64 {
            let sq = Square::try_from_primitive(sq).unwrap();
            assert_eq!(iter.next(), Some(sq));
        }

        assert_eq!(iter.next(), None);
    }
}
