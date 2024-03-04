use super::Board;
use crate::{
    board::{HistoryEntry, CASTLE_KEYS, COLOR_KEY, EN_PASSANT_KEYS, PIECE_KEYS},
    moves::Move32,
    types::{CastlePerms, Color, Piece, Square64},
};

impl Board {
    /// Makes the move, if it leads to a valid position
    /// (i.e. the moving side doesn't leave itself in check).
    ///
    /// If the move was invalid, the move is automatically taken back,
    /// so the board stays in its current state.
    ///
    /// Returns `true` if the move was successful and `false` otherwise.
    pub fn make_move(&mut self, m: Move32) -> bool {
        let from = m.m16.start();
        let to = m.m16.end();
        let color = self.color;

        #[cfg(debug_assertions)]
        {
            self.check_board_integrity();
            assert_ne!(from, Square64::Invalid);
            assert_ne!(to, Square64::Invalid);
            assert!(self.pieces[from].is_some());
        }

        // store old board data in the history table
        self.history.push(HistoryEntry {
            move32: m,
            fifty_move: self.fifty_move,
            en_passant: self.en_passant,
            castle_perms: self.castle_perms,
            position_key: self.position_key,
        });

        if m.m16.is_en_passant() {
            let dir: isize = if color == Color::White { -8 } else { 8 };
            let enemy_pawn_square = to + dir;
            self.clear_piece(enemy_pawn_square); // remove the captured pawn
        } else if m.m16.is_queenside_castle() {
            self.move_piece(from - 4usize, from - 1usize); // move the rook
        } else if m.m16.is_kingside_castle() {
            self.move_piece(from + 3usize, from + 1usize); // move the rook
        }

        // remove the en passant square and hash it out if necessary
        if let Some(sq) = self.en_passant.take() {
            self.position_key ^= EN_PASSANT_KEYS[sq];
        }

        // update castling permitions and update hash accordingly
        self.position_key ^= CASTLE_KEYS[self.castle_perms.as_u8() as usize];
        let castle_perms = self.castle_perms.as_u8() & CASTLE_PERM_MODIFIERS[from] & CASTLE_PERM_MODIFIERS[to];
        self.castle_perms = CastlePerms::from_u8(castle_perms);
        self.position_key ^= CASTLE_KEYS[self.castle_perms.as_u8() as usize];

        // update fifty move counter and ply
        self.fifty_move += 1;
        self.ply += 1;

        // remove any captured pieces and update fifty move counter accordingly
        if m.m16.is_capture() && !m.m16.is_en_passant() {
            self.clear_piece(to);
            self.fifty_move = 0;
        }

        // a pawn move resets the fifty move counter
        if let Some(Piece::WhitePawn | Piece::BlackPawn) = self.pieces[from] {
            self.fifty_move = 0;
        }

        // set en passant square and update hash, if the move is a double pawn push
        if m.m16.is_doube_pawn_push() {
            let dir: isize = if color == Color::White { 8 } else { -8 };
            self.en_passant = Some(from + dir);
            self.position_key ^= EN_PASSANT_KEYS[from + dir];
        }

        // do the actual move
        self.move_piece(from, to);

        // if the move is a promotion, switch the piece
        if let Some(promoted_piece) = m.m16.promoted_piece(color) {
            self.clear_piece(to);
            self.add_piece(to, promoted_piece);
        }

        // update the king square, if the move was a king move
        if let Some(Piece::WhiteKing | Piece::BlackKing) = self.pieces[to] {
            self.king_square[color] = to;
        }

        self.color = self.color.flipped();
        self.position_key ^= *COLOR_KEY;

        #[cfg(debug_assertions)]
        self.check_board_integrity();

        if self.is_square_attacked(self.king_square[color], self.color) {
            self.take_move();
            return false;
        }

        true
    }

