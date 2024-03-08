use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use bus::{Bus, BusReader};
use mattis::{
    board::{movegen::MoveList, Board},
    chess_move::ChessMove,
    hashtable::TranspositionTable,
    search::{iterative_deepening, SearchParams, SearchTables},
    types::Color,
    uci::{self, EngineMessage, Id},
};

const FEN_STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const HASHTABLE_SIZE_MB: usize = 256;

fn main() {
    let thread_count = std::env::args().nth(1).unwrap_or("1".to_string());
    let thread_count: usize = thread_count.parse().unwrap();

    let mut bus = Bus::new(1024);
    let search_stop = Arc::new(AtomicBool::new(false));
    let ttable = Arc::new(TranspositionTable::new(HASHTABLE_SIZE_MB));
    let mut threads = vec![];

    for i in 0..thread_count {
        let rx = bus.add_rx();
        let search_stop = Arc::clone(&search_stop);
        let ttable = Arc::clone(&ttable);
        let is_primary = i == 0;

        let handle = std::thread::spawn(move || search_thread(is_primary, rx, search_stop, ttable));
        threads.push(handle);
    }

    let go = uci::Go {
        depth: Some(9),
        ..Default::default()
    };

    ttable.next_age();
    bus.broadcast(ThreadCommand::Go(go.clone()));

    for t in threads {
        t.join().unwrap();
    }
}

#[derive(Debug, Clone)]
enum ThreadCommand {
    // Quit,
    // SetupPosition(uci::Position, Vec<String>),
    // NewGame,
    Go(uci::Go),
}

fn search_thread(
    is_primary: bool,
    mut rx: BusReader<ThreadCommand>,
    search_stop: Arc<AtomicBool>,
    ttable: Arc<TranspositionTable>,
) {
    let mut board = Board::from_fen(FEN_STARTPOS).unwrap();

    let mut search_tables = SearchTables {
        transposition_table: ttable,
        search_killers: vec![[ChessMove::default(); 2]; 1024],
        search_history: [[0; 64]; 12],
    };

    let command = rx.recv().unwrap();

    match command {
        //ThreadCommand::Quit => return,
        //ThreadCommand::SetupPosition(pos, moves) => setup_position(&mut board, pos, &moves),
        // ThreadCommand::NewGame => (),
        ThreadCommand::Go(go) => run_go(is_primary, &mut board, go, &mut search_tables, Arc::clone(&search_stop)),
    }
}

fn run_go(print_output: bool, board: &mut Board, go: uci::Go, search_tables: &mut SearchTables, stop: Arc<AtomicBool>) {
    search_tables.search_history = [[0; 64]; 12];
    search_tables
        .search_killers
        .iter_mut()
        .for_each(|k| *k = Default::default());

    let (time, inc) = match board.color {
        Color::White => (go.wtime, go.winc),
        Color::Black => (go.btime, go.binc),
    };

    let movestogo = go.movestogo.unwrap_or(30) as f64;
    let (time, inc) = (time.or(go.movetime), inc.unwrap_or(0) as f64);

    let max_time = time
        .map(|t| t as f64)
        .map(|t| (t + (movestogo * inc)) / (movestogo / 3.0 * 2.0) - inc)
        .map(|t| Duration::from_micros((t * 1000.0) as u64));

    stop.store(false, Ordering::Release);
    let params = SearchParams {
        max_time,
        max_nodes: go.nodes.map(|n| n as u64),
        max_depth: go.depth.map(|d| d as u16), // TODO: guard against too high numbers
        stop,
    };

    let mut bestmove = ChessMove::default();
    for stats in iterative_deepening(board, params, search_tables) {
        bestmove = stats.bestmove;
        let info = EngineMessage::Info(uci::Info {
            depth: Some(stats.depth as u32),
            nodes: Some(stats.nodes as u32),
            pv: stats.pv.into_iter().map(|m| format!("{m}")).collect(),
            score: Some(uci::Score(stats.score)),
            ..Default::default()
        });

        if print_output {
            println!("{info}");
            println!("Ordering: {:.2}", stats.fhf as f64 / stats.fh as f64);
        }
    }

    if print_output {
        let bestmove = EngineMessage::Bestmove {
            move_: format!("{bestmove}"),
            ponder: None,
        };

        println!("{bestmove}");
    }
}

fn _print_uci_info() {
    let name_msg: EngineMessage = EngineMessage::Id(Id::Name("Mattis".to_string()));
    let author_msg: EngineMessage = EngineMessage::Id(Id::Author("Anton Bornhoeft".to_string()));

    println!("{name_msg}",);
    println!("{author_msg}");
    println!("{}", EngineMessage::Uciok);
}

fn _setup_position(board: &mut Board, pos: uci::Position, moves: &[String]) {
    let fen = match &pos {
        uci::Position::Fen(fen) => fen,
        uci::Position::Startpos => FEN_STARTPOS,
    };

    *board = Board::from_fen(fen).unwrap();

    'outer: for m in moves {
        let mut movelist = MoveList::new();
        board.generate_all_moves(&mut movelist);

        for bm in movelist {
            if (format!("{bm}")) == *m {
                board.make_move(bm);
                continue 'outer;
            }
        }

        panic!("Invalid move");
    }

    board.ply = 0;
}
