//! Deterministic natural log using only IEEE-754 basic operations (+ - * / and bit casts), so results
//! do not depend on the platform libm. Geometric sampling floors ln(u)/ln(1-q); a 1-ulp libm difference
//! between devices could flip a collapse tick (finding F-3).

const LN2: f64 = 0.693_147_180_559_945_3;

/// ln(x) for finite x > 0.
pub fn ln(x: f64) -> f64 {
    debug_assert!(x > 0.0 && x.is_finite());
    let bits = x.to_bits();
    let mut e = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mut m = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000); // [1,2)
    if m > 1.414_213_562_373_095_1 {
        m *= 0.5;
        e += 1;
    }
    // ln(m) = 2*atanh(z), z = (m-1)/(m+1), |z| <= 0.1716
    let z = (m - 1.0) / (m + 1.0);
    let z2 = z * z;
    let mut term = z;
    let mut sum = 0.0;
    let mut k = 1.0;
    for _ in 0..16 {
        sum += term / k;
        term *= z2;
        k += 2.0;
    }
    (e as f64) * LN2 + 2.0 * sum
}

/// ln(1 - q) for 0 <= q < 1, accurate for tiny q.
pub fn ln_1m(q: f64) -> f64 {
    if q < 1e-3 {
        -(q + q * q / 2.0 + q * q * q / 3.0 + q * q * q * q / 4.0)
    } else {
        ln(1.0 - q)
    }
}

/// x^n for small non-negative integer n, by repeated multiplication (no libm).
pub fn powu(x: f64, n: u32) -> f64 {
    let mut r = 1.0;
    for _ in 0..n {
        r *= x;
    }
    r
}

/// Number of failures before the first success, success probability q per tick. q in (0,1).
pub fn geometric_failures(u_open0: f64, q: f64) -> u64 {
    (ln(u_open0) / ln_1m(q)).floor() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ln_matches_std_closely() {
        for &x in &[1e-300, 1e-9, 0.1, 0.5, 0.999, 1.0, 1.5, 2.0, 10.0, 12345.678, 1e300] {
            let d = (ln(x) - x.ln()).abs();
            assert!(d <= 1e-12 * x.ln().abs().max(1.0), "x={x} diff={d}");
        }
    }
    #[test]
    fn ln_1m_small_q() {
        for &q in &[1e-9, 1e-6, 1e-4, 5e-4, 2e-3, 0.3] {
            let d = (ln_1m(q) - (-q).ln_1p()).abs();
            assert!(d <= 1e-12 * q, "q={q} diff={d}");
        }
    }
}
