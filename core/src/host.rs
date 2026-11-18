//! Simulation host: runs a `World` on a worker thread driven by the Clock and the speed Governor.
//! The UI thread only sends Commands and reads the latest Snapshot (a small copy under a short lock); it never touches the world.
//! Real time lost while the thread was suspended (e.g. app in background) is NOT owed: capped at MAX_FRAME_US per loop
//! (no offline progress, per the design).
use crate::clock::Clock;
use crate::command::Command;
use crate::governor::{GovInput, GovParams, Governor, INTERVAL_US};
use crate::snapshot::Snapshot;
use crate::world::World;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Ticks per work chunk and per-loop work budget (PROVISIONAL, to be tuned with real rules).
pub const CHUNK_TICKS: u64 = 16;
pub const WORK_BUDGET_MS: u64 = 8;
pub const MAX_FRAME_US: u64 = 250_000;

struct Shared {
    snapshot: Mutex<Snapshot>,
    stop: AtomicBool,
}

pub struct SimHost {
    shared: Arc<Shared>,
    tx: Sender<Command>,
    handle: Option<JoinHandle<()>>,
}

impl SimHost {
    /// `start_tick` > 0 when restoring a save. The simulation starts running immediately (not paused).
    pub fn start<W: World>(mut world: W, start_tick: u64, requested_speed: u64) -> SimHost {
        let (tx, rx) = channel::<Command>();
        let shared = Arc::new(Shared { snapshot: Mutex::new(Snapshot::default()), stop: AtomicBool::new(false) });
        let sh = Arc::clone(&shared);
        let handle = thread::spawn(move || {
            let mut clock = Clock::restore(start_tick, requested_speed);
            world.step_to(clock.tick);
            let mut gov = Governor::new(GovParams::default());
            let mut last = Instant::now();
            let (mut seq, mut drops, mut raises) = (0u64, 0u64, 0u64);
            let (mut iv_us, mut iv_ticks, mut iv_busy_us) = (0u64, 0u64, 0u64);
            let mut busy_permille = 0u64;
            while !sh.stop.load(Ordering::Relaxed) {
                while let Ok(c) = rx.try_recv() {
                    match c {
                        Command::SetRequestedSpeed(s) => clock.set_requested(s),
                        Command::SetPaused(p) => clock.set_paused(p),
                    }
                }
                let now = Instant::now();
                let dt_us = (now.duration_since(last).as_micros() as u64).min(MAX_FRAME_US);
                last = now;
                clock.advance(dt_us);
                let work_start = Instant::now();
                let mut ran = 0u64;
                loop {
                    let n = clock.take(CHUNK_TICKS);
                    if n == 0 {
                        break;
                    }
                    world.step_to(clock.tick);
                    ran += n;
                    if work_start.elapsed() >= Duration::from_millis(WORK_BUDGET_MS) {
                        break;
                    }
                }
                if ran > 0 {
                    iv_busy_us += work_start.elapsed().as_micros() as u64;
                }
                iv_ticks += ran;
                if clock.paused() {
                    iv_us = 0;
                    iv_ticks = 0;
                    iv_busy_us = 0;
                } else {
                    iv_us += dt_us;
                }
                if iv_us >= INTERVAL_US {
                    let action = gov.update(GovInput {
                        requested: clock.requested(),
                        effective: clock.effective(),
                        owed: clock.owed(),
                        ticks_done: iv_ticks,
                        interval_us: iv_us,
                    });
                    if let Some(a) = action {
                        if a.new_effective < clock.effective() {
                            drops += 1;
                        } else if a.new_effective > clock.effective() {
                            raises += 1;
                        }
                        clock.set_effective(a.new_effective);
                        if a.forgive_owed {
                            clock.forgive_owed();
                        }
                    }
                    busy_permille = (iv_busy_us * 1000 / iv_us.max(1)).min(1000);
                    iv_us = 0;
                    iv_ticks = 0;
                    iv_busy_us = 0;
                }
                seq += 1;
                let snap = Snapshot {
                    seq,
                    tick: clock.tick,
                    requested_speed: clock.requested(),
                    effective_speed: clock.effective(),
                    paused: clock.paused(),
                    owed_ticks: clock.owed(),
                    governor_drops: drops,
                    governor_raises: raises,
                    busy_permille,
                };
                if let Ok(mut g) = sh.snapshot.lock() {
                    *g = snap;
                }
                if ran == 0 {
                    thread::sleep(Duration::from_micros(500));
                }
            }
        });
        SimHost { shared, tx, handle: Some(handle) }
    }

    pub fn send(&self, c: Command) {
        let _ = self.tx.send(c);
    }

    pub fn snapshot(&self) -> Snapshot {
        match self.shared.snapshot.lock() {
            Ok(g) => *g,
            Err(_) => Snapshot::default(),
        }
    }

    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for SimHost {
    fn drop(&mut self) {
        self.stop();
    }
}
