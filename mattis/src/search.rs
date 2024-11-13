use crate::{
    board::{movegen::MoveList, Board},
    chess_move::ChessMove,
    eval::evaluation,
    hashtable::{HEKind, Probe, TranspositionTable},
    time_man::TimeMan,
};
use history::SearchHistory;
use killers::SearchKillers;
use mattis_types::{Eval, Piece, PieceType};
use std::{collections::HashMap, sync::Arc};

pub mod history;
pub mod killers;
pub mod lazy_smp;

struct ABContext {
    time_man: TimeMan,
    stats: SearchStats,
    transposition_table: Arc<TranspositionTable>,
    search_killers: SearchKillers,
    search_history: SearchHistory,
    allow_null_pruning: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchStats {
    pub depth: u16,          // Search depth
    pub score: Eval,         // Score in centipawns
    pub nodes: u64,          // Total count of visited nodes
    pub leaves: u64,         // Total count of visited leaf nodes
    pub fh: u64,             // Count of fail-highs (beta cut off)
    pub fhf: u64,            // Count of fail-highs at the first move
    pub bestmove: ChessMove, // The best move
    pub pv: Vec<ChessMove>,  // Principle Variation Line
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            depth: 0,
            score: Eval::DRAW,
            nodes: 0,
            leaves: 0,
            fh: 0,
            fhf: 0,
            bestmove: ChessMove::default(),
            pv: vec![],
        }
    }
}

struct IterativeDeepening {
    next_depth: u16,
    last_eval: Eval,
}

impl IterativeDeepening {
    fn new(expected_eval: Eval, start_depth: u16) -> Self {
        Self {
            next_depth: start_depth,
            last_eval: expected_eval,
        }
    }

    fn next_depth(&mut self, board: &mut Board, ctx: &mut ABContext) -> Option<SearchStats> {
        ctx.stats.depth = self.next_depth;

        if !ctx.time_man.enough_time_for_next_depth(&ctx.stats) {
            return None;
        };

        let mut alpha = self.last_eval - PieceType::Pawn.value() / 2;
        let mut beta = self.last_eval + PieceType::Pawn.value() / 2;
        let mut loop_count = 0;

        let score = loop {
            let score = alpha_beta(alpha, beta, self.next_depth, board, ctx, ctx.allow_null_pruning, true);

            if ctx.time_man.stop(&ctx.stats, true) {
                return None;
            }

            let inc = 20_i16
                .saturating_mul(10_i16.saturating_pow(loop_count))
                .saturating_add(PieceType::Pawn.value() / 2);

            if score <= alpha {
                loop_count += 1;
                alpha = alpha.inner().checked_sub(inc).map(Into::into).unwrap_or(-Eval::MAX);
            } else if score >= beta {
                loop_count += 1;
                beta = beta.inner().checked_add(inc).map(Into::into).unwrap_or(Eval::MAX);
            } else {
                break score;
            }
        };

        self.last_eval = score;
        self.next_depth += 1;

        if ctx.time_man.stop(&ctx.stats, false) {
            None
        } else {
            ctx.stats.score = score;
            ctx.stats.pv = pv_line(&ctx.transposition_table, board);
            ctx.stats.bestmove = ctx.stats.pv.first().copied().unwrap_or_default();

            Some(ctx.stats.clone())
        }
    }
}

fn pv_line(tptable: &TranspositionTable, board: &mut Board) -> Vec<ChessMove> {
    let mut pos_key_counts = HashMap::new();
    let mut pvline = Vec::with_capacity(8);

    while let Some(cmove) = tptable.load_move(board.position_key) {
        if cmove.is_nomove() {
            break;
        }

        let kc_entry = pos_key_counts.entry(board.position_key).or_insert(0);
        *kc_entry += 1;

        if *kc_entry >= 3 {
            // We have seen a position 3 times.
            // Stop, because this is a threefold repetition and we'd be stuck in an infinite loop.
            break;
        }

        board.make_move(cmove);
        pvline.push(cmove);
    }

    for _ in 0..pvline.len() {
        board.take_move();
    }

    pvline
}

fn take_next_move(
    list: &mut MoveList,
    pv_move: Option<ChessMove>,
    ctx: &ABContext,
    board: &Board,
) -> Option<ChessMove> {
    let (idx, _) = list
        .iter()
        .enumerate()
        .min_by_key(|(_, m)| -score_move(**m, pv_move, ctx, board))?;

    let m = list.swap_remove(idx);
    Some(m)
}

