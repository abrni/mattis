use std::{
    fmt::Display,
    ops::{Add, Index, IndexMut, Sub},
};

pub use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};

macro_rules! impl_to_usize {
    ($type:ty, $repr:ty) => {
        impl From<$type> for usize {
            fn from(value: $type) -> usize {
                let value = <$repr>::from(value);
                value as usize
            }
        }
    };
}

macro_rules! impl_array_indexing {
    ($type:ty,  $len:expr) => {
        impl<T> Index<$type> for [T; $len] {
            type Output = T;

            fn index(&self, index: $type) -> &Self::Output {
                let index = usize::from(index);
                &self[index]
            }
        }

        impl<T> IndexMut<$type> for [T; $len] {
            fn index_mut(&mut self, index: $type) -> &mut Self::Output {
                let index = usize::from(index);
                &mut self[index]
            }
        }
    };
}

macro_rules! impl_iterators {
    ($type:ty, $itertype:ident, $repr:ty) => {
        pub struct $itertype {
            range: std::ops::RangeInclusive<$repr>,
        }

        impl Iterator for $itertype {
            type Item = $type;

            fn next(&mut self) -> Option<Self::Item> {
                self.range.next().map(|v| {
                    // Safety: `Self::range` always includes only valid values.
                    unsafe { <$type>::unchecked_transmute_from(v) }
                })
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }
        }

        impl DoubleEndedIterator for $itertype {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.range.next_back().map(|v| {
                    // Safety: `Self::range` always includes only valid values.
                    unsafe { <$type>::unchecked_transmute_from(v) }
                })
            }
        }

        impl ExactSizeIterator for $itertype {}

        impl $type {
            pub fn range_inclusive(a: Self, b: Self) -> $itertype {
                let a = <$repr>::from(a);
                let b = <$repr>::from(b);

                $itertype { range: a..=b }
            }

            pub fn iter_all() -> $itertype {
                Self::range_inclusive(Self::MIN, Self::MAX)
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive, Hash)]
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

    pub const fn is_big(self) -> bool {
        !matches!(self, Self::Pawn)
    }

    pub const fn is_major(self) -> bool {
        matches!(self, Self::Rook | Self::Queen | Self::King)
    }

    pub const fn is_minor(self) -> bool {
        matches!(self, Self::Bishop | Self::Knight)
    }

    pub const fn value(self) -> i16 {
        match self {
            Self::Pawn => 100,
            Self::Knight | PieceType::Bishop => 325,
            Self::Rook => 550,
            Self::Queen => 1000,
            Self::King => 15_000,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        }
    }
}

impl_to_usize!(PieceType, u8);
impl_array_indexing!(PieceType, 6);
impl_iterators!(PieceType, PieceTypeIter, u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive, Hash)]
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

    pub const fn new(ty: PieceType, color: Color) -> Self {
        // Safety:
        // - White pieces have the same bit repr as PieceType.
        // - For black pieces, we just need to add 6 (because there are 6 piece types).
        unsafe { std::mem::transmute(ty as u8 + 6 * color as u8) }
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
        self.piece_type().is_big()
    }

    pub const fn is_major(self) -> bool {
        self.piece_type().is_major()
    }

    pub const fn is_minor(self) -> bool {
        self.piece_type().is_minor()
    }

    pub const fn value(self) -> i16 {
        self.piece_type().value()
    }

    pub const fn piece_type(self) -> PieceType {
        match self {
            Self::WhitePawn | Self::BlackPawn => PieceType::Pawn,
            Self::WhiteKnight | Self::BlackKnight => PieceType::Knight,
            Self::WhiteBishop | Self::BlackBishop => PieceType::Bishop,
            Self::WhiteRook | Self::BlackRook => PieceType::Rook,
            Self::WhiteQueen | Self::BlackQueen => PieceType::Queen,
            Self::WhiteKing | Self::BlackKing => PieceType::King,
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

impl_to_usize!(Piece, u8);
impl_array_indexing!(Piece, 12);
impl_iterators!(Piece, PieceIter, u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive, Hash)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

impl Color {
    const MIN: Self = Self::White;
    const MAX: Self = Self::Black;

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
        }
    }

    #[must_use]
    pub fn flipped(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl_to_usize!(Color, u8);
impl_array_indexing!(Color, 2);
impl_array_indexing!(Color, 3);
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

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_char().fmt(f)
    }
}

