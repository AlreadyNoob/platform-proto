# TESTING (updated 2026-11-17, Milestone 01)

## What has actually been run
| Test | Where | Result |
|---|---|---|
| Event-driven vs brute-force equivalence, Python reference, 6 configs × 25,000 runs + 4 replicate configs | authoring sandbox | Correct method not rejected (α=0.001, min p=0.034, replicates ≥0.164); 4 negative-control configs rejected. `prototype/reference/results/` |
| Scaling benchmark, C reference | authoring sandbox (Xeon 2.1 GHz, 1 core) | table in MILESTONE_01_REPORT.md §2 |

| Rust core tests (5), phone Termux, MT6878, Rust 1.99.1 | creator's phone | **5/5 passed** (equivalence ratios <1, negative controls rejected) |
| Determinism hashes seeds 1-3, phone Rust vs x86 Python port | phone + sandbox | **identical** (`8fe431764a396449`, `05f5d4a4d9e575cf`, `6f401db9752f4ec1`) |
| Rust `bench`, phone: event-driven column | creator's phone | valid; table in MILESTONE_01_REPORT.md §7 |
| Rust `bench`, phone: brute-force columns, first run | creator's phone | INVALID (compiler removed loops, F-5) |
| Rust `bench`, phone, re-run after fix (`bench_out2.txt`) | creator's phone | **valid**, all 3 strategies; table + crossover finding F-6 in MILESTONE_01_REPORT.md §7 |

## Milestone 02a (v0.1.3) — run on the owner's phone, Termux, 2026-11-19
| Test | Result |
|---|---|
| `cargo test --release -p platform_core` (first compile) | **27/27 passed**: unit/clock 5, governor 3, host 3 (1.32 s), rng 4, save 10, wheel 2; cargo exit code 0 |
| Rust vs independent Python references | RNG vectors, governor traces (4 scenarios), save golden bytes: **identical** (`python_ref_m02a.txt`) |
| Python references | wheel vs model 214,126 events OK; governor properties OK |
Raw outputs: `core/results/phone_termux_02a/`. Not yet built/run: `bridge/`, `game/` check scene, `.github/workflows/ci.yml`, Android `.so`, anything on-device in Godot.

## Milestone 03a — run on the owner's phone, Termux, 2026-11-20 (v0.2.5)
| Test | Result |
|---|---|
| `cargo test --release -p platform_core` (first compile of `collapse.rs`) | **36/36 passed** (clock 5, collapse 9, governor 3, host 3 in 1.33 s, rng 4, save 10, wheel 2); cargo exit code 0 |
| Rust vs independent Python references | RNG vectors, governor traces, save golden bytes and the 3 collapse scenarios (61 records, state hashes): **identical** |
| Python reference | collapse self-tests pass; max chain size 3-4 even with inflated probabilities |
Raw outputs: `core/results/phone_termux_03a/`. Not wired to host/bridge/game; no on-device Godot test of collapse yet.

## Milestone 03b — run on the owner's phone, Termux, 2026-11-20 (v0.2.7)
| Test | Result |
|---|---|
| `cargo test --release -p platform_core` (first compile of the economy) | **44/44 passed** (clock 5, collapse 9, economy 8, governor 3, host 3 in 1.33 s, rng 4, save 10, wheel 2); cargo exit code 0 |
| Rust vs independent Python references | all 5 fixtures **identical**: RNG vectors, governor traces, save golden bytes, collapse scenarios (unchanged since 03a), economy scenarios (3 scenarios, replayed line by line) |
| Python references | economy self-tests pass; e1 count 317 / e2 826 particles / e3 24 chains |
Raw outputs: `core/results/phone_termux_03b/`. Not wired to host/bridge/game; world state is not saved (K-48).

