use mattis_bitboard::BitBoard;
use mattis_types::{File, Rank, Square, TryFromPrimitive};

pub fn file_bitboards() -> [BitBoard; 8] {
    let mut boards = [BitBoard::EMPTY; 8];

    for f in File::iter_all() {
        for r in mattis_types::Rank::iter_all() {
            boards[f].set(Square::from_file_rank(f, r));
        }
    }

    boards
}

pub fn not_file_bitboards() -> [BitBoard; 8] {
    let mut boards = file_bitboards();

    for m in &mut boards {
        *m = m.complement();
    }

    for board in &boards {
        println!("{board:x?}");
    }

    boards
}

pub fn rank_bitboards() -> [BitBoard; 8] {
    let mut boards = [BitBoard::EMPTY; 8];

    for r in Rank::iter_all() {
        for f in File::iter_all() {
            boards[r].set(Square::from_file_rank(f, r));
        }
    }

    boards
}

pub fn not_rank_bitboards() -> [BitBoard; 8] {
    let mut boards = rank_bitboards();

    for m in &mut boards {
        *m = m.complement();
    }

    boards
}

pub fn border() -> BitBoard {
    let file_bitboards = file_bitboards();

    file_bitboards[File::A]
        .union(file_bitboards[File::H])
        .union(file_bitboards[Rank::R1])
        .union(file_bitboards[Rank::R8])
}

pub fn white_pawn_passed_masks() -> [BitBoard; 64] {
    let mut bitboards = [BitBoard::EMPTY; 64];

    for (i, board) in bitboards.iter_mut().enumerate() {
        let square = Square::try_from_primitive(i as u8).unwrap();
        let (file, rank) = (square.file(), square.rank());

        for r in rank.iter_up().skip(1) {
            let sq = Square::from_file_rank(file, r);
            board.set(sq);
        }

        if let Some(file) = file.up() {
            for r in rank.iter_up().skip(1) {
                let sq = Square::from_file_rank(file, r);
                board.set(sq);
            }
        }

        if let Some(file) = file.down() {
            for r in rank.iter_up().skip(1) {
                let sq = Square::from_file_rank(file, r);
                board.set(sq);
            }
        }
    }

    bitboards
}

pub fn black_pawn_passed_masks() -> [BitBoard; 64] {
    let mut bitboards = [BitBoard::EMPTY; 64];

    for (i, board) in bitboards.iter_mut().enumerate() {
        let square = Square::try_from_primitive(i as u8).unwrap();
        let (file, rank) = (square.file(), square.rank());

        for r in rank.iter_down().skip(1) {
            let sq = Square::from_file_rank(file, r);
            board.set(sq);
        }

        if let Some(file) = file.up() {
            for r in rank.iter_down().skip(1) {
                let sq = Square::from_file_rank(file, r);
                board.set(sq);
            }
        }

        if let Some(file) = file.down() {
            for r in rank.iter_down().skip(1) {
                let sq = Square::from_file_rank(file, r);
                board.set(sq);
            }
        }
    }

    bitboards
}

pub fn isolated_pawn_masks() -> [BitBoard; 64] {
    let mut bitboards = [BitBoard::EMPTY; 64];

    for (i, board) in bitboards.iter_mut().enumerate() {
        let square = Square::try_from_primitive(i as u8).unwrap();
        let file = square.file();

        if let Some(f) = file.up() {
            *board = board.union(file_bitboards()[f]);
        }

        if let Some(f) = file.down() {
            *board = board.union(file_bitboards()[f]);
        }
    }

    bitboards
}

pub fn knight_move_patterns() -> [BitBoard; 64] {
    const DIRS: [(isize, Rank, Rank, File, File); 8] = [
        (6, Rank::R1, Rank::R7, File::C, File::H),
        (15, Rank::R1, Rank::R6, File::B, File::H),
        (17, Rank::R1, Rank::R6, File::A, File::G),
        (10, Rank::R1, Rank::R7, File::A, File::F),
        (-6, Rank::R2, Rank::R8, File::A, File::F),
        (-15, Rank::R3, Rank::R8, File::A, File::G),
        (-17, Rank::R3, Rank::R8, File::B, File::H),
        (-10, Rank::R2, Rank::R8, File::C, File::H),
    ];

    let mut boards = [BitBoard::EMPTY; 64];

    for (square_num, m) in boards.iter_mut().enumerate() {
        let mut result = BitBoard::EMPTY;
        let square = Square::try_from_primitive(square_num as u8).unwrap();
        let rank = square.rank();
        let file = square.file();

        for (dir, min_rank, max_rank, min_file, max_file) in DIRS {
            if file < min_file || file > max_file || rank < min_rank || rank > max_rank {
                continue;
            }

            let target = Square::try_from_primitive((square_num as isize + dir) as u8).unwrap();
            result.set(target);
        }

        *m = result;
    }

    boards
}

