use mattis_types::{CastlePerms, Color, Piece, PieceType, Square};

use super::Board;
use crate::{
    board::{
        movegen::{magic_bishop_moves, magic_rook_moves},
        HistoryEntry,
    },
    chess_move::ChessMove,
    tables::{
        KNIGHT_MOVE_PATTERNS, ZOBRIST_CASTLE_KEYS, ZOBRIST_COLOR_KEY, ZOBRIST_EN_PASSANT_KEYS, ZOBRIST_PIECE_KEYS,
    },
};

impl Board {
    /// Makes the move, if it leads to a valid position
    /// (i.e. the moving side doesn't leave itself in check).
    ///
    /// If the move was invalid, the move is automatically taken back,
    /// so the board stays in its current state.
    ///
    /// Returns `true` if the move was successful and `false` otherwise.
    pub fn make_move(&mut self, m: ChessMove) -> bool {
        let start_square = m.start();
        let end_square = m.end();

        #[cfg(debug_assertions)]
        {
            self.check_board_integrity();
            assert!(self.pieces[start_square].is_some());
        }

        let moving_piece = unsafe { self.pieces[start_square].unwrap_unchecked() };
        let color = self.color;

        // If the king moves into check, the move is definitely not legal
        if moving_piece.piece_type() == PieceType::King && self.is_square_attacked(end_square, self.color.flipped()) {
            return false;
        }

        if !self.in_check && moving_piece.piece_type() != PieceType::King && self.will_be_discovered_check(m) {
            return false;
        }

        let captured = if m.is_en_passant() {
            Some(PieceType::Pawn)
        } else {
            self.pieces[end_square].map(Piece::piece_type)
        };

        // store old board data in the history table
        self.history.push(HistoryEntry {
            move16: m,
            captured,
            fifty_move: self.fifty_move,
            en_passant: self.en_passant,
            castle_perms: self.castle_perms,
            position_key: self.position_key,
            in_check: self.in_check,
        });

        if m.is_en_passant() {
            let dir: i8 = if color == Color::White { -8 } else { 8 };
            // Safety: Always a valid square.
            let enemy_pawn_square = unsafe { end_square.add_unchecked(dir) };
            self.clear_piece(enemy_pawn_square); // remove the captured pawn
        } else if m.is_queenside_castle() {
            // Safety: Always a valid square.
            let rook_from = unsafe { start_square.add_unchecked(-4) };
            // Safety: Always a valid square.
            let rook_to = unsafe { start_square.add_unchecked(-1) };
            self.move_piece(rook_from, rook_to); // Move the rook
        } else if m.is_kingside_castle() {
            // Safety: Always a valid square.
            let rook_from = unsafe { start_square.add_unchecked(3) };
            // Safety: Always a valid square.
            let rook_to = unsafe { start_square.add_unchecked(1) };
            self.move_piece(rook_from, rook_to); // Move the rook
        }

        // remove the en passant square and hash it out if necessary
        if let Some(sq) = self.en_passant.take() {
            self.position_key ^= ZOBRIST_EN_PASSANT_KEYS[sq];
        }

        // update castling permitions and update hash accordingly
        self.position_key ^= ZOBRIST_CASTLE_KEYS[self.castle_perms.as_u8() as usize];
        let castle_perms =
            self.castle_perms.as_u8() & CASTLE_PERM_MODIFIERS[start_square] & CASTLE_PERM_MODIFIERS[end_square];
        self.castle_perms = CastlePerms::from_u8(castle_perms);
        self.position_key ^= ZOBRIST_CASTLE_KEYS[self.castle_perms.as_u8() as usize];

        // update fifty move counter and ply
        self.fifty_move += 1;
        self.ply += 1;

        // remove any captured pieces and update fifty move counter accordingly
        if m.is_capture() && !m.is_en_passant() {
            self.clear_piece(end_square);
            self.fifty_move = 0;
        }

        // a pawn move resets the fifty move counter
        if let Some(Piece::WhitePawn | Piece::BlackPawn) = self.pieces[start_square] {
            self.fifty_move = 0;
        }

        // set en passant square and update hash, if the move is a double pawn push
        if m.is_doube_pawn_push() {
            let dir: i8 = if color == Color::White { 8 } else { -8 };
            // Safety: Always a valid square.
            let en_pas = unsafe { start_square.add_unchecked(dir) };
            self.en_passant = Some(en_pas);
            self.position_key ^= ZOBRIST_EN_PASSANT_KEYS[en_pas];
        }

        // do the actual move
        self.move_piece(start_square, end_square);

        // if the move is a promotion, switch the piece
        if let Some(promoted) = m.promoted() {
            self.clear_piece(end_square);
            self.add_piece(end_square, Piece::new(promoted, color));
        }

        // update the king square, if the move was a king move
        if let Some(Piece::WhiteKing | Piece::BlackKing) = self.pieces[end_square] {
            self.king_square[color] = end_square;
        }

        self.color = self.color.flipped();
        self.position_key ^= ZOBRIST_COLOR_KEY;

        let was_in_check = self.in_check;
        self.in_check = self.gave_check(m);

        if was_in_check && self.is_square_attacked(self.king_square[color], self.color) {
            self.take_move();
            return false;
        }

        #[cfg(debug_assertions)]
        self.check_board_integrity();

        true
    }

