//! Hashed timing wheel (scheduler skeleton). Items are scheduled for an absolute tick; `pop_tick` returns everything due at
//! the current tick and advances time by one. Measured faster than a binary heap in Milestone 01b (2.3-3.1x); this is the
//! clean re-implementation. Order of items due on the same tick is deterministic but NOT insertion order: game rules must
//! sort by a stable key themselves. Mirrors tools/ref/platform_ref.py (class Wheel).
pub struct TimingWheel<T> {
    ring: Vec<Vec<(u64, T)>>,
    mask: u64,
    size: u64,
    now: u64,
    overflow: Vec<(u64, T)>,
    len: usize,
}

impl<T> TimingWheel<T> {
    /// `size` is rounded up to a power of two (minimum 2). `start` is the first tick that will be popped.
    pub fn new(size: usize, start: u64) -> Self {
        let mut s = 2usize;
        while s < size {
            s *= 2;
        }
        let mut ring = Vec::with_capacity(s);
        for _ in 0..s {
            ring.push(Vec::new());
        }
        TimingWheel { ring, mask: (s as u64) - 1, size: s as u64, now: start, overflow: Vec::new(), len: 0 }
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Schedule for tick `due`. A tick in the past is treated as "now".
    pub fn schedule(&mut self, due: u64, item: T) {
        let due = due.max(self.now);
        if due < self.now + self.size {
            let idx = (due & self.mask) as usize;
            self.ring[idx].push((due, item));
        } else {
            self.overflow.push((due, item));
        }
        self.len += 1;
    }

    /// Append everything due at the current tick to `out`, then advance to the next tick.
    pub fn pop_tick(&mut self, out: &mut Vec<T>) {
        let idx = (self.now & self.mask) as usize;
        let n = self.ring[idx].len();
        for (_, item) in self.ring[idx].drain(..) {
            out.push(item);
        }
        self.len -= n;
        self.now += 1;
        if self.now & (self.size / 2 - 1) == 0 {
            self.rescan();
        }
    }

    // Moves overflow items that now fit inside the ring window. Called every size/2 ticks, which guarantees an item
    // is moved before its due tick arrives.
    fn rescan(&mut self) {
        let limit = self.now + self.size;
        let mut i = 0;
        while i < self.overflow.len() {
            if self.overflow[i].0 < limit {
                let (d, item) = self.overflow.swap_remove(i);
                let idx = (d & self.mask) as usize;
                self.ring[idx].push((d, item));
            } else {
                i += 1;
            }
        }
    }
}
