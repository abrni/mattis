use crate::types::{Color, Piece, Square64};
use num_enum::{FromPrimitive, UnsafeFromPrimitive};
use std::fmt::{Debug, Display};

/// # Fields
/// ```text
/// 0000 0000 0000 0000 XXXX XXXX XXXX XXXX  -  Move16
/// 0000 0000 0000 XXXX 0000 0000 0000 0000  -  Captured Piece
/// XXXX XXXX XXXX 0000 0000 0000 0000 0000  -  *Unused*
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Move32 {
    v: u16,
    pub m16: Move16,
}

impl Move32 {
    pub fn new(m16: Move16, captured: Option<Piece>) -> Self {
        let v = if let Some(piece) = captured {
            Into::<u8>::into(piece) as u16 + 1
        } else {
            0
        };

        Self { v, m16 }
    }

    pub fn captured(self) -> Option<Piece> {
        if self.v == 0 {
            None
        } else {
            unsafe { Some(Piece::unchecked_transmute_from(self.v as u8 - 1)) }
        }
    }
}

impl Display for Move32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.m16, f)
    }
}

/// # Fields
/// ```text
/// // 0000 0000 00XX XXXX  -  Start square (64-index)
/// // 0000 XXXX XX00 0000  -  End square (64-index)
/// // 000X 0000 0000 0000  -  Promotion Flag
/// // 00X0 0000 0000 0000  -  Capture Flag
/// // 0X00 0000 0000 0000  -  Special Flag 1 (used to encode the promoted pieces, en passant, castling, etc)
/// // X000 0000 0000 0000  -  Special Flag 2 (used to encode the promoted pieces, en passant, castling, etc)
/// ```
///
/// If all bits are set to zero, the move is considered a No-Move.
/// Note, that both Start and End square are set to A1 in this case.
///
/// # Flags
/// ```text
/// // Pro Cap Sp1 Sp2                          Pro Cap Sp1 Sp2
/// //   0   0   0   0  -  Quiet Move             1   0   0   0  -  Knight promotion
/// //   0   0   0   1  -  Doube Pawn Push        1   0   0   1  -  Bishop promotion
/// //   0   0   1   0  -  Kingside Castle        1   0   1   0  -  Rook promotion
/// //   0   0   1   1  -  Queenside Castle       1   0   1   1  -  Queen promotion
/// //   0   1   0   0  -  Capture                1   1   0   0  -  Knight promo capture
/// //   0   1   0   1  -  En passant capture     1   1   0   1  -  Bishop promo capture
/// //   0   1   1   0  -  *Unused*               1   1   1   0  -  Rook promo capture
/// //   0   1   0   1  -  *Unused*               1   1   1   1  -  Queen promo capture
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Move16(u16);

impl Move16 {
    pub fn build() -> Move16Builder {
        Move16Builder(0)
    }

    pub fn is_nomove(self) -> bool {
        self.0 == 0
    }

    pub fn is_doube_pawn_push(self) -> bool {
        self.0 & 0xF000 == 0x1000
    }

    pub fn is_kingside_castle(self) -> bool {
        self.0 & 0xF000 == 0x2000
    }

    pub fn is_queenside_castle(self) -> bool {
        self.0 & 0xF000 == 0x3000
    }

    pub fn is_capture(self) -> bool {
        self.0 & 0x4000 != 0
    }

    pub fn is_promotion(self) -> bool {
        self.0 & 0x8000 != 0
    }

    pub fn is_en_passant(self) -> bool {
        self.0 & 0xF000 == 0x5000
    }

    pub fn start(self) -> Square64 {
        Square64::from_primitive((self.0 & 0x3F) as usize)
    }

    pub fn end(self) -> Square64 {
        Square64::from_primitive(((self.0 & 0xFC0) >> 6) as usize)
    }

