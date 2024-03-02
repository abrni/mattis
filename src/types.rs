use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign},
};

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

macro_rules! impl_iterators {
    ($type:ty, $itertype:ident, $repr:ty) => {
        pub struct $itertype {
            range: std::ops::RangeInclusive<u8>,
        }

        impl Iterator for $itertype {
            type Item = $type;

            fn next(&mut self) -> Option<Self::Item> {
                self.range
                    .next()
                    .map(|v| unsafe { <$type>::unchecked_transmute_from(v) })
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }
        }

        impl DoubleEndedIterator for $itertype {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.range
                    .next_back()
                    .map(|v| unsafe { <$type>::unchecked_transmute_from(v) })
            }
        }

        impl ExactSizeIterator for $itertype {}

        impl $type {
            pub fn range_inclusive(a: Self, b: Self) -> $itertype {
                let a: u8 = a.into();
                let b: u8 = b.into();

                $itertype { range: a..=b }
            }

            pub fn iter_all() -> $itertype {
                Self::range_inclusive(Self::MIN, Self::MAX)
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    const MIN: Self = Self::Pawn;
    const MAX: Self = Self::King;

    pub const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];
}

impl_array_indexing!(PieceType, u8, 6);
impl_iterators!(PieceType, PieceTypeIter, u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive)]
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
    const MIN: Self = Self::WhitePawn;
    const MAX: Self = Self::BlackKing;

    pub const ALL: [Self; 12] = [
        Self::WhitePawn,
        Self::WhiteKnight,
        Self::WhiteBishop,
        Self::WhiteRook,
        Self::WhiteQueen,
        Self::WhiteKing,
        Self::BlackPawn,
        Self::BlackKnight,
        Self::BlackBishop,
        Self::BlackRook,
        Self::BlackQueen,
        Self::BlackKing,
    ];

    pub const ALL_WHITE: [Self; 6] = [
        Self::WhitePawn,
        Self::WhiteKnight,
        Self::WhiteBishop,
        Self::WhiteRook,
        Self::WhiteQueen,
        Self::WhiteKing,
    ];

    pub const ALL_BLACK: [Self; 6] = [
        Self::BlackPawn,
        Self::BlackKnight,
        Self::BlackBishop,
        Self::BlackRook,
        Self::BlackQueen,
        Self::BlackKing,
    ];

    pub fn new(ty: PieceType, color: Color) -> Self {
        match (ty, color) {
            (_, Color::Both) => panic!("cant create a piece with Color::BOTH"),
            (PieceType::Pawn, Color::White) => Piece::WhitePawn,
            (PieceType::Pawn, Color::Black) => Piece::BlackPawn,
            (PieceType::Knight, Color::White) => Piece::WhiteKnight,
            (PieceType::Knight, Color::Black) => Piece::BlackKnight,
            (PieceType::Bishop, Color::White) => Piece::WhiteBishop,
            (PieceType::Bishop, Color::Black) => Piece::BlackBishop,
            (PieceType::Rook, Color::White) => Piece::WhiteRook,
            (PieceType::Rook, Color::Black) => Piece::BlackRook,
            (PieceType::Queen, Color::White) => Piece::WhiteQueen,
            (PieceType::Queen, Color::Black) => Piece::BlackQueen,
            (PieceType::King, Color::White) => Piece::WhiteKing,
            (PieceType::King, Color::Black) => Piece::BlackKing,
        }
    }

    pub const fn from_char(c: char) -> Option<Self> {
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

    pub const fn to_char(self) -> char {
        match self {
            Self::WhitePawn => 'P',
            Self::WhiteKnight => 'N',
            Self::WhiteBishop => 'B',
            Self::WhiteRook => 'R',
            Self::WhiteQueen => 'Q',
            Self::WhiteKing => 'K',
            Self::BlackPawn => 'p',
            Self::BlackKnight => 'n',
            Self::BlackBishop => 'b',
            Self::BlackRook => 'r',
            Self::BlackQueen => 'q',
            Self::BlackKing => 'k',
        }
    }

    pub const fn is_big(self) -> bool {
        !matches!(self, Self::BlackPawn | Self::WhitePawn)
    }

    pub const fn is_major(self) -> bool {
        matches!(
            self,
            Self::WhiteRook | Self::WhiteQueen | Self::WhiteKing | Self::BlackRook | Self::BlackQueen | Self::BlackKing
        )
    }

    pub const fn is_minor(self) -> bool {
        matches!(
            self,
            Self::WhiteBishop | Self::WhiteKnight | Self::BlackBishop | Self::BlackKnight
        )
    }

    pub const fn value(self) -> i16 {
        match self {
            Self::WhitePawn | Self::BlackPawn => 100,
            Self::WhiteKnight | Self::WhiteBishop | Self::BlackKnight | Self::BlackBishop => 325,
            Self::WhiteRook | Self::BlackRook => 550,
            Self::WhiteQueen | Self::BlackQueen => 1000,
            Self::WhiteKing | Self::BlackKing => 15_000,
        }
    }

    pub const fn color(self) -> Color {
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
impl_iterators!(Piece, PieceIter, u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
    Both,
}

impl Color {
    const MIN: Self = Self::White;
    const MAX: Self = Self::Both;

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
            Self::Both => '-',
        }
    }

    pub fn flipped(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
            Self::Both => Self::Both,
        }
    }
}

