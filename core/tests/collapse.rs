//! Replays tools/ref/collapse_ref.py (independent Python reference) against the Rust collapse core, plus direct rule checks.
use platform_core::collapse::*;
use platform_core::world::World;
use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("fixtures/collapse_scenarios.txt");

fn block(w: &mut ParticleWorld, x0: i32, y0: i32, wd: i32, ht: i32) {
    for y in 0..ht {
        for x in 0..wd {
            w.place(x0 + x, y0 + y);
        }
    }
}

// Scenario definitions: must be identical to tools/ref/collapse_ref.py.
fn build(name: &str) -> (ParticleWorld, u64, Vec<(u64, i32, i32)>) {
    match name {
        "s1_block_inflated" => {
            let mut w = ParticleWorld::new(1, CollapseModel::from_ppm(20_000, 300_000));
            block(&mut w, 0, 0, 6, 6);
            for x in 0..5 {
                w.place(10 + x, 10);
            }
            (w, 40, vec![])
        }
        "s2_default_model_small_groups" => {
            let mut w = ParticleWorld::new(7, CollapseModel::default());
            for x in 0..4 {
                w.place(x, 0);
            }
            w.place(10, 10);
            w.place(11, 10);
            w.place(20, 20);
            w.place(30, 30);
            w.place(31, 30);
            w.place(30, 31);
            w.place(31, 31);
            (w, 20_000, vec![])
        }
        "s3_placements_over_time" => {
            let w = ParticleWorld::new(99, CollapseModel::from_ppm(50_000, 500_000));
            let pl = vec![
                (0, 0, 0), (0, 1, 0), (5, 2, 0), (5, 3, 0), (10, 0, 1), (10, 1, 1), (10, 2, 1), (10, 3, 1),
                (20, 5, 5), (20, 5, 6), (20, 6, 5), (20, 6, 6), (30, 0, 0), (30, 1, 0),
            ];
            (w, 60, pl)
        }
        other => panic!("unknown scenario {other}"),
    }
}

fn run(name: &str) -> (ParticleWorld, Vec<CollapseRecord>) {
    let (mut w, ticks, pl) = build(name);
    let mut all = Vec::new();
    for t in 0..ticks {
        for (pt, x, y) in pl.iter() {
            if *pt == t {
                w.place(*x, *y);
            }
        }
        w.step_tick();
        all.extend(w.take_log());
    }
    (w, all)
}

#[test]
fn matches_python_reference_scenarios() {
    let mut cur: Option<String> = None;
    let mut recs: Vec<CollapseRecord> = Vec::new();
    let mut checked = 0;
    for line in FIXTURE.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.is_empty() {
            continue;
        }
        match t[0] {
            "scenario" => {
                cur = Some(t[1].to_string());
                recs.clear();
            }
            "rec" => recs.push(CollapseRecord {
                tick: t[1].parse().unwrap(),
                x: t[2].parse().unwrap(),
                y: t[3].parse().unwrap(),
                chain: t[4].parse().unwrap(),
                depth: t[5].parse().unwrap(),
            }),
            "end" => {
                let name = cur.clone().expect("end before scenario");
                let get = |key: &str| -> String {
                    t.iter().find(|s| s.starts_with(key)).unwrap().trim_start_matches(key).to_string()
                };
                let (w, got) = run(&name);
                assert_eq!(got.len(), recs.len(), "{name}: record count");
                for (i, (g, r)) in got.iter().zip(recs.iter()).enumerate() {
                    assert_eq!(g, r, "{name}: record {i}");
                }
                assert_eq!(format!("{:016x}", w.state_hash()), get("hash="), "{name}: state hash");
                assert_eq!(w.tick().to_string(), get("tick="), "{name}: tick");
                assert_eq!(w.len().to_string(), get("cells="), "{name}: cells left");
                assert_eq!(w.chains_started().to_string(), get("chains="), "{name}: chains");
                assert_eq!(w.dropped_records().to_string(), get("dropped="), "{name}: dropped");
                checked += 1;
            }
            other => panic!("unknown fixture line {other}"),
        }
    }
    assert_eq!(checked, 3);
}