    pub fn promoted_piece(self, color: Color) -> Option<Piece> {
        match (self.0 & 0xF000, color) {
            (0x8000, Color::White) => Some(Piece::WhiteKnight),
            (0x9000, Color::White) => Some(Piece::WhiteBishop),
            (0xA000, Color::White) => Some(Piece::WhiteRook),
            (0xB000, Color::White) => Some(Piece::WhiteQueen),

            (0x8000, Color::Black) => Some(Piece::BlackKnight),
            (0x9000, Color::Black) => Some(Piece::BlackBishop),
            (0xA000, Color::Black) => Some(Piece::BlackRook),
            (0xB000, Color::Black) => Some(Piece::BlackQueen),

            _ => None,
        }
    }
}

pub struct Move16Builder(u16);

impl Move16Builder {
    pub fn start(mut self, square: Square64) -> Self {
        let square: usize = square.into();
        self.0 &= !0x3f; // Clear the bits first
        self.0 |= square as u16 & 0x3f; // Set the square
        self
    }

    pub fn end(mut self, square: Square64) -> Self {
        let square: usize = square.into();
        self.0 &= !0xFC0; // Clear the bits first
        self.0 |= (square as u16 & 0x3F) << 6; // Set the square
        self
    }

    pub fn double_pawn_push(mut self) -> Self {
        // Current flags must signal a quiet move
        debug_assert!(self.0 < 0x1000);

        self.0 &= !0xF000; // Clear all flags
        self.0 |= 0x1000; // Set Special2
        self
    }

    pub fn castle(mut self, kingside: bool) -> Self {
        // Current flags must signal a quiet move
        debug_assert!(self.0 < 0x1000);

        if kingside {
            self.0 |= 0x2000; // Set Special1
        } else {
            self.0 |= 0x3000; // Set Special1 & Special2
        }

        self
    }

    pub fn capture(mut self) -> Self {
        // Current flags must signal a quiet move or a promotion
        debug_assert!(self.0 < 0x1000 || self.0 > 0x3FFF);

        self.0 |= 0x4000;
        self
    }

    pub fn en_passant(mut self) -> Self {
        // Current flags must signal a quiet move or a non-promoting capture
        debug_assert!(self.0 & 0xF000 == 0 || self.0 & 0xF000 == 0x4000);

        self.0 |= 0x5000;
        self
    }

    pub fn promote(mut self, piece: Piece) -> Self {
        // Current flags must signal a quiet move or a capture
        debug_assert!(self.0 < 0x1000 || self.0 > 0x3FFF);

        debug_assert!(matches!(
            piece,
            Piece::WhiteKnight
                | Piece::BlackKnight
                | Piece::WhiteBishop
                | Piece::BlackBishop
                | Piece::WhiteRook
                | Piece::BlackRook
                | Piece::WhiteQueen
                | Piece::BlackQueen
        ));

        let sp = match piece {
            Piece::WhiteKnight | Piece::BlackKnight => 0x0000,
            Piece::WhiteBishop | Piece::BlackBishop => 0x1000,
            Piece::WhiteRook | Piece::BlackRook => 0x2000,
            Piece::WhiteQueen | Piece::BlackQueen => 0x3000,
            _ => 0,
        };

        self.0 |= dbg!(0x8000 | sp);

        self
    }

    pub fn finish(self) -> Move16 {
        let m = Move16(self.0);

        debug_assert_ne!(m.0 & 0xF000, 0x6000); // unused flag configuration
        debug_assert_ne!(m.0 & 0xF000, 0x7000); // unused flag configuration
        debug_assert!(m.is_nomove() || m.start() != m.end()); // start and end should be different, unless it is a No-Move

        m
    }
}

impl Debug for Move16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Move16")
            .field("raw", &format!("{:016b}", self.0))
            .field("start", &self.start())
            .field("end", &self.end())
            .field("promotion", &self.promoted_piece(Color::White))
            .field("capture", &self.is_capture())
            .field("double_pawn_push", &self.is_doube_pawn_push())
            .field("en_passant", &self.is_en_passant())
            .field("kingside_castle", &self.is_kingside_castle())
            .field("queenside_castle", &self.is_queenside_castle())
            .field("nomove", &self.is_nomove())
            .finish()
    }
}

