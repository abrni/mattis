use crate::{
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

    let (my_fn, op_fn): (PieceSquareFn, PieceSquareFn) = if my_color == Color::White {
        (piece_square, piece_square_mirrored)
    } else {
        (piece_square_mirrored, piece_square)
    };

    eval += my_fn(Piece::pawn(my_color), board, &PAWN_SQUARE_TABLE);
    eval += my_fn(Piece::knight(my_color), board, &KNIGHT_SQUARE_TABLE);
    eval += my_fn(Piece::bishop(my_color), board, &BISHOP_SQUARE_TABLE);
    eval += my_fn(Piece::rook(my_color), board, &ROOK_SQUARE_TABLE);
    eval += my_fn(Piece::queen(my_color), board, &QUEEN_SQUARE_TABLE);
    eval += my_fn(Piece::king(my_color), board, &KING_SQUARE_TABLE);

    eval -= op_fn(Piece::pawn(op_color), board, &PAWN_SQUARE_TABLE);
    eval -= op_fn(Piece::knight(op_color), board, &KNIGHT_SQUARE_TABLE);
    eval -= op_fn(Piece::bishop(op_color), board, &BISHOP_SQUARE_TABLE);
    eval -= op_fn(Piece::rook(op_color), board, &ROOK_SQUARE_TABLE);
    eval -= op_fn(Piece::queen(op_color), board, &QUEEN_SQUARE_TABLE);
    eval -= op_fn(Piece::king(op_color), board, &KING_SQUARE_TABLE);

    eval
}

type PieceSquareFn = fn(Piece, &Board, &[i32; 64]) -> i32;

fn piece_square(piece: Piece, board: &Board, table: &[i32; 64]) -> i32 {
    board.bitboards[piece]
        .iter_bit_indices()
        .map(|square| table[square])
        .sum()
}

fn piece_square_mirrored(piece: Piece, board: &Board, table: &[i32; 64]) -> i32 {
    board.bitboards[piece]
        .iter_bit_indices()
        .map(|square| table[INDEX_MIRROR[square]])
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
const INDEX_MIRROR: [Square64; 64] = { use Square64::*; [
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
]};

#[rustfmt::skip]
const PAWN_SQUARE_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0, // 1  
     5, 10, 10,-20,-20, 10, 10,  5, // 2
     5, -5,-10,  0,  0,-10, -5,  5, // 3 
     0,  0,  0, 20, 20,  0,  0,  0, // 4 
     5,  5, 10, 25, 25, 10,  5,  5, // 5 
    10, 10, 20, 30, 30, 20, 10, 10, // 6 
    50, 50, 50, 50, 50, 50, 50, 50, // 7 
     0,  0,  0,  0,  0,  0,  0,  0, // 8 
];

#[rustfmt::skip]
const KNIGHT_SQUARE_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50, // 1
    -40,-20,  0,  5,  5,  0,-20,-40, // 2
    -30,  5, 10, 15, 15, 10,  5,-30, // 3
    -30,  0, 15, 20, 20, 15,  0,-30, // 4
    -30,  5, 15, 20, 20, 15,  5,-30, // 5
    -30,  0, 10, 15, 15, 10,  0,-30, // 6
    -40,-20,  0,  0,  0,  0,-20,-40, // 7
    -50,-40,-30,-30,-30,-30,-40,-50, // 8
];

#[rustfmt::skip]
const BISHOP_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20, // 1
    -10,  5,  0,  0,  0,  0,  5,-10, // 2
    -10, 10, 10, 10, 10, 10, 10,-10, // 3
    -10,  0, 10, 10, 10, 10,  0,-10, // 4
    -10,  5,  5, 10, 10,  5,  5,-10, // 5
    -10,  0,  5, 10, 10,  5,  0,-10, // 6
    -10,  0,  0,  0,  0,  0,  0,-10, // 7
    -20,-10,-10,-10,-10,-10,-10,-20, // 8
];

#[rustfmt::skip]
const ROOK_SQUARE_TABLE: [i32; 64] = [
     0,  0,  0,  5,  5,  0,  0,  0,  // 1
    -5,  0,  0,  0,  0,  0,  0, -5,  // 2
    -5,  0,  0,  0,  0,  0,  0, -5,  // 3
    -5,  0,  0,  0,  0,  0,  0, -5,  // 4
    -5,  0,  0,  0,  0,  0,  0, -5,  // 5
    -5,  0,  0,  0,  0,  0,  0, -5,  // 6
     5, 10, 10, 10, 10, 10, 10,  5,  // 7
     0,  0,  0,  0,  0,  0,  0,  0,  // 8
];

#[rustfmt::skip]
const QUEEN_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,  // 1
    -10,  0,  5,  0,  0,  0,  0,-10,  // 2
    -10,  5,  5,  5,  5,  5,  0,-10,  // 3
      0,  0,  5,  5,  5,  5,  0, -5,  // 4
     -5,  0,  5,  5,  5,  5,  0, -5,  // 5
    -10,  0,  5,  5,  5,  5,  0,-10,  // 6
    -10,  0,  0,  0,  0,  0,  0,-10,  // 7
    -20,-10,-10, -5, -5,-10,-10,-20,  // 8
];

#[rustfmt::skip]
const KING_SQUARE_TABLE: [i32; 64] = [
     20, 30, 10,  0,  0, 10, 30, 20,  // 1
     20, 20,  0,  0,  0,  0, 20, 20,  // 2
    -10,-20,-20,-20,-20,-20,-20,-10,  // 3
    -20,-30,-30,-40,-40,-30,-30,-20,  // 4
    -30,-40,-40,-50,-50,-40,-40,-30,  // 5
    -30,-40,-40,-50,-50,-40,-40,-30,  // 6
    -30,-40,-40,-50,-50,-40,-40,-30,  // 7
    -30,-40,-40,-50,-50,-40,-40,-30,  // 8
];

#[rustfmt::skip]
const KING_ENDGAME_SQUARE_TABLE: [i32; 64] = [
    -50,-30,-30,-30,-30,-30,-30,-50,  // 1 
    -30,-30,  0,  0,  0,  0,-30,-30,  // 2
    -30,-10, 20, 30, 30, 20,-10,-30,  // 3
    -30,-10, 30, 40, 40, 30,-10,-30,  // 4
    -30,-10, 30, 40, 40, 30,-10,-30,  // 5
    -30,-10, 20, 30, 30, 20,-10,-30,  // 6
    -30,-20,-10,  0,  0,-10,-20,-30,  // 7
    -50,-40,-30,-20,-20,-30,-40,-50,  // 8
];