fn score_move(m: ChessMove, pv_move: Option<ChessMove>, ctx: &ABContext, board: &Board) -> i32 {
    let captured = if m.is_en_passant() {
        Some(PieceType::Pawn)
    } else {
        board.pieces[m.end()].map(Piece::piece_type)
    };

    if Some(m) == pv_move {
        2_000_000
    } else if let Some(victim) = captured {
        //SAFETY: A chess move always moves a piece
        let attacker = unsafe { board.pieces[m.start()].unwrap_unchecked().piece_type() };
        1_000_000 + mvv_lva(attacker, victim)
    } else if ctx.search_killers.slot1(board.ply) == m {
        900_000
    } else if ctx.search_killers.slot2(board.ply) == m {
        800_000
    } else {
        let piece = board.pieces[m.start()].unwrap();
        ctx.search_history.entry(piece, m.end()) as i32
    }
}

fn mvv_lva(attacker: PieceType, victim: PieceType) -> i32 {
    const SCORES: [i32; PieceType::ALL.len()] = [1, 2, 3, 4, 5, 6];
    (SCORES[victim] << 3) - SCORES[attacker]
}

#[allow(clippy::too_many_arguments)] // TODO: reduce the number of arguments into an args struct or something
fn alpha_beta(
    mut alpha: Eval,
    beta: Eval,
    mut depth: u16,
    board: &mut Board,
    ctx: &mut ABContext,
    allow_null_move: bool,
    is_pv: bool,
) -> Eval {
    // We frequently check, if the search should stop
    // (e.g. because of time running out or a gui command).
    if ctx.time_man.stop(&ctx.stats, true) {
        return Eval::DRAW;
    }

    if depth == 0 {
        ctx.stats.leaves += 1;
        return quiescence(alpha, beta, board, ctx);
    }

    ctx.stats.nodes += 1;

    // Check if we reached a draw by fifty move rule or 3-fold-repetition.
    // We actually evaluate a single repetition as a draw, so we can find
    // drawn positions earlier.
    if board.ply >= 1 && (board.is_repetition() || board.fifty_move >= 100) {
        return Eval::DRAW;
    }

    // We extend the depth, if we are in check. This increases the chance to
    // properly evaluate, whether we are able to get out of check or not.
    // Even though we handle being in check in the quiescence search, this still
    // seems to yield positive results.
    if board.in_check() {
        depth += 1;
    }

    // Probe the transposition table. There a two kinds of hashtable hits:
    // A CutOff-Hit allows us to safely perform a branch cutoff and return early.
    // Otherwise we can still use the table hit for move ordering.
    let hashtable_probe = ctx.transposition_table.probe(board, alpha, beta, depth);
    let pv_move = match hashtable_probe {
        Probe::NoHit => None,
        Probe::Pv(cmove) => Some(cmove),
        Probe::CutOff(score) => return score,
    };

    // Null move pruning optimization.
    // We do a nothing move (passing move) and see if we are still much better than the oponent (by causing a beta cutoff).
    // In that case we can be sure to have found a good position and return early.
    // We don't want null move pruning, if we are in check, because that would cause an illegal position.
    if allow_null_move
        && !is_pv
        && !board.in_check()
        && board.ply != 0
        && board.count_big_pieces[board.color] > 1
        && depth >= 4
    {
        board.make_null_move();
        let score = -alpha_beta(-beta, -beta + 1i16, depth - 4, board, ctx, false, false);
        board.take_null_move();

        // Don't use the results, if we entered stop-mode in the meantime.
        if ctx.time_man.stop(&ctx.stats, true) {
            return Eval::DRAW;
        }

        // Finding a mate during null-move pruning is likely caused by a zugzwang,
        // which would not have occured without the null move.
        // Do not use the result in that case.
        if score >= beta && !score.is_mate() {
            return beta;
        }
    }

    let mut moves = MoveList::default();
    board.generate_all_moves(&mut moves);

    let mut best_move = ChessMove::default(); // Will contain the best move we found during the search.
    let mut best_score = -Eval::MAX; // TODO: do we really need this?
    let mut legal_moves = 0; // Counts the number of legal moves. Not every generated move is necessarily legal.
    let mut alpha_changed = false; // signals if alpha has changed during the evaluation of each move

    while let Some(m) = take_next_move(&mut moves, pv_move, ctx, board) {
        let is_legal_move = board.make_move(m);

        // The move might have been illegal. in that case the move was not made and we can skip to the next one.
        if !is_legal_move {
            continue;
        }

        legal_moves += 1;

        let score = if !alpha_changed {
            -alpha_beta(-beta, -alpha, depth - 1, board, ctx, ctx.allow_null_pruning, is_pv)
        } else {
            let est = -alpha_beta(
                -alpha - 1_i16,
                -alpha,
                depth - 1,
                board,
                ctx,
                ctx.allow_null_pruning,
                false,
            );
            if est > alpha {
                -alpha_beta(-beta, -alpha, depth - 1, board, ctx, ctx.allow_null_pruning, true)
            } else {
                -Eval::MAX
            }
        };

        // let score = -alpha_beta(-beta, -alpha, depth - 1, board, ctx, ctx.allow_null_pruning);
        board.take_move();

        // Don't use the result of alpha-beta if we entered stop-mode in the meantime. The result is probably nonsense.
        // Instead just return ASAP.
        if ctx.time_man.stop(&ctx.stats, true) {
            return Eval::DRAW;
        }

        // A score higher than beta allows us to perform a beta cutoff (fail-high)
        if score >= beta {
            ctx.stats.fh += 1; // Track that number of fail-highs

            // Track, if we caused the fail-high on the first legal move
            if legal_moves == 1 {
                ctx.stats.fhf += 1;
            };

            // A quiet move, that caused a beta-cutoff is labeled a 'killer-move'.
            // If the same move is encountered at the same ply but in a different position, it will be
            // prefered by move ordering. We use two killer slots, to not forget good moves in some situations.
            if !m.is_capture() && !m.is_promotion() {
                ctx.search_killers.store(board.ply, m);
            }

            // Store the move in the hashtable and mark it as a beta-cutoff
            ctx.transposition_table.store(board, beta, m, depth, HEKind::Beta);

            return beta; // fail hard beta-cutoff
        } else if score > alpha {
            alpha = score;
            alpha_changed = true;

            // If we improved alpha with this move, we increase a corresponding score in our history.
            // This helps move ordering by prefering moves that are similar to moves which caused alpha improvements before.
            // TODO: I am not sure, we are doing this right. I should test not using the history heuristic or using a
            // different added value.
            if !m.is_capture() {
                let piece = board.pieces[m.start()].unwrap();
                *ctx.search_history.entry_mut(piece, m.end()) += depth as u64; // TODO: is this better: += depth * depth or 2^depth?
            }
        }

        if score > best_score {
            best_move = m;
            best_score = score;
        }
    }

    // If we haven't found any legal move, we are either in checkmate or in a stalemate.
    if legal_moves == 0 {
        if board.in_check() {
            return -Eval::mate_in(board.ply as u8);
        }

        return Eval::DRAW;
    }

    // Store the best move we found in the hashtable.
    // If we have not improved alpha, we mark the best move as an alpha-cutoff.
    // Otherwise we can return the exact score.
    let hashentry_kind = if alpha_changed { HEKind::Exact } else { HEKind::Alpha };
    let score = if alpha_changed { alpha } else { best_score }; // TODO: I think, weh should be able to always use alpha here?
    ctx.transposition_table
        .store(board, score, best_move, depth, hashentry_kind);

    alpha
}

fn quiescence(mut alpha: Eval, beta: Eval, board: &mut Board, ctx: &mut ABContext) -> Eval {
    ctx.stats.nodes += 1;

    if board.is_repetition() || board.fifty_move >= 100 {
        return Eval::DRAW;
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

    let mut moves = MoveList::with_capacity(64);

    if in_check {
        board.generate_all_moves(&mut moves);
    } else {
        board.generate_capture_moves(&mut moves);
    }

    let mut legal_moves = 0;
    while let Some(m) = take_next_move(&mut moves, None, ctx, board) {
        let is_valid_move = board.make_move(m);

        if !is_valid_move {
            continue;
        }

        legal_moves += 1;
        let score = -quiescence(-beta, -alpha, board, ctx);
        board.take_move();

        if ctx.time_man.stop(&ctx.stats, true) {
            return Eval::DRAW;
        }

        if score >= beta {
            return beta; // fail hard beta-cutoff
        }

        if score > alpha {
            alpha = score;
        }
    }

    if legal_moves == 0 && in_check {
        return -Eval::mate_in(board.ply as u8);
    }

    alpha
}
