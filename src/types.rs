use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive,
)]
#[repr(u8)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive,
)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
    Both,
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive,
)]
#[repr(u8)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive,
)]
#[repr(u8)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive, IntoPrimitive)]
#[repr(usize)]
#[rustfmt::skip]
pub enum Square120 {
    A1 = 21, B1, C1, D1, E1, F1, G1, H1,
    A2 = 31, B2, C2, D2, E2, F2, G2, H2,
    A3 = 41, B3, C3, D3, E3, F3, G3, H3,
    A4 = 51, B4, C4, D4, E4, F4, G4, H4,
    A5 = 61, B5, C5, D5, E5, F5, G5, H5,
    A6 = 71, B6, C6, D6, E6, F6, G6, H6,
    A7 = 81, B7, C7, D7, E7, F7, G7, H7,
    A8 = 91, B8, C8, D8, E8, F8, G8, H8,
    #[num_enum(default)]
    Invalid
}

impl Square120 {
    pub fn from_file_rank(file: File, rank: Rank) -> Self {
        let file: u8 = file.into();
        let rank: u8 = rank.into();
        let square = 21 + file + rank * 10;
        Square120::from(square as usize)
    }

    pub fn file(self) -> Option<File> {
        if Self::Invalid == self {
            return None;
        };

        let sq: usize = self.into();
        let file = (sq - 21) % 10;

        unsafe { Some(File::unchecked_transmute_from(file as u8)) }
    }

    pub fn rank(self) -> Option<Rank> {
        if Self::Invalid == self {
            return None;
        };

        let sq: usize = self.into();
        let rank = (sq - 21) / 10;
        unsafe { Some(Rank::unchecked_transmute_from(rank as u8)) }
    }
}

impl TryFrom<Square64> for Square120 {
    type Error = ();

    fn try_from(value: Square64) -> Result<Self, Self::Error> {
        Ok(Self::from_file_rank(
            value.file().ok_or(())?,
            value.rank().ok_or(())?,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive, IntoPrimitive)]
#[repr(usize)]
#[rustfmt::skip]
pub enum Square64 {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
    #[num_enum(default)]
    Invalid
}

impl Square64 {
    pub fn from_file_rank(file: File, rank: Rank) -> Self {
        let file: u8 = file.into();
        let rank: u8 = rank.into();
        let square = file + rank * 8;
        Square64::from(square as usize)
    }

    pub fn file(self) -> Option<File> {
        if Self::Invalid == self {
            return None;
        };

        let sq: usize = self.into();
        let file = sq % 8;
        unsafe { Some(File::unchecked_transmute_from(file as u8)) }
    }

    pub fn rank(self) -> Option<Rank> {
        if Self::Invalid == self {
            return None;
        };

        let sq: usize = self.into();
        let rank = sq / 8;
        unsafe { Some(Rank::unchecked_transmute_from(rank as u8)) }
    }
}

impl TryFrom<Square120> for Square64 {
    type Error = ();

    fn try_from(value: Square120) -> Result<Self, Self::Error> {
        Ok(Self::from_file_rank(
            value.file().ok_or(())?,
            value.rank().ok_or(())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{Square120, Square64};
    use num_enum::FromPrimitive;

    #[test]
    fn convert_valid_square_indices() {
        for sq64 in 0..64 {
            // Convert 64-index to 120-index and assert that file and rank match up
            let sq64 = Square64::from_primitive(sq64);
            let sq120 = Square120::try_from(sq64).unwrap();
            assert_eq!(sq64.file().unwrap(), sq120.file().unwrap());
            assert_eq!(sq64.rank().unwrap(), sq120.rank().unwrap());

            // Make the conversion in the other direction with the same assertions
            let sq64 = Square64::try_from(sq120).unwrap();
            assert_eq!(sq64.file().unwrap(), sq120.file().unwrap());
            assert_eq!(sq64.rank().unwrap(), sq120.rank().unwrap());
        }
    }

    #[test]
    fn sq64_invalid() {
        let sq = Square64::Invalid;
        assert_eq!(None, sq.file());
        assert_eq!(None, sq.rank());
    }

    #[test]
    fn sq120_invalid() {
        let sq = Square120::Invalid;
        assert_eq!(None, sq.file());
        assert_eq!(None, sq.rank());
    }

    #[test]
    fn convert_invalid() {
        let sq64 = Square64::Invalid;
        assert_eq!(Err(()), Square120::try_from(sq64));

        let sq120 = Square120::Invalid;
        assert_eq!(Err(()), Square64::try_from(sq120));
    }
}
