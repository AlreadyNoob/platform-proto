//! Speed governor (D-019, PROVISIONAL numbers): when the device cannot keep up, lower the effective speed and
//! say so; probe upward carefully. It never changes rules, only how fast game time runs against the wall clock.
//! A probe that makes lag build up is reverted at once and the wait before the next probe doubles.
//! Mirrors tools/ref/platform_ref.py (class Governor); core/tests/governor.rs replays the same scenarios.

pub const INTERVAL_US: u64 = 250_000;

#[derive(Clone, Copy, Debug)]
pub struct GovParams {
    pub high_lag_us: u64,
    pub low_lag_us: u64,
    pub margin_pct: u64,
    pub up_pct: u64,
    pub base_wait: u32,
    pub backoff_max: u32,
    pub ok_reset: u32,
}

impl Default for GovParams {
    fn default() -> Self {
        GovParams { high_lag_us: 1_000_000, low_lag_us: 100_000, margin_pct: 90, up_pct: 125, base_wait: 6, backoff_max: 80, ok_reset: 40 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GovInput {
    pub requested: u64,
    pub effective: u64,
    pub owed: u64,
    pub ticks_done: u64,
    pub interval_us: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GovAction {
    pub new_effective: u64,
    pub forgive_owed: bool,
}

#[derive(Clone, Debug)]
pub struct Governor {
    p: GovParams,
    calm: u32,
    wait: u32,
    since_raise: Option<u32>,
    pre_raise: u64,
}

impl Governor {
    pub fn new(p: GovParams) -> Self {
        Governor { p, calm: 0, wait: p.base_wait, since_raise: None, pre_raise: 1 }
    }

    fn failed_probe(&mut self) {
        self.wait = (self.wait * 2).min(self.p.backoff_max);
        self.since_raise = None;
        self.calm = 0;
    }

    /// Call once per INTERVAL_US of real running time (not while paused).
    pub fn update(&mut self, i: GovInput) -> Option<GovAction> {
        let eff = i.effective.max(1);
        let lag_us = (i.owed as u128 * 1_000_000u128 / eff as u128) as u64; // BASE_TICKS_PER_SEC == 1
        if lag_us > self.p.high_lag_us {
            let achieved = (i.ticks_done as u128 * 1_000_000u128 / i.interval_us.max(1) as u128) as u64;
            let new = (achieved * self.p.margin_pct / 100).min(eff * self.p.margin_pct / 100).max(1);
            if self.since_raise.is_some() {
                self.failed_probe();
            } else {
                self.calm = 0;
            }
            return Some(GovAction { new_effective: new, forgive_owed: true });
        }
        if lag_us <= self.p.low_lag_us {
            self.calm += 1;
            if let Some(k) = self.since_raise {
                let k = k + 1;
                if k > self.p.ok_reset {
                    self.since_raise = None;
                    self.wait = self.p.base_wait;
                } else {
                    self.since_raise = Some(k);
                }
            }
            if eff < i.requested && self.calm >= self.wait {
                let new = i.requested.min((eff + 1).max(eff * self.p.up_pct / 100));
                self.pre_raise = eff;
                self.calm = 0;
                self.since_raise = Some(0);
                return Some(GovAction { new_effective: new, forgive_owed: false });
            }
        } else if self.since_raise.is_some() {
            let back = self.pre_raise;
            self.failed_probe();
            return Some(GovAction { new_effective: back, forgive_owed: true });
        } else {
            self.calm = 0;
        }
        None
    }
}
