use super::{alpha_beta, pv_line, report_after_search, ABContext, SearchStats};
use crate::{
    board::Board,
    chess_move::ChessMove,
    hashtable::TranspositionTable,
    search::{report_after_depth, IterativeDeepening, ReportMode},
    time_man::{Limits, TimeMan},
};
use bus::{Bus, BusReader};
use mattis_types::{Color, Eval};
use mattis_uci as uci;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub report_mode: ReportMode,
    pub allow_null_pruning: bool,
    pub go: uci::Go,
}

#[derive(Debug, Clone)]
struct ThreadConfig {
    report_mode: ReportMode,
    time_man: TimeMan,
    estimate_eval: Eval,
    estimate_bestmove: ChessMove,
    allow_null_pruning: bool,
}

#[derive(Debug, Clone)]
enum Message {
    StartSearch(Arc<ThreadConfig>),
    SetupBoard(Box<Board>),
    Quit,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
enum ThreadKind {
    Main,
    Supporter(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AlreadyRunning;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LazySMPSetup {
    thread_count: usize,
    ttable_size_mb: usize,
}

impl Default for LazySMPSetup {
    fn default() -> Self {
        Self {
            thread_count: 12,
            ttable_size_mb: 256,
        }
    }
}

impl LazySMPSetup {
    pub fn thread_count(&mut self, thread_count: usize) -> &mut Self {
        self.thread_count = thread_count;
        self
    }

    pub fn ttable_size(&mut self, size_mb: usize) -> &mut Self {
        self.ttable_size_mb = size_mb;
        self
    }

    pub fn create(&self) -> LazySMP {
        assert!(self.thread_count > 0, "At least 1 search thread is necessary.");

        let ttable = Arc::new(TranspositionTable::new(self.ttable_size_mb));
        let mut bus = Bus::new(1);

        // Spawn the main search thread
        let main = {
            let ttable = Arc::clone(&ttable);
            let rx = bus.add_rx();

            Some(std::thread::spawn(|| search_thread(ThreadKind::Main, ttable, rx)))
        };

        // Spawn all the supporter threads
        let supporters = (0..self.thread_count - 1)
            .map(|i| {
                let ttable = Arc::clone(&ttable);
                let thread_kind = ThreadKind::Supporter(i as u32);
                let rx = bus.add_rx();

                std::thread::spawn(move || search_thread(thread_kind, ttable, rx))
            })
            .collect();

        LazySMP {
            main,
            supporters,
            ttable,
            search_stop_flag: None,
            board: Board::startpos(),
            bus,
        }
    }
}

pub struct LazySMP {
    main: Option<JoinHandle<()>>,
    supporters: Vec<JoinHandle<()>>,
    ttable: Arc<TranspositionTable>,
    search_stop_flag: Option<Arc<AtomicBool>>,
    board: Board,
    bus: Bus<Message>,
}

impl LazySMP {
    pub fn reset_ttable(&mut self) {
        self.ttable.reset();
    }

    pub fn set_board(&mut self, board: Board) {
        self.board = board.clone();

        let message = Message::SetupBoard(Box::new(board));
        self.bus.broadcast(message);
    }

    /// Starts a new search.
    ///
    /// Fails, if a search is already running
    pub fn start_search(&mut self, search_config: SearchConfig) -> Result<(), AlreadyRunning> {
        if self.is_search_running() {
            return Err(AlreadyRunning);
        }

        // Advance the transposition table to the next age
        // TODO: Check if this is actually valid
        // (this only makes sense, if the previous search was from the same game and only at most a few plies ago)
        self.ttable.next_age();

        // Calculate the time limit and create the time manager
        let (hard_time, soft_time) = calculate_time_limit(&search_config.go, self.board.color).unzip();

        let time_man = Limits::new()
            .depth(search_config.go.depth.map(|d| d as u16))
            .nodes(search_config.go.nodes.map(|n| n as u64))
            .hard_time(hard_time)
            .soft_time(soft_time)
            .start_now();

        // Make sure, we extract the stop flag from the time manager, so we can stop the search at will
        let stop_flag = time_man.raw_stop_flag();
        self.search_stop_flag = Some(stop_flag);

        // Estimate a very rough evaluation result for the first aspiration window
        // TODO: maybe the main search thread should do this?
        // TODO: Or maybe test, if this is even worth it at all?
        let estimate = self.estimate_search(&search_config);

        // Create the Message for telling the threads to start searching
        let message = Message::StartSearch(Arc::new(ThreadConfig {
            report_mode: search_config.report_mode,
            time_man: time_man.clone(),
            estimate_eval: estimate.score,
            estimate_bestmove: estimate.bestmove,
            allow_null_pruning: search_config.allow_null_pruning,
        }));

        // Tell each thread to start searching
        self.bus.broadcast(message);

        Ok(())
    }

    /// Stops the search, if it is running. Otherwise nothing happens.
    pub fn stop_search(&mut self) {
        if let Some(stop_flag) = self.search_stop_flag.take() {
            stop_flag.store(true, Ordering::Relaxed)
        }
    }

    /// Is there currently a search running on the thread pool?
    pub fn is_search_running(&self) -> bool {
        // A search is running if:
        //   - a search stop flag exists
        //   - and this flag is set to `false`, meaning the search hasn't stopped.
        self.search_stop_flag
            .as_ref()
            .map(|s| !s.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    fn estimate_search(&self, config: &SearchConfig) -> SearchStats {
        let mut ctx = ABContext {
            time_man: Limits::new().start_now(),
            stats: SearchStats::default(),
            transposition_table: Arc::clone(&self.ttable),
            search_killers: Default::default(),
            search_history: Default::default(),
            allow_null_pruning: config.allow_null_pruning,
        };

        let score = alpha_beta(
            -Eval::MAX,
            Eval::MAX,
            1,
            &mut self.board.clone(),
            &mut ctx,
            config.allow_null_pruning,
            false,
        );

        assert_eq!(score, ctx.stats.score);
        ctx.stats
    }
}

impl Drop for LazySMP {
    fn drop(&mut self) {
        self.bus.broadcast(Message::Quit);

        self.supporters.drain(..).for_each(|handle| {
            handle.join().unwrap();
        });

        self.main.take().unwrap().join().unwrap();
    }
}

fn search_thread(kind: ThreadKind, ttable: Arc<TranspositionTable>, mut rx: BusReader<Message>) {
    let mut board = Board::startpos();

    loop {
        match rx.recv().unwrap() {
            Message::SetupBoard(new_board) => board = *new_board,
            Message::Quit => break,
            Message::StartSearch(config) => {
                let ctx = ABContext {
                    time_man: config.time_man.clone(),
                    stats: SearchStats::default(),
                    transposition_table: Arc::clone(&ttable),
                    search_killers: Default::default(),
                    search_history: Default::default(),
                    allow_null_pruning: config.allow_null_pruning,
                };

                match kind {
                    ThreadKind::Main => search_as_main(
                        config.estimate_eval,
                        config.estimate_bestmove,
                        config.report_mode,
                        &mut board,
                        ctx,
                    ),
                    ThreadKind::Supporter(thread_num) => {
                        search_as_supporter(thread_num, config.estimate_eval, &mut board, ctx)
                    }
                }
            }
        };
    }
}

fn search_as_main(
    estimate_eval: Eval,
    estimate_bestmove: ChessMove,
    report_mode: ReportMode,
    board: &mut Board,
    mut ctx: ABContext,
) {
    let mut iterative_deepening = IterativeDeepening::new(estimate_eval, 1);

    while let Some(stats) = iterative_deepening.next_depth(board, &mut ctx) {
        report_after_depth(report_mode, stats);
    }

    // Under extreme time pressure, the iterative deepening can be stopped very early.
    // In this case, the stats do not contain a valid bestmove.
    // Return the estimated bestmove instead.
    if ctx.stats.bestmove.is_nomove() {
        ctx.stats.bestmove = estimate_bestmove;
        ctx.stats.pv = pv_line(&ctx.transposition_table, board, Some(estimate_bestmove));
    }

    report_after_search(report_mode, ctx.stats);
    ctx.time_man.force_stop();
}

fn search_as_supporter(thread_num: u32, expected_eval: Eval, board: &mut Board, mut ctx: ABContext) {
    let start_depth = u16::min(thread_num as u16 + 1, ctx.time_man.depth_limit());
    loop {
        let mut iterative_deepening = IterativeDeepening::new(expected_eval, start_depth);
        while iterative_deepening.next_depth(board, &mut ctx).is_some() {}

        if ctx.time_man.stop(&ctx.stats, false) {
            break;
        }
    }
}

pub fn calculate_time_limit(go: &uci::Go, color: Color) -> Option<(Duration, Duration)> {
    let (time, inc) = match color {
        Color::White => (go.wtime, go.winc),
        Color::Black => (go.btime, go.binc),
    };

    let time = time.or(go.movetime).map(|t| t as f64);
    let inc = inc.unwrap_or(0) as f64;
    let movestogo = go.movestogo.unwrap_or(30) as f64;

    time.map(|t| {
        let hard_limit = t / 2.0;
        let hard_limit = Duration::from_micros((hard_limit * 1000.0) as u64);

        let soft_limit = (t + (movestogo * inc)) / movestogo;
        let soft_limit = Duration::from_micros((soft_limit * 1000.0) as u64);

        (hard_limit, soft_limit)
    })
}