pub fn king_move_patterns() -> [BitBoard; 64] {
    const DIRS: [(isize, Rank, Rank, File, File); 8] = [
        (7, Rank::R1, Rank::R7, File::B, File::H),
        (8, Rank::R1, Rank::R7, File::A, File::H),
        (9, Rank::R1, Rank::R7, File::A, File::G),
        (1, Rank::R1, Rank::R8, File::A, File::G),
        (-7, Rank::R2, Rank::R8, File::A, File::G),
        (-8, Rank::R2, Rank::R8, File::A, File::H),
        (-9, Rank::R2, Rank::R8, File::B, File::H),
        (-1, Rank::R1, Rank::R8, File::B, File::H),
    ];

    let mut boards = [BitBoard::EMPTY; 64];

    for (square_num, m) in boards.iter_mut().enumerate() {
        let mut result = BitBoard::EMPTY;
        let square = Square::try_from_primitive(square_num as u8).unwrap();
        let rank = square.rank();
        let file = square.file();

        for (dir, min_rank, max_rank, min_file, max_file) in DIRS {
            if file < min_file || file > max_file || rank < min_rank || rank > max_rank {
                continue;
            }

            let target = Square::try_from_primitive((square_num as isize + dir) as u8).unwrap();
            result.set(target);
        }

        *m = result;
    }

    boards
}

pub fn rook_move_patterns() -> [BitBoard; 64] {
    let mut boards = [BitBoard::EMPTY; 64];

    for (i, m) in boards.iter_mut().enumerate() {
        let mut result = BitBoard::EMPTY;
        let square = Square::try_from_primitive(i as u8).unwrap();
        let rank = square.rank();
        let file = square.file();

        if let Some(r) = rank.up() {
            for r in Rank::range_inclusive(r, Rank::R8) {
                result.set(Square::from_file_rank(file, r));
            }
        }

        if let Some(r) = rank.down() {
            for r in Rank::range_inclusive(Rank::R1, r) {
                result.set(Square::from_file_rank(file, r));
            }
        }

        if let Some(f) = file.up() {
            for f in File::range_inclusive(f, File::H) {
                result.set(Square::from_file_rank(f, rank));
            }
        }

        if let Some(f) = file.down() {
            for f in File::range_inclusive(File::A, f) {
                result.set(Square::from_file_rank(f, rank));
            }
        }

        *m = result;
    }

    boards
}

pub fn bishop_move_patterns() -> [BitBoard; 64] {
    let mut boards = [BitBoard::EMPTY; 64];

    for (i, m) in boards.iter_mut().enumerate() {
        let mut result = BitBoard::EMPTY;
        let square = Square::try_from_primitive(i as u8).unwrap();
        let rank = square.rank();
        let file = square.file();

        if let Some((r, f)) = rank.up().zip(file.up()) {
            for (r, f) in std::iter::zip(Rank::range_inclusive(r, Rank::R8), File::range_inclusive(f, File::H)) {
                result.set(Square::from_file_rank(f, r));
            }
        }

        if let Some((r, f)) = rank.up().zip(file.down()) {
            for (r, f) in std::iter::zip(
                Rank::range_inclusive(r, Rank::R8),
                File::range_inclusive(File::A, f).rev(),
            ) {
                result.set(Square::from_file_rank(f, r));
            }
        }

        if let Some((r, f)) = rank.down().zip(file.up()) {
            for (r, f) in std::iter::zip(
                Rank::range_inclusive(Rank::R1, r).rev(),
                File::range_inclusive(f, File::H),
            ) {
                result.set(Square::from_file_rank(f, r));
            }
        }

        if let Some((r, f)) = rank.down().zip(file.down()) {
            for (r, f) in std::iter::zip(
                Rank::range_inclusive(Rank::R1, r).rev(),
                File::range_inclusive(File::A, f).rev(),
            ) {
                result.set(Square::from_file_rank(f, r));
            }
        }

        *m = result;
    }

    boards
}
