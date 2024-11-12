#![warn(clippy::return_self_not_must_use)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod board;
pub mod chess_move;
pub mod eval;
pub mod hashtable;
pub mod notation;
pub mod perft;
pub mod search;
pub mod tables;
pub mod time_man;
