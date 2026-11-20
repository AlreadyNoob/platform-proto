//! Replays tools/ref/economy_ref.py (independent Python reference) against the Rust economy core, plus direct rule checks.
//! ALL economy numbers here are placeholders (D-037).
use platform_core::collapse::*;
use platform_core::world::World;

const FIXTURE: &str = include_str!("fixtures/economy_scenarios.txt");

fn cfg(id: u16, cost: u64, cpt: u64, spawn: u64, blocked: BlockedPolicy, free_first: bool) -> TypeConfig {
    TypeConfig { id, cost, count_per_tick: cpt, spawn_p32: spawn, spawn_type: id, blocked, free_first }
}

type Ops = Vec<(u64, i32, i32, u16)>;

// Scenario definitions: must be identical to tools/ref/economy_ref.py.
fn build(name: &str) -> (ParticleWorld, u64, Ops) {
    match name {
        "e1_costs_and_payment_order" => {
            let mut w = ParticleWorld::new(11, CollapseModel::from_ppm(0, 0));
            w.define_type(cfg(1, 3, 2, 0, BlockedPolicy::Skip, true));
            w.define_type(cfg(4, 3, 1, 0, BlockedPolicy::Skip, false));
            w.define_type(cfg(5, 1000, 0, 0, BlockedPolicy::Skip, true));
            w.set_count(10);
            w.grant_free_placements(2);
            let ops = vec![(0, 0, 0, 1), (0, 1, 0, 1), (0, 2, 0, 1), (0, 0, 0, 1), (0, 3, 0, 9), (1, 4, 0, 4), (2, 5, 0, 5), (3, 6, 0, 4)];
            (w, 40, ops)
        }
        "e2_spawning_and_blocked_policy" => {
            let mut w = ParticleWorld::new(22, CollapseModel::from_ppm(0, 0));
            w.define_type(cfg(2, 0, 1, p32_from_ppm(500_000), BlockedPolicy::Skip, true));
            w.define_type(cfg(3, 0, 0, p32_from_ppm(1_000_000), BlockedPolicy::ConvertToCount(5), true));
            for (x, y) in [(10, 9), (11, 10), (10, 11), (9, 10)] {
                w.place(x, y);
            }
            (w, 40, vec![(0, 0, 0, 2), (0, 10, 10, 3)])
        }
        "e3_collapse_with_economy" => {
            let mut w = ParticleWorld::new(33, CollapseModel::from_ppm(8_000, 150_000));
            w.define_type(cfg(6, 0, 1, p32_from_ppm(100_000), BlockedPolicy::Skip, true));
            let mut ops: Ops = Vec::new();
            for y in 0..3 {
                for x in 0..4 {
                    ops.push((0, x, y, 6));
                }
            }
            (w, 60, ops)
        }
        other => panic!("unknown scenario {other}"),
    }
}

fn run(name: &str) -> Vec<String> {
    let (mut w, ticks, ops) = build(name);
    let mut out: Vec<String> = Vec::new();
    for t in 0..ticks {
        for (ot, x, y, ty) in ops.iter() {
            if *ot == t {
                let res = match w.place_typed(*x, *y, *ty) {
                    Ok(Paid::Free) => "free",
                    Ok(Paid::Count) => "count",
                    Err(PlaceError::Occupied) => "occupied",
                    Err(PlaceError::UnknownType) => "unknown",
                    Err(PlaceError::Insufficient) => "insufficient",
                };
                out.push(format!("op {} {} {} {} {}", t, x, y, ty, res));
            }
        }
        w.step_tick();
        for r in w.take_log() {
            out.push(format!("rec {} {} {} {} {}", r.tick, r.x, r.y, r.chain, r.depth));
        }
        if w.tick() % 10 == 0 {
            out.push(format!("chk {} {} {} {} {:016x}", w.tick(), w.count(), w.free_placements(), w.len(), w.full_hash()));
        }
    }
    out.push(format!(
        "end full={:016x} count={} free={} tick={} cells={} chains={}",
        w.full_hash(),
        w.count(),
        w.free_placements(),
        w.tick(),
        w.len(),
        w.chains_started()
    ));
    out
}

#[test]
fn matches_python_reference_scenarios() {
    let mut blocks: Vec<(String, Vec<String>)> = Vec::new();
    for line in FIXTURE.lines() {
        if let Some(name) = line.strip_prefix("scenario ") {
            blocks.push((name.to_string(), Vec::new()));
        } else if !line.trim().is_empty() {
            blocks.last_mut().expect("line before scenario").1.push(line.to_string());
        }
    }
    assert_eq!(blocks.len(), 3);
    for (name, want) in blocks.iter() {
        let got = run(name);
        assert_eq!(got.len(), want.len(), "{name}: line count");
        for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
            assert_eq!(g, w, "{name}: line {i}");
        }
    }
}

