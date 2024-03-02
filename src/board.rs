use std::sync::Mutex;

use crate::types::{CastlePerms, Color, Piece, Square120};
use lazy_static::lazy_static;
use rand::{rngs::StdRng, Rng, SeedableRng};

lazy_static! {
    static ref KEY_RNG: Mutex<StdRng> = Mutex::new(StdRng::seed_from_u64(0)); // always produce the same keys

    static ref PIECE_KEYS: [[u64; 12]; 120] = [KEY_RNG.lock().unwrap().gen(); 120];
    static ref COLOR_KEY: u64 = KEY_RNG.lock().unwrap().gen();
    static ref CASTLE_KEY: [u64; 16] = KEY_RNG.lock().unwrap().gen();
    static ref EN_PASSANT_KEY: [u64; 120] = [KEY_RNG.lock().unwrap().gen(); 120];
}

struct Board {
    pieces: [Option<Piece>; 120],

    king_square: [Square120; 2],
    color: Color,
    en_passant: Option<Square120>,
    fifty_move: usize,
    castle_perms: CastlePerms,

    ply: usize,
    position_key: u64,
}

impl Board {
    fn generate_position_key(&self) -> u64 {
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
            key ^= EN_PASSANT_KEY[sq];
        }

        key ^= CASTLE_KEY[self.castle_perms.as_u8() as usize];

        key
    }
}