impl_array_indexing!(Color, u8, 2);
impl_array_indexing!(Color, u8, 3);
impl_iterators!(Color, ColorIter, u8);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive)]
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
    const MIN: Self = Self::A;
    const MAX: Self = Self::H;

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

    pub fn to_char(self) -> char {
        match self {
            Self::A => 'a',
            Self::B => 'b',
            Self::C => 'c',
            Self::D => 'd',
            Self::E => 'e',
            Self::F => 'f',
            Self::G => 'g',
            Self::H => 'h',
        }
    }

    pub fn up(self) -> Option<Self> {
        let f: u8 = self.into();
        Self::try_from_primitive(f + 1).ok()
    }

    pub fn down(self) -> Option<Self> {
        let f: u8 = self.into();
        Self::try_from_primitive(f.checked_sub(1)?).ok()
    }

    pub fn iter_up(self) -> impl Iterator<Item = Self> {
        let mut file = Some(self);
        std::iter::from_fn(move || {
            let old_file = file;
            file = file.and_then(File::up);
            old_file
        })
    }

    pub fn iter_down(self) -> impl Iterator<Item = Self> {
        let mut file = Some(self);
        std::iter::from_fn(move || {
            let old_file = file;
            file = file.and_then(File::down);
            old_file
        })
    }
}

impl_array_indexing!(File, u8, 8);
impl_iterators!(File, FileIter, u8);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive)]
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
    const MIN: Self = Self::R1;
    const MAX: Self = Self::R8;

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

    pub fn to_char(self) -> char {
        match self {
            Self::R1 => '1',
            Self::R2 => '2',
            Self::R3 => '3',
            Self::R4 => '4',
            Self::R5 => '5',
            Self::R6 => '6',
            Self::R7 => '7',
            Self::R8 => '8',
        }
    }

    pub fn up(self) -> Option<Self> {
        let r: u8 = self.into();
        Self::try_from_primitive(r + 1).ok()
    }

    pub fn down(self) -> Option<Self> {
        let r: u8 = self.into();
        Self::try_from_primitive(r.checked_sub(1)?).ok()
    }

    pub fn iter_up(self) -> impl Iterator<Item = Self> {
        let mut rank = Some(self);
        std::iter::from_fn(move || {
            let old_rank = rank;
            rank = rank.and_then(Rank::up);
            old_rank
        })
    }

    pub fn iter_down(self) -> impl Iterator<Item = Self> {
        let mut rank = Some(self);
        std::iter::from_fn(move || {
            let old_rank = rank;
            rank = rank.and_then(Rank::down);
            old_rank
        })
    }

    pub fn mirrored(self) -> Self {
        match self {
            Rank::R1 => Rank::R8,
            Rank::R2 => Rank::R7,
            Rank::R3 => Rank::R6,
            Rank::R4 => Rank::R5,
            Rank::R5 => Rank::R4,
            Rank::R6 => Rank::R3,
            Rank::R7 => Rank::R2,
            Rank::R8 => Rank::R1,
        }
    }
}

