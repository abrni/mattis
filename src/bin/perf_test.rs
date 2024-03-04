use mattis::board::{movegen::MoveList, Board};
use std::{fs, io::Write};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
struct Statistics {
    leaves: u32,
    captures: u32,
    ep: u32,
    checks: u32,
    castles: u32,
}

fn main() {
    const MAX_LEAVES: u32 = 9_999_999;
    let testsuite = fs::read_to_string("perftsuite.epd").unwrap();
    let print_width = (1 + MAX_LEAVES.ilog10()) as usize;

    for line in testsuite.lines() {
        let mut parts = line.split(';');
        let fen = parts.next().unwrap();
        let mut board = Board::from_fen(fen).unwrap();
        println!("{fen}:");

        for (depth, p) in parts.enumerate() {
            let depth = depth + 1;
            let expected_leaves: u32 = p.split_whitespace().nth(1).unwrap().parse().unwrap();

            if expected_leaves > MAX_LEAVES {
                break;
            }

            let mut stats = Statistics::default();

            print!("    Depth {depth}: expect {expected_leaves:>print_width$} leaves");
            std::io::stdout().flush().unwrap();

            let mut lists = vec![MoveList::with_capacity(32); 8];
            perft(&mut board, depth, &mut stats, lists.as_mut_slice());
            let actual_leaves = stats.leaves;
            let success = expected_leaves == actual_leaves;

            let symbol = if success { '✓' } else { '✗' };
            println!(", got {actual_leaves:>print_width$} -> {symbol}");

            if !success {
                eprintln!("Test failed!");
                eprintln!("{board:#}");
                println!("{stats:#?}");
                panic!("Test failed!");
            }
        }
    }
}

fn perft(board: &mut Board, depth: usize, stats: &mut Statistics, lists: &mut [MoveList]) {
    #[cfg(debug_assertions)]
    board.check_board_integrity();

    if depth == 0 {
        stats.leaves += 1;
        return;
    }

    let (first, rest) = lists.split_first_mut().unwrap();

    first.clear();
    board.generate_all_moves(first);
    for m in first {
        if !board.make_move(*m) {
            continue;
        }

        if depth == 1 {
            if m.is_capture() {
                stats.captures += 1;
            }

            if m.is_en_passant() {
                stats.ep += 1;
            }

            if m.is_kingside_castle() || m.is_queenside_castle() {
                stats.castles += 1;
            }

            if board.is_square_attacked(board.king_square[board.color], board.color.flipped()) {
                stats.checks += 1;
            }
        }

        perft(board, depth - 1, stats, rest);
        board.take_move();
    }
}
