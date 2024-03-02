use std::{fmt::Display, sync::Mutex};

use crate::{
    bitboard::BitBoard,
    types::{CastlePerm, CastlePerms, Color, File, Piece, Rank, Square120, Square64},
};
use lazy_static::lazy_static;
use num_enum::{FromPrimitive, TryFromPrimitive};
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
    pub pieces: [Option<Piece>; 120], // the main representation of pieces on the board
    pub color: Color,                 // the current active color
    pub en_passant: Option<Square120>, // the current en passant square, if there is one
    pub castle_perms: CastlePerms,    // the current castle permitions

    pub fifty_move: usize, // the amount of *halfmoves* (triggers the rule at 100) since a fifty-move-rule reset
    pub ply: usize,        // the number of halfmoves since the start of the game (currently unused)
    pub position_key: u64, // the current zobrist position key

    pub king_square: [Square120; 2], // the position of the white and black kings
    pub bitboards: [BitBoard; 12],   // bitboards for each piece type
    pub count_pieces: [usize; 12], // counts the number of pieces on the board fore each piece type
    pub count_big_pieces: [usize; 2], // counts the number of big pieces for both sides (everything exept pawns)
    pub count_major_pieces: [usize; 2], // counts the number of major pieces for both sides (rooks, queens, king)
    pub count_minor_pieces: [usize; 2], // counts the number of minor pieces for both sides (bishops, knights)
    pub material: [u32; 2],             // the material in centipawns for both sides
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
            bitboards: [BitBoard::EMPTY; 12],
            count_pieces: [0; 12],
            count_big_pieces: [0; 2],
            count_major_pieces: [0; 2],
            count_minor_pieces: [0; 2],
            material: [0; 2],
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
        board.update_redundant_data();
        Ok(board)
    }

    pub fn update_redundant_data(&mut self) {
        // clear all redundant data first
        self.bitboards = [BitBoard::EMPTY; 12];
        self.count_pieces = [0; 12];
        self.count_big_pieces = [0; 2];
        self.count_major_pieces = [0; 2];
        self.count_minor_pieces = [0; 2];
        self.material = [0; 2];

        for i in 0..120 {
            let square = Square120::from_primitive(i);
            let piece = self.pieces[square];

            if square == Square120::Invalid || piece.is_none() {
                continue;
            }

            let piece = piece.unwrap(); // safe, because we test if it is none before
            let sq64 = Square64::try_from(square).unwrap(); // safe, because we test if the quare is invalid before
            let color = piece.color();

            self.bitboards[piece].set(sq64);
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
                let sq = Square120::from_file_rank(file, rank);

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

    pub fn is_square_attacked(&self, square: Square120, color: Color) -> bool {
        if square == Square120::Invalid {
            return false;
        }

        // attacked by white pawns?
        if self.color == Color::White
            && (self.pieces[square - 11usize] == Some(Piece::WhitePawn)
                || self.pieces[square - 9usize] == Some(Piece::WhitePawn))
        {
            return true;
        }

        // attacked by black pawns?
        if self.color == Color::Black
            && (self.pieces[square + 11usize] == Some(Piece::BlackPawn)
                || self.pieces[square + 9usize] == Some(Piece::BlackPawn))
        {
            return true;
        }

        // attacked by a knight?
        const KNIGHT_DIRS: [isize; 8] = [-8, -19, -21, -12, 8, 19, 21, 12];
        for dir in KNIGHT_DIRS {
            let piece = self.pieces[square + dir];
            if (piece == Some(Piece::WhiteKnight) && color == Color::White)
                || (piece == Some(Piece::BlackKnight) && color == Color::Black)
            {
                return true;
            }
        }

        // attacked by a rook or queen?
        const ROOK_DIRS: [isize; 4] = [-1, -10, 1, 10];
        for dir in ROOK_DIRS {
            let mut temp_square = square + dir;

            while temp_square != Square120::Invalid {
                let piece = self.pieces[temp_square];

                if piece.is_none() {
                    temp_square += dir;
                    continue;
                }

                if (matches!(piece, Some(Piece::WhiteRook | Piece::WhiteQueen))
                    && color == Color::White)
                    || (matches!(piece, Some(Piece::BlackRook | Piece::BlackQueen))
                        && color == Color::Black)
                {
                    return true;
                }

                break;
            }
        }

        // attacked by a bishop or queen?
        const BISHOP_DIRS: [isize; 4] = [-9, -11, 9, 11];
        for dir in BISHOP_DIRS {
            let mut temp_square = square + dir;

            while temp_square != Square120::Invalid {
                let piece = self.pieces[temp_square];

                if piece.is_none() {
                    temp_square += dir;
                    continue;
                }

                if (matches!(piece, Some(Piece::WhiteBishop | Piece::WhiteQueen))
                    && color == Color::White)
                    || (matches!(piece, Some(Piece::BlackBishop | Piece::BlackQueen))
                        && color == Color::Black)
                {
                    return true;
                }

                break;
            }
        }

        // attacked by a king?
        const KING_DIRS: [isize; 8] = [-1, -10, 1, 10, -9, -11, 11, 9];
        for dir in KING_DIRS {
            let piece = self.pieces[square + dir];
            if (piece == Some(Piece::WhiteKing) && color == Color::White)
                || (piece == Some(Piece::BlackKing) && color == Color::Black)
            {
                return true;
            }
        }

        false
    }

    pub fn check_board_integrity(&self) {
        let mut check_bitboards = [BitBoard::EMPTY; 12];
        let mut check_count_pieces = [0; 12];
        let mut check_count_big_pieces = [0; 2];
        let mut check_count_major_pieces = [0; 2];
        let mut check_count_minor_pieces = [0; 2];
        let mut check_material = [0; 2];

        for i in 0..120 {
            let square = Square120::from_primitive(i);
            let piece = self.pieces[square];

            if square == Square120::Invalid {
                assert!(piece.is_none());
                continue;
            }

            if let Some(piece) = piece {
                let color = piece.color();
                let sq64 = Square64::try_from(square).unwrap();

                check_bitboards[piece].set(sq64);
                check_count_pieces[piece] += 1;
                check_count_big_pieces[color] += piece.is_big() as usize;
                check_count_major_pieces[color] += piece.is_major() as usize;
                check_count_minor_pieces[color] += piece.is_minor() as usize;
                check_material[color] += piece.value();
            }
        }

        assert_eq!(check_bitboards, self.bitboards);
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
                let sq = Square120::from_file_rank(file, rank);
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
                4 => write!(f, " * ply: {}", self.fifty_move)?,
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
