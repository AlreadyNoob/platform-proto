//! Simulation clock: pause, requested speed, effective (governed) speed, exact integer time accounting.
//! 1 tick = 1 sim-second at 1x (PROVISIONAL assumption A-1; GDD question Q-4).
//! `requested` is what the player asked for; `effective` is what the speed governor currently allows.
//! Mirrors the Python reference in tools/ref/platform_ref.py (class Clock).

pub const BASE_TICKS_PER_SEC: u64 = 1;
pub const MAX_SPEED: u64 = 10_000;

#[derive(Clone, Debug)]
pub struct Clock {
    pub tick: u64,
    requested: u64,
    effective: u64,
    paused: bool,
    acc: u128, // microseconds * ticks-per-second; the remainder below 1_000_000 is never lost
    owed: u64,
}

impl Clock {
    pub fn new(requested: u64) -> Self {
        let r = requested.clamp(1, MAX_SPEED);
        Clock { tick: 0, requested: r, effective: r, paused: false, acc: 0, owed: 0 }
    }

    /// Restore from a save: the tick counter continues where it stopped; no catch-up is owed.
    pub fn restore(tick: u64, requested: u64) -> Self {
        let mut c = Clock::new(requested);
        c.tick = tick;
        c
    }

    /// Player changes the speed. If the governor was not limiting, the new speed applies fully;
    /// if it was, the limit stays but never exceeds the new request.
    pub fn set_requested(&mut self, speed: u64) {
        let s = speed.clamp(1, MAX_SPEED);
        if self.effective == self.requested {
            self.effective = s;
        } else {
            self.effective = self.effective.min(s);
        }
        self.requested = s;
    }

    /// Used by the governor. Always within 1..=requested.
    pub fn set_effective(&mut self, speed: u64) {
        self.effective = speed.clamp(1, self.requested);
    }

    pub fn requested(&self) -> u64 {
        self.requested
    }

    pub fn effective(&self) -> u64 {
        self.effective
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Real time passes. While paused nothing accumulates, so resuming never causes a catch-up burst.
    pub fn advance(&mut self, dt_us: u64) {
        if self.paused {
            return;
        }
        self.acc += dt_us as u128 * self.effective as u128 * BASE_TICKS_PER_SEC as u128;
        let due = self.acc / 1_000_000;
        self.acc %= 1_000_000;
        self.owed += due as u64;
    }

    /// Ticks the caller may run now (at most `budget`). Returns 0 while paused. Leftover ticks stay owed ("lag").
    pub fn take(&mut self, budget: u64) -> u64 {
        if self.paused {
            return 0;
        }
        let n = self.owed.min(budget);
        self.owed -= n;
        self.tick += n;
        n
    }

    pub fn owed(&self) -> u64 {
        self.owed
    }

    /// Governor only: stop trying to catch up. No tick is skipped or altered: the tick counter simply stays
    /// where it is and the game runs slower than the requested speed relative to the wall clock. Rules never change.
    pub fn forgive_owed(&mut self) {
        self.owed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_stops_ticks_and_resume_has_no_burst() {
        let mut c = Clock::new(100);
        c.advance(1_000_000);
        assert_eq!(c.owed(), 100);
        assert_eq!(c.take(1000), 100);
        c.set_paused(true);
        c.advance(5_000_000);
        assert_eq!(c.owed(), 0);
        assert_eq!(c.take(1000), 0);
        c.set_paused(false);
        c.advance(1_000_000);
        assert_eq!(c.owed(), 100);
    }

    #[test]
    fn frame_splitting_does_not_change_total() {
        for (dt, speed) in [(1_000u64, 123u64), (2_000, 123), (16_000, 123), (10_000_000, 123), (8_000, 10_000), (1_000, 1)] {
            let mut c = Clock::new(speed);
            for _ in 0..(10_000_000 / dt) {
                c.advance(dt);
            }
            assert_eq!(c.owed(), speed * 10, "dt={dt} speed={speed}");
        }
    }

    #[test]
    fn lag_is_kept_until_forgiven() {
        let mut c = Clock::new(10_000);
        c.advance(1_000_000);
        assert_eq!(c.take(100), 100);
        assert_eq!(c.owed(), 9_900);
        c.forgive_owed();
        assert_eq!(c.owed(), 0);
        assert_eq!(c.tick, 100, "forgiving lag must not change the tick counter");
    }

    #[test]
    fn requested_and_effective_interact_correctly() {
        let mut c = Clock::new(1000);
        c.set_effective(400);
        assert_eq!((c.requested(), c.effective()), (1000, 400));
        c.set_requested(2000); // governed: limit stays
        assert_eq!((c.requested(), c.effective()), (2000, 400));
        c.set_requested(100); // request below the limit: follows the request
        assert_eq!((c.requested(), c.effective()), (100, 100));
        c.set_requested(500); // not governed any more: follows fully
        assert_eq!((c.requested(), c.effective()), (500, 500));
        c.set_effective(9999); // never above requested
        assert_eq!(c.effective(), 500);
    }

    #[test]
    fn restore_keeps_tick_and_owes_nothing() {
        let c = Clock::restore(777, 50);
        assert_eq!((c.tick, c.owed(), c.requested(), c.effective()), (777, 0, 50, 50));
    }
}
