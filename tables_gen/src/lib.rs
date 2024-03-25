use mattis_bitboard::BitBoard;
use mattis_types::{File, Rank, Square, TryFromPrimitive};
use rand::Rng;
use std::ops::BitAnd;

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
    let rank_bitboards = rank_bitboards();

    file_bitboards[File::A]
        .union(file_bitboards[File::H])
        .union(rank_bitboards[Rank::R1])
        .union(rank_bitboards[Rank::R8])
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

pub fn rook_magic_bit_count() -> [u32; 64] {
    #[rustfmt::skip]
    pub const ROOK_MAGIC_BIT_COUNT: [u32; 64] = [
        12, 11, 11, 11, 11, 11, 11, 12,
        11, 10, 10, 10, 10, 10, 10, 11,
        11, 10, 10, 10, 10, 10, 10, 11,
        11, 10, 10, 10, 10, 10, 10, 11,
        11, 10, 10, 10, 10, 10, 10, 11,
        11, 10, 10, 10, 10, 10, 10, 11,
        11, 10, 10, 10, 10, 10, 10, 11,
        12, 11, 11, 11, 11, 11, 11, 12,
    ];

    ROOK_MAGIC_BIT_COUNT
}

pub fn bishop_magic_bit_count() -> [u32; 64] {
    #[rustfmt::skip]
    pub const BISHOP_MAGIC_BIT_COUNT: [u32; 64] = [
        6, 5, 5, 5, 5, 5, 5, 6,
        5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 7, 7, 7, 7, 5, 5,
        5, 5, 7, 9, 9, 7, 5, 5,
        5, 5, 7, 9, 9, 7, 5, 5,
        5, 5, 7, 7, 7, 7, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5,
        6, 5, 5, 5, 5, 5, 5, 6,
    ];

    BISHOP_MAGIC_BIT_COUNT
}

pub fn rook_magic_masks() -> [BitBoard; 64] {
    let mut boards = [BitBoard::EMPTY; 64];

    for (i, m) in boards.iter_mut().enumerate() {
        let mut result = BitBoard::EMPTY;
        let square = Square::try_from_primitive(i as u8).unwrap();
        let rank = square.rank();
        let file = square.file();

        if let Some(r) = rank.up() {
            for r in Rank::range_inclusive(r, Rank::R7) {
                result.set(Square::from_file_rank(file, r));
            }
        }

        if let Some(r) = rank.down() {
            for r in Rank::range_inclusive(Rank::R2, r) {
                result.set(Square::from_file_rank(file, r));
            }
        }

        if let Some(f) = file.up() {
            for f in File::range_inclusive(f, File::G) {
                result.set(Square::from_file_rank(f, rank));
            }
        }

        if let Some(f) = file.down() {
            for f in File::range_inclusive(File::B, f) {
                result.set(Square::from_file_rank(f, rank));
            }
        }

        *m = result;
    }

    boards
}

pub fn bishop_magic_masks() -> [BitBoard; 64] {
    let mut masks = bishop_move_patterns();

    for m in &mut masks {
        *m = m.without(border());
    }

    masks
}

pub fn rook_magics() -> [u64; 64] {
    let mut magics = [0; 64];

    for square in 0..64 {
        let square = Square::try_from_primitive(square).unwrap();

        let rmagic = loop {
            if let Some(m) = find_magic(square, rook_magic_bit_count()[square as usize], false) {
                break m;
            };
        };

        magics[square] = rmagic;
    }

    magics
}

pub fn bishop_magics() -> [u64; 64] {
    let mut magics = [0; 64];

    for square in 0..64 {
        let square = Square::try_from_primitive(square).unwrap();

        let bmagic = loop {
            if let Some(m) = find_magic(square, bishop_magic_bit_count()[square as usize], true) {
                break m;
            };
        };

        magics[square] = bmagic;
    }

    magics
}

fn find_magic(square: Square, m: u32, is_bishop: bool) -> Option<u64> {
    let mut b = [BitBoard::EMPTY; 4096];
    let mut a = [BitBoard::EMPTY; 4096];

    let mask = if is_bishop {
        bishop_magic_masks()[square as usize]
    } else {
        rook_magic_masks()[square as usize]
    };

    let n = mask.bit_count();

    for i in 0..(1 << n) {
        b[i] = index_to_bb(i, n, mask);
        a[i] = if is_bishop {
            batt(square, b[i])
        } else {
            ratt(square, b[i])
        };
    }

    for _ in 0..100_000_000 {
        let magic = rand_u64_fewbits();

        if mask
            .to_u64()
            .wrapping_mul(magic)
            .bitand(0xFF00000000000000)
            .count_ones()
            < 6
        {
            continue;
        }

        let mut used = [BitBoard::EMPTY; 4096];
        let mut fail = false;

        for i in 0..(1 << n) {
            if fail {
                break;
            }

            let j = transform(b[i], magic, m);

            if used[j as usize] == BitBoard::EMPTY {
                used[j as usize] = a[i];
            } else if used[j as usize] != a[i] {
                fail = true;
            }
        }

        if !fail {
            return Some(magic);
        }
    }

    None
}

fn index_to_bb(index: usize, bits: u32, mut mask: BitBoard) -> BitBoard {
    let mut result = 0;

    for i in 0..bits {
        let j: usize = mask.pop().unwrap().into();

        if (index & (1 << i)) > 0 {
            result |= 1 << j;
        }
    }

    BitBoard::from_u64(result)
}

fn ratt(square: Square, block: BitBoard) -> BitBoard {
    let mut result = 0;
    let block = block.to_u64();
    let rank: u8 = square.rank().into();
    let file: u8 = square.file().into();

    for r in rank + 1..=7 {
        result |= 1 << (file + r * 8);
        if (block & (1 << (file + r * 8))) > 0 {
            break;
        }
    }

    for r in (0..=rank.saturating_sub(1)).rev() {
        result |= 1 << (file + r * 8);
        if (block & (1 << (file + r * 8))) > 0 {
            break;
        }
    }

    for f in file + 1..=7 {
        result |= 1 << (f + rank * 8);
        if (block & (1 << (f + rank * 8))) > 0 {
            break;
        }
    }

    for f in (0..=file.saturating_sub(1)).rev() {
        result |= 1 << (f + rank * 8);
        if (block & (1 << (f + rank * 8))) > 0 {
            break;
        }
    }

    BitBoard::from_u64(result)
}

fn batt(square: Square, block: BitBoard) -> BitBoard {
    let mut result = 0;
    let block = block.to_u64();
    let rank: u8 = square.rank().into();
    let file: u8 = square.file().into();

    for (r, f) in (rank + 1..=7).zip(file + 1..=7) {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    for (r, f) in (rank + 1..=7).zip((0..=file.saturating_sub(1)).rev()) {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    for (r, f) in (0..=rank.saturating_sub(1)).rev().zip(file + 1..=7) {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    for (r, f) in (0..=rank.saturating_sub(1))
        .rev()
        .zip((0..=file.saturating_sub(1)).rev())
    {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    BitBoard::from_u64(result)
}

fn rand_u64_fewbits() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>()
}

fn transform(b: BitBoard, magic: u64, bits: u32) -> u32 {
    // Faster methods?
    // ((b as i32) * (magic as i32) ^ ((b >> 32) as i32) * ((magic >> 32) as i32)) as u32 >> (32 - bits)

    ((b.to_u64().wrapping_mul(magic)) >> (64 - bits)) as u32
}
