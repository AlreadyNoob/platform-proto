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
