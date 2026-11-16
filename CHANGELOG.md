# CHANGELOG
## Milestone 00 — 2026-11-16 (v0.0.1)
Added project documentation set, engine evaluation, proposed architecture, decision log, prototype plan. No code.

## Milestone 01 — 2026-11-17 (v0.0.2) — PARTIAL
Added `prototype/` (Python equivalence reference, C scaling benchmark, Rust core + tests, gdext bridge, Godot harness), CI workflow, `MILESTONE_01_REPORT.md`. Updated decisions D-002/D-003 (approved for investigation), D-009..D-011, TESTING, KNOWN_ISSUES. Rust, bridge, Godot and CI are uncompiled/unrun.

## v0.0.4 — 2026-11-17 — valid phone benchmark
Bench re-run on phone after fix; all three strategies valid; found crossover F-6 (event-driven slower than per-component rolling above ~0.25-0.45% collapses/tick); D-011 amended, D-012 proposed.

## v0.0.3 — 2026-11-17 — phone results for the Rust core
Rust core compiled and passed 5/5 tests on the creator's phone; hashes reproduced on x86 (detport.py); event-driven bench measured; bench dead-code bug found and fixed; results added to report §7 and TESTING.
