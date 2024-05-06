#![cfg_attr(rustfmt, rustfmt_skip)]

use mattis_bitboard::BitBoard;

pub const ZOBRIST_PIECE_KEYS:      [[u64; 12]; 64] = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/zobrist_piece_keys")) };
pub const ZOBRIST_COLOR_KEY:       u64             = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/zobrist_color_key")) };
pub const ZOBRIST_CASTLE_KEYS:     [u64; 16]       = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/zobrist_castle_keys")) };
pub const ZOBRIST_EN_PASSANT_KEYS: [u64; 64]       = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/zobrist_en_passant_keys")) };
pub const BORDER:                   BitBoard       = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/border")) };
pub const FILE_BITBOARDS:          [BitBoard;  8]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/file_bitboards")) };
pub const NOT_FILE_BITBOARDS:      [BitBoard;  8]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/not_file_bitboards")) };
pub const RANK_BITBOARDS:          [BitBoard;  8]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/rank_bitboards")) };
pub const NOT_RANK_BITBOARDS:      [BitBoard;  8]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/not_rank_bitboards")) };
pub const WHITE_PAWN_PASSED_MASKS: [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/white_pawn_passed_masks")) };
pub const BLACK_PAWN_PASSED_MASKS: [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/black_pawn_passed_masks")) };
pub const ISOLATED_PAWN_MASKS:     [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/isolated_pawn_masks")) };
pub const KNIGHT_MOVE_PATTERNS:    [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/knight_move_patterns")) };
pub const KING_MOVE_PATTERNS:      [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/king_move_patterns")) };
pub const ROOK_MOVE_PATTERNS:      [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/rook_move_patterns")) };
pub const BISHOP_MOVE_PATTERNS:    [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/bishop_move_patterns")) };
pub const ROOK_MAGIC_BIT_COUNT:    [u32; 64]       = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/rook_magic_bit_count")) };
pub const BISHOP_MAGIC_BIT_COUNT:  [u32; 64]       = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/bishop_magic_bit_count")) };
pub const ROOK_MAGIC_MASKS:        [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/rook_magic_masks")) };
pub const BISHOP_MAGIC_MASKS:      [BitBoard; 64]  = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/bishop_magic_masks")) };
pub const ROOK_MAGICS:             [u64; 64]       = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/rook_magics")) };
pub const BISHOP_MAGICS:           [u64; 64]       = unsafe { std::mem::transmute(*include_bytes!("../../target/generated_tables/bishop_magics")) };