use platform_core_proto::clock::Clock;

#[test]
fn pause_stops_ticks_and_resume_has_no_burst() {
    let mut c = Clock::new(100);
    c.advance(1_000_000);
    assert_eq!(c.owed(), 100);
    assert_eq!(c.take(1000), 100);
    assert_eq!(c.tick, 100);
    c.set_paused(true);
    c.advance(5_000_000);
    assert_eq!(c.owed(), 0, "real time must not accumulate while paused");
    assert_eq!(c.take(1000), 0);
    assert_eq!(c.tick, 100);
    c.set_paused(false);
    c.advance(1_000_000);
    assert_eq!(c.owed(), 100, "resume must not catch up the paused time");
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
fn speed_change_is_exact() {
    let mut c = Clock::new(10);
    c.advance(1_000_000);
    c.set_speed(10_000);
    c.advance(1_000_000);
    assert_eq!(c.owed(), 10_010);
}

#[test]
fn lag_is_kept_not_dropped() {
    let mut c = Clock::new(10_000);
    c.advance(1_000_000);
    assert_eq!(c.take(100), 100);
    assert_eq!(c.owed(), 9_900);
}

#[test]
fn remainder_is_never_lost() {
    let mut c = Clock::new(1);
    for _ in 0..1000 {
        c.advance(1_000);
    }
    assert_eq!(c.owed(), 1);
}