impl_array_indexing!(Rank, u8, 8);
impl_iterators!(Rank, RankIter, u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive)]
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

    pub fn to_char(self) -> char {
        match self {
            Self::WhiteKingside => 'K',
            Self::WhiteQueenside => 'Q',
            Self::BlackKingside => 'k',
            Self::BlackQueenside => 'q',
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

    pub fn from_u8(v: u8) -> Self {
        Self(v)
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

impl Add<usize> for Square120 {
    type Output = Square120;

    fn add(self, rhs: usize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive(this + rhs)
    }
}

impl AddAssign<usize> for Square120 {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for Square120 {
    type Output = Square120;

    fn sub(self, rhs: usize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive(this - rhs)
    }
}

impl SubAssign<usize> for Square120 {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl Add<isize> for Square120 {
    type Output = Square120;

    fn add(self, rhs: isize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive((this as isize + rhs) as usize)
    }
}

impl AddAssign<isize> for Square120 {
    fn add_assign(&mut self, rhs: isize) {
        *self = *self + rhs;
    }
}

impl Sub<isize> for Square120 {
    type Output = Square120;

    fn sub(self, rhs: isize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive((this as isize - rhs) as usize)
    }
}

impl SubAssign<isize> for Square120 {
    fn sub_assign(&mut self, rhs: isize) {
        *self = *self - rhs;
    }
}

impl TryFrom<Square64> for Square120 {
    type Error = ();

    fn try_from(value: Square64) -> Result<Self, Self::Error> {
        Ok(Self::from_file_rank(value.file().ok_or(())?, value.rank().ok_or(())?))
    }
}

impl Display for Square120 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Self::Invalid {
            write!(f, "!!")
        } else {
            write!(
                f,
                "{}{}",
                self.file().unwrap().to_char(),
                self.rank().unwrap().to_char()
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive)]
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

impl FromPrimitive for Square64 {
    type Primitive = usize;

    fn from_primitive(number: Self::Primitive) -> Self {
        if number <= Self::Invalid as usize {
            unsafe { std::mem::transmute(number) }
        } else {
            Self::Invalid
        }
    }
}

impl<T> Index<Square64> for Vec<T> {
    type Output = T;

    fn index(&self, index: Square64) -> &Self::Output {
        let index: usize = index.into();
        &self[index]
    }
}

impl<T> IndexMut<Square64> for Vec<T> {
    fn index_mut(&mut self, index: Square64) -> &mut Self::Output {
        let index: usize = index.into();
        &mut self[index]
    }
}

impl Square64 {
    pub fn from_file_rank(file: File, rank: Rank) -> Self {
        let file: u8 = file.into();
        let rank: u8 = rank.into();
        let square = file + rank * 8;
        Square64::from_primitive(square as usize)
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

impl Add<usize> for Square64 {
    type Output = Square64;

    fn add(self, rhs: usize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive(this + rhs)
    }
}

impl AddAssign<usize> for Square64 {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for Square64 {
    type Output = Square64;

    fn sub(self, rhs: usize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive(this - rhs)
    }
}

impl SubAssign<usize> for Square64 {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl Add<isize> for Square64 {
    type Output = Square64;

    fn add(self, rhs: isize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive((this as isize + rhs) as usize)
    }
}

impl AddAssign<isize> for Square64 {
    fn add_assign(&mut self, rhs: isize) {
        *self = *self + rhs;
    }
}

impl Sub<isize> for Square64 {
    type Output = Square64;

    fn sub(self, rhs: isize) -> Self::Output {
        let this: usize = self.into();
        Self::from_primitive((this as isize - rhs) as usize)
    }
}

impl SubAssign<isize> for Square64 {
    fn sub_assign(&mut self, rhs: isize) {
        *self = *self - rhs;
    }
}

impl TryFrom<Square120> for Square64 {
    type Error = ();

    fn try_from(value: Square120) -> Result<Self, Self::Error> {
        Ok(Self::from_file_rank(value.file().ok_or(())?, value.rank().ok_or(())?))
    }
}

impl Display for Square64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Self::Invalid {
            write!(f, "!!")
        } else {
            write!(
                f,
                "{}{}",
                self.file().unwrap().to_char(),
                self.rank().unwrap().to_char()
            )
        }
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
    fn convert_invalid_squares() {
        let sq64 = Square64::Invalid;
        assert_eq!(Err(()), Square120::try_from(sq64));

        let sq120 = Square120::Invalid;
        assert_eq!(Err(()), Square64::try_from(sq120));
    }
}
