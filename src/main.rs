use std::{
    io::{BufRead, BufReader},
    time::Duration,
};

use mattis::{
    board::Board,
    eval::evaluation,
    moves::Move32,
    search::{iterative_deepening, SearchParams, SearchTables},
    tptable::TranspositionTable,
    types::Color,
    uci::{self, EngineMessage, GuiMessage, Id},
};

const FEN_STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let mut board = Board::from_fen(FEN_STARTPOS).unwrap();
    let mut stdin = BufReader::new(std::io::stdin());
    let mut search_tables = SearchTables {
        transposition_table: TranspositionTable::new(),
        search_killers: vec![[Move32::default(); 2]; 1024],
        search_history: [[0; 64]; 12],
    };

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
            GuiMessage::Isready => println!("{}", EngineMessage::Readyok),
            GuiMessage::Position { pos, moves } => {
                setup_position(&mut board, pos, &moves);
                dbg!(evaluation(&board));
            }
            GuiMessage::Go(go) => run_go(&mut board, go, &mut search_tables),
            _ => println!("This uci command is currently not supported."),
        }
    }
}

fn run_go(board: &mut Board, go: uci::Go, search_tables: &mut SearchTables) {
    let (time, inc) = match board.color {
        Color::White => (go.wtime, go.winc),
        Color::Black => (go.btime, go.binc),
        Color::Both => todo!(),
    };

    let movestogo = go.movestogo.unwrap_or(30) as f64;
    let (time, inc) = (time.or(go.movetime), inc.unwrap_or(0) as f64);

    let max_time = time
        .map(|t| t as f64)
        .map(|t| (t + (movestogo * inc)) / (movestogo / 3.0 * 2.0) - inc)
        .map(|t| Duration::from_micros((t * 1000.0) as u64));

    let params = SearchParams {
        max_time,
        max_nodes: go.nodes.map(|n| n as u64),
        max_depth: go.depth,
    };

    let mut bestmove = Move32::default();
    for stats in iterative_deepening(board, params, search_tables) {
        let info = EngineMessage::Info(uci::Info {
            depth: Some(stats.depth),
            nodes: Some(stats.nodes as u32),
            pv: stats.pv.into_iter().map(|m| format!("{m}")).collect(),
            score: Some(uci::Score::Cp(stats.score)),
            ..Default::default()
        });

        println!("{info}");
        println!("Ordering: {:.2}", stats.fhf as f64 / stats.fh as f64);
        bestmove = stats.bestmove;
    }

    // let hashfull = tptable.len() as f64 / tptable.capacity() as f64;
    // let hashfull = (hashfull * 1000.0) as u32;
    // let info = EngineMessage::Info(uci::Info {
    //     hashfull: Some(hashfull),
    //     string: Some(format!(
    //         "f {} c {}",
    //         tptable.fill_level(),
    //         tptable.collisions()
    //     )),
    //     ..Default::default()
    // });
    //
    // println!("{info}");

    let bestmove = EngineMessage::Bestmove {
        move_: format!("{bestmove}"),
        ponder: None,
    };

    println!("{bestmove}");
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
        let mut movelist = vec![];
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
    board.history.clear();
}
