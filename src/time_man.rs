use crate::{search::SearchStats, types::Color, uci};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct TimeMan {
    start_time: Instant,
    time_limit: Option<Duration>,
    node_limit: Option<u64>,
    depth_limit: Option<u16>,
    stop: Arc<AtomicBool>,
    cached_stop: bool,
}

impl TimeMan {
    pub fn new(go: uci::Go, color: Color) -> Self {
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

        TimeMan {
            start_time: Instant::now(),
            time_limit: max_time,
            node_limit: go.nodes.map(|n| n as u64),
            depth_limit: go.depth.map(|d| d as u16),
            stop: Arc::new(AtomicBool::new(false)),
            cached_stop: false,
        }
    }

    pub fn node_limit(&self) -> Option<u64> {
        self.node_limit
    }

    pub fn time_limit(&self) -> Option<Duration> {
        self.time_limit
    }

    pub fn depth_limit(&self) -> Option<u16> {
        self.depth_limit
    }

    pub fn get_stop(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop)
    }

    pub fn check_stop(&mut self, stats: &SearchStats, use_cached: bool) -> bool {
        if use_cached && stats.nodes.trailing_zeros() < 10 {
            return self.cached_stop;
        }

        let max_nodes = self.node_limit.unwrap_or(u64::MAX);
        let max_time = self.time_limit.unwrap_or(Duration::MAX);
        let max_depth = self.depth_limit.unwrap_or(u16::MAX);

        let should_stop = stats.nodes > max_nodes
            || self.start_time.elapsed() >= max_time
            || stats.depth > max_depth
            || self.stop.load(Ordering::Relaxed);

        self.cached_stop = should_stop;
        should_stop
    }

    pub fn force_stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cached_stop = true;
    }
}
