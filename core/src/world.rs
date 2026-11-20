//! The simulated world. M02a has no game rules: `NullWorld` only counts ticks, `BurnWorld` burns a configurable amount of
//! CPU per tick so the speed governor can be tested on a real device (it is a test tool, not gameplay).
use crate::command::Command;
use crate::save::WorldState;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Number of world counters in a snapshot: count, free placements, particles, chains, view revision (set by the host),
/// count of rejected commands (matters to the player), code of the latest rejection, undo depth.
pub const WORLD_STATS: usize = 8;

/// Everything that must be saved besides the save header: the clock tick the world stands at, the RNG states and the world state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldExport {
    pub tick: u64,
    pub rng_states: Vec<[u64; 4]>,
    pub state: WorldState,
}

pub trait World: Send + 'static {
    /// Advance the simulation so that all ticks before `tick` have been processed.
    fn step_to(&mut self, tick: u64);
    /// A player command that is not about the clock (placing, undo). Called by the host between ticks, never in the middle of one.
    fn command(&mut self, _c: Command) {}
    /// World counters for the snapshot (see `WORLD_STATS`).
    fn stats(&self) -> [i64; WORLD_STATS] {
        [0; WORLD_STATS]
    }
    /// Changes whenever the particles change; the host republishes the cells for the presentation when it moves.
    fn view_rev(&self) -> u64 {
        0
    }
    /// Flat `[x, y, type, ...]` list of at most `max_cells` particles for the presentation.
    fn view_cells(&self, _max_cells: usize) -> Vec<i32> {
        Vec::new()
    }
    /// Moves the collapse records produced since the last call into `out` as `[tick, x, y, chain, depth, ...]` (the animation feed, D-025).
    fn drain_events(&mut self, _out: &mut Vec<i64>) {}
    /// A copy of everything needed to save this world, or None for worlds that have no saveable state.
    fn export(&self) -> Option<WorldExport> {
        None
    }
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