impl_to_usize!(File, u8);
impl_array_indexing!(File, 8);
impl_iterators!(File, FileIter, u8);

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive, Hash,
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

    #[must_use]
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

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_char().fmt(f)
    }
}

impl_to_usize!(Rank, u8);
impl_array_indexing!(Rank, 8);
impl_iterators!(Rank, RankIter, u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, UnsafeFromPrimitive, Hash)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, Hash, UnsafeFromPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl_to_usize!(Square, u8);
impl_array_indexing!(Square, 64);

impl<T> Index<Square> for Vec<T> {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        let index = usize::from(index);
        &self[index]
    }
}

impl<T> IndexMut<Square> for Vec<T> {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        let index = usize::from(index);
        &mut self[index]
    }
}

impl Square {
    pub const MIN: Self = Self::A1;
    pub const MAX: Self = Self::H8;

    pub fn from_file_rank(file: File, rank: Rank) -> Self {
        let file: u8 = file.into();
        let rank: u8 = rank.into();
        let square = file + rank * 8;
        // Safety: `square` is always lower than `Square::MAX`.
        unsafe { Self::unchecked_transmute_from(square) }
    }

    pub fn file(self) -> File {
        let sq = u8::from(self);
        let file = sq % 8;
        // Safety: `file` is always lower than `File::MAX`.
        unsafe { File::unchecked_transmute_from(file) }
    }

    pub fn rank(self) -> Rank {
        let sq = u8::from(self);
        let rank = sq / 8;
        // Safety: `rank` is always lower than `Rank::MAX`.
        unsafe { Rank::unchecked_transmute_from(rank) }
    }

    /// # Safety
    /// Leads to UB, if the result is not in range (0..=64).
    #[must_use]
    pub unsafe fn add_unchecked<T>(self, rhs: T) -> Self
    where
        T: Into<i8>,
    {
        let this = u8::from(self) as i8;
        let rhs = rhs.into();
        Self::unchecked_transmute_from((this + rhs) as u8)
    }

    /// # Safety
    /// Leads to UB, if the result is not in range (0..=64).
    #[must_use]
    pub unsafe fn sub_unchecked<T>(self, rhs: T) -> Self
    where
        T: Into<i8>,
    {
        let this = u8::from(self) as i8;
        let rhs = rhs.into();
        Self::unchecked_transmute_from((this - rhs) as u8)
    }
}

impl Add<u8> for Square {
    type Output = Option<Square>;

    fn add(self, rhs: u8) -> Self::Output {
        let this = u8::from(self);
        Self::try_from_primitive(this + rhs).ok()
    }
}

impl Sub<u8> for Square {
    type Output = Option<Square>;

    fn sub(self, rhs: u8) -> Self::Output {
        let this = u8::from(self);
        Self::try_from_primitive(this - rhs).ok()
    }
}

impl Add<i8> for Square {
    type Output = Option<Square>;

    fn add(self, rhs: i8) -> Self::Output {
        let this = u8::from(self) as i8;
        Self::try_from_primitive((this + rhs) as u8).ok()
    }
}

impl Sub<i8> for Square {
    type Output = Option<Square>;

    fn sub(self, rhs: i8) -> Self::Output {
        let this = u8::from(self) as i8;
        Self::try_from_primitive((this - rhs) as u8).ok()
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Piece, PieceType};

    #[test]
    fn convert_piece_types() {
        for color in [Color::White, Color::Black] {
            for piece_type in PieceType::ALL {
                let piece = Piece::new(piece_type, color);
                println!("{color:?}, {piece_type:?} -> {piece:?}");
                assert_eq!(piece.piece_type(), piece_type);
                assert_eq!(piece.color(), color);
            }
        }
    }
}
