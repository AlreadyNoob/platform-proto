//! Simulation clock: pause, speed, exact integer time accounting (no floating point).
//! 1 tick = 1 sim-second at 1x speed (provisional assumption A-1); ticks per real second = speed * BASE_TICKS_PER_SEC.
//! Frame splitting cannot change the number of ticks owed: the accumulator keeps the exact remainder.

pub const BASE_TICKS_PER_SEC: u64 = 1;
pub const MAX_SPEED: u64 = 10_000;

#[derive(Clone, Debug)]
pub struct Clock {
    pub tick: u64,
    speed: u64,
    paused: bool,
    acc: u128, // microseconds * ticks-per-second, remainder below 1_000_000
    owed: u64,
}

impl Clock {
    pub fn new(speed: u64) -> Self {
        Clock { tick: 0, speed: speed.clamp(1, MAX_SPEED), paused: false, acc: 0, owed: 0 }
    }

    pub fn set_speed(&mut self, speed: u64) {
        self.speed = speed.clamp(1, MAX_SPEED);
    }

    pub fn speed(&self) -> u64 {
        self.speed
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Real time passes. While paused nothing accumulates, so resuming never produces a catch-up burst.
    pub fn advance(&mut self, dt_us: u64) {
        if self.paused {
            return;
        }
        self.acc += dt_us as u128 * self.speed as u128 * BASE_TICKS_PER_SEC as u128;
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
}