    pub fn take_move(&mut self) {
        #[cfg(debug_assertions)]
        self.check_board_integrity();

        self.ply -= 1;
        let his = self.history.pop().unwrap();
        let m = his.move32;

        let from = m.m16.start();
        let to = m.m16.end();

        // Hash out current en passant square, if there is one
        if let Some(sq) = self.en_passant {
            self.position_key ^= EN_PASSANT_KEYS[sq];
        }

        self.fifty_move = his.fifty_move;

        // Reset castle permitions
        self.position_key ^= CASTLE_KEYS[self.castle_perms.as_u8() as usize];
        self.castle_perms = his.castle_perms;
        self.position_key ^= CASTLE_KEYS[self.castle_perms.as_u8() as usize];

        // Reset en passant square from history entry and update the hash
        self.en_passant = his.en_passant;
        if let Some(sq) = self.en_passant {
            self.position_key ^= EN_PASSANT_KEYS[sq];
        }

        self.color = self.color.flipped();
        self.position_key ^= *COLOR_KEY;

        if his.move32.m16.is_en_passant() {
            let (dir, enemy_pawn): (isize, _) = if self.color == Color::White {
                (-8, Piece::BlackPawn)
            } else {
                (8, Piece::WhitePawn)
            };

            let enemy_pawn_square = to + dir;
            self.add_piece(enemy_pawn_square, enemy_pawn); // add the captured pawn back in
        } else if his.move32.m16.is_queenside_castle() {
            self.move_piece(from - 1usize, from - 4usize); // move the rook back
        } else if his.move32.m16.is_kingside_castle() {
            self.move_piece(from + 1usize, from + 3usize); // move the rook back
        }

        // move the piece back
        self.move_piece(to, from);

        // reset the king square, if the move was a king move
        if let Some(Piece::WhiteKing | Piece::BlackKing) = self.pieces[from] {
            self.king_square[self.color] = from;
        }

        // add the captured piece back in, if there is one
        if m.m16.is_capture() && !m.m16.is_en_passant() {
            self.add_piece(to, Piece::new(m.captured.unwrap(), self.color.flipped()));
        }

        if m.m16.is_promotion() {
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
        debug_assert!(!self.in_check());

        self.ply += 1;
        self.history.push(HistoryEntry {
            move32: Move32::default(),
            fifty_move: self.fifty_move,
            en_passant: self.en_passant,
            castle_perms: self.castle_perms,
            position_key: self.position_key,
        });

        self.color = self.color.flipped();
        self.position_key ^= *COLOR_KEY;

        // remove the en passant square and hash it out if necessary
        if let Some(sq) = self.en_passant.take() {
            self.position_key ^= EN_PASSANT_KEYS[sq];
        }

        #[cfg(debug_assertions)]
        self.check_board_integrity();
    }

    pub fn take_null_move(&mut self) {
        #[cfg(debug_assertions)]
        self.check_board_integrity();

        self.ply -= 1;

        if let Some(sq) = self.en_passant {
            self.position_key ^= EN_PASSANT_KEYS[sq];
        }

        let his = self.history.pop().unwrap();
        self.castle_perms = his.castle_perms;
        self.fifty_move = his.fifty_move;
        self.en_passant = his.en_passant;

        if let Some(sq) = self.en_passant {
            self.position_key ^= EN_PASSANT_KEYS[sq];
        }

        self.color = self.color.flipped();
        self.position_key ^= *COLOR_KEY;

        #[cfg(debug_assertions)]
        self.check_board_integrity();
    }

    fn clear_piece(&mut self, square: Square64) {
        debug_assert_ne!(square, Square64::Invalid);
        let piece = self.pieces[square].take().unwrap();
        let color = piece.color();

        self.position_key ^= PIECE_KEYS[square][piece];
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

    fn add_piece(&mut self, square: Square64, piece: Piece) {
        debug_assert_ne!(square, Square64::Invalid);
        debug_assert_eq!(self.pieces[square], None);
        let color = piece.color();

        self.position_key ^= PIECE_KEYS[square][piece];
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

    fn move_piece(&mut self, from: Square64, to: Square64) {
        debug_assert_ne!(from, Square64::Invalid);
        debug_assert_ne!(to, Square64::Invalid);

        let piece = self.pieces[from].take().unwrap();
        let color = piece.color();
        self.pieces[to] = Some(piece);

        self.position_key ^= PIECE_KEYS[from][piece];
        self.position_key ^= PIECE_KEYS[to][piece];

        self.bitboards[piece].clear(from);
        self.bitboards[piece].set(to);

        self.bb_all_per_color[color].clear(from);
        self.bb_all.clear(from);

        self.bb_all_per_color[color].set(to);
        self.bb_all.set(to);
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
