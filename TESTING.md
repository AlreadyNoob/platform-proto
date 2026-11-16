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

## Written but NOT run
Bridge crate (gdext), Godot harness, CI workflow.
How to run: `prototype/README.md`.

## Device test matrix
| Device | Android | Rust core tests | Bench | Godot app |
|---|---|---|---|---|
| TECNO CAMON 40 Pro 5G (MT6878, Android 16, Termux) | 16 | PASS 5/5 | PASS; bench valid after fix | NOT RUN |
| lower-end device | — | NOT RUN | NOT RUN | NOT RUN |

## Benchmark template (fill from the phone; stored / simulated / visible kept separate)
| Device | Stored | Simulated/tick | Visible | Speed | Sim ticks/s (Rust) | Render avg/p95 ms | Call overhead ns | Memory | Thermal |
|---|---|---|---|---|---|---|---|---|---|
