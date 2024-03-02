use mattis::board::Board;

fn main() {
    let board =
        Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2").unwrap();

    println!("{board}");
    println!();
    println!("{board:#}");
}
