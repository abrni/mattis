use std::io::Write;

use crate::board::{movegen::MoveList, Board};

const MAX_LEAVES: u32 = u32::MAX;

pub fn perft_full() {
    let testsuite = std::fs::read_to_string("perftsuite.epd").unwrap();

    for line in testsuite.lines() {
        let mut parts = line.split(';');
        let fen = parts.next().unwrap();
        println!("{}", fen);

        for (depth, p) in parts.enumerate() {
            let depth = depth + 1;
            let expected_leaves: u32 = p.split_whitespace().nth(1).unwrap().parse().unwrap();

            print!(" - depth {depth:02}, expect {expected_leaves} leaves ... ");
            std::io::stdout().flush().unwrap();

            #[allow(clippy::absurd_extreme_comparisons)]
            if expected_leaves > MAX_LEAVES {
                println!("skipping");
                break;
            }

            let mut board = Board::from_fen(fen).unwrap();
            let actual_leaves = perft(&mut board, depth, false);
            println!("got {actual_leaves}");
            assert_eq!(expected_leaves, actual_leaves);
        }
    }
}

/// Makes all legal moves up to the given depth and returns the total number of reached leaf positions.
///
/// If `check_integrity` is set, the board structure is checked for correctness in each position.
/// This results in a significant runtime overhead and is much slower.
/// It is recommended to only enable this, when perft results don't match the expected result.
pub fn perft(board: &mut Board, depth: usize, check_integrity: bool) -> u32 {
    // Run integrity checking once at the beginning and the end of the function
    if check_integrity {
        board.check_board_integrity();
    }

    // At depth 0 we are already at a leave position, no matter what.
    // Nothing to do.
    if depth == 0 {
        return 1;
    }

    let mut movelist = MoveList::default();
    board.generate_all_moves(&mut movelist);
    let mut sum = 0;

    // Try to make each move in the movelist.
    for m in movelist {
        // Skip pseudomoves (moves, that cannot be made, because the would lead to an illegal position).
        if !board.make_move(m) {
            continue;
        };

        // Sum the leave count of each move for the final result
        sum += perft(board, depth - 1, check_integrity);
        board.take_move();
    }

    // Run integrity checking once at the beginning and the end of the function
    if check_integrity {
        board.check_board_integrity();
    }

    sum
}