    pub fn take_move(&mut self) {
        #[cfg(debug_assertions)]
        self.check_board_integrity();

        self.ply -= 1;
        let his = self.history.pop().unwrap();
        let m = his.move16;

        let from = m.start();
        let to = m.end();

        // Hash out current en passant square, if there is one
        if let Some(sq) = self.en_passant {
            self.position_key ^= ZOBRIST_EN_PASSANT_KEYS[sq];
        }

        self.fifty_move = his.fifty_move;
        self.in_check = his.in_check;

        // Reset castle permitions
        self.position_key ^= ZOBRIST_CASTLE_KEYS[self.castle_perms.as_u8() as usize];
        self.castle_perms = his.castle_perms;
        self.position_key ^= ZOBRIST_CASTLE_KEYS[self.castle_perms.as_u8() as usize];

        // Reset en passant square from history entry and update the hash
        self.en_passant = his.en_passant;
        if let Some(sq) = self.en_passant {
            self.position_key ^= ZOBRIST_EN_PASSANT_KEYS[sq];
        }

        self.color = self.color.flipped();
        self.position_key ^= ZOBRIST_COLOR_KEY;

        if his.move16.is_en_passant() {
            let enemy_pawn = Piece::new(PieceType::Pawn, self.color.flipped());
            let dir: i8 = if self.color == Color::White { -8 } else { 8 };

            // Safety: Always a valid square.
            let enemy_pawn_square = unsafe { to.add_unchecked(dir) };
            self.add_piece(enemy_pawn_square, enemy_pawn); // add the captured pawn back in
        } else if his.move16.is_queenside_castle() {
            // Safety: Always a valid square.
            let rook_from = unsafe { from.add_unchecked(-1) };
            // Safety: Always a valid square.
            let rook_to = unsafe { from.add_unchecked(-4) };
            self.move_piece(rook_from, rook_to); // move the rook back
        } else if his.move16.is_kingside_castle() {
            // Safety: Always a valid square.
            let rook_from = unsafe { from.add_unchecked(1) };
            // Safety: Always a valid square.
            let rook_to = unsafe { from.add_unchecked(3) };
            self.move_piece(rook_from, rook_to); // move the rook back
        }

        // move the piece back
        self.move_piece(to, from);

        // reset the king square, if the move was a king move
        if let Some(Piece::WhiteKing | Piece::BlackKing) = self.pieces[from] {
            self.king_square[self.color] = from;
        }

        // add the captured piece back in, if there is one
        if m.is_capture() && !m.is_en_passant() {
            self.add_piece(to, Piece::new(his.captured.unwrap(), self.color.flipped()));
        }

        if m.is_promotion() {
            let pawn = if self.color == Color::White {
                Piece::WhitePawn
            } else {
                Piece::BlackPawn
            };

            self.clear_piece(from);
            self.add_piece(from, pawn);
        }

        #[cfg(debug_assertions)]
        {
            self.check_board_integrity();
            assert_eq!(self.position_key, his.position_key);
        }
    }

    pub fn make_null_move(&mut self) {
        #[cfg(debug_assertions)]
        self.check_board_integrity();
        debug_assert!(!self.in_check);

        self.ply += 1;
        self.history.push(HistoryEntry {
            move16: ChessMove::default(),
            captured: None,
            fifty_move: self.fifty_move,
            en_passant: self.en_passant,
            castle_perms: self.castle_perms,
            position_key: self.position_key,
            in_check: self.in_check,
        });

        self.color = self.color.flipped();
        self.position_key ^= ZOBRIST_COLOR_KEY;

        // remove the en passant square and hash it out if necessary
        if let Some(sq) = self.en_passant.take() {
            self.position_key ^= ZOBRIST_EN_PASSANT_KEYS[sq];
        }

        self.in_check = false;

        #[cfg(debug_assertions)]
        self.check_board_integrity();
    }

    pub fn take_null_move(&mut self) {
        #[cfg(debug_assertions)]
        self.check_board_integrity();

        self.ply -= 1;

        if let Some(sq) = self.en_passant {
            self.position_key ^= ZOBRIST_EN_PASSANT_KEYS[sq];
        }

        let his = self.history.pop().unwrap();
        self.castle_perms = his.castle_perms;
        self.fifty_move = his.fifty_move;
        self.en_passant = his.en_passant;
        self.in_check = his.in_check;

        if let Some(sq) = self.en_passant {
            self.position_key ^= ZOBRIST_EN_PASSANT_KEYS[sq];
        }

        self.color = self.color.flipped();
        self.position_key ^= ZOBRIST_COLOR_KEY;

        #[cfg(debug_assertions)]
        self.check_board_integrity();
    }

