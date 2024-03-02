use crate::{board::Board, eval::evaluation, moves::Move32, tptable::TpTable, types::Piece};
use std::time::{Duration, Instant};

pub struct SearchParams {
    pub max_time: Option<Duration>,
    pub max_nodes: Option<u64>,
    pub max_depth: Option<u32>,
    // TODO: Support for Mate Search
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SearchStats {
    pub depth: u32,      // Search depth
    pub score: i32,      // Score in centipawns
    pub nodes: u64,      // Total count of visited nodes
    pub leaves: u64,     // Total count of visited leaf nodes
    pub fh: u64,         // Count of fail-highs (beta cut off)
    pub fhf: u64,        // Count of fail-highs at the first move
    pub pv: Vec<Move32>, // Principle Variation Line
}

pub fn pv_line(tptable: &TpTable, board: &mut Board) -> Vec<Move32> {
    let mut pvline = Vec::with_capacity(8);

    while let Some(m16) = tptable.get(board.position_key) {
        let m32 = board.move_16_to_32(m16);
        board.make_move(m32);
        pvline.push(m32);
    }

    for _ in 0..pvline.len() {
        board.take_move();
    }

    pvline
}

pub fn iterative_deepening<'a>(
    board: &'a mut Board,
    params: SearchParams,
    tptable: &'a mut TpTable,
) -> impl Iterator<Item = SearchStats> + 'a {
    let max_depth = params.max_depth.unwrap_or(u32::MAX);
    let max_nodes = params.max_nodes.unwrap_or(u64::MAX);
    let end_time = params.max_time.map(|t| Instant::now() + t);

    let mut stats = SearchStats::default();

    std::iter::from_fn(move || {
        stats.depth += 1;

        if stats.depth > max_depth {
            return None;
        }

        if stats.nodes > max_nodes {
            return None;
        }

        if let Some(t) = end_time {
            if Instant::now() > t {
                return None;
            }
        }

        let score = alpha_beta(-i32::MAX, i32::MAX, stats.depth, board, &mut stats, tptable);
        stats.score = score;
        stats.pv = pv_line(tptable, board);

        Some(stats.clone())
    })
}

pub fn alpha_beta(
    mut alpha: i32,
    beta: i32,
    depth: u32,
    board: &mut Board,
    stats: &mut SearchStats,
    tptable: &mut TpTable,
) -> i32 {
    stats.nodes += 1;

    if depth == 0 {
        stats.leaves += 1;
        return evaluation(board); // TODO: use quiescence search
    }

    let mut moves = Vec::with_capacity(32); // TODO: reuse a preallocated vec
    board.generate_all_moves(&mut moves);
    moves.sort_by_key(|m| -score_move(*m, tptable, board));

    let mut new_best_move = None;
    let mut legal_moves = 0;

    for m in moves.into_iter() {
        let is_valid_move = board.make_move(m);

        if !is_valid_move {
            continue;
        }

        legal_moves += 1;
        let score = -alpha_beta(-beta, -alpha, depth - 1, board, stats, tptable);
        board.take_move();

        if score >= beta {
            stats.fh += 1;

            if legal_moves == 1 {
                stats.fhf += 1
            };

            return beta; // fail hard beta-cutoff
        }

        if score > alpha {
            alpha = score;
            new_best_move = Some(m);
        }
    }

    if legal_moves == 0 {
        if board.is_square_attacked(board.king_square[board.color], board.color.flipped()) {
            return -30_000;
        } else {
            return 0;
        }
    }

    if let Some(m) = new_best_move {
        tptable.insert(board.position_key, m.m16);
    }

    alpha
}

fn score_move(m: Move32, tptable: &TpTable, board: &Board) -> i32 {
    if Some(m.m16) == tptable.get(board.position_key) {
        1_000_000
    } else if let Some(victim) = m.captured() {
        //SAFETY: A chess move always moves a piece
        let attacker = unsafe { board.pieces[m.m16.start()].unwrap_unchecked() };
        mvv_lva(attacker, victim)
    } else {
        0
    }
}

fn mvv_lva(attacker: Piece, victim: Piece) -> i32 {
    const SCORES: [i32; Piece::ALL.len()] = [1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6];
    (SCORES[victim] << 3) - SCORES[attacker]
}
