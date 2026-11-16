//! Port of reference/equivalence_test.py (6x6 board, orthogonal adjacency). TEST VALUES, not game balance.
use crate::detmath::{geometric_failures, ln_1m, ln, powu};
use crate::rng::Rng;

pub const W: usize = 6;
pub const H: usize = 6;
pub const BASE: f64 = 0.002;
pub const CAP: f64 = 0.25;
pub const PLACE_TICK: u64 = 60;
pub const T_MID: u64 = 75;
pub const T_END: u64 = 150;
const M_BOUNDS: [u64; 2] = [50, 100];
const INF: u64 = u64::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    /// each collapsing particle is removed; component may split
    A,
    /// a collapse takes the particle's whole connected component
    B,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bug {
    None,
    /// do not redraw schedules when the probability multiplier changes
    StaleP,
    /// use per-particle p instead of component probability 1-(1-p)^n
    ForgetN,
}

/// (first collapse tick or -1, particles at tick 75, particles at tick 150)
pub type Outcome = (i64, u32, u32);

fn bit(x: usize, y: usize) -> u64 {
    1u64 << (y * W + x)
}

pub fn init_mask() -> u64 {
    let cells = [(0, 0), (1, 0), (2, 0), (2, 1), (2, 2), (1, 2), (4, 0), (5, 0), (5, 1), (4, 4), (0, 5)];
    cells.iter().fold(0u64, |m, &(x, y)| m | bit(x, y))
}

fn m_at(t: u64) -> f64 {
    if (50..100).contains(&t) {
        2.5
    } else {
        1.0
    }
}

fn p_of(n: u32, m: f64) -> f64 {
    let p = BASE * m * powu(2.0, n - 1);
    if p > CAP {
        CAP
    } else {
        p
    }
}

pub fn components(mask: u64) -> Vec<u64> {
    let mut seen = 0u64;
    let mut out = Vec::new();
    for i in 0..(W * H) {
        let b = 1u64 << i;
        if mask & b != 0 && seen & b == 0 {
            let mut comp = 0u64;
            let mut stack = vec![i];
            while let Some(j) = stack.pop() {
                let bj = 1u64 << j;
                if comp & bj != 0 {
                    continue;
                }
                comp |= bj;
                let (x, y) = (j % W, j / W);
                let mut nb = Vec::with_capacity(4);
                if x + 1 < W { nb.push(y * W + x + 1); }
                if x > 0 { nb.push(y * W + x - 1); }
                if y + 1 < H { nb.push((y + 1) * W + x); }
                if y > 0 { nb.push((y - 1) * W + x); }
                for k in nb {
                    if mask & (1u64 << k) != 0 && comp & (1u64 << k) == 0 {
                        stack.push(k);
                    }
                }
            }
            seen |= comp;
            out.push(comp);
        }
    }
    out
}

fn cells_of(comp: u64) -> Vec<usize> {
    (0..(W * H)).filter(|i| comp >> i & 1 == 1).collect()
}

/// Reference: every particle rolls every tick.
pub fn brute(rng: &mut Rng, variant: Variant) -> Outcome {
    let mut mask = init_mask();
    let mut first = -1i64;
    let mut mid = 0u32;
    for t in 0..T_END {
        if t == T_MID {
            mid = mask.count_ones();
        }
        if t == PLACE_TICK {
            mask |= bit(3, 1);
        }
        let mt = m_at(t);
        let mut dead = 0u64;
        for comp in components(mask) {
            let p = p_of(comp.count_ones(), mt);
            let mut hit = 0u64;
            for i in cells_of(comp) {
                if rng.unit() < p {
                    hit |= 1u64 << i;
                }
            }
            if hit != 0 {
                dead |= if variant == Variant::B { comp } else { hit };
            }
        }
        if dead != 0 {
            mask &= !dead;
            if first < 0 {
                first = t as i64;
            }
        }
    }
    (first, mid, mask.count_ones())
}

struct Sched {
    items: Vec<(u64, u64)>, // (component mask, next collapse tick)
}

impl Sched {
    fn draw(&mut self, rng: &mut Rng, comp: u64, t0: u64, bug: Bug) {
        let n = comp.count_ones();
        let p = p_of(n, m_at(t0));
        let q = if bug == Bug::ForgetN { p } else { 1.0 - powu(1.0 - p, n) };
        let t = if q <= 0.0 {
            INF
        } else if q >= 1.0 {
            t0
        } else {
            t0 + geometric_failures(rng.unit_open0(), q)
        };
        self.items.push((comp, t));
    }

    fn refresh(&mut self, rng: &mut Rng, mask: u64, t0: u64, redraw_all: bool, bug: Bug) {
        let cur = components(mask);
        self.items.retain(|(c, _)| !redraw_all && cur.contains(c));
        for c in cur {
            if !self.items.iter().any(|(k, _)| *k == c) {
                self.draw(rng, c, t0, bug);
            }
        }
    }
}

/// Event-driven: schedule the next collapse tick per component (geometric), exact by memorylessness.
pub fn event_driven(rng: &mut Rng, variant: Variant, bug: Bug) -> Outcome {
    let mut mask = init_mask();
    let mut first = -1i64;
    let mut mid: Option<u32> = None;
    let mut s = Sched { items: Vec::new() };
    s.refresh(rng, mask, 0, false, bug);
    let keys = [50u64, PLACE_TICK, T_MID, 100];
    let mut cur = 0u64;
    loop {
        let mut t = T_END;
        for &(_, tt) in &s.items {
            if tt >= cur && tt < INF && tt < t { t = tt; }
        }
        for &k in &keys {
            if k >= cur && k < t { t = k; }
        }
        if t >= T_END { break; }
        if t == T_MID { mid = Some(mask.count_ones()); }
        if t == PLACE_TICK {
            mask |= bit(3, 1);
            s.refresh(rng, mask, t, false, bug);
        }
        if M_BOUNDS.contains(&t) && bug != Bug::StaleP {
            s.refresh(rng, mask, t, true, bug);
        }
        let due: Vec<u64> = s.items.iter().filter(|(_, tt)| *tt == t).map(|(c, _)| *c).collect();
        if !due.is_empty() {
            let mut dead = 0u64;
            for comp in due {
                if variant == Variant::B {
                    dead |= comp;
                    continue;
                }
                let cl = cells_of(comp);
                let n = cl.len();
                let p = p_of(n as u32, m_at(t));
                let q = 1.0 - powu(1.0 - p, n as u32);
                let u = rng.unit();
                // first success position ~ truncated geometric on 0..n-1
                let k = (ln(1.0 - u * q) / ln_1m(p)).ceil() as i64 - 1;
                let i = k.clamp(0, n as i64 - 1) as usize;
                dead |= 1u64 << cl[i];
                for j in (i + 1)..n {
                    if rng.unit() < p {
                        dead |= 1u64 << cl[j];
                    }
                }
            }
            mask &= !dead;
            if first < 0 { first = t as i64; }
            s.refresh(rng, mask, t + 1, false, bug);
        }
        cur = t + 1;
    }
    (first, mid.unwrap_or_else(|| mask.count_ones()), mask.count_ones())
}

/// FNV-1a over many runs: used to compare desktop vs Android bit-for-bit (acceptance A6).
pub fn determinism_hash(seed: u64, runs: u32) -> u64 {
    let mut rng = Rng::new(seed);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for variant in [Variant::A, Variant::B] {
        for _ in 0..runs {
            let (f, a, b) = event_driven(&mut rng, variant, Bug::None);
            for v in [f as u64, a as u64, b as u64] {
                h ^= v;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    h ^ rng.next_u64()
}
