# CHANGELOG
## v0.1.3 — 2026-11-19 — Milestone 02a core tested on the phone
Owner ran `tools/RUN_M02A_TERMUX.sh`: first compile succeeded; 27/27 tests passed (clock 5, governor 3, host 3, rng 4, save 10, wheel 2); Python references pass and all three fixtures are identical. Raw outputs in `core/results/phone_termux_02a/`. No code changes. Docs updated (K-26 narrowed, K-29 first pass).

## v0.1.2 — 2026-11-19 — Milestone 02a: core skeleton (WRITTEN, NOT COMPILED)
Owner committed to Godot 4.7.x + Rust core and said go for Milestone 02. Added `core/` (rng with named streams, clock with requested/effective speed, speed governor D-019, timing wheel, snapshot/command interface, save container v1 with atomic write and backup, simulation host on a worker thread, test worlds), `bridge/` (gdext class `PlatformCore`), `game/` (developer check scene, not the game shell), `tools/ref/platform_ref.py` (independent Python references + golden fixtures), `tools/RUN_M02A_TERMUX.sh`, CI `ci.yml`. Prototype workflow now manual only. Python references pass; Rust is uncompiled.

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

## v0.1.0 — 2026-11-18 — Milestones 01 and 01b complete
Final app results: snapshot 0.5-0.65 ms per 100k points; 10-minute combined soak at 10,000x with 100k drawn instances passed (sim 99.8%, lag 0, render 119 fps, RSS stable); overload test (lag grows without bound -> D-019 proposed); CI x86 results (14/14, identical hashes); harness v3 (buttons first, wrapped log, automatic clipboard); CLAUDE.md build commands filled in; D-002/D-003 validated.

## v0.1.1 — 2026-11-18 — handoff-ready package
No technical change. Added START_HERE.md, PROMPT_TO_PASTE.txt, PROJECT_STATE.json, PROJECT_LOG.md, CONTINUITY_PROTOCOL.md, ENVIRONMENT_AND_REPO.md, tools/check_handoff.py, docs/original/ (byte-identical copies + SHA-256), .gitignore; rewrote README, SESSION_HANDOFF, prototype/README; updated CLAUDE.md with the continuity protocol.
