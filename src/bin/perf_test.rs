use mattis::board::Board;
use std::fs;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
struct Statistics {
    leaves: u32,
    captures: u32,
    ep: u32,
    checks: u32,
    castles: u32,
}

fn main() {
    const MAX_LEAVES: u32 = 1_000_000;
    let testsuite = fs::read_to_string("perftsuite.epd").unwrap();

    for line in testsuite.lines() {
        let mut parts = line.split(';');
        let fen = parts.next().unwrap();
        let mut board = Board::from_fen(fen).unwrap();

        for (depth, p) in parts.enumerate() {
            let depth = depth + 1;
            let expected_leaves: u32 = p.split_whitespace().nth(1).unwrap().parse().unwrap();

            if expected_leaves > MAX_LEAVES {
                break;
            }

            let mut stats = Statistics::default();

            print!("Depth {depth}: expect {expected_leaves} leaves");

            perft(&mut board, depth, &mut stats);
            let actual_leaves = stats.leaves;
            let success = expected_leaves == actual_leaves;

            let symbol = if success { '✓' } else { '✗' };
            println!(", got {actual_leaves} --> {symbol}");

            if !success {
                eprintln!("Test failed!");
                eprintln!("{board:#}");
                println!("{stats:#?}");
                panic!("Test failed!");
            }
        }
    }
}

fn perft(board: &mut Board, depth: usize, stats: &mut Statistics) {
    #[cfg(debug_assertions)]
    board.check_board_integrity();

    if depth == 0 {
        stats.leaves += 1;
        return;
    }

    for m in board.generate_all_moves() {
        if !board.make_move(m) {
            continue;
        }

        if depth == 1 {
            if m.m16.is_capture() {
                stats.captures += 1;
            }

            if m.m16.is_en_passant() {
                stats.ep += 1;
            }

            if m.m16.is_kingside_castle() || m.m16.is_queenside_castle() {
                stats.castles += 1;
            }

            if board.is_square_attacked(board.king_square[board.color], board.color.flipped()) {
                stats.checks += 1;
            }
        }

        perft(board, depth - 1, stats);
        board.take_move();
    }
}
