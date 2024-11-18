use super::{alpha_beta, history::SearchHistory, killers::SearchKillers, report_after_search, ABContext, SearchStats};
use crate::{
    board::Board,
    chess_move::ChessMove,
    hashtable::TranspositionTable,
    search::{report_after_depth, IterativeDeepening, ReportMode},
    time_man::{Limits, TimeMan},
};
use mattis_types::{Color, Eval};
use mattis_uci as uci;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
    thread::JoinHandle,
    time::Duration,
};

pub type Shared<T> = Arc<RwLock<T>>;

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

        let ttable = Arc::new(TranspositionTable::new(self.ttable_size_mb)); // TODO: allow configuration
        let history = Arc::new(RwLock::new(SearchHistory::default()));
        let killers = Arc::new(RwLock::new(SearchKillers::default()));

        // Spawn the main search thread
        let (main, main_sender) = {
            let ttable = Arc::clone(&ttable);
            let history = Arc::clone(&history);
            let killers = Arc::clone(&killers);
            let (tx, rx) = std::sync::mpsc::channel();

            let main = Some(std::thread::spawn(|| {
                search_thread(ThreadKind::Main, ttable, history, killers, rx)
            }));

            (main, tx)
        };

        // Spawn all the supporter threads
        let supporters = (0..self.thread_count - 1)
            .map(|i| {
                let ttable = Arc::clone(&ttable);
                let history = Arc::clone(&history);
                let killers = Arc::clone(&killers);
                let (tx, rx) = std::sync::mpsc::channel();
                let handle = std::thread::spawn(move || {
                    search_thread(ThreadKind::Supporter(i as u32), ttable, history, killers, rx)
                });
                (handle, tx)
            })
            .collect();

        LazySMP {
            main,
            main_sender,
            supporters,
            ttable,
            history,
            killers,
            active_search: None,
            board: Board::startpos(),
        }
    }
}

pub struct LazySMP {
    main: Option<JoinHandle<()>>,
    main_sender: Sender<Message>,
    supporters: Vec<(JoinHandle<()>, Sender<Message>)>,
    ttable: Arc<TranspositionTable>,
    history: Shared<SearchHistory>,
    killers: Shared<SearchKillers>,
    active_search: Option<Arc<AtomicBool>>,
    board: Board,
}

impl LazySMP {
    pub fn reset_tables(&mut self) {
        self.ttable.reset();
        *self.history.write().unwrap() = SearchHistory::default();
        *self.killers.write().unwrap() = SearchKillers::default();
    }

    pub fn set_board(&mut self, board: Board) {
        self.board = board.clone();
        self.main_sender.send(Message::SetupBoard(board.clone())).unwrap();

        for (_, tx) in &self.supporters {
            tx.send(Message::SetupBoard(board.clone())).unwrap();
        }
    }

    /// Starts a search. Fails, if a search is already running
    pub fn start_search(&mut self, search_config: SearchConfig) -> Result<(), AlreadyRunning> {
        if self.is_search_running() {
            return Err(AlreadyRunning);
        }

        let (hard_time, soft_time) = calculate_time_limit(&search_config.go, self.board.color).unzip();

        let time_man = Limits::new()
            .depth(search_config.go.depth.map(|d| d as u16))
            .nodes(search_config.go.nodes.map(|n| n as u64))
            .hard_time(hard_time)
            .soft_time(soft_time)
            .start_now();

        let switch = time_man.raw_stop_flag();
        self.active_search = Some(switch);

        self.ttable.next_age();

        let mut ctx = ABContext {
            time_man: time_man.clone(),
            stats: SearchStats::default(),
            transposition_table: Arc::clone(&self.ttable),
            search_killers: self.killers.read().unwrap().clone(),
            search_history: self.history.read().unwrap().clone(),
            allow_null_pruning: search_config.allow_null_pruning,
        };

        let expected_eval = alpha_beta(
            -Eval::MAX,
            Eval::MAX,
            1,
            &mut self.board.clone(),
            &mut ctx,
            search_config.allow_null_pruning,
            false,
        );

        let config = ThreadConfig {
            report_mode: search_config.report_mode,
            time_man: time_man.clone(),
            expected_eval,
            allow_null_pruning: search_config.allow_null_pruning,
        };

        self.main_sender.send(Message::StartSearch(config)).unwrap();

        for (_, tx) in &self.supporters {
            let config = ThreadConfig {
                report_mode: search_config.report_mode,
                time_man: time_man.clone(),
                expected_eval,
                allow_null_pruning: search_config.allow_null_pruning,
            };

            tx.send(Message::StartSearch(config)).unwrap();
        }

        Ok(())
    }

