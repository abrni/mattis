use crate::{
    board::{movegen::MoveList, Board},
    chess_move::ChessMove,
    eval::{evaluation, Eval},
    hashtable::{HEKind, Probe, TranspositionTable},
    types::{Color, Piece, PieceType},
    uci::{self, EngineMessage},
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

pub struct SearchTables {
    pub transposition_table: Arc<TranspositionTable>,
    pub search_killers: Vec<[ChessMove; 2]>,
    pub search_history: [[u64; 64]; 12],
}

#[derive(Debug, Clone)]
pub struct SearchParams {
    max_time: Option<Duration>,
    max_nodes: Option<u64>,
    max_depth: Option<u16>,
    stop: Arc<AtomicBool>, // TODO: Support for Mate Search
    allow_null_pruning: bool,
}

impl SearchParams {
    fn new(go: uci::Go, color: Color, allow_null_pruning: bool, stop: Arc<AtomicBool>) -> Self {
        let (time, inc) = match color {
            Color::White => (go.wtime, go.winc),
            Color::Black => (go.btime, go.binc),
        };

        let movestogo = go.movestogo.unwrap_or(30) as f64;
        let (time, inc) = (time.or(go.movetime), inc.unwrap_or(0) as f64);

        let max_time = time
            .map(|t| t as f64)
            .map(|t| (t + (movestogo * inc)) / (movestogo / 3.0 * 2.0) - inc)
            .map(|t| Duration::from_micros((t * 1000.0) as u64));

        SearchParams {
            max_time,
            max_nodes: go.nodes.map(|n| n as u64),
            max_depth: go.depth.map(|d| d as u16),
            stop,
            allow_null_pruning,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchStats {
    pub start_time: Instant, // When we started the search
    pub depth: u16,          // Search depth
    pub score: Eval,         // Score in centipawns
    pub nodes: u64,          // Total count of visited nodes
    pub leaves: u64,         // Total count of visited leaf nodes
    pub fh: u64,             // Count of fail-highs (beta cut off)
    pub fhf: u64,            // Count of fail-highs at the first move
    pub bestmove: ChessMove, // The best move
    pub pv: Vec<ChessMove>,  // Principle Variation Line
    pub stop: bool,          // Should the search stop ASAP
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            depth: 0,
            score: Eval::DRAW,
            nodes: 0,
            leaves: 0,
            fh: 0,
            fhf: 0,
            bestmove: ChessMove::default(),
            pv: vec![],
            stop: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct KillSwitch {
    switch: Arc<AtomicBool>,
    join_handles: Vec<JoinHandle<()>>,
}

impl KillSwitch {
    pub fn kill(self) {
        self.switch.store(true, Ordering::Relaxed);

        for h in self.join_handles {
            h.join().unwrap();
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.switch.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct SearchConfig<'a> {
    pub allow_null_pruning: bool,
    pub thread_count: u32,
    pub go: uci::Go,
    pub board: &'a Board,
    pub tp_table: Arc<TranspositionTable>,
}

pub fn run_search(config: SearchConfig) -> KillSwitch {
    let switch = Arc::new(AtomicBool::new(false));
    let params = SearchParams::new(config.go, config.board.color, config.allow_null_pruning, switch.clone());

    let mut search_tables = SearchTables {
        transposition_table: Arc::clone(&config.tp_table),
        search_killers: vec![[ChessMove::default(); 2]; 1024],
        search_history: [[0; 64]; 12],
    };

    let mut stats = SearchStats::default();
    let expected_eval = alpha_beta(
        -Eval::MAX,
        Eval::MAX,
        1,
        &mut config.board.clone(),
        &params,
        &mut stats,
        &mut search_tables,
        false,
    );

    let join_handles: Vec<JoinHandle<()>> = (0..config.thread_count)
        .map(|i| {
            let thread_config = ThreadConfig {
                tp_table: Arc::clone(&config.tp_table),
                thread_num: i,
                params: params.clone(),
                expected_eval,
            };

            let board = config.board.clone();
            std::thread::spawn(move || search_thread(thread_config, board))
        })
        .collect();

    KillSwitch { switch, join_handles }
}

struct ThreadConfig {
    tp_table: Arc<TranspositionTable>,
    thread_num: u32,
    params: SearchParams,
    expected_eval: Eval,
}

fn search_thread(config: ThreadConfig, mut board: Board) {
    let mut search_tables = SearchTables {
        transposition_table: config.tp_table,
        search_killers: vec![[ChessMove::default(); 2]; 1024],
        search_history: [[0; 64]; 12],
    };

    let start_depth = u16::min(config.thread_num as u16, config.params.max_depth.unwrap_or(u16::MAX));

    let mut bestmove = ChessMove::default();
    // let mut iterative_deepening = IterativeDeepening::init(&mut board, config.params.clone(), &mut search_tables);
    let mut iterative_deepening = IterativeDeepening::new(config.params.clone(), config.expected_eval, start_depth);

    while let Some(stats) = iterative_deepening.next_depth(&mut board, &mut search_tables) {
        bestmove = stats.bestmove;

        let info = EngineMessage::Info(uci::Info {
            depth: Some(stats.depth as u32),
            nodes: Some(stats.nodes as u32),
            pv: stats.pv.into_iter().map(|m| format!("{m}")).collect(),
            // FIXME: Mate score can be off by 1 at low depths,
            // because the score comes straight from the hashtable which stored the entry one move ago.
            score: Some(uci::Score(stats.score)),
            ..Default::default()
        });

        if config.thread_num == 0 {
            println!("{info}");
        }
    }

    if config.thread_num == 0 {
        let bestmove = EngineMessage::Bestmove {
            move_: format!("{bestmove}"),
            ponder: None,
        };

        println!("{bestmove}");
        config.params.stop.store(true, Ordering::Relaxed);
    }
}

struct IterativeDeepening {
    last_eval: Eval,
    stats: SearchStats,
    params: SearchParams,
}

impl IterativeDeepening {
    fn new(params: SearchParams, expected_eval: Eval, start_depth: u16) -> Self {
        let stats = SearchStats {
            depth: start_depth.saturating_sub(1),
            ..Default::default()
        };

        Self {
            last_eval: expected_eval,
            stats,
            params,
        }
    }

    fn next_depth(&mut self, board: &mut Board, tables: &mut SearchTables) -> Option<SearchStats> {
        self.stats.depth += 1;

        if should_search_stop(&self.params, &self.stats) {
            return None;
        };

        let mut alpha = self.last_eval - PieceType::Pawn.value();
        let mut beta = self.last_eval + PieceType::Pawn.value();
        let mut loop_count = 0;

        let score = loop {
            let score = alpha_beta(
                alpha,
                beta,
                self.stats.depth,
                board,
                &self.params,
                &mut self.stats,
                tables,
                self.params.allow_null_pruning,
            );

            if self.stats.stop {
                return None;
            }

            if score <= alpha {
                loop_count += 1;
                alpha = alpha
                    .inner()
                    .checked_sub(10_i16 * 4_i16.pow(loop_count) + PieceType::Pawn.value())
                    .map(Into::into)
                    .unwrap_or(-Eval::MAX);
            } else if score >= beta {
                loop_count += 1;
                beta = beta
                    .inner()
                    .checked_add(10_i16 * 4_i16.pow(loop_count) + PieceType::Pawn.value())
                    .map(Into::into)
                    .unwrap_or(Eval::MAX);
            } else {
                break score;
            }
        };

        self.last_eval = score;
        self.stats.score = score;
        self.stats.pv = pv_line(&tables.transposition_table, board);
        self.stats.bestmove = self.stats.pv.get(0).copied().unwrap_or_default();

        Some(self.stats.clone())
    }
}

fn pv_line(tptable: &TranspositionTable, board: &mut Board) -> Vec<ChessMove> {
    let mut pos_key_counts = HashMap::new();
    let mut pvline = Vec::with_capacity(8);

    while let Some(cmove) = tptable.get(board.position_key) {
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
    tables: &SearchTables,
    board: &Board,
) -> Option<ChessMove> {
    let (idx, _) = list
        .iter()
        .enumerate()
        .min_by_key(|(_, m)| -score_move(**m, pv_move, tables, board))?;

    let m = list.swap_remove(idx);
    Some(m)
}

fn score_move(m: ChessMove, pv_move: Option<ChessMove>, tables: &SearchTables, board: &Board) -> i32 {
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
    } else if tables.search_killers[board.ply][0] == m {
        900_000
    } else if tables.search_killers[board.ply][1] == m {
        800_000
    } else {
        let piece = board.pieces[m.start()].unwrap();
        tables.search_history[piece][m.end()] as i32
    }
}

fn mvv_lva(attacker: PieceType, victim: PieceType) -> i32 {
    const SCORES: [i32; PieceType::ALL.len()] = [1, 2, 3, 4, 5, 6];
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

#[allow(clippy::too_many_arguments)] // TODO: reduce the number of arguments into an args struct or something
fn alpha_beta(
    mut alpha: Eval,
    beta: Eval,
    mut depth: u16,
    board: &mut Board,
    params: &SearchParams,
    stats: &mut SearchStats,
    tables: &mut SearchTables,
    allow_null_move: bool,
) -> Eval {
    if stats.stop {
        return Eval::DRAW;
    }

    // We frequently check, if the search should stop
    // (e.g. because of time running out or a gui command).
    if stats.nodes.trailing_zeros() == 10 {
        stats.stop = should_search_stop(params, stats);
    }

    if depth == 0 {
        stats.leaves += 1;
        return quiescence(alpha, beta, board, stats, tables);
    }

    stats.nodes += 1;

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
    let hashtable_probe = tables.transposition_table.probe(board, alpha, beta, depth);
    let pv_move = match hashtable_probe {
        Probe::NoHit => None,
        Probe::PV(m32, _) => Some(m32),
        Probe::CutOff(_, score) => return score,
    };

    // Null move pruning optimization.
    // We do a nothing move (passing move) and see if we are still much better than the oponent (by causing a beta cutoff).
    // In that case we can be sure to have found a good position and return early.
    // We don't want null move pruning, if we are in check, because that would cause an illegal position.
    if allow_null_move && !board.in_check() && board.ply != 0 && board.count_big_pieces[board.color] > 1 && depth >= 4 {
        board.make_null_move();
        let score = -alpha_beta(-beta, -beta + 1i16, depth - 4, board, params, stats, tables, false);
        board.take_null_move();

        // Don't use the results, if we entered stop-mode in the meantime.
        if stats.stop {
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

    while let Some(m) = take_next_move(&mut moves, pv_move, tables, board) {
        let is_legal_move = board.make_move(m);

        // The move might have been illegal. in that case the move was not made and we can skip to the next one.
        if !is_legal_move {
            continue;
        }

        legal_moves += 1;
        let score = -alpha_beta(-beta, -alpha, depth - 1, board, params, stats, tables, true);
        board.take_move();

        // Don't use the result of alpha-beta if we entered stop-mode in the meantime. The result is probably nonsense.
        // Instead just return ASAP.
        if stats.stop {
            return Eval::DRAW;
        }

        // A score higher than beta allows us to perform a beta cutoff (fail-high)
        if score >= beta {
            stats.fh += 1; // Track that number of fail-highs

            // Track, if we caused the fail-high on the first legal move
            if legal_moves == 1 {
                stats.fhf += 1;
            };

            // A quiet move, that caused a beta-cutoff is labeled a 'killer-move'.
            // If the same move is encountered at the same ply but in a different position, it will be
            // prefered by move ordering. We use two killer slots, to not forget good moves in some situations.
            if !m.is_capture() {
                tables.search_killers[board.ply][1] = tables.search_killers[board.ply][0];
                tables.search_killers[board.ply][0] = m;
            }

            // Store the move in the hashtable and mark it as a beta-cutoff
            tables
                .transposition_table
                .store(board.position_key, beta, m, depth, HEKind::Beta);

            return beta; // fail hard beta-cutoff
        }

        if score > alpha {
            alpha = score;
            alpha_changed = true;

            // If we improved alpha with this move, we increase a corresponding score in our history.
            // This helps move ordering by prefering moves that are similar to moves which caused alpha improvements before.
            // TODO: I am not sure, we are doing this right. I should test not using the history heuristic or using a
            // different added value.
            if !m.is_capture() {
                let piece = board.pieces[m.start()].unwrap();
                tables.search_history[piece][m.end()] += depth as u64; // TODO: is this better: += depth * depth or 2^depth?
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
    let score = if alpha_changed { best_score } else { alpha }; // TODO: I think, weh should be able to always use alpha here?
    tables
        .transposition_table
        .store(board.position_key, score, best_move, depth, hashentry_kind);

    alpha
}

fn quiescence(
    mut alpha: Eval,
    beta: Eval,
    board: &mut Board,
    stats: &mut SearchStats,
    tables: &mut SearchTables,
) -> Eval {
    stats.nodes += 1;

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
    while let Some(m) = take_next_move(&mut moves, None, tables, board) {
        let is_valid_move = board.make_move(m);

        if !is_valid_move {
            continue;
        }

        legal_moves += 1;
        let score = -quiescence(-beta, -alpha, board, stats, tables);
        board.take_move();

        if stats.stop {
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
