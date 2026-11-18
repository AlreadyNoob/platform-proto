//! Small statistics helpers for the equivalence tests (no dependencies).

/// Chi-square critical value at alpha = 0.001 (Wilson-Hilferty approximation).
pub fn crit_001(dof: usize) -> f64 {
    let k = dof.max(1) as f64;
    let z = 3.0902;
    let a = 2.0 / (9.0 * k);
    k * (1.0 - a + z * a.sqrt()).powi(3)
}

/// 2 x k homogeneity test after merging sparse bins (at least 5 in both rows). Returns (chi2, dof).
pub fn chi2_homogeneity(ha: &[u64], hb: &[u64]) -> (f64, usize) {
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
    if let Some(last) = rows.last_mut() {
        last.0 += ca;
        last.1 += cb;
    }
    if rows.len() < 2 {
        return (0.0, 1);
    }
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

/// Data-driven bin edges: every 5th percentile of the baseline sample (deduplicated).
pub fn percentile_edges(vals: &[i64]) -> Vec<i64> {
    let mut v = vals.to_vec();
    v.sort();
    let n = v.len();
    let mut e = Vec::new();
    for k in 1..20usize {
        let idx = (k * n / 20).min(n - 1);
        e.push(v[idx]);
    }
    e.dedup();
    e
}

pub fn bin_index(v: i64, edges: &[i64]) -> usize {
    edges.iter().take_while(|&&e| e <= v).count()
}

pub fn histogram(vals: &[i64], edges: &[i64]) -> Vec<u64> {
    let mut h = vec![0u64; edges.len() + 1];
    for &v in vals {
        h[bin_index(v, edges)] += 1;
    }
    h
}