    fn clear_piece(&mut self, square: Square) {
        let piece = self.pieces[square].take().unwrap();
        let color = piece.color();

        self.position_key ^= ZOBRIST_PIECE_KEYS[square][piece];
        self.material[color] -= piece.value();
        self.count_pieces[piece] -= 1;
        self.bitboards[piece].clear(square);
        self.bb_all_per_color[color].clear(square);
        self.bb_all.clear(square);

        if piece.is_major() {
            self.count_big_pieces[color] -= 1;
            self.count_major_pieces[color] -= 1;
        } else if piece.is_minor() {
            self.count_big_pieces[color] -= 1;
            self.count_minor_pieces[color] -= 1;
        }
    }

    fn add_piece(&mut self, square: Square, piece: Piece) {
        debug_assert_eq!(self.pieces[square], None);
        let color = piece.color();

        self.position_key ^= ZOBRIST_PIECE_KEYS[square][piece];
        self.pieces[square] = Some(piece);
        self.material[color] += piece.value();
        self.count_pieces[piece] += 1;
        self.bitboards[piece].set(square);
        self.bb_all_per_color[color].set(square);
        self.bb_all.set(square);

        if piece.is_major() {
            self.count_big_pieces[color] += 1;
            self.count_major_pieces[color] += 1;
        } else if piece.is_minor() {
            self.count_big_pieces[color] += 1;
            self.count_minor_pieces[color] += 1;
        }
    }

    fn move_piece(&mut self, from: Square, to: Square) {
        let piece = self.pieces[from].take().unwrap();
        let color = piece.color();
        self.pieces[to] = Some(piece);

        self.position_key ^= ZOBRIST_PIECE_KEYS[from][piece];
        self.position_key ^= ZOBRIST_PIECE_KEYS[to][piece];

        self.bitboards[piece].clear(from);
        self.bitboards[piece].set(to);

        self.bb_all_per_color[color].clear(from);
        self.bb_all.clear(from);

        self.bb_all_per_color[color].set(to);
        self.bb_all.set(to);
    }

    fn will_be_discovered_check(&self, cmove: ChessMove) -> bool {
        let king_square = self.king_square[self.color];

        let mut bb_all = self.bb_all;
        bb_all.clear(cmove.start());
        bb_all.set(cmove.end());

        let queen = Piece::new(PieceType::Queen, self.color.flipped());
        let bishop = Piece::new(PieceType::Bishop, self.color.flipped());
        let rook = Piece::new(PieceType::Rook, self.color.flipped());

        let bq_attacks = magic_bishop_moves(king_square, bb_all);

        if bq_attacks.get(cmove.start()) {
            let mut bishops_and_queens = self.bitboards[queen].union(self.bitboards[bishop]);
            bishops_and_queens.clear(cmove.end());

            if !bq_attacks.intersection(bishops_and_queens).is_empty() {
                return true;
            }
        }

        let rq_attacks = magic_rook_moves(king_square, bb_all);
        if rq_attacks.get(cmove.start()) {
            let mut rooks_and_queens = self.bitboards[queen].union(self.bitboards[rook]);
            rooks_and_queens.clear(cmove.end());

            if !rq_attacks.intersection(rooks_and_queens).is_empty() {
                return true;
            }
        }

        false
    }

    fn gave_check(&self, cmove: ChessMove) -> bool {
        let king_square = self.king_square[self.color];
        let piece = unsafe { self.pieces[cmove.end()].unwrap_unchecked() };

        let direct_check = match piece.piece_type() {
            PieceType::Pawn => match self.color {
                Color::White => {
                    let bb = self.bitboards[Piece::BlackPawn];
                    bb.shifted_southeast().get(king_square) || bb.shifted_southwest().get(king_square)
                }
                Color::Black => {
                    let bb = self.bitboards[Piece::WhitePawn];
                    bb.shifted_northeast().get(king_square) || bb.shifted_northwest().get(king_square)
                }
            },
            PieceType::Knight => KNIGHT_MOVE_PATTERNS[cmove.end()].get(king_square),
            PieceType::Bishop => magic_bishop_moves(cmove.end(), self.bb_all).get(king_square),
            PieceType::Rook => magic_rook_moves(cmove.end(), self.bb_all).get(king_square),
            PieceType::Queen => {
                magic_bishop_moves(cmove.end(), self.bb_all).get(king_square)
                    || magic_rook_moves(cmove.end(), self.bb_all).get(king_square)
            }
            PieceType::King => {
                if cmove.is_kingside_castle() {
                    let start_square = match self.color {
                        Color::White => Square::F8,
                        Color::Black => Square::F1,
                    };
                    magic_rook_moves(start_square, self.bb_all).get(king_square)
                } else if cmove.is_queenside_castle() {
                    let start_square = match self.color {
                        Color::White => Square::D8,
                        Color::Black => Square::D1,
                    };
                    magic_rook_moves(start_square, self.bb_all).get(king_square)
                } else {
                    false
                }
            }
        };

        direct_check || self.will_be_discovered_check(cmove)
    }
}

#[rustfmt::skip]
const CASTLE_PERM_MODIFIERS: [u8; 64] = [
    13, 15, 15, 15, 12, 15, 15, 14,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
     7, 15, 15, 15,  3, 15, 15, 11,
];
