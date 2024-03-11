use std::sync::Arc;

use mattis::{
    board::Board,
    hashtable::TranspositionTable,
    search::{self, SearchConfig},
    uci,
};

const FEN_STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const HASHTABLE_SIZE_MB: usize = 256;

fn main() {
    let thread_count = std::env::args().nth(1).unwrap_or("1".to_string());
    let thread_count: u32 = thread_count.parse().unwrap();
    let board = Board::from_fen(FEN_STARTPOS).unwrap();
    let ttable = Arc::new(TranspositionTable::new(HASHTABLE_SIZE_MB));

    let go = uci::Go {
        depth: Some(9),
        ..Default::default()
    };

    ttable.next_age();
    let config = SearchConfig {
        allow_null_pruning: true,
        thread_count,
        go,
        board: &board,
        tp_table: Arc::clone(&ttable),
    };

    let killswitch = search::run_search(config);
    while killswitch.is_alive() {}
}