## Milestone 03c (v0.2.8) — RAN on the phone 2026-11-20: 52/52, 7 fixtures identical (originally written as below)
`core/tests/save_world.rs` (7 tests: world save bytes equal the independent Python encoder's; golden bytes decode to the exact world; restore + 40 more ticks equals never saving, records included; every bit flip and truncation of a world file is rejected; inconsistent worlds refused; header-only files; slot roundtrip with backup) and updated `core/tests/save.rs` (schema 1 golden file still loads = migration test; schema 2 golden; newer schema refused). Python reference passes (export/restore continues exactly). Run: `sh tools/RUN_M03C_TERMUX.sh` — expect 52 Rust tests and 7 fixtures identical. Note: the GitHub CI run for the previous push did not start (billing, K-50); the phone is the test environment.

## Device test matrix
| Device | Android | Rust core tests | Bench | Godot app |
|---|---|---|---|---|
| TECNO CAMON 40 Pro 5G (MT6878, Android 16, Termux + Godot app) | 16 | PASS 5/5 | valid | **PASS**: extension loaded, hashes match, sim + render numbers in report §8 |
| lower-end device | — | NOT RUN | NOT RUN | NOT RUN |

## Benchmark template (fill from the phone; stored / simulated / visible kept separate)
| Device | Stored | Simulated/tick | Visible | Speed | Sim ticks/s (Rust) | Render avg/p95 ms | Call overhead ns | Memory | Thermal |
|---|---|---|---|---|---|---|---|---|---|

## Android app run (2026-11-17) — Godot 4.7.2 + Rust, owner's phone
All lines in `prototype/reference/results/phone_godot_app_4.7.2/app_output.txt`; interpretation in MILESTONE_01_REPORT.md §8. Summary: loaded OK; 233 ns/call; hashes identical; sim ≈ Termux speed; render 100k at 120 Hz vsync, 300k ≈ 70 fps, 1M ≈ 34 fps.
Not yet tested: combined sim+render, thermal soak, pan/zoom, snapshot transfer cost, memory, other devices.

## Milestone 01b (v0.0.6)
| Test | Status |
|---|---|
| Hybrid/wheel/heap vs brute force, Python reference, 20,000 runs + theory check | **ran** — all match; controls rejected (report §10.2) |
| Rust tests (14), `bench_hybrid`, `--determinism-hazard` on phone (`sh RUN_M01B_TERMUX.sh`) | **ran (v0.0.7)**: 14/14 pass; wheel best; hazard hashes match x86 Python port (report §10.5) |
| Same on GitHub CI (x86_64): log of the `core-desktop` job | NOT RUN |
| App: button 4 (snapshot cost), 5a (1-min soak), 5b (10-min soak), then Copy | NOT RUN |
Acceptance B1-B5: report §10.3 and §11.1 — all addressed (B1, B3, B4, B5 pass; B2 changed the design).
| `bench_hybrid --hi` (hot slots q=0.2, threshold 3%) on phone | **ran (v0.0.8)**: hybrid 1.2-2.6x faster than wheel at 0.4-83% hot; break-even about 5% hazard (report §10.6) |

## Final 01b app runs (2026-11-18, v0.1.0)
Snapshot cost, 1-minute soak, overload and a 10-minute soak on the owner's phone: see report §11 and `prototype/reference/results/phone_godot_app_01b/soak_log_summary.txt`. 10-minute soak: sim 99.8% of 10,000 ticks/s, lag 0, render 119 fps with p95 <= 9.09 ms, RSS stable 498-507 MB, no crash.
CI (x86_64 runner, Rust 1.100.0): 14/14 tests, hazard hashes identical to the phone, Android .so built with godot 0.5.5.

## Milestone 02a check app (v0.1.4) — owner's phone, Godot 4.7.2, 2026-11-19 — VERIFIED ON DEVICE
CI run 44406424568: core job (cargo test + Python fixtures equal) and android-arm64-so job green. App log: `core/results/phone_godot_app_02a/app_output.txt`.
- Extension loads; `platform_core 0.1.2 arch=aarch64 os=android`.
- 1000x, no load: 4996 ticks in 5 s, no governor action, busy 1.0%.
- Pause: tick equal over 1 s while paused; grows ~1015/s after resume.
- Overload (1 ms/tick, request 10000x): effective speed fell to 899 within 2 s, then oscillated 899/1123 (drops 3, raises 2 in 12 s), owed ticks 0-137, no unbounded lag. Load removed: eff 1123 -> 1403 -> 1753 over 15 s (rising, slowly).
- Save/load: two saves ok, backup exists, load ok (tick, speed, seed, 3 streams, label restored).
- Not tested: app lifecycle (background/kill), long-run steady state, thermal behaviour, a real world.

## Milestone 02b shell (v0.1.5) — NOT RUN. Owner checklist: app starts; top bar fits portrait and landscape; speed buttons change "Running at"; Pause/Play works; one-finger pan, two-finger pinch zoom; Menu > Dark theme and Debug overlay persist after restart; Developer checks opens and Close returns; Android Back closes Menu then quits.

## Milestone 02c — VERIFIED ON DEVICE by the owner (2026-11-19, TECNO CM7)
Checklist: CI green (new .so) OK; Menu > Saves shows 4 rows OK; Save slot 1 then Load slot 1 returns to the saved tick OK; Home/other app then return: no tick jump while away (background pause works, no offline progress) OK; Autosave row filled after backgrounding OK; Back closes Saves first OK. Reported in words by the owner; no log file.

## Milestone 03d (v0.2.9/v0.2.10) — core RAN 65/65 on the phone; app run by the owner: saves, placement, collapse work; remaining checklist items below not yet reported
Core: `sh tools/RUN_M03D_TERMUX.sh` (expect 65 tests: the 52 of 03c plus 13 in `core/tests/game.rs`; 7 fixtures identical; the Python references are unchanged because 03d adds no rule). The new tests check: place then undo restores the identical world (free and count payment), unpayable/unknown placements change nothing and are reported, occupied cells are never overwritten or charged twice, undo takes whole strokes, collapsed particles are skipped without refund, the cell feed is row-major and its revision moves only on change, save then restore continues identically, a save without a world becomes a new world, the host runs commands and publishes counters/cells/exports.
Phone checklist (after CI built the new `.so`; copy it and the new game files into the Godot project, export, install):
1. Start: Count 0, Free 20, Particles 0; hint line shows the tap hint.
2. Tap a cell: a ghost (green) shows while the finger is down; on release one particle appears, Free 19, Particles 1.
3. Tap the same cell again: nothing changes (no message). Place 20 particles: Free 0, Count grows by 1 per particle per tick; place more until Count runs out: message "Not enough free placements or count." and nothing is placed.
4. Mode button -> Paint: drag one finger across cells: a continuous line, no gaps, no double placement; the ghost follows the finger; two fingers pan/zoom.
5. Undo after a tap removes that particle and refunds; Undo after a drag removes the whole drag.
6. Let the sim run at 1x-10x for a while: particles collapse sometimes (about 0.1%/tick per lone particle) with a ring animation; at 1k/10k no animation but counters keep working.
7. Saves: Save to slot 1, place/undo some, Load slot 1: particles, count and free placements return exactly; the sim starts paused; slot rows show particles/count/free. Background the app and return: autosave row shows a world summary.
8. Report anything that looks wrong with the debug overlay on (Menu): it shows count, free, particles, view revision, drawn cells and wave effects.

9. (v0.2.10) Paint mode: keep one finger down, swipe over cells until some collapse, swipe back over them without lifting: the collapsed cells must fill again, occupied cells must not be charged twice.

## Milestone 03e (v0.2.12): starter particle with rising cost
Core: `sh tools/RUN_M03E_TERMUX.sh` (expect 71 tests: the 65 of 03d plus 6 in `core/tests/game.rs`; 7 fixtures identical; the Python references are unchanged). It also runs the balance simulation and copies `balance_report_m03e.txt` to Downloads (a report, not a test). New tests: the starter's values, the dev game's start (20 free, 120 count), the 21st placement needs 180 count and a refused one gives the message, the cost rises 60/66/120 with 0/1/10 particles, undoing a count purchase refunds what was paid, a starter world saved and restored continues identically with spawning.
Owner verified earlier on v0.2.9/v0.2.10/v0.2.11: paint fix, undo, ghost colours, collapse ring, high speed; v0.2.11 core 70/70. Autosave: written on background but not loaded on start (K-60).
Phone checklist (needs the new `.so` from CI AND the new `game/shell.gd`):
1. Start: Count 120, Free 20, Particles 0, Cost 60.
2. Place particles: Cost rises with every particle (66, 72, ... 260 at 20 particles).
3. After the 20 free placements the ghost is red until Count reaches the Cost shown; place a 21st when you can pay it; Count drops by the Cost shown.
4. Place one while you cannot afford it: the message "Not enough free placements or count" (first sighting).
5. Undo a count purchase: Count rises by what you paid; Cost goes back down.
6. Play 10 minutes: does growth slow down? Can you get stuck or lose particles faster than you can buy? Tell Claude how it feels.
