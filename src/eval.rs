use crate::{
    board::{
        movegen::{magic_bishop_moves, magic_rook_moves},
        Board,
    },
    types::{Color, Piece, Square64},
};

pub fn evaluation(board: &Board) -> i32 {
    if is_draw_by_material(board) {
        return 0;
    }

    let my_color = board.color;
    let op_color = board.color.flipped();
    let mut eval = board.material[my_color] - board.material[op_color];

    let (my_fn, op_fn): (PieceSquareFn, PieceSquareFn) = match my_color {
        Color::White => (piece_square, piece_square_mirrored),
        Color::Black => (piece_square_mirrored, piece_square),
        _ => unreachable!(),
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

fn is_draw_by_material(board: &Board) -> bool {
    let white_queens = board.count_pieces[Piece::WhiteQueen];
    let white_rooks = board.count_pieces[Piece::WhiteRook];
    let white_knights = board.count_pieces[Piece::WhiteKnight];
    let white_bishops = board.count_pieces[Piece::WhiteBishop];
    let white_pawns = board.count_pieces[Piece::WhitePawn];
    let white_minors = board.count_minor_pieces[Color::White]; // Knights + Bishops

    let black_queens = board.count_pieces[Piece::BlackQueen];
    let black_rooks = board.count_pieces[Piece::BlackRook];
    let black_knights = board.count_pieces[Piece::BlackKnight];
    let black_bishops = board.count_pieces[Piece::BlackBishop];
    let black_pawns = board.count_pieces[Piece::BlackPawn];
    let black_minors = board.count_minor_pieces[Color::Black]; // Knights + Bishops

    // Any Queens or Pawns on Board --> no draw
    if white_queens + black_queens + white_pawns + black_pawns != 0 {
        return false;
    }

    // If any Side has more than one Rook --> no draw
    if white_rooks > 1 || black_rooks > 1 {
        return false;
    }

    // No Side has enough material Advantage
    if white_rooks == 1 && black_rooks == 1 && white_minors < 2 && black_minors < 2 {
        return true;
    }

    // Only Rook against only 1 or 2 Minor pieces (White Perspective)
    if white_rooks == 1 && black_rooks == 0 && white_minors == 0 && [1, 2].contains(&black_minors) {
        return true;
    }

    // Only Rook against only 1 or 2 Minor pieces (Black Perspective)
    if black_rooks == 1 && white_rooks == 0 && black_minors == 0 && [1, 2].contains(&white_minors) {
        return true;
    }

    // At this point, we checked all possible draws with rooks
    // All other draws by material contain no rooks
    if black_rooks + white_rooks > 0 {
        return false;
    }

    // Only a few knights on board is a draw
    if white_bishops + black_bishops == 0 && white_knights < 3 && black_knights < 3 {
        return true;
    }

    // There are only a few bishops neither side has significantly more bishops than the other
    if white_knights + black_knights == 0 && usize::abs_diff(white_bishops, black_bishops) < 2 {
        return true;
    }

    if ((white_knights < 3 && white_bishops == 0) || (white_bishops == 1 && white_knights == 0))
        && ((black_knights < 3 && black_bishops == 0) || (black_bishops == 1 && black_knights == 0))
    {
        return true;
    }

    false
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
