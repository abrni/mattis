use std::ops::{BitAnd, Mul};

use mattis::{
    bitboard::{BitBoard, BISHOP_MAGIC_MASKS, ROOK_MAGIC_MASKS},
    types::Square64,
};
use num_enum::FromPrimitive;
use rand::{thread_rng, Rng};

fn main() {
    for square in 0..64 {
        let square = Square64::from_primitive(square);
        let rmagic = find_magic(square, ROOK_BITS[square as usize], false);
        println!("{rmagic:0x?}");
    }

    for square in 0..64 {
        let square = Square64::from_primitive(square);
        let bmagic = find_magic(square, BISHOP_BITS[square as usize], true);
        println!("{bmagic:0x?}");
    }
}

#[rustfmt::skip]
const ROOK_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

#[rustfmt::skip]
const BISHOP_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

fn find_magic(square: Square64, m: u32, is_bishop: bool) -> Option<u64> {
    let mut b = [BitBoard::EMPTY; 4096];
    let mut a = [BitBoard::EMPTY; 4096];

    let mask = if is_bishop {
        BISHOP_MAGIC_MASKS[square as usize]
    } else {
        ROOK_MAGIC_MASKS[square as usize]
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
            .mul(magic)
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
        let j: usize = mask.pop().into();

        if (index & (1 << i)) > 0 {
            result |= 1 << j;
        }
    }

    BitBoard::from_u64(result)
}

fn ratt(square: Square64, block: BitBoard) -> BitBoard {
    let mut result = 0;
    let block = block.to_u64();
    let rank: u8 = square.rank().unwrap().into();
    let file: u8 = square.file().unwrap().into();

    for r in rank + 1..=7 {
        result |= 1 << (file + r * 8);
        if (block & (1 << (file + r * 8))) > 0 {
            break;
        }
    }

    for r in (0..=rank - 1).rev() {
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

    for f in (0..=file - 1).rev() {
        result |= 1 << (f + rank * 8);
        if (block & (1 << (f + rank * 8))) > 0 {
            break;
        }
    }

    BitBoard::from_u64(result)
}

fn batt(square: Square64, block: BitBoard) -> BitBoard {
    let mut result = 0;
    let block = block.to_u64();
    let rank: u8 = square.rank().unwrap().into();
    let file: u8 = square.file().unwrap().into();

    for (r, f) in (rank + 1..=7).zip(file + 1..=7) {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    for (r, f) in (rank + 1..=7).zip((0..=file - 1).rev()) {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    for (r, f) in (0..=rank - 1).rev().zip(file + 1..=7) {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    for (r, f) in (0..=rank - 1).rev().zip((0..=file - 1).rev()) {
        result |= 1 << (f + r * 8);
        if (block & (1 << (f + r * 8))) > 0 {
            break;
        }
    }

    BitBoard::from_u64(result)
}

fn rand_u64_fewbits() -> u64 {
    let mut rng = thread_rng();
    rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>()
}

fn transform(b: BitBoard, magic: u64, bits: u32) -> u32 {
    // Faster methods?
    // ((b as i32) * (magic as i32) ^ ((b >> 32) as i32) * ((magic >> 32) as i32)) as u32 >> (32 - bits)

    ((b.to_u64() * magic) >> (64 - bits)) as u32
}