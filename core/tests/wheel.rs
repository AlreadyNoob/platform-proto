use platform_core::rng::Rng;
use platform_core::wheel::TimingWheel;
use std::collections::BTreeMap;

fn check(seed: u64, size: usize, ticks: u64, max_ops: u64) -> u64 {
    let mut rng = Rng::new(seed);
    let mut w: TimingWheel<u64> = TimingWheel::new(size, 0);
    let mut model: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut next_id = 0u64;
    let sz = size.next_power_of_two().max(2) as u64; // actual ring size, used to pick interesting horizons
    for t in 0..ticks {
        let ops = rng.below(max_ops + 1);
        for _ in 0..ops {
            let horizons = [0u64, 1, 3, sz - 1, sz, sz + 1, 3 * sz, rng.below(5 * sz)];
            let h = horizons[rng.below(horizons.len() as u64) as usize];
            let due = w.now() + h;
            w.schedule(due, next_id);
            model.entry(due).or_default().push(next_id);
            next_id += 1;
        }
        let mut got = Vec::new();
        w.pop_tick(&mut got);
        got.sort();
        let mut want = model.remove(&t).unwrap_or_default();
        want.sort();
        assert_eq!(got, want, "seed={seed} size={size} tick={t}");
    }
    next_id
}

#[test]
fn matches_a_simple_model_exactly() {
    let mut total = 0;
    for seed in 0..30u64 {
        for size in [2usize, 4, 8, 64] {
            total += check(seed, size, 600, 6);
        }
    }
    assert!(total > 100_000);
}

#[test]
fn past_ticks_run_now_and_len_tracks() {
    let mut w: TimingWheel<u32> = TimingWheel::new(8, 100);
    w.schedule(5, 1); // in the past -> due now (tick 100)
    w.schedule(101, 2);
    assert_eq!(w.len(), 2);
    let mut out = Vec::new();
    w.pop_tick(&mut out);
    assert_eq!(out, vec![1]);
    w.pop_tick(&mut out);
    assert_eq!(out, vec![1, 2]);
    assert!(w.is_empty());
}
