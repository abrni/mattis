use crate::{
    board::{movegen::MoveList, Board},
    eval::evaluation,
    hashtable::{HEKind, Probe, TranspositionTable},
    moves::Move32,
    types::Piece,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub struct SearchTables {
    pub transposition_table: Arc<TranspositionTable>,
    pub search_killers: Vec<[Move32; 2]>,
    pub search_history: [[u32; 64]; 12],
}

pub struct SearchParams {
    pub max_time: Option<Duration>,
    pub max_nodes: Option<u64>,
    pub max_depth: Option<u16>,
    pub stop: Arc<AtomicBool>, // TODO: Support for Mate Search
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchStats {
    pub start_time: Instant, // When we started the search
    pub depth: u16,          // Search depth
    pub score: i16,          // Score in centipawns
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

fn pv_line(tptable: &TranspositionTable, board: &mut Board) -> Vec<Move32> {
    let mut key_counts = HashMap::new();
    let mut pvline = Vec::with_capacity(8);

    while let Some(m16) = tptable.get(board.position_key) {
        if m16.is_nomove() {
            break;
        }

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

fn take_next_move(
    list: &mut MoveList,
    pv_move: Option<Move32>,
    tables: &SearchTables,
    board: &Board,
) -> Option<Move32> {
    let (idx, _) = list
        .iter()
        .enumerate()
        .min_by_key(|(_, m)| -score_move(**m, pv_move, tables, board))?;

    let m = list.swap_remove(idx);
    Some(m)
}

fn score_move(m: Move32, pv_move: Option<Move32>, tables: &SearchTables, board: &Board) -> i32 {
    if Some(m) == pv_move {
        2_000_000
    } else if let Some(victim) = m.captured() {
        //SAFETY: A chess move always moves a piece
        let attacker = unsafe { board.pieces[m.m16.start()].unwrap_unchecked() };
        1_000_000 + mvv_lva(attacker, victim)
    } else if tables.search_killers[board.ply][0] == m {
        900_000
    } else if tables.search_killers[board.ply][1] == m {
        800_000
    } else {
        let piece = board.pieces[m.m16.start()].unwrap();
        tables.search_history[piece][m.m16.end()] as i32
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

    let max_depth = params.max_depth.unwrap_or(u16::MAX);
    if stats.depth > max_depth {
        return true;
    }

    if params.stop.load(Ordering::Acquire) {
        return true;
    }

    false
}

pub fn iterative_deepening<'a>(
    board: &'a mut Board,
    params: SearchParams,
    tables: &'a mut SearchTables,
) -> impl Iterator<Item = SearchStats> + 'a {
    let mut stats = SearchStats::default();

    std::iter::from_fn(move || {
        stats.depth += 1;

        if should_search_stop(&params, &stats) {
            return None;
        };

        let score = alpha_beta(-30_002, 30_002, stats.depth, board, &params, &mut stats, tables, true);

        if stats.stop {
            return None;
        }

        stats.score = score;
        stats.pv = pv_line(&tables.transposition_table, board);
        stats.bestmove = stats.pv.get(0).copied().unwrap_or_default();

        Some(stats.clone())
    })
}

#[allow(clippy::too_many_arguments)] // TODO: reduce the number of arguments into an args struct or something
pub fn alpha_beta(
    mut alpha: i16,
    beta: i16,
    mut depth: u16,
    board: &mut Board,
    params: &SearchParams,
    stats: &mut SearchStats,
    tables: &mut SearchTables,
    allow_null_move: bool,
) -> i16 {
    if stats.stop {
        return 0;
    }

    if stats.nodes.trailing_zeros() == 10 {
        stats.stop = should_search_stop(params, stats);
    }

    if depth == 0 {
        stats.leaves += 1;
        return quiescence(alpha, beta, board, stats, tables);
    }

    stats.nodes += 1;

    if board.ply >= 1 && (board.is_repetition() || board.fifty_move >= 100) {
        return 0;
    }

    if board.in_check() {
        depth += 1;
    }

    let hashtable_probe = tables.transposition_table.probe(board, alpha, beta, depth);
    let pv_move = match hashtable_probe {
        Probe::NoHit => None,
        Probe::PV(m32, _) => Some(m32),
        Probe::CutOff(_, score) => return score,
    };

    if allow_null_move && !board.in_check() && board.ply != 0 && board.count_big_pieces[board.color] > 1 && depth >= 4 {
        board.make_null_move();
        let score = -alpha_beta(-beta, -beta + 1, depth - 4, board, params, stats, tables, false);
        board.take_null_move();

        if stats.stop {
            return 0;
        }

        if score >= beta && score.abs() < 29_000 {
            return beta;
        }
    }

    let mut moves = Vec::with_capacity(32); // TODO: reuse a preallocated vec
    board.generate_all_moves(&mut moves);
    // moves.sort_unstable_by_key(|m| -score_move(*m, pv_move, tables, board));

    let mut best_move = Move32::default();
    let mut best_score = -30_000;
    let mut legal_moves = 0;
    let mut alpha_changed = false;

    while let Some(m) = take_next_move(&mut moves, pv_move, tables, board) {
        let is_valid_move = board.make_move(m);

        if !is_valid_move {
            continue;
        }

        legal_moves += 1;
        let score = -alpha_beta(-beta, -alpha, depth - 1, board, params, stats, tables, true);
        board.take_move();

        if stats.stop {
            return 0;
        }

        if score >= beta {
            stats.fh += 1;

            if legal_moves == 1 {
                stats.fhf += 1
            };

            if !m.m16.is_capture() {
                tables.search_killers[board.ply][1] = tables.search_killers[board.ply][0];
                tables.search_killers[board.ply][0] = m;
            }

            tables
                .transposition_table
                .store(board.position_key, beta, m.m16, depth, HEKind::Beta);

            return beta; // fail hard beta-cutoff
        }

        if score > alpha {
            alpha = score;
            alpha_changed = true;

            if !m.m16.is_capture() {
                let piece = board.pieces[m.m16.start()].unwrap();
                tables.search_history[piece][m.m16.end()] += depth as u32;
            }
        }

        if score > best_score {
            best_move = m;
            best_score = score;
        }
    }

    if legal_moves == 0 {
        if board.in_check() {
            return -30_000 + board.ply as i16;
        } else {
            return 0;
        }
    }

    let hashentry_kind = if alpha_changed { HEKind::Exact } else { HEKind::Alpha };
    let score = if alpha_changed { best_score } else { alpha };
    tables
        .transposition_table
        .store(board.position_key, score, best_move.m16, depth, hashentry_kind);

    alpha
}

pub fn quiescence(
    mut alpha: i16,
    beta: i16,
    board: &mut Board,
    stats: &mut SearchStats,
    tables: &mut SearchTables,
) -> i16 {
    stats.nodes += 1;

    if board.is_repetition() || board.fifty_move >= 100 {
        return 0;
    }

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

    // moves.sort_by_key(|m| -score_move(*m, None, tables, board));

    let mut legal_moves = 0;
    while let Some(m) = take_next_move(&mut moves, None, tables, board) {
        let is_valid_move = board.make_move(m);

        if !is_valid_move {
            continue;
        }

        legal_moves += 1;
        let score = -quiescence(-beta, -alpha, board, stats, tables);
        board.take_move();

        if stats.stop {
            return 0;
        }

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
