use std::io::{BufRead, BufReader};

use mattis::{board::Board, uci::GuiMessage};

const FEN_STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let mut board = Board::from_fen(FEN_STARTPOS).unwrap();
    let mut stdin = BufReader::new(std::io::stdin());
    let mut input = String::new();

    loop {
        input.clear();
        stdin
            .read_line(&mut input)
            .expect("Could not read input line");

        let message = GuiMessage::parse(&input).expect("Received unknown command");
        dbg!(message);
    }
}
