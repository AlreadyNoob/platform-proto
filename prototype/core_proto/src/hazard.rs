//! Hybrid scheduler prototype (decision D-012). Geometry-free model: `comps` slots, each with a per-tick collapse
//! probability q. On collapse the slot is respawned with a freshly drawn q (hot w.p. p_hot, else cold) and starts
//! rolling at the NEXT tick. Four strategies produce the same stochastic process:
//!   Brute  - every slot rolls every tick
//!   Heap   - binary heap of next-collapse ticks (geometric sampling)
//!   Wheel  - hashed timing wheel with overflow list (O(1) schedule/pop)
//!   Hybrid - slots with q >= q_star roll every tick (hot list); the rest live in the wheel
//! Mirrors reference/hybrid_ref.py (validated there). STATUS: Rust port not compiled in the authoring sandbox.
use crate::detmath::geometric_failures;
use crate::rng::Rng;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const NONE: u32 = u32::MAX;
const NEVER: u64 = u64::MAX / 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strategy {
    Brute,
    Heap,
    Wheel,
    Hybrid,
}

impl Strategy {
    pub fn from_index(i: i64) -> Strategy {
        match i {
            0 => Strategy::Brute,
            1 => Strategy::Heap,
            2 => Strategy::Wheel,
            _ => Strategy::Hybrid,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Strategy::Brute => "brute",
            Strategy::Heap => "heap",
            Strategy::Wheel => "wheel",
            Strategy::Hybrid => "hybrid",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HazardParams {
    pub p_hot: f64,
    pub q_hot: f64,
    pub q_cold: f64,
    /// slots with q >= q_star are "hot" in the Hybrid strategy
    pub q_star: f64,
    /// wheel horizon = 2^wheel_bits ticks
    pub wheel_bits: u32,
    /// test control: multiplies q in Brute only (to prove the tests can detect a wrong process)
    pub brute_bias: f64,
    /// test control: skip the overflow rescan (a deliberate wheel bug)
    pub bug_no_rescan: bool,
    /// start with the steady-state mix of hot/cold slots (benchmarks) instead of drawing with p_hot (tests)
    pub steady_start: bool,
}

impl HazardParams {
    /// Parameters used by the equivalence tests (match hybrid_ref.py).
    pub fn test() -> Self {
        HazardParams { p_hot: 0.5, q_hot: 0.2, q_cold: 0.01, q_star: 0.05, wheel_bits: 5, brute_bias: 1.0, bug_no_rescan: false, steady_start: false }
    }
    /// Parameters used by benchmarks and the app: hot slots live ~50 ticks, cold ~10,000 ticks.
    pub fn bench(p_hot: f64) -> Self {
        HazardParams { p_hot, q_hot: 0.02, q_cold: 1e-4, q_star: 0.003, wheel_bits: 16, brute_bias: 1.0, bug_no_rescan: false, steady_start: true }
    }
    /// High-hazard experiment: hot slots collapse every ~5 ticks (q=0.2) and the hybrid threshold is 3% per tick.
    pub fn bench_hi(p_hot: f64) -> Self {
        HazardParams { p_hot, q_hot: 0.2, q_cold: 1e-4, q_star: 0.03, wheel_bits: 16, brute_bias: 1.0, bug_no_rescan: false, steady_start: true }
    }
    /// Long-run fraction of slots that are hot (time-weighted, since hot slots collapse sooner).
    pub fn steady_hot_fraction(&self) -> f64 {
        let h = self.p_hot / self.q_hot;
        let c = (1.0 - self.p_hot) / self.q_cold;
        h / (h + c)
    }
}

pub struct HazardSim {
    strat: Strategy,
    params: HazardParams,
    rng: Rng,
    q: Vec<f64>,
    thr: Vec<u64>,
    now: u64,
    pub events: u64,
    pub hash: u64,
    heap: BinaryHeap<Reverse<(u64, u32)>>,
    w: u64,
    mask: u64,
    head: Vec<u32>,
    nxt: Vec<u32>,
    due: Vec<u64>,
    over: Vec<u32>,
    hot: Vec<u32>,
    hot_pos: Vec<u32>,
    scratch: Vec<u32>,
    log_on: bool,
    log: Vec<(u64, u32)>,
}

impl HazardSim {
    pub fn new(strat: Strategy, comps: usize, params: HazardParams, seed: u64) -> Self {
        let wheel = strat == Strategy::Wheel || strat == Strategy::Hybrid;
        let w: u64 = 1u64 << params.wheel_bits;
        let mut s = HazardSim {
            strat,
            params,
            rng: Rng::new(seed),
            q: vec![0.0; comps],
            thr: vec![0; comps],
            now: 0,
            events: 0,
            hash: 0xcbf2_9ce4_8422_2325,
            heap: BinaryHeap::new(),
            w,
            mask: w - 1,
            head: if wheel { vec![NONE; w as usize] } else { Vec::new() },
            nxt: if wheel { vec![NONE; comps] } else { Vec::new() },
            due: if wheel { vec![0; comps] } else { Vec::new() },
            over: Vec::new(),
            hot: Vec::new(),
            hot_pos: if strat == Strategy::Hybrid { vec![NONE; comps] } else { Vec::new() },
            scratch: Vec::new(),
            log_on: false,
            log: Vec::new(),
        };
        let p_first = if params.steady_start { params.steady_hot_fraction() } else { params.p_hot };
        for c in 0..comps {
            let q = if s.rng.unit() < p_first { params.q_hot } else { params.q_cold };
            s.set_q(c, q);
        }
        for c in 0..comps {
            let q = s.q[c];
            match strat {
                Strategy::Brute => {}
                Strategy::Heap => {
                    let t = s.geo(q);
                    s.heap.push(Reverse((t, c as u32)));
                }
                Strategy::Wheel => {
                    let t = s.geo(q);
                    s.schedule(c as u32, t);
                }
                Strategy::Hybrid => {
                    if q >= params.q_star {
                        s.hot_add(c as u32);
                    } else {
                        let t = s.geo(q);
                        s.schedule(c as u32, t);
                    }
                }
            }
        }
        s
    }

    pub fn with_log(mut self) -> Self {
        self.log_on = true;
        self
    }

    pub fn take_log(&mut self) -> Vec<(u64, u32)> {
        std::mem::take(&mut self.log)
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn hot_count(&self) -> usize {
        self.hot.len()
    }

    pub fn comps(&self) -> usize {
        self.q.len()
    }

    fn set_q(&mut self, c: usize, q: f64) {
        self.q[c] = q;
        let qq = if self.strat == Strategy::Brute { q * self.params.brute_bias } else { q };
        self.thr[c] = if qq >= 1.0 { u64::MAX } else { (qq * 18446744073709551616.0) as u64 };
    }

    fn draw_q(&mut self) -> f64 {
        if self.rng.unit() < self.params.p_hot {
            self.params.q_hot
        } else {
            self.params.q_cold
        }
    }

    /// Failures before the first success, per-tick success probability q.
    fn geo(&mut self, q: f64) -> u64 {
        if q <= 0.0 {
            return NEVER;
        }
        if q >= 1.0 {
            return 0;
        }
        geometric_failures(self.rng.unit_open0(), q)
    }

    fn schedule(&mut self, c: u32, t: u64) {
        self.due[c as usize] = t;
        if t < self.now + self.w {
            let b = (t & self.mask) as usize;
            self.nxt[c as usize] = self.head[b];
            self.head[b] = c;
        } else {
            self.over.push(c);
        }
    }

    fn rescan(&mut self, t: u64) {
        let over = std::mem::take(&mut self.over);
        let mut keep: Vec<u32> = Vec::with_capacity(over.len());
        for c in over {
            let d = self.due[c as usize];
            if d < t + self.w {
                let b = (d & self.mask) as usize;
                self.nxt[c as usize] = self.head[b];
                self.head[b] = c;
            } else {
                keep.push(c);
            }
        }
        self.over = keep;
    }

    fn hot_add(&mut self, c: u32) {
        self.hot_pos[c as usize] = self.hot.len() as u32;
        self.hot.push(c);
    }

    fn hot_remove(&mut self, c: u32) {
        let p = self.hot_pos[c as usize];
        if p == NONE {
            return;
        }
        let last = self.hot.pop().unwrap();
        if last != c {
            self.hot[p as usize] = last;
            self.hot_pos[last as usize] = p;
        }
        self.hot_pos[c as usize] = NONE;
    }

    fn record(&mut self, t: u64, c: u32) {
        self.events += 1;
        self.hash ^= t;
        self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
        self.hash ^= c as u64;
        self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
        if self.log_on {
            self.log.push((t, c));
        }
    }

    /// Slot c collapses at tick t: record, respawn with a new q, reschedule according to the strategy.
    fn collapse(&mut self, c: u32, t: u64) {
        self.record(t, c);
        if self.strat == Strategy::Hybrid && self.q[c as usize] >= self.params.q_star {
            self.hot_remove(c); // only slots that were hot (q before this collapse) can be in the hot list
        }
        let q = self.draw_q();
        self.set_q(c as usize, q);
        match self.strat {
            Strategy::Brute => {}
            Strategy::Heap => {
                let tt = t.saturating_add(1).saturating_add(self.geo(q));
                self.heap.push(Reverse((tt, c)));
            }
            Strategy::Wheel => {
                let tt = t.saturating_add(1).saturating_add(self.geo(q));
                self.schedule(c, tt);
            }
            Strategy::Hybrid => {
                if q >= self.params.q_star {
                    self.hot_add(c); // rolls from t + 1
                } else {
                    let tt = t.saturating_add(1).saturating_add(self.geo(q));
                    self.schedule(c, tt);
                }
            }
        }
    }

    fn tick_wheel(&mut self) {
        let t = self.now;
        if t & self.mask == 0 && !self.over.is_empty() && !self.params.bug_no_rescan {
            self.rescan(t);
        }
        let b = (t & self.mask) as usize;
        let mut c = self.head[b];
        self.head[b] = NONE;
        self.scratch.clear();
        while c != NONE {
            self.scratch.push(c);
            c = self.nxt[c as usize];
        }
        if self.strat == Strategy::Hybrid {
            for i in 0..self.hot.len() {
                let h = self.hot[i];
                if self.rng.next_u64() < self.thr[h as usize] {
                    self.scratch.push(h);
                }
            }
        }
        let n = self.scratch.len();
        for i in 0..n {
            let c = self.scratch[i];
            self.collapse(c, t);
        }
        self.now = t + 1;
    }

    /// Advance the simulation so that ticks [now, target) have been processed. Batching does not change results.
    pub fn run_until(&mut self, target: u64) {
        match self.strat {
            Strategy::Brute => {
                let n = self.q.len();
                while self.now < target {
                    let t = self.now;
                    for c in 0..n {
                        if self.rng.next_u64() < self.thr[c] {
                            self.collapse(c as u32, t);
                        }
                    }
                    self.now = t + 1;
                }
            }
            Strategy::Heap => {
                while let Some(&Reverse((t, c))) = self.heap.peek() {
                    if t >= target {
                        break;
                    }
                    self.heap.pop();
                    self.collapse(c, t);
                }
                if target > self.now {
                    self.now = target;
                }
            }
            Strategy::Wheel | Strategy::Hybrid => {
                while self.now < target {
                    self.tick_wheel();
                }
            }
        }
    }
}