#[test]
fn risk_function_follows_the_default_model() {
    let m = CollapseModel::default();
    assert_eq!(m.risk_p32(1), 4_294_967);
    assert_eq!(m.risk_p32(2), 2 * m.risk_p32(1));
    assert_eq!(m.risk_p32(3), 4 * m.risk_p32(1));
    assert_eq!(m.risk_p32(8), p32_from_ppm(100_000));
    assert_eq!(m.risk_p32(1000), p32_from_ppm(100_000));
    assert_eq!(m.risk_p32(u32::MAX), p32_from_ppm(100_000));
    assert_eq!(CollapseModel::from_ppm(0, 100_000).risk_p32(50), 0);
    assert!(m.risk_p32(1) < P32_ONE);
}

#[test]
fn groups_are_orthogonal_and_share_one_risk() {
    let mut w = ParticleWorld::new(5, CollapseModel::default());
    for x in 0..3 {
        w.place(x, 0);
    }
    w.place(10, 10);
    w.place(20, 20);
    w.place(21, 21); // diagonal: NOT adjacent (D-007)
    let m = CollapseModel::default();
    let r = w.risks();
    assert_eq!(r[&(0, 0)], m.risk_p32(3));
    assert_eq!(r[&(0, 1)], m.risk_p32(3));
    assert_eq!(r[&(0, 2)], m.risk_p32(3));
    assert_eq!(r[&(10, 10)], m.risk_p32(1));
    assert_eq!(r[&(20, 20)], m.risk_p32(1));
    assert_eq!(r[&(21, 21)], m.risk_p32(1));
}

#[test]
fn placing_on_an_occupied_cell_changes_nothing() {
    let mut w = ParticleWorld::new(5, CollapseModel::default());
    assert!(w.place(3, 4));
    assert!(!w.place(3, 4));
    assert_eq!(w.len(), 1);
    assert!(w.contains(3, 4));
}

#[test]
fn an_empty_world_consumes_no_randomness() {
    let mut w = ParticleWorld::new(5, CollapseModel::default());
    let before = w.rng_states();
    for _ in 0..100 {
        w.step_tick();
    }
    assert_eq!(w.rng_states(), before);
    assert_eq!(w.tick(), 100);
}

#[test]
fn same_seed_same_result_and_different_seed_differs() {
    let (a, ra) = run("s1_block_inflated");
    let (b, rb) = run("s1_block_inflated");
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(ra, rb);
    let mut c = ParticleWorld::new(2, CollapseModel::from_ppm(20_000, 300_000));
    block(&mut c, 0, 0, 6, 6);
    for x in 0..5 {
        c.place(10 + x, 10);
    }
    for _ in 0..40 {
        c.step_tick();
    }
    assert_ne!(a.state_hash(), c.state_hash());
}

#[test]
fn chain_log_properties_hold() {
    for name in ["s1_block_inflated", "s2_default_model_small_groups", "s3_placements_over_time"] {
        let (_, recs) = run(name);
        let mut seen: BTreeSet<(u64, i32, i32)> = BTreeSet::new();
        let mut last_depth: BTreeMap<u64, u32> = BTreeMap::new();
        for r in recs.iter() {
            assert!(seen.insert((r.tick, r.x, r.y)), "{name}: a cell collapsed twice in one tick");
            match last_depth.get(&r.chain) {
                Some(d) => assert!(r.depth >= *d, "{name}: breadth-first order violated"),
                None => assert_eq!(r.depth, 0, "{name}: a chain must start at depth 0"),
            }
            last_depth.insert(r.chain, r.depth);
        }
    }
}

#[test]
fn step_to_equals_repeated_step_tick() {
    let (mut a, _, _) = build("s1_block_inflated");
    let (mut b, _, _) = build("s1_block_inflated");
    a.step_to(25);
    for _ in 0..25 {
        b.step_tick();
    }
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.tick(), 25);
}

#[test]
fn a_collapsed_particle_is_removed_and_isolated_ones_collapse_alone() {
    // with a risk of 100% every particle collapses on its first roll, and in a group the chain takes everything it can reach
    let mut w = ParticleWorld::new(1, CollapseModel::from_ppm(1_000_000, 1_000_000));
    w.place(0, 0);
    w.place(5, 5);
    w.step_tick();
    assert!(w.is_empty());
    let recs = w.take_log();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].chain, 0);
    assert_eq!(recs[1].chain, 1);
}
