//! The simulated world. M02a has no game rules: `NullWorld` only counts ticks, `BurnWorld` burns a configurable amount of
//! CPU per tick so the speed governor can be tested on a real device (it is a test tool, not gameplay).
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub trait World: Send + 'static {
    /// Advance the simulation so that all ticks before `tick` have been processed.
    fn step_to(&mut self, tick: u64);
}

pub struct NullWorld {
    pub tick: u64,
}

impl NullWorld {
    pub fn new() -> Self {
        NullWorld { tick: 0 }
    }
}

impl Default for NullWorld {
    fn default() -> Self {
        NullWorld::new()
    }
}

impl World for NullWorld {
    fn step_to(&mut self, tick: u64) {
        self.tick = tick;
    }
}

/// Spins for `ns_per_tick` nanoseconds per simulated tick (shared atomic so it can be changed while running).
pub struct BurnWorld {
    tick: u64,
    ns_per_tick: Arc<AtomicU64>,
}

pub const MAX_BURN_NS_PER_TICK: u64 = 1_000_000;

impl BurnWorld {
    pub fn new(ns_per_tick: Arc<AtomicU64>) -> Self {
        BurnWorld { tick: 0, ns_per_tick }
    }
}

impl World for BurnWorld {
    fn step_to(&mut self, tick: u64) {
        let n = tick.saturating_sub(self.tick);
        self.tick = tick;
        let ns = self.ns_per_tick.load(Ordering::Relaxed).min(MAX_BURN_NS_PER_TICK);
        if ns == 0 || n == 0 {
            return;
        }
        let end = Instant::now() + Duration::from_nanos(n.saturating_mul(ns));
        while Instant::now() < end {
            std::hint::spin_loop();
        }
    }
}
