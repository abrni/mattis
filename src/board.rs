pub mod makemove;
pub mod movegen;

use self::movegen::{magic_bishop_moves, magic_rook_moves};
use crate::{
    bitboard::{BitBoard, KING_MOVE_PATTERNS, KNIGHT_MOVE_PATTERNS},
    moves::Move32,
    types::{CastlePerm, CastlePerms, Color, File, Piece, Rank, Square64},
};
use lazy_static::lazy_static;
use num_enum::{FromPrimitive, TryFromPrimitive};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{fmt::Display, sync::Mutex};
use thiserror::Error;

lazy_static! {
    static ref __KEY_RNG: Mutex<StdRng> = Mutex::new(StdRng::seed_from_u64(0)); // always produce the same keys

    pub static ref PIECE_KEYS: [[u64; 12]; 64] = {
        let mut rng = __KEY_RNG.lock().unwrap();
        let mut keys = [[0; 12]; 64];
        keys.iter_mut().for_each(|l| *l = rng.gen());
        keys
    };
    pub static ref COLOR_KEY: u64 = __KEY_RNG.lock().unwrap().gen();
    pub static ref CASTLE_KEYS: [u64; 16] = {
        let mut keys: [u64; 16]  = __KEY_RNG.lock().unwrap().gen();
        // an unitialized board should always have position key 0
        keys[CastlePerms::NONE.as_u8() as usize] = 0;
        keys
    };
    pub static ref EN_PASSANT_KEYS: [u64; 64] = {
        let mut rng = __KEY_RNG.lock().unwrap();
        let mut keys = [0; 64];
        keys.iter_mut().for_each(|k| *k = rng.gen());
        keys
    };
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HistoryEntry {
    pub move32: Move32,
    pub fifty_move: usize,
    pub en_passant: Option<Square64>,
    pub castle_perms: CastlePerms,
    pub position_key: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board {
    pub pieces: [Option<Piece>; 64], // the main representation of pieces on the board
    pub color: Color,                // the current active color
    pub en_passant: Option<Square64>, // the current en passant square, if there is one
    pub castle_perms: CastlePerms,   // the current castle permitions

    pub fifty_move: usize, // the amount of *halfmoves* (triggers the rule at 100) since a fifty-move-rule reset
    pub ply: usize,        // the number of halfmoves since the start of the game (currently unused)
    pub position_key: u64, // the current zobrist position key

    pub king_square: [Square64; 2], // the position of the white and black kings
    pub bitboards: [BitBoard; 12],  // bitboards for each piece type
    pub bb_all_pieces: [BitBoard; 3], // bitboards of all pieces per color
    pub count_pieces: [usize; 12],  // counts the number of pieces on the board fore each piece type
    pub count_big_pieces: [usize; 2], // counts the number of big pieces for both sides (everything exept pawns)
    pub count_major_pieces: [usize; 2], // counts the number of major pieces for both sides (rooks, queens, king)
    pub count_minor_pieces: [usize; 2], // counts the number of minor pieces for both sides (bishops, knights)
    pub material: [i32; 2],             // the material in centipawns for both sides

    pub history: Vec<HistoryEntry>, // stores the board history
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: [None; 64],
            king_square: [Square64::Invalid; 2],
            color: Color::Both,
            en_passant: None,
            fifty_move: 0,
            castle_perms: CastlePerms::NONE,
            ply: 0,
            position_key: 0,
            bitboards: [BitBoard::EMPTY; 12],
            bb_all_pieces: [BitBoard::EMPTY; 3],
            count_pieces: [0; 12],
            count_big_pieces: [0; 2],
            count_major_pieces: [0; 2],
            count_minor_pieces: [0; 2],
            material: [0; 2],
            history: vec![],
        }
    }

    pub fn generate_position_key(&self) -> u64 {
        let mut key: u64 = 0;

        for sq in 0..64 {
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
                    let square = Square64::from_file_rank(file, rank);
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
            let square = Square64::from_file_rank(file, rank);
            board.en_passant = Some(square);
        }

        // TODO: Handle halfmove and fullmove clock from parts 5 and 6

        board.position_key = board.generate_position_key();
        board.update_redundant_data();
        Ok(board)
    }

    pub fn update_redundant_data(&mut self) {
        // clear all redundant data first
        self.bitboards = [BitBoard::EMPTY; 12];
        self.bb_all_pieces = [BitBoard::EMPTY; 3];
        self.count_pieces = [0; 12];
        self.count_big_pieces = [0; 2];
        self.count_major_pieces = [0; 2];
        self.count_minor_pieces = [0; 2];
        self.material = [0; 2];

        for i in 0..64 {
            let square = Square64::from_primitive(i);
            let piece = self.pieces[square];

            if piece.is_none() {
                continue;
            }

            let piece = piece.unwrap(); // safe, because we test if it is none before
            let color = piece.color();

            self.bitboards[piece].set(square);
            self.bb_all_pieces[color].set(square);
            self.bb_all_pieces[Color::Both].set(square);
            self.count_pieces[piece] += 1;
            self.count_big_pieces[color] += piece.is_big() as usize;
            self.count_major_pieces[color] += piece.is_major() as usize;
            self.count_minor_pieces[color] += piece.is_minor() as usize;
            self.material[color] += piece.value();

            if let Piece::WhiteKing | Piece::BlackKing = piece {
                self.king_square[color] = square;
            }
        }
    }

    pub fn as_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let rank = Rank::try_from_primitive(rank).unwrap();
            let mut empty = 0;

            for file in 0..8 {
                let file = File::try_from_primitive(file).unwrap();
                let sq = Square64::from_file_rank(file, rank);

                if let Some(piece) = self.pieces[sq] {
                    if empty > 0 {
                        fen.push(char::from_digit(empty, 10).unwrap());
                        empty = 0;
                    }

                    fen.push(piece.to_char());
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push(char::from_digit(empty, 10).unwrap());
            }

            if rank != Rank::R1 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(self.color.to_char());
        fen.push(' ');

        if self.castle_perms == CastlePerms::NONE {
            fen.push('-');
        } else {
            for p in [
                CastlePerm::WhiteKingside,
                CastlePerm::WhiteQueenside,
                CastlePerm::BlackKingside,
                CastlePerm::BlackQueenside,
            ] {
                if self.castle_perms.get(p) {
                    fen.push(p.to_char());
                }
            }
        }

        fen.push(' ');

        if let Some(sq) = self.en_passant {
            fen.push(sq.file().unwrap().to_char()); // safe, because en passant square is never an invalid square
            fen.push(sq.rank().unwrap().to_char());
        } else {
            fen.push('-');
        }

        // TODO: halfmove and fullmove clock
        fen.push(' ');
        fen.push('0');
        fen.push(' ');
        fen.push('0');

        fen
    }

    pub fn is_square_attacked(&self, square: Square64, color: Color) -> bool {
        if square == Square64::Invalid {
            return false;
        }

        // attacked by white pawns?
        if color == Color::White {
            let east = self.bitboards[Piece::WhitePawn]
                .shifted_northeast()
                .get(square);

            let west = self.bitboards[Piece::WhitePawn]
                .shifted_northwest()
                .get(square);

            if west || east {
                return true;
            }
        }

        // attacked by black pawns?
        if color == Color::Black {
            let east = self.bitboards[Piece::BlackPawn]
                .shifted_southeast()
                .get(square);

            let west = self.bitboards[Piece::BlackPawn]
                .shifted_southwest()
                .get(square);

            if west || east {
                return true;
            }
        }

        // attacked by a king?
        let knight_piece = match color {
            Color::Black => Piece::BlackKnight,
            Color::White => Piece::WhiteKnight,
            Color::Both => unreachable!(),
        };

        if !KNIGHT_MOVE_PATTERNS[square]
            .intersection(self.bitboards[knight_piece])
            .is_empty()
        {
            return true;
        }

        // attacked by a rook or queen?
        let (queen_piece, rook_piece) = match color {
            Color::Black => (Piece::BlackQueen, Piece::BlackRook),
            Color::White => (Piece::WhiteQueen, Piece::WhiteRook),
            Color::Both => unreachable!(),
        };

        let attack_pattern = magic_rook_moves(square, self.bb_all_pieces[Color::Both]);
        let rooks_and_queens = self.bitboards[queen_piece].union(self.bitboards[rook_piece]);
        if !attack_pattern.intersection(rooks_and_queens).is_empty() {
            return true;
        }

        // attacked by a bishop or queen?
        let (queen_piece, bishop_piece) = match color {
            Color::Black => (Piece::BlackQueen, Piece::BlackBishop),
            Color::White => (Piece::WhiteQueen, Piece::WhiteBishop),
            Color::Both => unreachable!(),
        };

        let attack_pattern = magic_bishop_moves(square, self.bb_all_pieces[Color::Both]);
        let bishops_and_queens = self.bitboards[queen_piece].union(self.bitboards[bishop_piece]);
        if !attack_pattern.intersection(bishops_and_queens).is_empty() {
            return true;
        }

        // attacked by a king?
        let king_piece = match color {
            Color::Black => Piece::BlackKing,
            Color::White => Piece::WhiteKing,
            Color::Both => unreachable!(),
        };

        if !KING_MOVE_PATTERNS[square]
            .intersection(self.bitboards[king_piece])
            .is_empty()
        {
            return true;
        }

        false
    }

    pub fn check_board_integrity(&self) {
        let mut check_bitboards = [BitBoard::EMPTY; 12];
        let mut check_bb_all_pieces = [BitBoard::EMPTY; 3];
        let mut check_count_pieces = [0; 12];
        let mut check_count_big_pieces = [0; 2];
        let mut check_count_major_pieces = [0; 2];
        let mut check_count_minor_pieces = [0; 2];
        let mut check_material = [0; 2];

        for i in 0..64 {
            let square = Square64::from_primitive(i);
            let piece = self.pieces[square];

            if let Some(piece) = piece {
                let color = piece.color();

                check_bitboards[piece].set(square);
                check_bb_all_pieces[color].set(square);
                check_bb_all_pieces[Color::Both].set(square);
                check_count_pieces[piece] += 1;
                check_count_big_pieces[color] += piece.is_big() as usize;
                check_count_major_pieces[color] += piece.is_major() as usize;
                check_count_minor_pieces[color] += piece.is_minor() as usize;
                check_material[color] += piece.value();
            }
        }

        for (check_bb, bb) in check_bitboards.iter().zip(self.bitboards.iter()) {
            assert_eq!(check_bb, bb);
        }

        assert_eq!(check_bb_all_pieces, self.bb_all_pieces);
        assert_eq!(check_count_pieces, self.count_pieces);
        assert_eq!(check_count_big_pieces, self.count_big_pieces);
        assert_eq!(check_count_major_pieces, self.count_major_pieces);
        assert_eq!(check_count_minor_pieces, self.count_minor_pieces);
        assert_eq!(check_material, self.material);

        assert!(self.color != Color::Both);
        assert_eq!(self.position_key, self.generate_position_key());

        if let Some(sq) = self.en_passant {
            assert!(
                (sq.rank().unwrap() == Rank::R6 && self.color == Color::White)
                    || (sq.rank().unwrap() == Rank::R3 && self.color == Color::Black)
            );
        }

        assert_eq!(
            self.pieces[self.king_square[Color::White]].unwrap(),
            Piece::WhiteKing
        );

        assert_eq!(
            self.pieces[self.king_square[Color::Black]].unwrap(),
            Piece::BlackKing
        );
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !f.alternate() {
            return write!(f, "{}", self.as_fen());
        }

        for rank in (0..8).rev() {
            for file in 0..8 {
                let rank = Rank::try_from_primitive(rank).unwrap();
                let file = File::try_from_primitive(file).unwrap();
                let sq = Square64::from_file_rank(file, rank);
                let piece = self.pieces[sq];

                if let Some(piece) = piece {
                    write!(f, "{} ", piece.to_char())?;
                } else {
                    write!(f, ". ")?;
                };
            }

            match rank {
                6 => write!(f, " * active color: {}", self.color.to_char())?,
                5 => {
                    if let Some(sq) = self.en_passant {
                        write!(
                            f,
                            " * en passant: {}{}",
                            sq.file().unwrap().to_char(),
                            sq.rank().unwrap().to_char()
                        )?;
                    } else {
                        write!(f, " * en passant: -")?;
                    }
                }
                4 => write!(f, " * ply: {}", self.ply)?,
                3 => write!(f, " * fifty-move: {}", self.fifty_move)?,
                2 => {
                    write!(f, " * castle permitions: ")?;
                    if self.castle_perms == CastlePerms::NONE {
                        write!(f, "-")?;
                    } else {
                        for p in [
                            CastlePerm::WhiteKingside,
                            CastlePerm::WhiteQueenside,
                            CastlePerm::BlackKingside,
                            CastlePerm::BlackQueenside,
                        ] {
                            if self.castle_perms.get(p) {
                                write!(f, "{}", p.to_char())?;
                            }
                        }
                    }
                }
                1 => write!(f, " * tpos key: {:#08x}", self.position_key)?,
                _ => (),
            }

            writeln!(f)?;
        }

        Ok(())
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
