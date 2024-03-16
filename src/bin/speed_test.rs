use std::sync::{Arc, Mutex};

use mattis::{
    board::Board,
    chess_move::ChessMove,
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
    let search_killers = Arc::new(Mutex::new(vec![[ChessMove::default(); 2]; 1024].into_boxed_slice()));
    let search_history = Arc::new(Mutex::new([[0; 64]; 12]));

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
        search_killers: Arc::clone(&search_killers),
        search_history: Arc::clone(&search_history),
    };

    let killswitch = search::run_search(config);
    while killswitch.is_alive() {}
}
