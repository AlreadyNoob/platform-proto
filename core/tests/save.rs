use platform_core::rng::RngStreams;
use platform_core::save::*;
use std::fs;
use std::path::PathBuf;

const GOLDEN: &str = include_str!("fixtures/save_v1_golden.hex");

fn unhex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn golden_data() -> SaveData {
    let seed = 0x0123_4567_89ab_cdefu64;
    SaveData { seed, tick: 42, requested_speed: 100, rng_states: RngStreams::new(seed).states(), label: "slot-one \u{e9}".to_string() }
}

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("platform_save_test_{}_{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn encoder_matches_independent_python_bytes() {
    assert_eq!(encode(&golden_data()), unhex(GOLDEN));
}

#[test]
fn golden_bytes_decode_to_the_expected_data() {
    assert_eq!(decode(&unhex(GOLDEN)).unwrap(), golden_data());
}

#[test]
fn every_single_byte_flip_is_rejected() {
    let good = unhex(GOLDEN);
    for i in 0..good.len() {
        let mut bad = good.clone();
        bad[i] ^= 0x01;
        assert!(decode(&bad).is_err(), "flip at byte {i} was accepted");
    }
}

#[test]
fn every_truncation_is_rejected() {
    let good = unhex(GOLDEN);
    for n in 0..good.len() {
        assert!(decode(&good[..n]).is_err(), "truncation to {n} bytes was accepted");
    }
}

#[test]
fn newer_schema_is_refused_with_a_clear_error() {
    let mut b = unhex(GOLDEN);
    b[4] = 2; // schema 2 (checksum is now wrong too, but the version check comes first)
    assert_eq!(decode(&b), Err(SaveError::TooNew(2)));
}

#[test]
fn bad_magic_is_refused() {
    let mut b = unhex(GOLDEN);
    b[0] = b'X';
    assert_eq!(decode(&b), Err(SaveError::BadMagic));
}

#[test]
fn save_then_load_roundtrip_and_backup_generation() {
    let dir = tmpdir("roundtrip");
    let path = dir.join("slot1.sav");
    let mut d = golden_data();
    save_slot(&path, &d).unwrap();
    assert_eq!(load_slot(&path).unwrap(), (d.clone(), LoadSource::Main));
    d.tick = 43;
    save_slot(&path, &d).unwrap();
    assert!(dir.join("slot1.bak").exists(), "previous generation must be kept");
    assert_eq!(load_slot(&path).unwrap().0.tick, 43);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn corrupt_main_falls_back_to_backup() {
    let dir = tmpdir("fallback");
    let path = dir.join("slot1.sav");
    let mut d = golden_data();
    save_slot(&path, &d).unwrap();
    d.tick = 50;
    save_slot(&path, &d).unwrap();
    fs::write(&path, b"garbage").unwrap();
    let (loaded, src) = load_slot(&path).unwrap();
    assert_eq!(src, LoadSource::Backup);
    assert_eq!(loaded.tick, 42);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn crash_between_renames_is_recovered_from_the_temp_file() {
    let dir = tmpdir("crash");
    let path = dir.join("slot1.sav");
    let mut d = golden_data();
    save_slot(&path, &d).unwrap();
    // simulate: new data fully written to .tmp, old main moved to .bak, crash before the final rename
    d.tick = 99;
    fs::write(dir.join("slot1.tmp"), encode(&d)).unwrap();
    fs::rename(&path, dir.join("slot1.bak")).unwrap();
    let (loaded, src) = load_slot(&path).unwrap();
    assert_eq!((loaded.tick, src), (99, LoadSource::Temp));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn nothing_to_load_is_an_error() {
    let dir = tmpdir("empty");
    assert!(load_slot(&dir.join("nope.sav")).is_err());
    fs::remove_dir_all(&dir).unwrap();
}
