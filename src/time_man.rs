use crate::search::SearchStats;
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
    time_limit: Duration,
    node_limit: u64,
    depth_limit: u16,
    stop: Arc<AtomicBool>,
    cached_stop: bool,
}

pub struct Limits {
    time_limit: Duration,
    node_limit: u64,
    depth_limit: u16,
    stop: Arc<AtomicBool>,
}

impl Limits {
    pub fn new() -> Limits {
        Limits {
            time_limit: Duration::MAX,
            node_limit: u64::MAX,
            depth_limit: u16::MAX,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn time(&mut self, limit: Option<Duration>) -> &mut Self {
        if let Some(limit) = limit {
            self.time_limit = limit;
        }

        self
    }

    pub fn nodes(&mut self, limit: Option<u64>) -> &mut Self {
        if let Some(limit) = limit {
            self.node_limit = limit;
        }

        self
    }

    pub fn depth(&mut self, limit: Option<u16>) -> &mut Self {
        if let Some(limit) = limit {
            self.depth_limit = limit;
        }

        self
    }

    pub fn start_now(&self) -> TimeMan {
        TimeMan {
            start_time: Instant::now(),
            time_limit: self.time_limit,
            node_limit: self.node_limit,
            depth_limit: self.depth_limit,
            stop: Arc::clone(&self.stop),
            cached_stop: self.stop.load(Ordering::Relaxed),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeMan {
    pub fn node_limit(&self) -> u64 {
        self.node_limit
    }

    pub fn time_limit(&self) -> Duration {
        self.time_limit
    }

    pub fn depth_limit(&self) -> u16 {
        self.depth_limit
    }

    pub fn get_stop(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop)
    }

    pub fn check_stop(&mut self, stats: &SearchStats, use_cached: bool) -> bool {
        if use_cached && stats.nodes.trailing_zeros() < 10 {
            return self.cached_stop;
        }

        let should_stop = stats.nodes > self.node_limit
            || stats.depth > self.depth_limit
            || self.start_time.elapsed() >= self.time_limit
            || self.stop.load(Ordering::Relaxed);

        self.cached_stop = should_stop;
        should_stop
    }

    pub fn force_stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cached_stop = true;
    }
}
