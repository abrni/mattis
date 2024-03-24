#![cfg_attr(rustfmt, rustfmt_skip)]

use mattis_bitboard::BitBoard;

pub const FILE_BITBOARDS:          [BitBoard;  8] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/file_bitboards")) };
pub const NOT_FILE_BITBOARDS:      [BitBoard;  8] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/not_file_bitboards")) };
pub const RANK_BITBOARDS:          [BitBoard;  8] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/rank_bitboards")) };
pub const NOT_RANK_BITBOARDS:      [BitBoard;  8] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/not_rank_bitboards")) };
pub const BORDER:                   BitBoard      = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/border")) };
pub const WHITE_PAWN_PASSED_MASKS: [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/white_pawn_passed_masks")) };
pub const BLACK_PAWN_PASSED_MASKS: [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/black_pawn_passed_masks")) };
pub const ISOLATED_PAWN_MASKS:     [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/isolated_pawn_masks")) };
pub const KNIGHT_MOVE_PATTERNS:    [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/knight_move_patterns")) };
pub const KING_MOVE_PATTERNS:      [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/king_move_patterns")) };
pub const ROOK_MOVE_PATTERNS:      [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/rook_move_patterns")) };
pub const BISHOP_MOVE_PATTERNS:    [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("../_static_tables/bishop_move_patterns")) };