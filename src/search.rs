use crate::{board::Board, eval::evaluation, tptable::TpTable};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SearchStats {
    pub nodes: usize,     //  total count of visited nodes
    pub leaves: usize,    // total count of visited leaf nodes
    pub fh: usize,        // counts fail-highs (beta cut off)
    pub fhf: usize,       // counts fail-highs at the first move
    pub tptable: TpTable, // Transposition table
}

pub fn iterative_deepening(board: &mut Board, depth: usize, stats: &mut SearchStats) -> i32 {
    let mut score = 0;
    for i in 1..=depth {
        score = alpha_beta(-i32::MAX, i32::MAX, i, board, stats);
    }

    score
}

pub fn alpha_beta(
    mut alpha: i32,
    beta: i32,
    depth: usize,
    board: &mut Board,
    stats: &mut SearchStats,
) -> i32 {
    stats.nodes += 1;

    if depth == 0 {
        stats.leaves += 1;
        return evaluation(board); // TODO: use quiescence search
    }

    let mut moves = Vec::with_capacity(32); // TODO: reuse a preallocated vec
    board.generate_all_moves(&mut moves);

    if let Some(m) = stats.tptable.get(board.position_key) {
        let pos = moves.iter().position(|m32| m32.m16 == m).unwrap();
        moves.swap(0, pos);
    }

    let mut new_best_move = None;

    for (i, m) in moves.into_iter().enumerate() {
        let is_valid_move = board.make_move(m);

        if !is_valid_move {
            continue;
        }

        let score = -alpha_beta(-beta, -alpha, depth - 1, board, stats);
        board.take_move();

        if score >= beta {
            stats.fh += 1;

            if i == 0 {
                stats.fhf += 1
            };

            return beta; // fail hard beta-cutoff
        }

        if score > alpha {
            alpha = score;
            new_best_move = Some(m);
        }
    }

    if let Some(m) = new_best_move {
        stats.tptable.insert(board.position_key, m.m16);
    }

    alpha
}
