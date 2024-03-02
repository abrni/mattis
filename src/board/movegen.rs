use crate::{
    bitboard::RANK_BITBOARDS,
    moves::{Move16, Move32},
    types::{Color, Piece, Rank},
};

use super::Board;

impl Board {
    pub fn generate_all_moves(&self) -> Vec<Move32> {
        let mut list = vec![];

        self.generate_pawn_pushes(&mut list);

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
            list.push(Move32::new(
                Move16::build().start(end - 8usize).end(end).finish(),
                None,
            ));
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
            list.push(Move32::new(
                Move16::build().start(end + 8usize).end(end).finish(),
                None,
            ));
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
}
