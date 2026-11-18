# Platform

Android 2D top-down particle-and-universe simulation. Design source of truth: `PLATFORM_GDD.md` (unmodified v0.1; original concept in its Appendix A).

## Status (2026-11-17): Milestone 01 complete for feasibility; Milestone 01b core part verified on the phone, app part pending (v0.0.8)
No game exists. Engine/language remain provisional (Godot 4.7.2 + engine-independent core, Rust leading; owner-approved for investigation only). `prototype/` holds isolated validation code. Real results so far: event-driven collapse scheduling statistically matches per-tick rolls (Python reference) and scaling behaviour of the strategy (C reference, x86). **Verified on the owner's phone:** Rust core compiled and tested, Godot 4.7.2 loads it on Android, determinism identical across 4 environments, sim and render limits measured. **Open:** combined sim+render/thermal, worker thread, snapshot cost, hybrid scheduler (report §9). Read `MILESTONE_01_REPORT.md`. Status vocabulary used everywhere: **designed / planned / implemented / tested / verified on device**.

## Read in this order
1. `SESSION_HANDOFF.md` — what to do next
1a. `MILESTONE_01_REPORT.md` — results, limits, what is unverified; `prototype/README.md` — how to run it
2. `PROJECT_UNDERSTANDING.md` — what Platform is, and the open design questions
3. `ENGINE_EVALUATION.md` + `DECISIONS.md` — provisional technology choice and why
4. `ARCHITECTURE.md` — proposed layering and simulation model
5. `PROTOTYPE_PLAN.md` — how the engine gets validated (Milestone 01)
6. `DEVELOPMENT_PROGRESS.md`, `TODO.md`, `TESTING.md`, `KNOWN_ISSUES.md`, `CHANGELOG.md`

## Build / run
Not applicable yet. Prerequisites will be recorded here when Milestone 01 produces a buildable project.

## Principle
Architecture and engine validation first. Particle mechanics only after the simulation core, clock, determinism and save skeleton are proven. Timeline: months, not days.
