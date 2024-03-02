use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mattis::board::Board;

const MAX_LEAVES: u32 = 999_999;

fn perf_bench(c: &mut Criterion) {
    let testsuite = std::fs::read_to_string("perftsuite.epd").unwrap();
    let mut group = c.benchmark_group("perft_group");

    for line in testsuite.lines().take(10) {
        let mut parts = line.split(';');
        let fen = parts.next().unwrap();

        for (depth, p) in parts.enumerate().take(1) {
            let depth = depth + 1;
            let expected_leaves: u32 = p.split_whitespace().nth(1).unwrap().parse().unwrap();

            if expected_leaves > MAX_LEAVES {
                break;
            }

            let id = BenchmarkId::from_parameter(format!("{fen}: {depth}"));

            group.bench_with_input(id, &(fen, depth), |b, (fen, depth)| {
                let mut board = Board::from_fen(fen).unwrap();

                b.iter(|| {
                    let actual_leaves = perft(&mut board, *depth);
                    assert_eq!(expected_leaves, actual_leaves);
                });
            });
        }
    }

    group.finish();
}

fn perft(board: &mut Board, depth: usize) -> u32 {
    #[cfg(debug_assertions)]
    board.check_board_integrity();

    if depth == 0 {
        return 1;
    }

    let mut leaves = 0;
    let moves = board.generate_all_moves();

    for m in moves {
        if !board.make_move(m) {
            continue;
        }

        leaves += perft(board, depth - 1);
        board.take_move();
    }

    leaves
}

criterion_group!(benches, perf_bench);
criterion_main!(benches);
