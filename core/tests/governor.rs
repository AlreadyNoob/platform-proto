//! Replays the Python reference scenarios (tools/ref/platform_ref.py) against the real Clock + Governor.
use platform_core::clock::Clock;
use platform_core::governor::{GovInput, GovParams, Governor, INTERVAL_US};

const TRACES: &str = include_str!("fixtures/governor_traces.txt");
const UNLIMITED: u64 = 1_000_000_000;

fn scenario(name: &str) -> (u64, Vec<u64>) {
    match name {
        "s1_const_1000_req10000" => (10_000, vec![1000; 400]),
        "s2_unlimited_req10000" => (10_000, vec![UNLIMITED; 100]),
        "s3_step_5000_500_8000_req10000" => {
            let mut v = vec![5000u64; 100];
            v.extend(vec![500u64; 100]);
            v.extend(vec![8000u64; 200]);
            (10_000, v)
        }
        "s4_req100_cap50_then_1000" => {
            let mut v = vec![50u64; 80];
            v.extend(vec![1000u64; 200]);
            (100, v)
        }
        other => panic!("unknown scenario {other}"),
    }
}

fn run(req: u64, caps: &[u64]) -> Vec<u64> {
    let mut c = Clock::new(req);
    let mut g = Governor::new(GovParams::default());
    let mut effs = Vec::new();
    for cap in caps {
        c.advance(INTERVAL_US);
        let done = c.take(cap * INTERVAL_US / 1_000_000);
        let out = g.update(GovInput { requested: c.requested(), effective: c.effective(), owed: c.owed(), ticks_done: done, interval_us: INTERVAL_US });
        effs.push(c.effective());
        if let Some(a) = out {
            c.set_effective(a.new_effective);
            if a.forgive_owed {
                c.forgive_owed();
            }
        }
    }
    effs
}

#[test]
fn matches_python_reference_traces() {
    let mut n = 0;
    for line in TRACES.lines() {
        let mut it = line.split_whitespace();
        let name = it.next().unwrap();
        let want: Vec<u64> = it.next().unwrap().split(',').map(|x| x.parse().unwrap()).collect();
        let (req, caps) = scenario(name);
        let got = run(req, &caps);
        assert_eq!(got.len(), want.len(), "{name}");
        for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
            assert_eq!(g, w, "{name} interval {i}");
        }
        n += 1;
    }
    assert_eq!(n, 4);
}

#[test]
fn does_not_engage_when_device_keeps_up() {
    let (req, caps) = scenario("s2_unlimited_req10000");
    assert!(run(req, &caps).iter().all(|e| *e == 10_000));
}

#[test]
fn never_exceeds_request_and_never_drops_below_one() {
    for name in ["s1_const_1000_req10000", "s3_step_5000_500_8000_req10000", "s4_req100_cap50_then_1000"] {
        let (req, caps) = scenario(name);
        for e in run(req, &caps) {
            assert!(e >= 1 && e <= req, "{name}");
        }
    }
}
