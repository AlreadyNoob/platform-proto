//! Thread-level checks. Timing based, so the limits are generous; they exist to catch hangs and gross errors, not to benchmark.
use platform_core::command::Command;
use platform_core::host::SimHost;
use platform_core::world::{BurnWorld, NullWorld};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn wait_until<F: Fn() -> bool>(secs: u64, f: F) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        sleep(Duration::from_millis(20));
    }
    f()
}

#[test]
fn runs_pauses_and_resumes_without_a_burst() {
    let mut h = SimHost::start(NullWorld::new(), 0, 1000);
    assert!(wait_until(5, || h.snapshot().tick >= 300), "ticks did not advance");
    h.send(Command::SetPaused(true));
    sleep(Duration::from_millis(200));
    let t0 = h.snapshot().tick;
    sleep(Duration::from_millis(500));
    let s = h.snapshot();
    assert_eq!(s.tick, t0, "ticks advanced while paused");
    assert!(s.paused);
    h.send(Command::SetPaused(false));
    sleep(Duration::from_millis(300));
    let t1 = h.snapshot().tick;
    assert!(t1 - t0 < 1500, "resume produced a catch-up burst: {} ticks", t1 - t0);
    h.stop();
}

#[test]
fn restore_continues_from_the_saved_tick() {
    let mut h = SimHost::start(NullWorld::new(), 5000, 1000);
    assert!(wait_until(5, || h.snapshot().tick > 5000));
    assert!(h.snapshot().tick < 7000);
    h.stop();
}

#[test]
fn governor_lowers_speed_when_the_device_cannot_keep_up() {
    let ns = Arc::new(AtomicU64::new(1_000_000)); // about 1000 ticks/s at most
    let mut h = SimHost::start(BurnWorld::new(ns), 0, 10_000);
    let reacted = wait_until(8, || {
        let s = h.snapshot();
        s.governor_drops >= 1 && s.effective_speed < s.requested_speed
    });
    let s = h.snapshot();
    assert!(reacted, "governor never reacted: {:?}", s);
    assert_eq!(s.requested_speed, 10_000, "the request itself must not change");
    h.stop();
}
