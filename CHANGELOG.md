# CHANGELOG
## Milestone 00 — 2026-11-16 (v0.0.1)
Added project documentation set, engine evaluation, proposed architecture, decision log, prototype plan. No code.

## Milestone 01 — 2026-11-17 (v0.0.2) — PARTIAL
Added `prototype/` (Python equivalence reference, C scaling benchmark, Rust core + tests, gdext bridge, Godot harness), CI workflow, `MILESTONE_01_REPORT.md`. Updated decisions D-002/D-003 (approved for investigation), D-009..D-011, TESTING, KNOWN_ISSUES. Rust, bridge, Godot and CI are uncompiled/unrun.

## v0.0.4 — 2026-11-17 — valid phone benchmark
Bench re-run on phone after fix; all three strategies valid; found crossover F-6 (event-driven slower than per-component rolling above ~0.25-0.45% collapses/tick); D-011 amended, D-012 proposed.

## v0.0.3 — 2026-11-17 — phone results for the Rust core
Rust core compiled and passed 5/5 tests on the creator's phone; hashes reproduced on x86 (detport.py); event-driven bench measured; bench dead-code bug found and fixed; results added to report §7 and TESTING.

## v0.0.5 — 2026-11-17 — Android app validated
Godot 4.7.2 + Rust (gdext 0.5.5) APK exported on the phone and run: extension loaded, 233 ns/call, hashes identical across 4 environments, sim speed unchanged, render limits measured. Workflow publishes the .so as a release asset; gdext pinned; patches moved to prototype/patches/. Report §8-9, decisions D-013..D-015, acceptance table in PROTOTYPE_PLAN.

## v0.0.6 — 2026-11-17 — Milestone 01b prepared
Added hybrid scheduler (Python validated, Rust written), clock, worker thread, snapshot/soak harness v2, Termux runner, CI steps, report section 10, decisions D-012 update and D-016/D-017. Rust and app additions unrun.

## v0.0.7 — 2026-11-17 — Milestone 01b core results
Phone results recorded: 14/14 tests, wheel 2.4-3.1x faster than heap, hybrid slower than wheel (D-012 revised, D-018), hazard hashes matched by an independent x86 port (hazport.py). Added `bench_hybrid --hi`, soak now uses the wheel and has an overload button.

## v0.0.8 — 2026-11-17 — hot-list break-even measured
`bench_hybrid --hi` results: hybrid 1.2-2.6x faster than wheel for slots with hazard 0.2/tick; cost model gives break-even about 5%/tick; D-012 revised again (q_star = 0.05); hybrid no-hot overhead fix (behaviour-preserving, hashes unchanged in the Python port).
