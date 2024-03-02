use std::sync::Mutex;

use crate::types::{CastlePerms, Color, Piece, Square120};
use lazy_static::lazy_static;
use rand::{rngs::StdRng, Rng, SeedableRng};

lazy_static! {
    static ref KEY_RNG: Mutex<StdRng> = Mutex::new(StdRng::seed_from_u64(0)); // always produce the same keys

    static ref PIECE_KEYS: [[u64; 12]; 120] = [KEY_RNG.lock().unwrap().gen(); 120];
    static ref COLOR_KEY: u64 = KEY_RNG.lock().unwrap().gen();
    static ref CASTLE_KEYS: [u64; 16] = {
        let mut keys: [u64; 16]  = KEY_RNG.lock().unwrap().gen();
        // an unitialized board should always have position key 0
        keys[CastlePerms::NONE.as_u8() as usize] = 0;
        keys
    };
    static ref EN_PASSANT_KEYS: [u64; 120] = [KEY_RNG.lock().unwrap().gen(); 120];
}

pub struct Board {
    pub pieces: [Option<Piece>; 120],

    pub king_square: [Square120; 2],
    pub color: Color,
    pub en_passant: Option<Square120>,
    pub fifty_move: usize,
    pub castle_perms: CastlePerms,

    pub ply: usize,
    pub position_key: u64,
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: [None; 120],
            king_square: [Square120::Invalid; 2],
            color: Color::Both,
            en_passant: None,
            fifty_move: 0,
            castle_perms: CastlePerms::NONE,
            ply: 0,
            position_key: 0,
        }
    }

    pub fn generate_position_key(&self) -> u64 {
        let mut key: u64 = 0;

        for sq in 0..120 {
            let piece = self.pieces[sq];

            if let Some(piece) = piece {
                key ^= PIECE_KEYS[sq][piece];
            }
        }

        if self.color == Color::White {
            key ^= *COLOR_KEY;
        }

        if let Some(sq) = self.en_passant {
            key ^= EN_PASSANT_KEYS[sq];
        }

        key ^= CASTLE_KEYS[self.castle_perms.as_u8() as usize];

        key
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Board;

    #[test]
    fn empty_board() {
        let board = Board::new();
        assert_eq!(board.generate_position_key(), 0);
        assert_eq!(board.position_key, 0);
    }
}
