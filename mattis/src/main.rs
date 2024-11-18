use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use clap::{Parser, Subcommand};
use mattis::{
    board::Board,
    hashtable::TranspositionTable,
    notation::SmithNotation,
    perft::perft_full,
    search::{
        history::SearchHistory,
        killers::SearchKillers,
        lazy_smp::{LazySMP, SearchConfig},
        ReportMode,
    },
};
use mattis_uci::{self as uci, EngineMessage, GuiMessage, Id};

const FEN_STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const HASHTABLE_SIZE_MB: usize = 256;
const THREAD_COUNT: u32 = 12;

#[derive(Debug, Parser, Clone)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Default, Subcommand, Clone)]
enum Command {
    /// Starts the engine in UCI mode. (Default)
    #[default]
    Uci,

    /// Runs a perft testsuite.
    Perft {
        /// Skip tests with this many or more expected leaf nodes.
        #[arg(long, short)]
        skip: Option<u32>,
        /// Read testcases from a file. Otherwise a default builtin testsuite is used.
        #[arg(long, short)]
        file: Option<PathBuf>,
    },

    /// Runs a single search.
    Search {
        /// Start position in FEN format.
        #[arg(long, short, default_value_t = FEN_STARTPOS.to_string())]
        startpos: String,

        /// Disable null pruning
        #[arg(long)]
        no_null_pruning: bool,
    },
}

fn main() {
    let args = Args::parse();
    let command = args.command.unwrap_or(Command::Uci); // Default to UCI, if no command is given

    match command {
        Command::Uci => uci_loop(),
        Command::Perft { file, skip } => perft_full(file.as_deref(), skip),
        Command::Search {
            startpos,
            no_null_pruning,
        } => single_search(&startpos, !no_null_pruning),
    }
}

fn single_search(startpos: &str, null_pruning: bool) {
    let ttable = Arc::new(TranspositionTable::new(HASHTABLE_SIZE_MB));
    let search_killers = Arc::new(RwLock::new(SearchKillers::default()));
    let search_history = Arc::new(RwLock::new(SearchHistory::default()));
    let board = Board::from_fen(startpos).unwrap();

    let go = uci::Go {
        depth: Some(13),
        ..Default::default()
    };

    let search_config = SearchConfig {
        report_mode: ReportMode::Full,
        allow_null_pruning: null_pruning,
        thread_count: THREAD_COUNT,
        go,
        board: &board,
        tp_table: Arc::clone(&ttable),
        search_killers: Arc::clone(&search_killers),
        search_history: Arc::clone(&search_history),
    };
    let config = search_config;

    let mut lazysmp = LazySMP::create(THREAD_COUNT as usize);
    lazysmp.start_search(config).unwrap();
    while lazysmp.is_search_running() {}
}

fn uci_loop() {
    let ttable = Arc::new(TranspositionTable::new(HASHTABLE_SIZE_MB));
    let search_killers = Arc::new(RwLock::new(SearchKillers::default()));
    let search_history = Arc::new(RwLock::new(SearchHistory::default()));
    let mut board = Board::from_fen(FEN_STARTPOS).unwrap();
    let mut lazysmp = LazySMP::create(THREAD_COUNT as usize);

    let mut stdin = BufReader::new(std::io::stdin());
    let mut input = String::new();

    loop {
        input.clear();
        stdin.read_line(&mut input).expect("Must be able to read from stdin");

        let Ok(message) = GuiMessage::parse(&input) else {
            println!("Received unknown command");
            continue;
        };

        match message {
            GuiMessage::Uci => print_uci_info(),
            GuiMessage::Ucinewgame => {
                ttable.reset();
                *search_history.write().unwrap() = SearchHistory::default();
                *search_killers.write().unwrap() = SearchKillers::default();
                board = Board::from_fen(FEN_STARTPOS).unwrap();
                let _ = lazysmp.stop_search();
            }
            GuiMessage::Isready => println!("{}", EngineMessage::Readyok),
            GuiMessage::Position { pos, moves } => setup_position(&mut board, pos, &moves),
            GuiMessage::Go(go) => {
                if lazysmp.is_search_running() {
                    println!("Already searching");
                    continue;
                }

                let config = SearchConfig {
                    report_mode: ReportMode::Uci,
                    allow_null_pruning: true,
                    thread_count: THREAD_COUNT,
                    go,
                    board: &board,
                    tp_table: Arc::clone(&ttable),
                    search_killers: Arc::clone(&search_killers),
                    search_history: Arc::clone(&search_history),
                };

                lazysmp.start_search(config).unwrap();
            }
            GuiMessage::Stop => {
                let _ = lazysmp.stop_search();
            }
            GuiMessage::Quit => {
                let _ = lazysmp.stop_search();
                return;
            }
            _ => println!("This uci command is currently not supported."),
        }
    }
}

fn print_uci_info() {
    let name_msg: EngineMessage = EngineMessage::Id(Id::Name("Mattis".to_string()));
    let author_msg: EngineMessage = EngineMessage::Id(Id::Author("Anton Bornhoeft".to_string()));

    println!("{name_msg}",);
    println!("{author_msg}");
    println!("{}", EngineMessage::Uciok);
}

fn setup_position(board: &mut Board, pos: uci::Position, moves: &[String]) {
    let fen = match &pos {
        uci::Position::Fen(fen) => fen,
        uci::Position::Startpos => FEN_STARTPOS,
    };

    *board = Board::from_fen(fen).unwrap();

    for move_str in moves {
        let chess_move = board.find_move::<SmithNotation>(move_str);

        if let Some(cm) = chess_move {
            board.make_move(cm);
        } else {
            *board = Board::from_fen(FEN_STARTPOS).unwrap();
            println!("Invalid move `{move_str}`. Setting up `startpos` instead.");
            break;
        }
    }

    board.ply = 0;
}
