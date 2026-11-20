//! World save/restore (Milestone 03c): golden bytes from tools/ref/save2_ref.py, exact continuation after a restore, rejection of broken files.
use platform_core::collapse::*;
use platform_core::rng::NUM_STREAMS;
use platform_core::save::*;
use std::fs;

const GOLDEN_WORLD: &str = include_str!("fixtures/save_v2_world_golden.hex");

fn unhex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn cfg6() -> TypeConfig {
    TypeConfig { id: 6, cost: 0, count_per_tick: 1, spawn_p32: p32_from_ppm(100_000), spawn_type: 6, blocked: BlockedPolicy::Skip, free_first: true }
}

// Same scenario as e3 in tools/ref/economy_ref.py, run for `ticks` ticks.
fn e3_at(ticks: u64) -> ParticleWorld {
    let mut w = ParticleWorld::new(33, CollapseModel::from_ppm(8_000, 150_000));
    w.define_type(cfg6());
    for t in 0..ticks {
        if t == 0 {
            for y in 0..3 {
                for x in 0..4 {
                    w.place_typed(x, y, 6).unwrap();
                }
            }
        }
        w.step_tick();
        w.take_log();
    }
    w
}

fn header(w: &ParticleWorld) -> SaveData {
    SaveData { seed: 33, tick: w.tick(), requested_speed: 1000, rng_states: w.rng_states(), label: "e3 at tick 30".to_string() }
}

#[test]
fn world_save_matches_independent_python_bytes() {
    let w = e3_at(30);
    let bytes = encode_game(&header(&w), Some(&w.export_state()));
    assert_eq!(bytes, unhex(GOLDEN_WORLD));
}

#[test]
fn world_golden_bytes_decode_to_the_expected_world() {
    let w = e3_at(30);
    let (d, ws) = decode_game(&unhex(GOLDEN_WORLD)).unwrap();
    assert_eq!(d, header(&w));
    assert_eq!(ws, Some(w.export_state()));
    // the header-only decoder still works on a file that has a world
    assert_eq!(decode(&unhex(GOLDEN_WORLD)).unwrap(), header(&w));
}

#[test]
fn restoring_and_continuing_equals_never_saving() {
    let a = e3_at(30);
    let bytes = encode_game(&header(&a), Some(&a.export_state()));
    let (d, ws) = decode_game(&bytes).unwrap();
    let mut restored = ParticleWorld::restore(d.tick, &d.rng_states, &ws.unwrap()).unwrap();
    assert_eq!(restored.full_hash(), a.full_hash());
    let mut straight = e3_at(30);
    let mut rec_a = Vec::new();
    let mut rec_b = Vec::new();
    for _ in 0..40 {
        restored.step_tick();
        rec_a.extend(restored.take_log());
        straight.step_tick();
        rec_b.extend(straight.take_log());
    }
    assert_eq!(restored.full_hash(), straight.full_hash());
    assert_eq!(rec_a, rec_b);
    assert_eq!(restored.export_state(), straight.export_state());
}

#[test]
fn every_bit_flip_and_truncation_of_a_world_file_is_rejected() {
    let good = unhex(GOLDEN_WORLD);
    for i in 0..good.len() {
        let mut bad = good.clone();
        bad[i] ^= 0x01;
        assert!(decode_game(&bad).is_err(), "flip at byte {i} was accepted");
    }
    for n in 0..good.len() {
        assert!(decode_game(&good[..n]).is_err(), "truncation to {n} bytes was accepted");
    }
}

fn sample_state() -> WorldState {
    let w = e3_at(5);
    w.export_state()
}

fn sealed(ws: &WorldState) -> Vec<u8> {
    let d = SaveData { seed: 1, tick: 5, requested_speed: 1, rng_states: ParticleWorld::new(1, CollapseModel::default()).rng_states(), label: String::new() };
    encode_game(&d, Some(ws))
}

#[test]
fn inconsistent_worlds_are_refused_not_guessed() {
    // unsorted / duplicate cells and unsorted types are refused at decode time (the checksum is valid: the encoder wrote them)
    let mut ws = sample_state();
    assert!(ws.cells.len() >= 2);
    ws.cells.swap(0, 1);
    assert!(matches!(decode_game(&sealed(&ws)), Err(SaveError::Malformed(_))));
    let mut ws = sample_state();
    let dup = ws.cells[0];
    ws.cells.insert(1, dup);
    assert!(matches!(decode_game(&sealed(&ws)), Err(SaveError::Malformed(_))));
    let mut ws = sample_state();
    ws.types.push(TypeConfig::placeholder(1)); // id 1 after id 6: not ascending
    assert!(matches!(decode_game(&sealed(&ws)), Err(SaveError::Malformed(_))));
    // restore refuses a particle of an undefined type, a duplicate cell, and a wrong number of RNG streams
    let mut ws = sample_state();
    ws.cells[0].2 = 99;
    let rng = ParticleWorld::new(1, CollapseModel::default()).rng_states();
    assert!(ParticleWorld::restore(5, &rng, &ws).is_err());
    let mut ws = sample_state();
    let dup = ws.cells[0];
    ws.cells.push(dup);
    assert!(ParticleWorld::restore(5, &rng, &ws).is_err());
    assert!(ParticleWorld::restore(5, &rng[..NUM_STREAMS - 1], &sample_state()).is_err());
}

#[test]
fn an_old_header_only_file_restores_as_an_empty_world_by_the_caller() {
    // a schema 1 / header-only file has no world: decode_game says so and the caller decides what to do
    let w = e3_at(1);
    let d = header(&w);
    let (back, ws) = decode_game(&encode(&d)).unwrap();
    assert_eq!(back, d);
    assert_eq!(ws, None);
}

#[test]
fn world_save_slot_roundtrip_with_backup() {
    let dir = std::env::temp_dir().join(format!("platform_world_save_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("slot1.sav");
    let a = e3_at(10);
    save_slot_game(&path, &header(&a), Some(&a.export_state())).unwrap();
    let b = e3_at(20);
    save_slot_game(&path, &header(&b), Some(&b.export_state())).unwrap();
    let (d, ws, src) = load_slot_game(&path).unwrap();
    assert_eq!(src, LoadSource::Main);
    assert_eq!(d.tick, 20);
    assert_eq!(ws, Some(b.export_state()));
    fs::write(&path, b"garbage").unwrap();
    let (d2, ws2, src2) = load_slot_game(&path).unwrap();
    assert_eq!((d2.tick, src2), (10, LoadSource::Backup));
    assert_eq!(ws2, Some(a.export_state()));
    fs::remove_dir_all(&dir).unwrap();
}
