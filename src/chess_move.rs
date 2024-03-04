use crate::types::{Piece, PieceType, Square64};
use num_enum::FromPrimitive;
use std::fmt::{Debug, Display};

/// `ChessMove` contains the start and end field of a move and information about castling, piece promotion and captures.
/// For captures it only encodes if a piece was captured but *not* which one,
/// since this allows us to encode the move in 16 bits.
///
/// # Internal Representation
/// ```text
/// // 0000 0000 00XX XXXX  -  Start square (64-index)
/// // 0000 XXXX XX00 0000  -  End square (64-index)
/// // 000X 0000 0000 0000  -  Promotion Flag
/// // 00X0 0000 0000 0000  -  Capture Flag
/// // 0X00 0000 0000 0000  -  Special Flag 1 (encodes promoted pieces, en passant, castling, etc)
/// // X000 0000 0000 0000  -  Special Flag 2 (encodes promoted pieces, en passant, castling, etc)
/// ```
///
/// If all bits are set to zero, the move is considered a No-Move.
/// Note, that both Start and End square are set to A1 in this case.
///
/// ## Flags
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
pub struct ChessMove(u16);

impl ChessMove {
    pub fn build() -> ChessMoveBuilder {
        ChessMoveBuilder(0)
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

    pub fn promoted(self) -> Option<PieceType> {
        match self.0 & 0xB000 {
            0x8000 => Some(PieceType::Knight),
            0x9000 => Some(PieceType::Bishop),
            0xA000 => Some(PieceType::Rook),
            0xB000 => Some(PieceType::Queen),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChessMoveBuilder(u16);

impl ChessMoveBuilder {
    #[must_use]
    pub fn start(mut self, square: Square64) -> Self {
        let square: usize = square.into();
        self.0 &= !0x3f; // Clear the bits first
        self.0 |= square as u16 & 0x3f; // Set the square
        self
    }

    #[must_use]
    pub fn end(mut self, square: Square64) -> Self {
        let square: usize = square.into();
        self.0 &= !0xFC0; // Clear the bits first
        self.0 |= (square as u16 & 0x3F) << 6; // Set the square
        self
    }

    #[must_use]
    pub fn double_pawn_push(mut self) -> Self {
        // Current flags must signal a quiet move
        debug_assert!(self.0 < 0x1000);

        self.0 &= !0xF000; // Clear all flags
        self.0 |= 0x1000; // Set Special2
        self
    }

    #[must_use]
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

    #[must_use]
    pub fn capture(mut self) -> Self {
        // Current flags must signal a quiet move or a promotion
        debug_assert!(self.0 < 0x1000 || self.0 > 0x3FFF);

        self.0 |= 0x4000;
        self
    }

    #[must_use]
    pub fn en_passant(mut self) -> Self {
        // Current flags must signal a quiet move or a non-promoting capture
        debug_assert!(self.0 & 0xF000 == 0 || self.0 & 0xF000 == 0x4000);

        self.0 |= 0x5000;
        self
    }

    #[must_use]
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
            _ => unreachable!(),
        };

        self.0 |= 0x8000 | sp;

        self
    }

    pub fn finish(self) -> ChessMove {
        let m = ChessMove(self.0);

        debug_assert_ne!(m.0 & 0xF000, 0x6000); // unused flag configuration
        debug_assert_ne!(m.0 & 0xF000, 0x7000); // unused flag configuration
        debug_assert!(m.is_nomove() || m.start() != m.end()); // start and end should be different, unless it is a No-Move

        m
    }
}

impl Default for ChessMove {
    fn default() -> Self {
        Self::build().finish()
    }
}

impl Debug for ChessMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Move16")
            .field("raw", &format!("{:016b}", self.0))
            .field("start", &self.start())
            .field("end", &self.end())
            .field("promotion", &self.promoted())
            .field("capture", &self.is_capture())
            .field("double_pawn_push", &self.is_doube_pawn_push())
            .field("en_passant", &self.is_en_passant())
            .field("kingside_castle", &self.is_kingside_castle())
            .field("queenside_castle", &self.is_queenside_castle())
            .field("nomove", &self.is_nomove())
            .finish()
    }
}

impl Display for ChessMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.start(), self.end())?;

        if let Some(pt) = self.promoted() {
            write!(f, "{}", pt.to_char())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ChessMove;
    use crate::types::*;
    use num_enum::FromPrimitive;

    #[test]
    fn type_size() {
        assert_eq!(std::mem::size_of::<ChessMove>(), 2);
    }

    #[test]
    fn m16_nomove() {
        let m = ChessMove::build().finish();

        assert!(m.is_nomove());
        assert!(!m.is_doube_pawn_push());
        assert!(!m.is_capture());
        assert!(!m.is_en_passant());
        assert!(!m.is_kingside_castle());
        assert!(!m.is_queenside_castle());
        assert!(!m.is_promotion());

        assert_eq!(m.start(), Square64::A1);
        assert_eq!(m.end(), Square64::A1);
        assert_eq!(m.promoted(), None);
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
                let m = ChessMove::build().start(start).end(end).finish();

                assert!(!m.is_nomove());
                assert!(!m.is_doube_pawn_push());
                assert!(!m.is_capture());
                assert!(!m.is_en_passant());
                assert!(!m.is_kingside_castle());
                assert!(!m.is_queenside_castle());
                assert!(!m.is_promotion());

                assert_eq!(m.start(), start);
                assert_eq!(m.end(), end);
                assert_eq!(m.promoted(), None);
            }
        }
    }

    #[test]
    fn m16_capture() {
        let m = ChessMove::build()
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
        let m = ChessMove::build()
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
        const CASES: [Piece; 8] = [
            Piece::WhiteKnight,
            Piece::WhiteBishop,
            Piece::WhiteRook,
            Piece::WhiteQueen,
            Piece::BlackKnight,
            Piece::BlackBishop,
            Piece::BlackRook,
            Piece::BlackQueen,
        ];

        for piece in CASES {
            let m = ChessMove::build()
                .start(Square64::H7)
                .end(Square64::H8)
                .promote(piece)
                .finish();

            assert_eq!(m.promoted(), Some(piece.piece_type()));
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
