use crate::{
    bitboard::{KNIGHT_MOVE_PATTERNS, RANK_BITBOARDS},
    moves::{Move16, Move16Builder, Move32},
    types::{Color, Piece, Rank, Square120, Square64},
};

use super::Board;

impl Board {
    pub fn generate_all_moves(&self) -> Vec<Move32> {
        let mut list = vec![];

        self.generate_pawn_pushes(&mut list);
        self.generate_pawn_attacks(&mut list);
        self.generate_en_passant(&mut list);
        self.generate_knight_moves(&mut list);

        list
    }

    fn generate_pawn_pushes(&self, list: &mut Vec<Move32>) {
        match self.color {
            Color::White => self.generate_white_pawn_pushes(list),
            Color::Black => self.generate_black_pawn_pushes(list),
            _ => (),
        }
    }

    fn generate_white_pawn_pushes(&self, list: &mut Vec<Move32>) {
        let target_squares_single = self.bitboards[Piece::WhitePawn]
            .shifted_north()
            .without(self.bb_all_pieces[Color::Both]);

        let target_squares_double = target_squares_single
            .shifted_north()
            .without(self.bb_all_pieces[Color::Both])
            .intersection(RANK_BITBOARDS[Rank::R4]);

        for end in target_squares_single.iter_bit_indices() {
            let m16 = Move16::build().start(end - 8usize).end(end);

            if end.rank().unwrap() == Rank::R8 {
                insert_promotions(list, m16, Color::White, None);
            } else {
                list.push(Move32::new(m16.finish(), None));
            }
        }

        for end in target_squares_double.iter_bit_indices() {
            list.push(Move32::new(
                Move16::build()
                    .start(end - 16usize)
                    .end(end)
                    .double_pawn_push()
                    .finish(),
                None,
            ));
        }
    }

    fn generate_black_pawn_pushes(&self, list: &mut Vec<Move32>) {
        let target_squares_single = self.bitboards[Piece::BlackPawn]
            .shifted_south()
            .without(self.bb_all_pieces[Color::Both]);

        let target_squares_double = target_squares_single
            .shifted_south()
            .without(self.bb_all_pieces[Color::Both])
            .intersection(RANK_BITBOARDS[Rank::R5]);

        for end in target_squares_single.iter_bit_indices() {
            let m16 = Move16::build().start(end + 8usize).end(end);

            if end.rank().unwrap() == Rank::R1 {
                insert_promotions(list, m16, Color::Black, None);
            } else {
                list.push(Move32::new(m16.finish(), None));
            }
        }

        for end in target_squares_double.iter_bit_indices() {
            list.push(Move32::new(
                Move16::build()
                    .start(end + 16usize)
                    .end(end)
                    .double_pawn_push()
                    .finish(),
                None,
            ));
        }
    }

    fn generate_pawn_attacks(&self, list: &mut Vec<Move32>) {
        match self.color {
            Color::White => self.generate_white_pawn_attacks(list),
            Color::Black => self.generate_black_pawn_attacks(list),
            _ => (),
        }
    }

    fn generate_white_pawn_attacks(&self, list: &mut Vec<Move32>) {
        let targets_east = self.bitboards[Piece::WhitePawn]
            .shifted_northeast()
            .intersection(self.bb_all_pieces[Color::Black]);

        for end in targets_east.iter_bit_indices() {
            let m16 = Move16::build().start(end - 9usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R8 {
                insert_promotions(list, m16, Color::White, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }

        let targets_west = self.bitboards[Piece::WhitePawn]
            .shifted_northwest()
            .intersection(self.bb_all_pieces[Color::Black]);

        for end in targets_west.iter_bit_indices() {
            let m16 = Move16::build().start(end - 7usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R8 {
                insert_promotions(list, m16, Color::White, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }
    }

    fn generate_black_pawn_attacks(&self, list: &mut Vec<Move32>) {
        let targets_east = self.bitboards[Piece::BlackPawn]
            .shifted_southeast()
            .intersection(self.bb_all_pieces[Color::White]);

        for end in targets_east.iter_bit_indices() {
            let m16 = Move16::build().start(end + 7usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R1 {
                insert_promotions(list, m16, Color::Black, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }

        let targets_west = self.bitboards[Piece::BlackPawn]
            .shifted_southwest()
            .intersection(self.bb_all_pieces[Color::White]);

        for end in targets_west.iter_bit_indices() {
            let m16 = Move16::build().start(end + 9usize).end(end).capture();
            let capture = self.pieces[Square120::try_from(end).unwrap()];

            if end.rank().unwrap() == Rank::R1 {
                insert_promotions(list, m16, Color::Black, capture);
            } else {
                list.push(Move32::new(m16.finish(), capture));
            }
        }
    }

    fn generate_en_passant(&self, list: &mut Vec<Move32>) {
        let Some(en_pas_sq) = self.en_passant else { return };

        if self.color == Color::Both {
            return;
        }

        let candidates = match self.color {
            Color::White => [en_pas_sq - 9usize, en_pas_sq - 11usize],
            Color::Black => [en_pas_sq + 9usize, en_pas_sq + 11usize],
            _ => unreachable!(),
        };

        let moving_piece = match self.color {
            Color::White => Piece::WhitePawn,
            Color::Black => Piece::BlackPawn,
            _ => unreachable!(),
        };

        let captured_piece = match self.color {
            Color::White => Piece::BlackPawn,
            Color::Black => Piece::WhitePawn,
            _ => unreachable!(),
        };

        for candidate in candidates {
            if self.pieces[candidate] != Some(moving_piece) {
                continue;
            }

            let candidate = Square64::try_from(candidate).unwrap();
            let en_pas_sq = Square64::try_from(en_pas_sq).unwrap();

            list.push(Move32::new(
                Move16::build()
                    .start(candidate)
                    .end(en_pas_sq)
                    .en_passant()
                    .finish(),
                Some(captured_piece),
            ))
        }
    }

    fn generate_knight_moves(&self, list: &mut Vec<Move32>) {
        let knights = match self.color {
            Color::Both => return,
            Color::White => self.bitboards[Piece::WhiteKnight],
            Color::Black => self.bitboards[Piece::BlackKnight],
        };

        for start in knights.iter_bit_indices() {
            let targets = KNIGHT_MOVE_PATTERNS[start].without(self.bb_all_pieces[self.color]);

            for end in targets.iter_bit_indices() {
                let capture = self.pieces[Square120::try_from(end).unwrap()];

                let m = Move16::build().start(start).end(end);
                let m = if capture.is_some() { m.capture() } else { m };

                list.push(Move32::new(m.finish(), capture));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// UTILITY -------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

fn insert_promotions(
    list: &mut Vec<Move32>,
    builder: Move16Builder,
    color: Color,
    capture: Option<Piece>,
) {
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
        list.push(Move32::new(builder.promote(p).finish(), capture));
    }
}
