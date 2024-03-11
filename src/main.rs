use std::{
    io::{BufRead, BufReader},
    sync::Arc,
};

use mattis::{
    board::{movegen::MoveList, Board},
    hashtable::TranspositionTable,
    search::{self, SearchConfig},
    uci::{self, EngineMessage, GuiMessage, Id},
};

const FEN_STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const HASHTABLE_SIZE_MB: usize = 256;
const THREAD_COUNT: u32 = 12;

fn main() {
    let ttable = Arc::new(TranspositionTable::new(HASHTABLE_SIZE_MB));
    let mut board = Board::from_fen(FEN_STARTPOS).unwrap();
    let mut active_search_kill = None;

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
                ttable.next_age();
                let config = SearchConfig {
                    allow_null_pruning: true,
                    thread_count: THREAD_COUNT,
                    go,
                    board: &board,
                    tp_table: Arc::clone(&ttable),
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