#[test]
fn placement_pays_in_the_configured_order_and_fails_cleanly() {
    let mut w = ParticleWorld::new(1, CollapseModel::from_ppm(0, 0));
    w.define_type(cfg(1, 3, 0, 0, BlockedPolicy::Skip, true));
    w.define_type(cfg(2, 3, 0, 0, BlockedPolicy::Skip, false));
    w.set_count(3);
    w.grant_free_placements(1);
    assert_eq!(w.place_typed(0, 0, 9), Err(PlaceError::UnknownType));
    assert_eq!(w.place_typed(0, 0, 1), Ok(Paid::Free));
    assert_eq!((w.count(), w.free_placements()), (3, 0));
    assert_eq!(w.place_typed(0, 0, 1), Err(PlaceError::Occupied));
    assert_eq!((w.count(), w.free_placements()), (3, 0));
    assert_eq!(w.place_typed(1, 0, 2), Ok(Paid::Count));
    assert_eq!((w.count(), w.free_placements()), (0, 0));
    assert_eq!(w.place_typed(2, 0, 2), Err(PlaceError::Insufficient));
    w.grant_free_placements(2);
    assert_eq!(w.free_placements(), 2);
    assert_eq!(w.place_typed(2, 0, 2), Ok(Paid::Free)); // count is short, so a free placement is used
    assert_eq!(w.free_placements(), 1);
    assert_eq!(w.kind_at(2, 0), Some(2));
}

#[test]
fn generation_adds_count_and_uses_no_genesis_randomness_without_spawns() {
    let mut w = ParticleWorld::new(1, CollapseModel::from_ppm(0, 0));
    w.define_type(cfg(1, 0, 2, 0, BlockedPolicy::Skip, true));
    let before = w.rng_states();
    w.place_typed(0, 0, 1).unwrap();
    w.place_typed(1, 0, 1).unwrap();
    for _ in 0..5 {
        w.step_tick();
    }
    assert_eq!(w.count(), 20);
    assert_eq!(w.len(), 2);
    // stream 2 (genesis) untouched; streams 0 (collapse) was used by the rolls at risk 0
    assert_eq!(w.rng_states()[2], before[2]);
}

#[test]
fn blocked_spawn_converts_and_unknown_spawn_type_is_skipped() {
    let mut b = ParticleWorld::new(1, CollapseModel::from_ppm(0, 0));
    b.define_type(cfg(3, 0, 0, p32_from_ppm(1_000_000), BlockedPolicy::ConvertToCount(5), true));
    for (x, y) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
        b.place(x, y);
    }
    b.place_typed(0, 0, 3).unwrap();
    b.step_tick();
    assert_eq!(b.count(), 5);
    assert_eq!(b.len(), 5);

    let mut u = ParticleWorld::new(1, CollapseModel::from_ppm(0, 0));
    let mut c = cfg(3, 0, 0, p32_from_ppm(1_000_000), BlockedPolicy::Skip, true);
    c.spawn_type = 77;
    u.define_type(c);
    u.place_typed(5, 5, 3).unwrap();
    u.step_tick();
    assert_eq!(u.len(), 1);
}

#[test]
fn collapsed_particles_stop_producing_and_lose_their_type() {
    let mut c = ParticleWorld::new(1, CollapseModel::from_ppm(1_000_000, 1_000_000));
    c.define_type(cfg(1, 0, 7, 0, BlockedPolicy::Skip, true));
    c.place_typed(0, 0, 1).unwrap();
    c.step_tick();
    assert!(c.is_empty());
    assert_eq!(c.kind_at(0, 0), None);
    assert_eq!(c.count(), 0);
}

#[test]
fn spawned_particles_start_generating_next_tick() {
    let mut s = ParticleWorld::new(1, CollapseModel::from_ppm(0, 0));
    s.define_type(cfg(2, 0, 1, p32_from_ppm(1_000_000), BlockedPolicy::Skip, true));
    s.place_typed(0, 0, 2).unwrap();
    s.step_tick();
    assert_eq!(s.len(), 2);
    assert_eq!(s.count(), 1);
    s.step_tick();
    assert_eq!(s.count(), 3);
}

#[test]
fn placeholder_defaults_are_the_documented_ones() {
    let t = TypeConfig::placeholder(5);
    assert_eq!((t.id, t.cost, t.count_per_tick, t.spawn_p32, t.spawn_type, t.blocked, t.free_first), (5, 1, 1, 0, 5, BlockedPolicy::Skip, true));
}

#[test]
fn step_to_still_works_with_the_economy() {
    let (mut a, _, ops) = build("e3_collapse_with_economy");
    for (_, x, y, ty) in ops.iter() {
        a.place_typed(*x, *y, *ty).unwrap();
    }
    let (mut b, _, ops2) = build("e3_collapse_with_economy");
    for (_, x, y, ty) in ops2.iter() {
        b.place_typed(*x, *y, *ty).unwrap();
    }
    a.step_to(30);
    for _ in 0..30 {
        b.step_tick();
    }
    assert_eq!(a.full_hash(), b.full_hash());
}
