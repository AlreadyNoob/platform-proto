//! Milestone 03d: placement, undo and the presentation feeds of the game world. No new rules: undo is the exact inverse of placement,
//! so the main checks are "place then undo gives back the identical world" and "a restored world continues identically".
use platform_core::collapse::{CollapseModel, ParticleWorld, TypeConfig};
use platform_core::command::Command;
use platform_core::game::*;
use platform_core::host::SimHost;
use platform_core::save::{decode_game, encode_game, SaveData};
use platform_core::snapshot::Snapshot;
use platform_core::world::{NullWorld, World};
use std::time::{Duration, Instant};

fn place(w: &mut GameWorld, x: i32, y: i32, stroke: u32) {
    w.command(Command::Place { x, y, type_id: BASIC_TYPE, stroke });
}

fn stat(w: &GameWorld, i: usize) -> i64 {
    w.stats()[i]
}
const COUNT: usize = 0;
const FREE: usize = 1;
const PARTICLES: usize = 2;
const REJECT_SEQ: usize = 5;
const REJECT_CODE: usize = 6;
const UNDO_DEPTH: usize = 7;
const NEXT_COST: usize = 8;

#[test]
fn snapshot_field_names_match_the_array() {
    assert_eq!(Snapshot::FIELDS.len(), Snapshot::LEN);
    assert_eq!(Snapshot::default().to_array().len(), Snapshot::LEN);
    assert_eq!(Snapshot::FIELDS[9], "count");
    assert_eq!(Snapshot::FIELDS[Snapshot::LEN - 1], "next_cost");
}

#[test]
fn place_then_undo_restores_the_identical_world_free_payment() {
    let mut w = GameWorld::new_game(&GameSetup { seed: 5, start_free: 3, start_count: 0 });
    let before = w.export().unwrap();
    place(&mut w, 2, 3, 1);
    assert_eq!(stat(&w, PARTICLES), 1);
    assert_eq!(stat(&w, FREE), 2);
    w.command(Command::Undo);
    assert_eq!(w.export().unwrap(), before);
    assert_eq!(stat(&w, UNDO_DEPTH), 0);
    assert_eq!(stat(&w, REJECT_SEQ), 0);
}

#[test]
fn place_then_undo_restores_the_identical_world_count_payment() {
    let mut w = GameWorld::new_game(&GameSetup { seed: 5, start_free: 0, start_count: 0 });
    w.inner_mut().add_count(5);
    let before = w.export().unwrap();
    place(&mut w, -4, 7, 1);
    assert_eq!(stat(&w, COUNT), 4); // the placeholder type costs 1 count
    assert_eq!(stat(&w, FREE), 0);
    w.command(Command::Undo);
    assert_eq!(stat(&w, COUNT), 5);
    assert_eq!(w.export().unwrap(), before);
}

#[test]
fn a_placement_that_cannot_be_paid_changes_nothing_and_is_reported() {
    let mut w = GameWorld::new_game(&GameSetup { seed: 5, start_free: 0, start_count: 0 });
    let before = w.export().unwrap();
    place(&mut w, 0, 0, 1);
    assert_eq!(w.export().unwrap(), before);
    assert_eq!(stat(&w, REJECT_SEQ), 1);
    assert_eq!(stat(&w, REJECT_CODE), REJECT_INSUFFICIENT);
    assert_eq!(stat(&w, UNDO_DEPTH), 0);
}

#[test]
fn an_unknown_type_is_rejected() {
    let mut w = GameWorld::new_game(&GameSetup::dev_default(5));
    let before = w.export().unwrap();
    w.command(Command::Place { x: 0, y: 0, type_id: 99, stroke: 1 });
    assert_eq!(w.export().unwrap(), before);
    assert_eq!(stat(&w, REJECT_CODE), REJECT_UNKNOWN_TYPE);
}

#[test]
fn occupied_cells_are_never_overwritten_or_charged_twice() {
    let mut w = GameWorld::new_game(&GameSetup { seed: 5, start_free: 10, start_count: 0 });
    place(&mut w, 1, 1, 1);
    let after_first = w.export().unwrap();
    place(&mut w, 1, 1, 1);
    place(&mut w, 1, 1, 2);
    assert_eq!(w.export().unwrap(), after_first);
    assert_eq!(stat(&w, PARTICLES), 1);
    assert_eq!(stat(&w, FREE), 9);
    assert_eq!(stat(&w, UNDO_DEPTH), 1);
    assert_eq!(stat(&w, REJECT_SEQ), 0); // occupied is ignored quietly
}

