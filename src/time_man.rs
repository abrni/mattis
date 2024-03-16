use crate::search::SearchStats;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

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
            last_depth_reached: Instant::now(),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TimeMan {
    start_time: Instant,
    time_limit: Duration,
    node_limit: u64,
    depth_limit: u16,
    stop: Arc<AtomicBool>,
    cached_stop: bool,
    last_depth_reached: Instant,
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

    pub fn raw_stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop)
    }

    pub fn stop(&mut self, stats: &SearchStats, use_cached: bool) -> bool {
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

    pub fn finished_depth(&mut self) {
        self.last_depth_reached = Instant::now();
    }

    pub fn enough_time_for_next_depth(&self) -> bool {
        if self.time_limit == Duration::MAX {
            return true;
        }

        let time_used = self.last_depth_reached.duration_since(self.start_time);
        let time_left = (self.start_time + self.time_limit).duration_since(self.last_depth_reached);
        let expected_next_time = time_used * 10;
        expected_next_time < time_left
    }

    pub fn force_stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cached_stop = true;
    }
}
