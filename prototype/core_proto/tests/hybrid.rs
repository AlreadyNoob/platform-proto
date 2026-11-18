//! Hybrid scheduler: statistical equivalence with brute force (mirrors reference/hybrid_ref.py) and batching/clock tests (A8).
use platform_core_proto::clock::Clock;
use platform_core_proto::hazard::{HazardParams, HazardSim, Strategy};
use platform_core_proto::rng::Rng;
use platform_core_proto::stats::{chi2_homogeneity, crit_001, histogram, percentile_edges};

const RUNS: usize = 20_000;
const T: u64 = 150;

type Obs = (i64, i64, i64); // total events, slot-0 events, slot-0 first-event tick (-1 none)

fn observe(strat: Strategy, params: HazardParams, seed: u64) -> Obs {
    let mut s = HazardSim::new(strat, 8, params, seed).with_log();
    s.run_until(T);
    let log = s.take_log();
    let total = log.iter().filter(|e| e.0 < T).count() as i64;
    let s0: Vec<u64> = log.iter().filter(|e| e.1 == 0 && e.0 < T).map(|e| e.0).collect();
    let first = s0.iter().copied().min().map(|x| x as i64).unwrap_or(-1);
    (total, s0.len() as i64, first)
}

fn sample(strat: Strategy, params: HazardParams, base_seed: u64) -> Vec<Obs> {
    (0..RUNS).map(|i| observe(strat, params, base_seed + i as u64)).collect()
}

fn field(v: &[Obs], i: usize) -> Vec<i64> {
    v.iter().map(|t| match i { 0 => t.0, 1 => t.1, _ => t.2 }).collect()
}

fn worst_ratio(label: &str, a: &[Obs], b: &[Obs]) -> f64 {
    let mut worst: f64 = 0.0;
    for i in 0..3 {
        let (fa, fb) = (field(a, i), field(b, i));
        let edges = percentile_edges(&fa);
        let (x, dof) = chi2_homogeneity(&histogram(&fa, &edges), &histogram(&fb, &edges));
        let r = x / crit_001(dof);
        println!("{label} stat={i} chi2={x:.1} dof={dof} crit={:.1} ratio={r:.2}", crit_001(dof));
        worst = worst.max(r);
    }
    worst
}

#[test]
fn event_strategies_match_brute_force() {
    // Deterministic seeds, so this either always passes or always fails; chance failure probability is ~1%.
    // If it fails, re-run with other seeds before suspecting the code.
    let base = sample(Strategy::Brute, HazardParams::test(), 1_000_000);
    for (strat, seed) in [(Strategy::Heap, 5_000_000u64), (Strategy::Wheel, 6_000_000), (Strategy::Hybrid, 7_000_000)] {
        let other = sample(strat, HazardParams::test(), seed);
        let r = worst_ratio(strat.name(), &base, &other);
        assert!(r <= 1.0, "{} differs from brute force (worst ratio {r:.2})", strat.name());
    }
}

#[test]
fn negative_controls_are_rejected() {
    let base = sample(Strategy::Brute, HazardParams::test(), 1_000_000);
    let mut p = HazardParams::test();
    p.brute_bias = 1.15;
    let biased = sample(Strategy::Brute, p, 9_000_000);
    assert!(worst_ratio("brute bias 1.15", &base, &biased) > 1.0, "biased process was NOT detected");
    let mut p2 = HazardParams::test();
    p2.bug_no_rescan = true;
    let buggy = sample(Strategy::Wheel, p2, 6_000_000);
    assert!(worst_ratio("wheel no_rescan", &base, &buggy) > 1.0, "wheel overflow bug was NOT detected");
}

fn direct(strat: Strategy, comps: usize, n: u64, seed: u64) -> (u64, u64) {
    let mut s = HazardSim::new(strat, comps, HazardParams::bench(0.9), seed);
    s.run_until(n);
    (s.hash, s.events)
}

/// Drive the simulation through a Clock with odd frame sizes, speeds and pauses; the result must equal one direct run.
fn driven(strat: Strategy, comps: usize, n: u64, seed: u64, pattern: u32) -> (u64, u64) {
    let mut s = HazardSim::new(strat, comps, HazardParams::bench(0.9), seed);
    let mut clock = Clock::new(1);
    let mut rng = Rng::new(99 + pattern as u64);
    let mut steps = 0u64;
    while clock.tick < n {
        let (dt_us, speed, paused) = match pattern {
            0 => (1_000_000u64, 1u64, false),
            1 => (50_000, 10_000, false),
            _ => (rng.next_u64() % 40_000 + 1, rng.next_u64() % 10_000 + 1, rng.next_u64() % 4 == 0),
        };
        clock.set_speed(speed);
        clock.set_paused(paused);
        clock.advance(dt_us);
        let budget = (n - clock.tick).min(77);
        if clock.take(budget) > 0 {
            s.run_until(clock.tick);
        }
        steps += 1;
        assert!(steps < 5_000_000, "clock made no progress");
    }
    (s.hash, s.events)
}

#[test]
fn speed_pause_and_batching_do_not_change_state_a8() {
    for strat in [Strategy::Brute, Strategy::Heap, Strategy::Wheel, Strategy::Hybrid] {
        let want = direct(strat, 500, 5_000, 4242);
        for pattern in 0..3 {
            let got = driven(strat, 500, 5_000, 4242, pattern);
            assert_eq!(got, want, "{} pattern {pattern}", strat.name());
        }
    }
}

#[test]
fn same_seed_same_hash_every_strategy() {
    for strat in [Strategy::Brute, Strategy::Heap, Strategy::Wheel, Strategy::Hybrid] {
        assert_eq!(direct(strat, 300, 2_000, 7), direct(strat, 300, 2_000, 7));
    }
}