#[test]
fn undo_takes_back_one_whole_stroke_at_a_time() {
    let mut w = GameWorld::new_game(&GameSetup { seed: 5, start_free: 10, start_count: 0 });
    let empty = w.export().unwrap();
    place(&mut w, 0, 0, 7);
    place(&mut w, 1, 0, 7);
    place(&mut w, 2, 0, 7);
    place(&mut w, 10, 10, 8);
    assert_eq!(stat(&w, PARTICLES), 4);
    w.command(Command::Undo);
    assert_eq!(stat(&w, PARTICLES), 3); // only stroke 8
    assert!(w.inner().contains(0, 0) && w.inner().contains(2, 0) && !w.inner().contains(10, 10));
    w.command(Command::Undo);
    assert_eq!(stat(&w, PARTICLES), 0); // all of stroke 7
    assert_eq!(w.export().unwrap(), empty);
    w.command(Command::Undo);
    assert_eq!(stat(&w, REJECT_CODE), REJECT_NOTHING_TO_UNDO);
    assert_eq!(stat(&w, REJECT_SEQ), 1);
}

#[test]
fn undo_skips_particles_that_have_collapsed_and_gives_no_refund() {
    // risk 100%: every lone particle collapses in the first tick
    let mut pw = ParticleWorld::new(1, CollapseModel::from_ppm(1_000_000, 1_000_000));
    pw.define_type(TypeConfig::placeholder(BASIC_TYPE));
    pw.grant_free_placements(5);
    let mut w = GameWorld::from_world(pw);
    place(&mut w, 0, 0, 1);
    assert_eq!(stat(&w, FREE), 4);
    w.step_to(1);
    assert_eq!(stat(&w, PARTICLES), 0);
    let mut ev: Vec<i64> = Vec::new();
    w.drain_events(&mut ev);
    assert_eq!(ev, vec![0, 0, 0, 0, 0]); // tick 0, cell (0,0), chain 0, depth 0
    w.command(Command::Undo);
    assert_eq!(stat(&w, FREE), 4);
    assert_eq!(stat(&w, REJECT_CODE), REJECT_NOTHING_TO_UNDO);
}

#[test]
fn view_revision_moves_when_particles_change_and_cells_are_listed_row_major() {
    let mut w = GameWorld::new_game(&GameSetup::dev_default(5));
    let r0 = w.view_rev();
    place(&mut w, 5, 2, 1);
    let r1 = w.view_rev();
    assert!(r1 > r0);
    place(&mut w, 1, 0, 2);
    assert_eq!(w.view_cells(100), vec![1, 0, BASIC_TYPE as i32, 5, 2, BASIC_TYPE as i32]); // y 0 before y 2
    assert_eq!(w.view_cells(1), vec![1, 0, BASIC_TYPE as i32]);
    let r2 = w.view_rev();
    w.command(Command::Undo);
    assert!(w.view_rev() > r2);
    // a command that changes nothing does not touch the revision; one that does, does
    let r3 = w.view_rev();
    place(&mut w, 5, 2, 3); // occupied
    assert_eq!(w.view_rev(), r3);
    place(&mut w, 7, 7, 4);
    assert!(w.view_rev() > r3);
}

#[test]
fn saving_and_restoring_through_the_save_format_continues_identically() {
    let setup = GameSetup { seed: 99, start_free: 30, start_count: 7 };
    let mut a = GameWorld::new_game(&setup);
    for i in 0..12 {
        place(&mut a, i % 4, i / 4, (i % 3) as u32);
    }
    a.step_to(25);
    let e = a.export().unwrap();
    let d = SaveData { seed: setup.seed, tick: e.tick, requested_speed: 1, rng_states: e.rng_states.clone(), label: "t".to_string() };
    let bytes = encode_game(&d, Some(&e.state));
    let (d2, ws2) = decode_game(&bytes).unwrap();
    let mut b = GameWorld::restore(d2.tick, &d2.rng_states, &ws2.unwrap()).unwrap();
    assert_eq!(b.export().unwrap(), e);
    a.step_to(80);
    b.step_to(80);
    assert_eq!(a.export().unwrap(), b.export().unwrap());
}

#[test]
fn a_save_without_a_world_becomes_a_new_world_at_that_tick() {
    let setup = GameSetup::dev_default(3);
    let w = GameWorld::new_game_at(&setup, 5000).unwrap();
    let e = w.export().unwrap();
    assert_eq!(e.tick, 5000);
    assert_eq!(stat(&w, PARTICLES), 0);
    assert_eq!(stat(&w, FREE) as u64, DEV_START_FREE);
}

