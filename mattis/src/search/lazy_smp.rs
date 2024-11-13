use super::{alpha_beta, history::SearchHistory, killers::SearchKillers, ABContext, SearchStats};
use crate::{
    board::Board,
    chess_move::ChessMove,
    hashtable::TranspositionTable,
    search::IterativeDeepening,
    time_man::{Limits, TimeMan},
};
use mattis_types::{Color, Eval};
use mattis_uci as uci;
use mattis_uci::EngineMessage;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::JoinHandle,
    time::Duration,
};

pub type Shared<T> = Arc<RwLock<T>>;

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
    pub search_killers: Shared<SearchKillers>,
    pub search_history: Shared<SearchHistory>,
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

pub fn run_search(config: SearchConfig) -> KillSwitch {
    let (hard_time, soft_time) = calculate_time_limit(&config.go, config.board.color).unzip();

    let time_man = Limits::new()
        .depth(config.go.depth.map(|d| d as u16))
        .nodes(config.go.nodes.map(|n| n as u64))
        .hard_time(hard_time)
        .soft_time(soft_time)
        .start_now();

    let mut ctx = ABContext {
        time_man: time_man.clone(),
        stats: SearchStats::default(),
        transposition_table: Arc::clone(&config.tp_table),
        search_killers: config.search_killers.read().unwrap().clone(),
        search_history: config.search_history.read().unwrap().clone(),
        allow_null_pruning: config.allow_null_pruning,
    };

    let expected_eval = alpha_beta(
        -Eval::MAX,
        Eval::MAX,
        1,
        &mut config.board.clone(),
        &mut ctx,
        config.allow_null_pruning,
        false,
    );

    let join_handles: Vec<JoinHandle<()>> = (0..config.thread_count)
        .map(|i| {
            let thread_config = ThreadConfig {
                tp_table: Arc::clone(&config.tp_table),
                search_killers: Arc::clone(&config.search_killers),
                search_history: Arc::clone(&config.search_history),
                thread_num: i,
                time_man: time_man.clone(),
                expected_eval,
                allow_null_pruning: config.allow_null_pruning,
            };

            let board = config.board.clone();
            std::thread::spawn(move || search_thread(thread_config, board))
        })
        .collect();

    KillSwitch {
        switch: time_man.raw_stop_flag(),
        join_handles,
    }
}

pub struct ThreadConfig {
    tp_table: Arc<TranspositionTable>,
    search_killers: Shared<SearchKillers>,
    search_history: Shared<SearchHistory>,
    thread_num: u32,
    time_man: TimeMan,
    expected_eval: Eval,
    allow_null_pruning: bool,
}

pub fn search_thread(config: ThreadConfig, mut board: Board) {
    let mut ctx = ABContext {
        time_man: config.time_man,
        stats: SearchStats::default(),
        transposition_table: config.tp_table,
        search_killers: config.search_killers.read().unwrap().clone(),
        search_history: config.search_history.read().unwrap().clone(),
        allow_null_pruning: config.allow_null_pruning,
    };

    if config.thread_num == 0 {
        let mut bestmove = ChessMove::default();
        let mut iterative_deepening = IterativeDeepening::new(config.expected_eval, 1);

        while let Some(stats) = iterative_deepening.next_depth(&mut board, &mut ctx) {
            bestmove = stats.bestmove;

            let info = EngineMessage::Info(uci::Info {
                depth: Some(stats.depth as u32),
                nodes: Some(stats.nodes as u32),
                pv: stats.pv.into_iter().map(|m| format!("{}", m.display_smith())).collect(),
                // FIXME: Mate score can be off by 1 at low depths,
                // because the score comes straight from the hashtable which stored the entry one move ago.
                score: Some(uci::Score(stats.score)),
                ..Default::default()
            });

            println!("{info}");
        }

        let bestmove = EngineMessage::Bestmove {
            move_: format!("{}", bestmove.display_smith()),
            ponder: None,
        };

        println!("{bestmove}");
        *config.search_killers.write().unwrap() = ctx.search_killers;
        *config.search_history.write().unwrap() = ctx.search_history;
        ctx.time_man.force_stop();
    } else {
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
