use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use std::ops::{Index, IndexMut};

macro_rules! impl_array_indexing {
    ($type:ty, $repr:ty, $len:expr) => {
        impl<T> Index<$type> for [T; $len] {
            type Output = T;

            fn index(&self, index: $type) -> &Self::Output {
                let index: $repr = index.into();
                &self[index as usize]
            }
        }

        impl<T> IndexMut<$type> for [T; $len] {
            fn index_mut(&mut self, index: $type) -> &mut Self::Output {
                let index: $repr = index.into();
                &mut self[index as usize]
            }
        }
    };
}

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

impl Piece {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'P' => Some(Self::WhitePawn),
            'N' => Some(Self::WhiteKnight),
            'B' => Some(Self::WhiteBishop),
            'R' => Some(Self::WhiteRook),
            'Q' => Some(Self::WhiteQueen),
            'K' => Some(Self::WhiteKing),
            'p' => Some(Self::BlackPawn),
            'n' => Some(Self::BlackKnight),
            'b' => Some(Self::BlackBishop),
            'r' => Some(Self::BlackRook),
            'q' => Some(Self::BlackQueen),
            'k' => Some(Self::BlackKing),
            _ => None,
        }
    }

    pub fn is_big(self) -> bool {
        !matches!(self, Self::BlackPawn | Self::WhitePawn)
    }

    pub fn is_major(self) -> bool {
        matches!(
            self,
            Self::WhiteRook
                | Self::WhiteQueen
                | Self::WhiteKing
                | Self::BlackRook
                | Self::BlackQueen
                | Self::BlackKing
        )
    }

    pub fn is_minor(self) -> bool {
        matches!(
            self,
            Self::WhiteBishop | Self::WhiteKnight | Self::BlackBishop | Self::BlackKnight
        )
    }

    pub fn value(self) -> u32 {
        match self {
            Self::WhitePawn | Self::BlackPawn => 100,
            Self::WhiteKnight | Self::WhiteBishop | Self::BlackKnight | Self::BlackBishop => 325,
            Self::WhiteRook | Self::BlackRook => 550,
            Self::WhiteQueen | Self::BlackQueen => 1000,
            Self::WhiteKing | Self::BlackKing => 50_000,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::WhitePawn
            | Self::WhiteKnight
            | Self::WhiteBishop
            | Self::WhiteRook
            | Self::WhiteQueen
            | Self::WhiteKing => Color::White,

            Self::BlackPawn
            | Self::BlackKnight
            | Self::BlackBishop
            | Self::BlackRook
            | Self::BlackQueen
            | Self::BlackKing => Color::Black,
        }
    }
}

impl_array_indexing!(Piece, u8, 12);

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive,
)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
    Both,
}

impl Color {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }
}

impl_array_indexing!(Color, u8, 2);
impl_array_indexing!(Color, u8, 3);

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

impl File {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'a' => Some(Self::A),
            'b' => Some(Self::B),
            'c' => Some(Self::C),
            'd' => Some(Self::D),
            'e' => Some(Self::E),
            'f' => Some(Self::F),
            'g' => Some(Self::G),
            'h' => Some(Self::H),
            _ => None,
        }
    }
}

impl_array_indexing!(File, u8, 8);

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

impl Rank {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::R1),
            '2' => Some(Self::R2),
            '3' => Some(Self::R3),
            '4' => Some(Self::R4),
            '5' => Some(Self::R5),
            '6' => Some(Self::R6),
            '7' => Some(Self::R7),
            '8' => Some(Self::R8),
            _ => None,
        }
    }
}

impl_array_indexing!(Rank, u8, 8);

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive,
)]
#[repr(u8)]
pub enum CastlePerm {
    WhiteKingside = 1,
    WhiteQueenside = 2,
    BlackKingside = 4,
    BlackQueenside = 8,
}

impl CastlePerm {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'K' => Some(Self::WhiteKingside),
            'Q' => Some(Self::WhiteQueenside),
            'k' => Some(Self::BlackKingside),
            'q' => Some(Self::BlackQueenside),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CastlePerms(u8);

impl CastlePerms {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(0x0F);

    pub fn set(&mut self, perm: CastlePerm) {
        let perm: u8 = perm.into();
        self.0 |= perm;
    }

    pub fn clear(&mut self, perm: CastlePerm) {
        let perm: u8 = perm.into();
        self.0 &= !perm;
    }

    pub fn get(&self, perm: CastlePerm) -> bool {
        let perm: u8 = perm.into();
        (self.0 & perm) > 0
    }

    pub fn as_u8(self) -> u8 {
        self.0
    }
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

impl_array_indexing!(Square120, usize, 120);

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

impl_array_indexing!(Square64, usize, 64);

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
