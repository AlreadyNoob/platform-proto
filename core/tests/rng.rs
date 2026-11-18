use platform_core::rng::{stream_seed, Rng, RngStreams};

const VECTORS: &str = include_str!("fixtures/rng_vectors.txt");

fn hex(s: &str) -> u64 {
    u64::from_str_radix(s, 16).unwrap()
}

#[test]
fn matches_python_reference_vectors() {
    let mut checked = 0;
    for line in VECTORS.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.is_empty() {
            continue;
        }
        match t[0] {
            "rng" => {
                let seed = hex(t[1].trim_start_matches("seed="));
                let mut r = Rng::new(seed);
                for want in &t[2..] {
                    assert_eq!(r.next_u64(), hex(want), "line: {line}");
                }
            }
            "below" => {
                let seed: u64 = t[1].trim_start_matches("seed=").parse().unwrap();
                let n: u64 = t[2].trim_start_matches("n=").parse().unwrap();
                let mut r = Rng::new(seed);
                for want in &t[3..] {
                    assert_eq!(r.below(n), want.parse::<u64>().unwrap(), "line: {line}");
                }
            }
            "stream_seed" => {
                let seed = hex(t[1].trim_start_matches("seed="));
                let id: usize = t[2].trim_start_matches("id=").parse().unwrap();
                assert_eq!(stream_seed(seed, id), hex(t[3]), "line: {line}");
            }
            "stream_first" => {
                let seed = hex(t[1].trim_start_matches("seed="));
                let mut s = RngStreams::new(seed);
                for (i, want) in t[2..].iter().enumerate() {
                    assert_eq!(s.stream(i).next_u64(), hex(want), "stream {i}");
                }
            }
            other => panic!("unknown fixture line kind {other}"),
        }
        checked += 1;
    }
    assert!(checked >= 8, "fixture file looks incomplete");
}

#[test]
fn state_roundtrip_continues_the_same_sequence() {
    let mut a = Rng::new(99);
    for _ in 0..10 {
        a.next_u64();
    }
    let mut b = Rng::from_state(a.state());
    for _ in 0..100 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn streams_are_independent() {
    let mut s = RngStreams::new(5);
    let before = s.states();
    s.stream(0).next_u64();
    s.stream(0).next_u64();
    let after = s.states();
    assert_ne!(before[0], after[0]);
    assert_eq!(before[1], after[1]);
    assert_eq!(before[2], after[2]);
}

#[test]
fn below_stays_in_range_and_hits_all_values() {
    let mut r = Rng::new(3);
    let mut seen = [false; 7];
    for _ in 0..1000 {
        let v = r.below(7);
        assert!(v < 7);
        seen[v as usize] = true;
    }
    assert!(seen.iter().all(|x| *x));
}
