use crate::{
    bitboard::BitBoard,
    board::{
        movegen::{magic_bishop_moves, magic_rook_moves},
        Board,
    },
    types::{Color, Piece, Square64},
};

pub fn evaluation(board: &Board) -> i32 {
    let my_color = board.color;
    let op_color = board.color.flipped();

    let mut eval = board.material[my_color] - board.material[op_color];

    eval += piece_square(Piece::pawn(my_color), board, &PAWN_SQUARE_TABLE);
    eval += piece_square(Piece::knight(my_color), board, &KNIGHT_SQUARE_TABLE);
    eval += piece_square(Piece::bishop(my_color), board, &BISHOP_SQUARE_TABLE);
    eval += piece_square(Piece::rook(my_color), board, &ROOK_SQUARE_TABLE);
    eval += piece_square(Piece::queen(my_color), board, &QUEEN_SQUARE_TABLE);
    eval += piece_square(Piece::king(my_color), board, &KING_SQUARE_TABLE);

    eval -= piece_square(Piece::pawn(op_color), board, &PAWN_SQUARE_TABLE);
    eval -= piece_square(Piece::knight(op_color), board, &KNIGHT_SQUARE_TABLE);
    eval -= piece_square(Piece::bishop(op_color), board, &BISHOP_SQUARE_TABLE);
    eval -= piece_square(Piece::rook(op_color), board, &ROOK_SQUARE_TABLE);
    eval -= piece_square(Piece::queen(op_color), board, &QUEEN_SQUARE_TABLE);
    eval -= piece_square(Piece::king(op_color), board, &KING_SQUARE_TABLE);

    eval
}

fn piece_square(piece: Piece, board: &Board, table: &[i32; 64]) -> i32 {
    board.bitboards[piece]
        .iter_bit_indices()
        .map(|square| table[square])
        .sum()
}

fn rook_queen_mobility(square: Square64, color: Color, board: &Board) -> i32 {
    magic_rook_moves(square, board.bb_all_pieces[Color::Both])
        .without(board.bb_all_pieces[color])
        .bit_count() as i32
}

fn bishop_queen_mobility(square: Square64, color: Color, board: &Board) -> i32 {
    magic_bishop_moves(square, board.bb_all_pieces[Color::Both])
        .without(board.bb_all_pieces[color])
        .bit_count() as i32
}

// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// TABLES --------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------
// ---------------------------------------------------------------------------------------------------------------------

#[rustfmt::skip]
const PAWN_SQUARE_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
const KNIGHT_SQUARE_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_SQUARE_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0
];

#[rustfmt::skip]
const QUEEN_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[rustfmt::skip]
const KING_SQUARE_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

#[rustfmt::skip]
const KING_ENDGAME_SQUARE_TABLE: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50
];
