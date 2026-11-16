//! Statistical equivalence: event-driven vs brute force. Run with `cargo test --release`.
//! Mirrors reference/equivalence_test.py. Chi-square critical value at alpha=0.001 via Wilson-Hilferty.
use platform_core_proto::model::{brute, event_driven, Bug, Variant};
use platform_core_proto::rng::Rng;

const RUNS: usize = 20_000;

fn bin_first(x: i64) -> usize {
    if x < 0 { 0 } else if x < 20 { 1 + x as usize } else { 21 + ((x - 20) / 10) as usize }
}

fn crit(dof: usize) -> f64 {
    let k = dof as f64;
    let z = 3.0902; // 0.001 upper tail
    let a = 2.0 / (9.0 * k);
    k * (1.0 - a + z * a.sqrt()).powi(3)
}

/// Returns (chi2, dof) for a 2 x k homogeneity test after merging sparse bins (min count 5 in both rows).
fn chi2(ha: &[u64], hb: &[u64]) -> (f64, usize) {
    let mut rows: Vec<(f64, f64)> = Vec::new();
    let (mut ca, mut cb) = (0f64, 0f64);
    for i in 0..ha.len() {
        ca += ha[i] as f64;
        cb += hb[i] as f64;
        if ca >= 5.0 && cb >= 5.0 {
            rows.push((ca, cb));
            ca = 0.0;
            cb = 0.0;
        }
    }
    if let Some(last) = rows.last_mut() { last.0 += ca; last.1 += cb; }
    let (ta, tb): (f64, f64) = rows.iter().fold((0.0, 0.0), |s, r| (s.0 + r.0, s.1 + r.1));
    let tot = ta + tb;
    let mut x = 0.0;
    for (a, b) in &rows {
        let col = a + b;
        let (ea, eb) = (col * ta / tot, col * tb / tot);
        x += (a - ea).powi(2) / ea + (b - eb).powi(2) / eb;
    }
    (x, rows.len() - 1)
}

fn hist(vals: &[usize], len: usize) -> Vec<u64> {
    let mut h = vec![0u64; len];
    for &v in vals { h[v] += 1; }
    h
}

/// Returns the worst chi2/crit ratio over the 3 statistics (<= 1.0 means not rejected).
fn worst_ratio(variant: Variant, bug: Bug, seed: u64) -> f64 {
    let (mut rb, mut re) = (Rng::new(seed), Rng::new(seed + 7919));
    let (mut fb, mut fe, mut mb, mut me, mut eb, mut ee) = (vec![], vec![], vec![], vec![], vec![], vec![]);
    for _ in 0..RUNS {
        let (f, m, e) = brute(&mut rb, variant);
        fb.push(bin_first(f)); mb.push(m as usize); eb.push(e as usize);
        let (f, m, e) = event_driven(&mut re, variant, bug);
        fe.push(bin_first(f)); me.push(m as usize); ee.push(e as usize);
    }
    let mut worst: f64 = 0.0;
    for (a, b, len) in [(&fb, &fe, 40usize), (&mb, &me, 40), (&eb, &ee, 40)] {
        let (x, dof) = chi2(&hist(a, len), &hist(b, len));
        let r = x / crit(dof.max(1));
        println!("variant={variant:?} bug={bug:?} seed={seed} chi2={x:.1} dof={dof} crit={:.1} ratio={r:.2}", crit(dof.max(1)));
        worst = worst.max(r);
    }
    worst
}

#[test]
fn correct_scheduler_matches_brute_force() {
    for (v, seed) in [(Variant::A, 1001u64), (Variant::B, 2002)] {
        assert!(worst_ratio(v, Bug::None, seed) <= 1.0, "variant {v:?} rejected");
    }
}

#[test]
fn negative_controls_are_rejected() {
    for (v, bug, seed) in [(Variant::A, Bug::StaleP, 3003u64), (Variant::B, Bug::StaleP, 4004),
                           (Variant::A, Bug::ForgetN, 5005), (Variant::B, Bug::ForgetN, 6006)] {
        assert!(worst_ratio(v, bug, seed) > 1.0, "control {bug:?} {v:?} was NOT detected: test has no power");
    }
}

#[test]
fn same_seed_same_hash() {
    assert_eq!(platform_core_proto::model::determinism_hash(7, 50), platform_core_proto::model::determinism_hash(7, 50));
}