    /// Stops the search, if it is running. Otherwise nothing happens.
    pub fn stop_search(&mut self) {
        if let Some(switch) = self.active_search.take() {
            switch.store(true, Ordering::Relaxed)
        }
    }

    pub fn is_search_running(&self) -> bool {
        self.active_search
            .as_ref()
            .map(|s| !s.load(Ordering::Relaxed))
            .unwrap_or(false)
    }
}

impl Drop for LazySMP {
    fn drop(&mut self) {
        // Make the supporter threads quit
        self.supporters.drain(..).for_each(|(h, tx)| {
            tx.send(Message::Quit).unwrap();
            h.join().unwrap();
        });

        // Make the main thread quit
        self.main_sender.send(Message::Quit).unwrap();
        self.main.take().unwrap().join().unwrap();
    }
}

enum Message {
    StartSearch(ThreadConfig),
    SetupBoard(Board),
    Quit,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
enum ThreadKind {
    Main,
    Supporter(u32),
}

fn search_thread(
    kind: ThreadKind,
    ttable: Arc<TranspositionTable>,
    history: Shared<SearchHistory>,
    killers: Shared<SearchKillers>,
    rx: Receiver<Message>,
) {
    let mut board = Board::startpos();

    loop {
        match rx.recv().unwrap() {
            Message::SetupBoard(new_board) => board = new_board,
            Message::Quit => break,
            Message::StartSearch(config) => {
                let ctx = ABContext {
                    time_man: config.time_man.clone(),
                    stats: SearchStats::default(),
                    transposition_table: Arc::clone(&ttable),
                    search_killers: killers.read().unwrap().clone(),
                    search_history: history.read().unwrap().clone(),
                    allow_null_pruning: config.allow_null_pruning,
                };

                match kind {
                    ThreadKind::Main => search_as_main(config, &mut board, ctx, &history, &killers),
                    ThreadKind::Supporter(thread_num) => search_as_supporter(thread_num, config, &mut board, ctx),
                }
            }
        };
    }
}

fn search_as_main(
    config: ThreadConfig,
    board: &mut Board,
    mut ctx: ABContext,
    history: &Shared<SearchHistory>,
    killers: &Shared<SearchKillers>,
) {
    let mut bestmove = ChessMove::default();
    let mut iterative_deepening = IterativeDeepening::new(config.expected_eval, 1);

    while let Some(stats) = iterative_deepening.next_depth(board, &mut ctx) {
        bestmove = stats.bestmove;
        report_after_depth(config.report_mode, stats);
    }

    ctx.stats.bestmove = bestmove; // TODO: Do we need this assignment?
    report_after_search(config.report_mode, ctx.stats);

    *killers.write().unwrap() = ctx.search_killers;
    *history.write().unwrap() = ctx.search_history;
    ctx.time_man.force_stop();
}

fn search_as_supporter(thread_num: u32, config: ThreadConfig, board: &mut Board, mut ctx: ABContext) {
    let start_depth = u16::min(thread_num as u16 + 1, ctx.time_man.depth_limit());
    loop {
        let mut iterative_deepening = IterativeDeepening::new(config.expected_eval, start_depth);
        while iterative_deepening.next_depth(board, &mut ctx).is_some() {}

        if ctx.time_man.stop(&ctx.stats, false) {
            break;
        }
    }
}

#[derive(Clone)]
pub struct SearchConfig {
    pub report_mode: ReportMode,
    pub allow_null_pruning: bool,
    pub go: uci::Go,
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

pub struct ThreadConfig {
    report_mode: ReportMode,
    time_man: TimeMan,
    expected_eval: Eval,
    allow_null_pruning: bool,
}