impl Display for Move16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.start(), self.end())?;

        if let Some(piece) = self.promoted_piece(Color::Black) {
            write!(f, "{}", piece.to_char())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Move16;
    use crate::{moves::Move32, types::*};
    use num_enum::FromPrimitive;

    #[test]
    fn type_size() {
        assert_eq!(std::mem::size_of::<Move16>(), 2);
        assert_eq!(std::mem::size_of::<Move32>(), 4);
    }

    #[test]
    fn m32_quiet() {
        let m = Move32::new(Move16::build().finish(), None);
        assert_eq!(m.captured(), None);
        assert!(m.m16.is_nomove());
    }

    #[test]
    fn m32_capture() {
        for piece in Piece::ALL {
            let m = Move32::new(
                Move16::build()
                    .start(Square64::A1)
                    .end(Square64::A2)
                    .capture()
                    .finish(),
                Some(piece),
            );

            assert_eq!(m.captured(), Some(piece));
            assert!(m.m16.is_capture());
        }
    }

    #[test]
    fn m16_nomove() {
        let m = Move16::build().finish();

        assert!(m.is_nomove());
        assert!(!m.is_doube_pawn_push());
        assert!(!m.is_capture());
        assert!(!m.is_en_passant());
        assert!(!m.is_kingside_castle());
        assert!(!m.is_queenside_castle());
        assert!(!m.is_promotion());

        assert_eq!(m.start(), Square64::A1);
        assert_eq!(m.end(), Square64::A1);

        assert_eq!(m.promoted_piece(Color::White), None);
        assert_eq!(m.promoted_piece(Color::Black), None);
        assert_eq!(m.promoted_piece(Color::Both), None);
    }

    #[test]
    fn m16_quiet_move() {
        for start in 0..64 {
            for end in 0..64 {
                if start == end {
                    continue;
                }

                let start = Square64::from_primitive(start);
                let end = Square64::from_primitive(end);
                let m = Move16::build().start(start).end(end).finish();

                assert!(!m.is_nomove());
                assert!(!m.is_doube_pawn_push());
                assert!(!m.is_capture());
                assert!(!m.is_en_passant());
                assert!(!m.is_kingside_castle());
                assert!(!m.is_queenside_castle());
                assert!(!m.is_promotion());

                assert_eq!(m.start(), start);
                assert_eq!(m.end(), end);

                assert_eq!(m.promoted_piece(Color::White), None);
                assert_eq!(m.promoted_piece(Color::Black), None);
                assert_eq!(m.promoted_piece(Color::Both), None);
            }
        }
    }

    #[test]
    fn m16_capture() {
        let m = Move16::build()
            .start(Square64::A1)
            .end(Square64::A2)
            .capture()
            .finish();

        assert_eq!(m.start(), Square64::A1);
        assert_eq!(m.end(), Square64::A2);
        assert!(m.is_capture());
        assert!(!m.is_en_passant());
    }

    #[test]
    fn m16_en_passant_capture() {
        let m = Move16::build()
            .start(Square64::A4)
            .end(Square64::B3)
            .en_passant()
            .finish();

        assert_eq!(m.start(), Square64::A4);
        assert_eq!(m.end(), Square64::B3);
        assert!(m.is_capture());
        assert!(m.is_en_passant());
    }

    #[test]
    fn m16_promotion() {
        const CASES: [(Piece, Color); 8] = [
            (Piece::WhiteKnight, Color::White),
            (Piece::WhiteBishop, Color::White),
            (Piece::WhiteRook, Color::White),
            (Piece::WhiteQueen, Color::White),
            (Piece::BlackKnight, Color::Black),
            (Piece::BlackBishop, Color::Black),
            (Piece::BlackRook, Color::Black),
            (Piece::BlackQueen, Color::Black),
        ];

        for (piece, color) in CASES {
            let m = Move16::build()
                .start(Square64::H7)
                .end(Square64::H8)
                .promote(piece)
                .finish();

            assert_eq!(m.promoted_piece(color), Some(piece));
            assert!(m.is_promotion());
            assert!(!m.is_capture());
            assert!(!m.is_doube_pawn_push());
            assert!(!m.is_nomove());
            assert!(!m.is_kingside_castle());
            assert!(!m.is_queenside_castle());
            assert!(!m.is_en_passant());
        }
    }
}
