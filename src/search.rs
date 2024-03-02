use crate::{board::Board, eval::evaluation, moves::Move32, tptable::TpTable, types::Piece};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct SearchParams {
    pub max_time: Option<Duration>,
    pub max_nodes: Option<u64>,
    pub max_depth: Option<u32>,
    // TODO: Support for Mate Search
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchStats {
    pub start_time: Instant, // When we started the search
    pub depth: u32,          // Search depth
    pub score: i32,          // Score in centipawns
    pub nodes: u64,          // Total count of visited nodes
    pub leaves: u64,         // Total count of visited leaf nodes
    pub fh: u64,             // Count of fail-highs (beta cut off)
    pub fhf: u64,            // Count of fail-highs at the first move
    pub bestmove: Move32,    // The best move
    pub pv: Vec<Move32>,     // Principle Variation Line
    pub stop: bool,          // Should the search stop ASAP
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            depth: 0,
            score: 0,
            nodes: 0,
            leaves: 0,
            fh: 0,
            fhf: 0,
            bestmove: Move32::default(),
            pv: vec![],
            stop: false,
        }
    }
}

fn pv_line(tptable: &TpTable, board: &mut Board) -> Vec<Move32> {
    let mut key_counts = HashMap::new();
    let mut pvline = Vec::with_capacity(8);

    while let Some(m16) = tptable.get(board.position_key) {
        let kc = key_counts.entry(board.position_key).or_insert(0);
        *kc += 1;

        if *kc >= 3 {
            break;
        }

        let m32 = board.move_16_to_32(m16);
        board.make_move(m32);
        pvline.push(m32);
    }

    for _ in 0..pvline.len() {
        board.take_move();
    }

    pvline
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

fn should_search_stop(params: &SearchParams, stats: &SearchStats) -> bool {
    let max_nodes = params.max_nodes.unwrap_or(u64::MAX);
    if stats.nodes > max_nodes {
        return true;
    }

    let max_time = params.max_time.unwrap_or(Duration::MAX);
    if stats.start_time.elapsed() >= max_time {
        return true;
    };

    let max_depth = params.max_depth.unwrap_or(u32::MAX);
    if stats.depth > max_depth {
        return true;
    }

    false
}

pub fn iterative_deepening<'a>(
    board: &'a mut Board,
    params: SearchParams,
    tptable: &'a mut TpTable,
) -> impl Iterator<Item = SearchStats> + 'a {
    let mut stats = SearchStats::default();

    std::iter::from_fn(move || {
        stats.depth += 1;

        if should_search_stop(&params, &stats) {
            return None;
        };

        let score = alpha_beta(-i32::MAX, i32::MAX, stats.depth, board, &params, &mut stats, tptable);

        if stats.stop {
            return None;
        }

        stats.score = score;
        stats.pv = pv_line(tptable, board);
        stats.bestmove = stats.pv[0];

        Some(stats.clone())
    })
}

pub fn alpha_beta(
    mut alpha: i32,
    beta: i32,
    depth: u32,
    board: &mut Board,
    params: &SearchParams,
    stats: &mut SearchStats,
    tptable: &mut TpTable,
) -> i32 {
    if stats.stop {
        return 0;
    }

    if stats.nodes.trailing_zeros() == 10 {
        stats.stop = should_search_stop(params, stats);
    }

    stats.nodes += 1;

    if depth == 0 {
        stats.leaves += 1;
        return quiescence(alpha, beta, board, stats, tptable);
    }

    if board.is_repetition() || board.fifty_move >= 100 {
        return 0;
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
        let score = -alpha_beta(-beta, -alpha, depth - 1, board, params, stats, tptable);
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
        if board.in_check() {
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

pub fn quiescence(mut alpha: i32, beta: i32, board: &mut Board, stats: &mut SearchStats, tptable: &mut TpTable) -> i32 {
    stats.nodes += 1;
    let standing_pat = evaluation(board);
    let in_check = board.in_check();

    if !in_check {
        if standing_pat >= beta {
            return beta;
        } else if alpha < standing_pat {
            alpha = standing_pat;
        }
    }

    let mut moves = Vec::with_capacity(32); // TODO: reuse a preallocated vec

    if in_check {
        board.generate_all_moves(&mut moves);
    } else {
        board.generate_capture_moves(&mut moves);
    }

    moves.sort_by_key(|m| -score_move(*m, tptable, board));

    let mut legal_moves = 0;
    for m in moves.into_iter() {
        let is_valid_move = board.make_move(m);

        if !is_valid_move {
            continue;
        }

        legal_moves += 1;
        let score = -quiescence(-beta, -alpha, board, stats, tptable);
        board.take_move();

        if score >= beta {
            return beta; // fail hard beta-cutoff
        }

        if score > alpha {
            alpha = score;
        }
    }

    if legal_moves == 0 && in_check {
        return -30_000;
    }

    alpha
}
