//! Simulation on a worker thread driven by the Clock. The UI thread only reads atomics (no locks, no snapshots here).
use crate::clock::{Clock, BASE_TICKS_PER_SEC};
use crate::hazard::{HazardParams, HazardSim, Strategy};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Default)]
struct Shared {
    tick: AtomicU64,
    owed: AtomicU64,
    events: AtomicU64,
    hot: AtomicU64,
    busy_us: AtomicU64,
    speed: AtomicU64,
    paused: AtomicBool,
    stop: AtomicBool,
}

#[derive(Clone, Copy, Debug)]
pub struct WorkerStats {
    pub tick: u64,
    pub owed: u64,
    pub events: u64,
    pub hot: u64,
    pub busy_us: u64,
    pub speed: u64,
    pub target_ticks_per_sec: u64,
}

pub struct SimWorker {
    shared: Arc<Shared>,
    handle: Option<JoinHandle<()>>,
}

impl SimWorker {
    pub fn start(strategy: Strategy, comps: usize, params: HazardParams, seed: u64, speed: u64) -> SimWorker {
        let shared = Arc::new(Shared::default());
        shared.speed.store(speed, Ordering::Relaxed);
        let sh = Arc::clone(&shared);
        let handle = thread::spawn(move || {
            let mut sim = HazardSim::new(strategy, comps, params, seed);
            let mut clock = Clock::new(speed);
            let mut last = Instant::now();
            while !sh.stop.load(Ordering::Relaxed) {
                let now = Instant::now();
                let dt_us = now.duration_since(last).as_micros() as u64;
                last = now;
                clock.set_speed(sh.speed.load(Ordering::Relaxed));
                clock.set_paused(sh.paused.load(Ordering::Relaxed));
                clock.advance(dt_us);
                let work_start = Instant::now();
                let mut ran = 0u64;
                loop {
                    let n = clock.take(100);
                    if n == 0 {
                        break;
                    }
                    sim.run_until(clock.tick);
                    ran += n;
                    if work_start.elapsed() >= Duration::from_millis(8) {
                        break;
                    }
                }
                if ran > 0 {
                    sh.busy_us.fetch_add(work_start.elapsed().as_micros() as u64, Ordering::Relaxed);
                }
                sh.tick.store(clock.tick, Ordering::Relaxed);
                sh.owed.store(clock.owed(), Ordering::Relaxed);
                sh.events.store(sim.events, Ordering::Relaxed);
                sh.hot.store(sim.hot_count() as u64, Ordering::Relaxed);
                if ran == 0 {
                    thread::sleep(Duration::from_micros(500));
                }
            }
        });
        SimWorker { shared, handle: Some(handle) }
    }

    pub fn set_speed(&self, speed: u64) {
        self.shared.speed.store(speed, Ordering::Relaxed);
    }

    pub fn set_paused(&self, paused: bool) {
        self.shared.paused.store(paused, Ordering::Relaxed);
    }

    pub fn stats(&self) -> WorkerStats {
        let speed = self.shared.speed.load(Ordering::Relaxed);
        WorkerStats {
            tick: self.shared.tick.load(Ordering::Relaxed),
            owed: self.shared.owed.load(Ordering::Relaxed),
            events: self.shared.events.load(Ordering::Relaxed),
            hot: self.shared.hot.load(Ordering::Relaxed),
            busy_us: self.shared.busy_us.load(Ordering::Relaxed),
            speed,
            target_ticks_per_sec: speed * BASE_TICKS_PER_SEC,
        }
    }

    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for SimWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
