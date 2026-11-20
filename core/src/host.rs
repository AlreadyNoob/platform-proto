//! Simulation host: runs a `World` on a worker thread driven by the Clock and the speed Governor.
//! The UI thread only sends Commands and reads the latest Snapshot (a small copy under a short lock); it never touches the world.
//! Real time lost while the thread was suspended (e.g. app in background) is NOT owed: capped at MAX_FRAME_US per loop
//! (no offline progress, per the design).
use crate::clock::Clock;
use crate::command::Command;
use crate::governor::{GovInput, GovParams, Governor, INTERVAL_US};
use crate::snapshot::Snapshot;
use crate::world::{World, WorldExport};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Ticks per work chunk and per-loop work budget (PROVISIONAL, to be tuned with real rules).
pub const CHUNK_TICKS: u64 = 16;
pub const WORK_BUDGET_MS: u64 = 8;
pub const MAX_FRAME_US: u64 = 250_000;
/// The presentation gets at most this many particles (PROVISIONAL: real viewport chunks replace this later, D-013, K-52).
pub const MAX_VIEW_CELLS: usize = 20_000;
/// The presentation copy of the cells is refreshed at most this often.
pub const VIEW_MIN_INTERVAL_MS: u64 = 30;
/// Collapse records waiting for the presentation: oldest are dropped first (5 values each).
pub const MAX_EVENT_VALUES: usize = 5 * 20_000;

struct Shared {
    snapshot: Mutex<Snapshot>,
    stop: AtomicBool,
    /// (revision, [x, y, type, ...]) of the particles, refreshed when the world changed.
    view: Mutex<(u64, Vec<i32>)>,
    /// Collapse records for the wave animation: [tick, x, y, chain, depth, ...].
    events: Mutex<Vec<i64>>,
    /// Answer to the latest `Command::Export`: (request id, export or None if the world has nothing to save).
    export: Mutex<Option<(u64, Option<WorldExport>)>>,
}

pub struct SimHost {
    shared: Arc<Shared>,
    tx: Sender<Command>,
    handle: Option<JoinHandle<()>>,
    next_req: AtomicU64,
}

impl SimHost {
    /// `start_tick` > 0 when restoring a save. The simulation starts running immediately (not paused).
    pub fn start<W: World>(mut world: W, start_tick: u64, requested_speed: u64) -> SimHost {
        let (tx, rx) = channel::<Command>();
        let shared = Arc::new(Shared {
            snapshot: Mutex::new(Snapshot::default()),
            stop: AtomicBool::new(false),
            view: Mutex::new((0, Vec::new())),
            events: Mutex::new(Vec::new()),
            export: Mutex::new(None),
        });
        let sh = Arc::clone(&shared);
        let handle = thread::spawn(move || {
            let mut clock = Clock::restore(start_tick, requested_speed);
            world.step_to(clock.tick);
            let mut gov = Governor::new(GovParams::default());
            let mut last = Instant::now();
            let (mut seq, mut drops, mut raises) = (0u64, 0u64, 0u64);
            let (mut iv_us, mut iv_ticks, mut iv_busy_us) = (0u64, 0u64, 0u64);
            let mut busy_permille = 0u64;
            let mut view_published: u64 = u64::MAX; // forces the first publication
            let mut view_last = Instant::now();
            let mut evbuf: Vec<i64> = Vec::new();
            while !sh.stop.load(Ordering::Relaxed) {
                while let Ok(c) = rx.try_recv() {
                    match c {
                        Command::SetRequestedSpeed(s) => clock.set_requested(s),
                        Command::SetPaused(p) => clock.set_paused(p),
                        Command::Export(id) => {
                            let e = world.export();
                            if let Ok(mut g) = sh.export.lock() {
                                *g = Some((id, e));
                            }
                        }
                        other => world.command(other),
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
                // Presentation channels: the particle cells (change-driven, throttled) and the collapse records.
                let rev = world.view_rev();
                if rev != view_published && (view_published == u64::MAX || view_last.elapsed() >= Duration::from_millis(VIEW_MIN_INTERVAL_MS)) {
                    let cells = world.view_cells(MAX_VIEW_CELLS);
                    if let Ok(mut g) = sh.view.lock() {
                        *g = (rev, cells);
                    }
                    view_published = rev;
                    view_last = Instant::now();
                }
                evbuf.clear();
                world.drain_events(&mut evbuf);
                if !evbuf.is_empty() {
                    if let Ok(mut g) = sh.events.lock() {
                        g.extend_from_slice(&evbuf);
                        if g.len() > MAX_EVENT_VALUES {
                            let cut = g.len() - MAX_EVENT_VALUES;
                            drop(g.drain(0..cut));
                        }
                    }
                }
                let mut ws = world.stats();
                ws[4] = if view_published == u64::MAX { 0 } else { view_published as i64 };
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
                    world: ws,
                };
                if let Ok(mut g) = sh.snapshot.lock() {
                    *g = snap;
                }
                if ran == 0 {
                    thread::sleep(Duration::from_micros(500));
                }
            }
        });
        SimHost { shared, tx, handle: Some(handle), next_req: AtomicU64::new(1) }
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

    /// The latest published particle cells with their revision (the revision is also in the snapshot as `view_rev`).
    pub fn view(&self) -> (u64, Vec<i32>) {
        match self.shared.view.lock() {
            Ok(g) => (g.0, g.1.clone()),
            Err(_) => (0, Vec::new()),
        }
    }

    /// Takes the collapse records published since the last call: [tick, x, y, chain, depth, ...].
    pub fn take_events(&self) -> Vec<i64> {
        match self.shared.events.lock() {
            Ok(mut g) => std::mem::take(&mut *g),
            Err(_) => Vec::new(),
        }
    }

    /// Asks the worker for a consistent copy of the world and waits for it (at most `timeout_ms`).
    /// `None` = the worker did not answer in time (the caller must NOT treat that as "nothing to save");
    /// `Some(None)` = the world has no saveable state; `Some(Some(e))` = the copy.
    pub fn export_blocking(&self, timeout_ms: u64) -> Option<Option<WorldExport>> {
        let id = self.next_req.fetch_add(1, Ordering::Relaxed);
        self.send(Command::Export(id));
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            if let Ok(mut g) = self.shared.export.lock() {
                let mine = matches!(&*g, Some((rid, _)) if *rid == id);
                if mine {
                    return g.take().map(|(_, e)| e);
                }
            }
            if Instant::now() >= deadline {
                return None;
            }
            thread::sleep(Duration::from_millis(1));
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
