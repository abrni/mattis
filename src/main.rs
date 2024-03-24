use std::{
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
};

use mattis::{
    board::Board,
    chess_move::{ChessMove, Notation},
    hashtable::TranspositionTable,
    search::{self, KillSwitch, SearchConfig},
    uci::{self, EngineMessage, GuiMessage, Id},
};

const FEN_STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const HASHTABLE_SIZE_MB: usize = 256;
const THREAD_COUNT: u32 = 12;

fn main() {
    let ttable = Arc::new(TranspositionTable::new(HASHTABLE_SIZE_MB));
    let search_killers = Arc::new(Mutex::new(vec![[ChessMove::default(); 2]; 1024].into_boxed_slice()));
    let search_history = Arc::new(Mutex::new([[0; 64]; 12]));
    let mut board = Board::from_fen(FEN_STARTPOS).unwrap();
    let mut active_search_kill: Option<KillSwitch> = None;

    let mut stdin = BufReader::new(std::io::stdin());
    let mut input = String::new();

    loop {
        input.clear();
        stdin.read_line(&mut input).expect("Could not read input line");

        let Ok(message) = GuiMessage::parse(&input) else {
            println!("Received unknown command");
            continue;
        };

        match message {
            GuiMessage::Uci => print_uci_info(),
            GuiMessage::Ucinewgame => {
                ttable.reset();
            }
            GuiMessage::Isready => println!("{}", EngineMessage::Readyok),
            GuiMessage::Position { pos, moves } => setup_position(&mut board, pos, &moves),
            GuiMessage::Go(go) => {
                if active_search_kill.as_ref().is_some_and(|k| k.is_alive()) {
                    println!("Already searching");
                    continue;
                }

                ttable.next_age();
                let config = SearchConfig {
                    allow_null_pruning: true,
                    thread_count: THREAD_COUNT,
                    go,
                    board: &board,
                    tp_table: Arc::clone(&ttable),
                    search_killers: Arc::clone(&search_killers),
                    search_history: Arc::clone(&search_history),
                };

                active_search_kill = Some(search::run_search(config));
            }
            GuiMessage::Stop => {
                if let Some(s) = active_search_kill.take() {
                    s.kill()
                }
            }
            GuiMessage::Quit => {
                if let Some(s) = active_search_kill.take() {
                    s.kill()
                }
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
        let chess_move = board.find_move(move_str, Notation::Smith);

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