fn wait_until<F: FnMut() -> bool>(mut f: F) -> bool {
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

#[test]
fn host_runs_commands_publishes_counters_cells_and_exports() {
    let w = GameWorld::new_game(&GameSetup { seed: 1, start_free: 5, start_count: 0 });
    let mut h = SimHost::start(w, 0, 1);
    h.send(Command::SetPaused(true));
    h.send(Command::Place { x: 3, y: 4, type_id: BASIC_TYPE, stroke: 1 });
    h.send(Command::Place { x: 4, y: 4, type_id: BASIC_TYPE, stroke: 1 });
    assert!(wait_until(|| h.snapshot().world[2] == 2), "particles never reached 2");
    assert_eq!(h.snapshot().world[1], 3); // free placements left
    assert!(wait_until(|| {
        let s = h.snapshot();
        let (rev, cells) = h.view();
        s.world[4] as u64 == rev && cells.len() == 6
    }), "view never caught up");
    let (_, cells) = h.view();
    assert_eq!(cells, vec![3, 4, BASIC_TYPE as i32, 4, 4, BASIC_TYPE as i32]);
    let e = h.export_blocking(2000).expect("worker answered").expect("world can be exported");
    assert_eq!(e.state.cells.len(), 2);
    assert_eq!(e.state.free, 3);
    h.send(Command::Undo);
    assert!(wait_until(|| h.snapshot().world[2] == 0), "undo did not run");
    assert_eq!(h.snapshot().world[1], 5);
    h.stop();
}

#[test]
fn host_without_a_saveable_world_answers_export_with_none() {
    let mut h = SimHost::start(NullWorld::new(), 0, 1);
    assert_eq!(h.export_blocking(2000), Some(None));
    h.stop();
}

// ---- Milestone 03e: the starter particle and the dev game's start (all numbers are PLACEHOLDERS) ----

#[test]
fn the_starter_type_has_the_documented_placeholder_values() {
    use platform_core::collapse::{p32_from_ppm, BlockedPolicy};
    let t = TypeConfig::starter(BASIC_TYPE);
    assert_eq!(t.id, BASIC_TYPE);
    assert_eq!(t.cost, 60);
    assert_eq!(t.count_per_tick, 1);
    assert_eq!(t.spawn_p32, p32_from_ppm(10_000));
    assert_eq!(t.spawn_type, BASIC_TYPE);
    assert_eq!(t.blocked, BlockedPolicy::Skip);
    assert!(t.free_first);
}

#[test]
fn the_dev_game_starts_with_20_free_placements_and_120_count() {
    let w = GameWorld::new_dev_game(5);
    assert_eq!(stat(&w, FREE) as u64, DEV_START_FREE);
    assert_eq!(stat(&w, COUNT) as u64, DEV_START_COUNT);
    assert_eq!(stat(&w, PARTICLES), 0);
    assert_eq!((DEV_START_FREE, DEV_START_COUNT), (20, 120));
}

#[test]
fn the_dev_start_budget_is_20_free_placements_and_the_21st_needs_180_count() {
    let mut w = GameWorld::new_dev_game(5);
    for i in 0..20 {
        place(&mut w, i * 2, 0, i as u32 + 1); // spaced cells, no ticks in between
    }
    assert_eq!(stat(&w, PARTICLES), 20);
    assert_eq!(stat(&w, FREE), 0);
    assert_eq!(stat(&w, COUNT), 120);
    assert_eq!(stat(&w, NEXT_COST), 180); // 60 + 60 * 20 / 10
    assert_eq!(stat(&w, REJECT_SEQ), 0);
    place(&mut w, 100, 0, 99);
    assert_eq!(stat(&w, PARTICLES), 20);
    assert_eq!(stat(&w, REJECT_SEQ), 1);
    assert_eq!(stat(&w, REJECT_CODE), REJECT_INSUFFICIENT);
}

#[test]
fn the_cost_rises_with_the_number_of_particles() {
    let mut w = GameWorld::new_dev_game(5);
    assert_eq!(stat(&w, NEXT_COST), 60);
    place(&mut w, 0, 0, 1);
    assert_eq!(stat(&w, NEXT_COST), 66); // 60 + 60 * 1 / 10
    for i in 1..10 {
        place(&mut w, i * 2, 0, i as u32 + 1);
    }
    assert_eq!(stat(&w, PARTICLES), 10);
    assert_eq!(stat(&w, NEXT_COST), 120); // 60 + 60 * 10 / 10
}

#[test]
fn undoing_a_count_purchase_refunds_what_was_paid_at_that_time() {
    let mut w = GameWorld::new_dev_game(5);
    for i in 0..20 {
        place(&mut w, i * 2, 0, i as u32 + 1);
    }
    w.inner_mut().add_count(100); // count is now 220
    place(&mut w, 100, 0, 21); // costs 180
    assert_eq!(stat(&w, PARTICLES), 21);
    assert_eq!(stat(&w, COUNT), 40);
    assert_eq!(stat(&w, NEXT_COST), 186); // 60 + 60 * 21 / 10
    w.command(Command::Undo);
    assert_eq!(stat(&w, COUNT), 220);
    assert_eq!(stat(&w, PARTICLES), 20);
    assert_eq!(stat(&w, NEXT_COST), 180);
}

#[test]
fn a_starter_world_saved_and_restored_continues_identically_with_spawning() {
    let mut a = GameWorld::new_dev_game(21);
    for i in 0..6 {
        place(&mut a, i * 3, 0, i as u32 + 1);
    }
    a.step_to(200);
    let e = a.export().unwrap();
    let mut b = GameWorld::restore(e.tick, &e.rng_states, &e.state).unwrap();
    assert_eq!(b.export().unwrap(), e);
    a.step_to(400);
    b.step_to(400);
    assert_eq!(a.export().unwrap(), b.export().unwrap());
}
