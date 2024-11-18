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

pub struct LazySMP {
    main: Option<JoinHandle<()>>,
    main_sender: Sender<Message>,
    supporters: Vec<(JoinHandle<()>, Sender<Message>)>,
    ttable: Arc<TranspositionTable>,
    history: Shared<SearchHistory>,
    killers: Shared<SearchKillers>,
    active_search: Option<Arc<AtomicBool>>,
}

impl LazySMP {
    pub fn create(threads: usize) -> Self {
        assert!(threads > 0, "At least 1 search thread is necessary.");

        let ttable = Arc::new(TranspositionTable::new(256)); // TODO: allow configuration
        let history = Arc::new(RwLock::new(SearchHistory::default()));
        let killers = Arc::new(RwLock::new(SearchKillers::default()));

        // Spawn the main search thread
        let (main, main_sender) = {
            let ttable = Arc::clone(&ttable);
            let history = Arc::clone(&history);
            let killers = Arc::clone(&killers);
            let (tx, rx) = std::sync::mpsc::channel();
            let main = Some(std::thread::spawn(|| main_search_thread(ttable, history, killers, rx)));

            (main, tx)
        };

        // Spawn all the supporter threads
        let supporters = (0..threads - 1)
            .map(|_| {
                let ttable = Arc::clone(&ttable);
                let history = Arc::clone(&history);
                let killers = Arc::clone(&killers);
                let (tx, rx) = std::sync::mpsc::channel();
                let handle = std::thread::spawn(|| supporter_search_thread(ttable, history, killers, rx));
                (handle, tx)
            })
            .collect();

        Self {
            main,
            main_sender,
            supporters,
            ttable,
            history,
            killers,
            active_search: None,
        }
    }

    pub fn reset_tables(&mut self) {
        self.ttable.reset();
        *self.history.write().unwrap() = SearchHistory::default();
        *self.killers.write().unwrap() = SearchKillers::default();
    }

    /// Starts a search. Fails, if a search is already running
    pub fn start_search(&mut self, search_config: SearchConfig) -> Result<(), AlreadyRunning> {
        if self.is_search_running() {
            return Err(AlreadyRunning);
        }

        let (hard_time, soft_time) = calculate_time_limit(&search_config.go, search_config.board.color).unzip();

        let time_man = Limits::new()
            .depth(search_config.go.depth.map(|d| d as u16))
            .nodes(search_config.go.nodes.map(|n| n as u64))
            .hard_time(hard_time)
            .soft_time(soft_time)
            .start_now();

        let switch = time_man.raw_stop_flag();
        self.active_search = Some(switch);

        self.ttable.next_age();

        let board = search_config.board.clone();

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
            &mut search_config.board.clone(),
            &mut ctx,
            search_config.allow_null_pruning,
            false,
        );

        let config = ThreadConfig {
            report_mode: search_config.report_mode,
            thread_num: 0,
            time_man: time_man.clone(),
            expected_eval,
            allow_null_pruning: search_config.allow_null_pruning,
        };

        self.main_sender
            .send(Message::StartSearch(config, board.clone()))
            .unwrap();

        for (thread_num, (_, tx)) in self.supporters.iter().enumerate() {
            let config = ThreadConfig {
                report_mode: search_config.report_mode,
                thread_num: thread_num as u32,
                time_man: time_man.clone(),
                expected_eval,
                allow_null_pruning: search_config.allow_null_pruning,
            };

            tx.send(Message::StartSearch(config, board.clone())).unwrap();
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
    StartSearch(ThreadConfig, Board),
    Quit,
}

fn main_search_thread(
    ttable: Arc<TranspositionTable>,
    history: Shared<SearchHistory>,
    killers: Shared<SearchKillers>,
    rx: Receiver<Message>,
) {
    loop {
        let (config, mut board) = match rx.recv().unwrap() {
            Message::StartSearch(thread_config, board) => (thread_config, board),
            Message::Quit => {
                println!("main thread quits");
                break;
            }
        };

        let mut ctx = ABContext {
            time_man: config.time_man,
            stats: SearchStats::default(),
            transposition_table: Arc::clone(&ttable),
            search_killers: killers.read().unwrap().clone(),
            search_history: history.read().unwrap().clone(),
            allow_null_pruning: config.allow_null_pruning,
        };

        let mut bestmove = ChessMove::default();
        let mut iterative_deepening = IterativeDeepening::new(config.expected_eval, 1);

        while let Some(stats) = iterative_deepening.next_depth(&mut board, &mut ctx) {
            bestmove = stats.bestmove;
            report_after_depth(config.report_mode, stats);
        }

        ctx.stats.bestmove = bestmove; // TODO: Do we need this assignment?
        report_after_search(config.report_mode, ctx.stats);

        *killers.write().unwrap() = ctx.search_killers;
        *history.write().unwrap() = ctx.search_history;
        ctx.time_man.force_stop();
    }
}

fn supporter_search_thread(
    ttable: Arc<TranspositionTable>,
    history: Shared<SearchHistory>,
    killers: Shared<SearchKillers>,
    rx: Receiver<Message>,
) {
    loop {
        let (config, mut board) = match rx.recv().unwrap() {
            Message::StartSearch(thread_config, board) => (thread_config, board),
            Message::Quit => {
                println!("supporter thread quits");
                break;
            }
        };

        let mut ctx = ABContext {
            time_man: config.time_man,
            stats: SearchStats::default(),
            transposition_table: Arc::clone(&ttable),
            search_killers: killers.read().unwrap().clone(),
            search_history: history.read().unwrap().clone(),
            allow_null_pruning: config.allow_null_pruning,
        };

        let start_depth = u16::min(config.thread_num as u16, ctx.time_man.depth_limit());
        loop {
            let mut iterative_deepening = IterativeDeepening::new(config.expected_eval, start_depth);
            while iterative_deepening.next_depth(&mut board, &mut ctx).is_some() {}

            if ctx.time_man.stop(&ctx.stats, false) {
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct SearchConfig<'a> {
    pub report_mode: ReportMode,
    pub allow_null_pruning: bool,
    pub go: uci::Go,
    pub board: &'a Board,
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
    thread_num: u32,
    time_man: TimeMan,
    expected_eval: Eval,
    allow_null_pruning: bool,
}
