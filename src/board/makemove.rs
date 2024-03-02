use super::Board;
use crate::{
    board::{HistoryEntry, CASTLE_KEYS, COLOR_KEY, EN_PASSANT_KEYS, PIECE_KEYS},
    moves::Move32,
    types::{CastlePerms, Color, Piece, Square120, Square64},
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
        let from64 = m.m16.start();
        let from120 = Square120::try_from(from64).unwrap();
        let to64 = m.m16.end();
        let to120 = Square120::try_from(to64).unwrap();
        let color = self.color;

        #[cfg(debug_assertions)]
        {
            self.check_board_integrity();
            assert_ne!(from64, Square64::Invalid);
            assert_ne!(to64, Square64::Invalid);
            assert_ne!(color, Color::Both);
            assert!(self.pieces[from120].is_some());
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
            let dir: isize = if color == Color::White { -10 } else { 10 };
            let enemy_pawn_square = to120 + dir;
            self.clear_piece(enemy_pawn_square); // remove the captured pawn
        } else if m.m16.is_queenside_castle() {
            self.move_piece(from120 - 4usize, from120 - 1usize); // move the rook
        } else if m.m16.is_kingside_castle() {
            self.move_piece(from120 + 3usize, from120 + 1usize); // move the rook
        }

        // remove the en passant square and hash it out if necessary
        if let Some(sq) = self.en_passant.take() {
            self.position_key ^= EN_PASSANT_KEYS[sq];
        }

        // update castling permitions and update hash accordingly
        self.position_key ^= CASTLE_KEYS[self.castle_perms.as_u8() as usize];
        self.castle_perms = CastlePerms::from_u8(
            self.castle_perms.as_u8()
                & CASTLE_PERM_MODIFIERS[from120]
                & CASTLE_PERM_MODIFIERS[to120],
        );
        self.position_key ^= CASTLE_KEYS[self.castle_perms.as_u8() as usize];

        // update fifty move counter and ply
        self.fifty_move += 1;
        self.ply += 1;

        // remove any captured pieces and update fifty move counter accordingly
        if m.m16.is_capture() && !m.m16.is_en_passant() {
            self.clear_piece(to120);
            self.fifty_move = 0;
        }

        // a pawn move resets the fifty move counter
        if let Some(Piece::WhitePawn | Piece::BlackPawn) = self.pieces[from120] {
            self.fifty_move = 0;
        }

        // set en passant square and update hash, if the move is a double pawn push
        if m.m16.is_doube_pawn_push() {
            let dir: isize = if color == Color::White { 10 } else { -10 };
            self.en_passant = Some(from120 + dir);
            self.position_key ^= EN_PASSANT_KEYS[from120 + dir];
        }

        // do the actual move
        self.move_piece(from120, to120);

        // if the move is a promotion, switch the piece
        if let Some(promoted_piece) = m.m16.promoted_piece(color) {
            self.clear_piece(to120);
            self.add_piece(to120, promoted_piece);
        }

        // update the king square, if the move was a king move
        if let Some(Piece::WhiteKing | Piece::BlackKing) = self.pieces[to120] {
            self.king_square[color] = to120;
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

        let from64 = m.m16.start();
        let to64 = m.m16.end();
        let from120 = Square120::try_from(from64).unwrap();
        let to120 = Square120::try_from(to64).unwrap();

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
                (-10, Piece::BlackPawn)
            } else {
                (10, Piece::WhitePawn)
            };

            let enemy_pawn_square = to120 + dir;
            self.add_piece(enemy_pawn_square, enemy_pawn) // add the captured pawn back in
        } else if his.move32.m16.is_queenside_castle() {
            self.move_piece(from120 - 1usize, from120 - 4usize); // move the rook back
        } else if his.move32.m16.is_kingside_castle() {
            self.move_piece(from120 + 1usize, from120 + 3usize); // move the rook back
        }

        // move the piece back
        self.move_piece(to120, from120);

        // reset the king square, if the move was a king move
        if let Some(Piece::WhiteKing | Piece::BlackKing) = self.pieces[from120] {
            self.king_square[self.color] = from120;
        }

        // add the captured piece back in, if there is one
        if m.m16.is_capture() && !m.m16.is_en_passant() {
            self.add_piece(to120, m.captured().unwrap());
        }

        if m.m16.is_promotion() {
            let pawn = if self.color == Color::White {
                Piece::WhitePawn
            } else {
                Piece::BlackPawn
            };

            self.clear_piece(from120);
            self.add_piece(from120, pawn);
        }

        #[cfg(debug_assertions)]
        {
            self.check_board_integrity();
            assert_eq!(self.position_key, his.position_key);
        }
    }

    fn clear_piece(&mut self, sq120: Square120) {
        debug_assert_ne!(sq120, Square120::Invalid);
        let sq64 = Square64::try_from(sq120).unwrap();
        let piece = self.pieces[sq120].take().unwrap();
        let color = piece.color();

        self.position_key ^= PIECE_KEYS[sq120][piece];
        self.material[color] -= piece.value();
        self.count_pieces[piece] -= 1;
        self.bitboards[piece].clear(sq64);
        self.bb_all_pieces[color].clear(sq64);
        self.bb_all_pieces[Color::Both].clear(sq64);

        if piece.is_major() {
            self.count_big_pieces[color] -= 1;
            self.count_major_pieces[color] -= 1;
        } else if piece.is_minor() {
            self.count_big_pieces[color] -= 1;
            self.count_minor_pieces[color] -= 1;
        }
    }

    fn add_piece(&mut self, sq120: Square120, piece: Piece) {
        debug_assert_ne!(sq120, Square120::Invalid);
        debug_assert_eq!(self.pieces[sq120], None);
        let sq64 = Square64::try_from(sq120).unwrap();
        let color = piece.color();

        self.position_key ^= PIECE_KEYS[sq120][piece];
        self.pieces[sq120] = Some(piece);
        self.material[color] += piece.value();
        self.count_pieces[piece] += 1;
        self.bitboards[piece].set(sq64);
        self.bb_all_pieces[color].set(sq64);
        self.bb_all_pieces[Color::Both].set(sq64);

        if piece.is_major() {
            self.count_big_pieces[color] += 1;
            self.count_major_pieces[color] += 1;
        } else if piece.is_minor() {
            self.count_big_pieces[color] += 1;
            self.count_minor_pieces[color] += 1;
        }
    }

    fn move_piece(&mut self, from120: Square120, to120: Square120) {
        debug_assert_ne!(from120, Square120::Invalid);
        debug_assert_ne!(to120, Square120::Invalid);

        let from64 = Square64::try_from(from120).unwrap();
        let to64 = Square64::try_from(to120).unwrap();

        let piece = self.pieces[from120].take().unwrap();
        let color = piece.color();
        self.pieces[to120] = Some(piece);

        self.position_key ^= PIECE_KEYS[from120][piece];
        self.position_key ^= PIECE_KEYS[to120][piece];

        self.bitboards[piece].clear(from64);
        self.bitboards[piece].set(to64);

        self.bb_all_pieces[color].clear(from64);
        self.bb_all_pieces[Color::Both].clear(from64);

        self.bb_all_pieces[color].set(to64);
        self.bb_all_pieces[Color::Both].set(to64);
    }
}

#[rustfmt::skip]
const CASTLE_PERM_MODIFIERS: [u8; 120] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 13, 15, 15, 15, 12, 15, 15, 14, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15,  7, 15, 15, 15,  3, 15, 15, 11, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
];
