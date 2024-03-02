use std::sync::Mutex;

use crate::types::{CastlePerm, CastlePerms, Color, File, Piece, Rank, Square120};
use lazy_static::lazy_static;
use num_enum::TryFromPrimitive;
use rand::{rngs::StdRng, Rng, SeedableRng};
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum FenError {
    #[error("fen string does not contain exactly 6 fields separated by spaces")]
    WrongFieldCount,

    #[error("fen string does not contain exactly 8 ranks separated by slashes")]
    WrongRankCount,

    #[error("fen string does not contain exactly 8 files per rank")]
    WrongFileCount,

    #[error(
        "fen string does not specify a correct active color (use 'w' for white, 'b' for black)"
    )]
    InvalidColor,

    #[error("fen string does not contain valid castle permitions (use '-' for none)")]
    InvalidCastlePerms,

    #[error("fen string does not contain a valid en passant square (use '-' for none)")]
    InvalidEnPassantSquare,
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

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut board = Board::new();

        let parts: Vec<&str> = fen.split_ascii_whitespace().collect();

        if parts.len() != 6 {
            return Err(FenError::WrongFieldCount);
        }

        let ranks: Vec<&str> = parts[0].split('/').collect();

        if ranks.len() != 8 {
            return Err(FenError::WrongRankCount);
        }

        for (rank_num, chars) in ranks.iter().enumerate() {
            let mut file_num = 0;
            for c in chars.chars() {
                if let Some(d) = c.to_digit(10) {
                    file_num += d;
                } else if let Some(piece) = Piece::from_char(c) {
                    let file = File::try_from_primitive(file_num as u8)
                        .map_err(|_| FenError::WrongFileCount)?;
                    let rank = Rank::try_from_primitive((7 - rank_num) as u8).unwrap();
                    let square = Square120::from_file_rank(file, rank);
                    board.pieces[square] = Some(piece);
                    file_num += 1;
                }
            }

            if file_num != 8 {
                return Err(FenError::WrongFileCount);
            }
        }

        if parts[1].chars().count() != 1 {
            return Err(FenError::InvalidColor);
        }

        let color = parts[1].chars().next().unwrap();
        board.color = Color::from_char(color).ok_or(FenError::InvalidColor)?;

        if parts[2].chars().count() > 4 {
            return Err(FenError::InvalidCastlePerms);
        }

        if parts[2] != "-" {
            for c in parts[2].chars() {
                let perm = CastlePerm::from_char(c).ok_or(FenError::InvalidCastlePerms)?;
                board.castle_perms.set(perm);
            }
        }

        if parts[3].chars().count() > 2 {
            return Err(FenError::InvalidCastlePerms);
        }

        if parts[3] != "-" {
            let file = File::from_char(parts[3].chars().next().unwrap())
                .ok_or(FenError::InvalidEnPassantSquare)?;

            let rank = Rank::from_char(parts[3].chars().nth(1).unwrap())
                .ok_or(FenError::InvalidEnPassantSquare)?;
            let square = Square120::from_file_rank(file, rank);
            board.en_passant = Some(square);
        }

        // TODO: Handle halfmove and fullmove clock from parts 5 and 6

        board.position_key = board.generate_position_key();
        Ok(board)
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
