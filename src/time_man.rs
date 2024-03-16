use crate::{search::SearchStats, types::Color, uci};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct Limits {
    time: Option<Duration>,
    nodes: Option<u64>,
    depth: Option<u16>,
    stop: Arc<AtomicBool>,
    cached_stop: bool,
}

impl Limits {
    pub fn new(go: uci::Go, color: Color, stop: Arc<AtomicBool>) -> Self {
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

        let cached_stop = stop.load(Ordering::Relaxed);

        Limits {
            time: max_time,
            nodes: go.nodes.map(|n| n as u64),
            depth: go.depth.map(|d| d as u16),
            stop,
            cached_stop,
        }
    }

    pub fn check_stop(&mut self, stats: &SearchStats, use_cached: bool) -> bool {
        if use_cached && stats.nodes.trailing_zeros() < 10 {
            return self.cached_stop;
        }

        let max_nodes = self.nodes.unwrap_or(u64::MAX);
        let max_time = self.time.unwrap_or(Duration::MAX);
        let max_depth = self.depth.unwrap_or(u16::MAX);

        let should_stop = stats.nodes > max_nodes
            || stats.start_time.elapsed() >= max_time
            || stats.depth > max_depth
            || self.stop.load(Ordering::Relaxed);

        self.cached_stop = should_stop;
        should_stop
    }

    pub fn force_stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cached_stop = true;
    }

    pub fn nodes(&self) -> Option<u64> {
        self.nodes
    }

    pub fn time(&self) -> Option<Duration> {
        self.time
    }

    pub fn depth(&self) -> Option<u16> {
        self.depth
    }
}
