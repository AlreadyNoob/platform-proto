# Platform

Android 2D top-down particle-and-universe simulation. Design source of truth: `PLATFORM_GDD.md` (unmodified v0.1; the owner's original concept is its Appendix A).

## Status (v0.2.7, 2026-11-19): Milestones 00, 01, 01b COMPLETE; Milestone 02a core TESTED on the phone
Engine and language are committed (Godot 4.7.x + engine-independent Rust core). On the owner's phone the new `core/` compiled on the first try and passed 27/27 tests, including the checks against the independent Python references (`core/results/phone_termux_02a/`). `bridge/`, `game/` and `ci.yml` are still never built. **There are still no game rules.** Vocabulary: **designed / planned / implemented / tested / verified on device**.

## For a new Claude session
Read `START_HERE.md` first. The project is resumable from two files: this milestone ZIP + the owner's original proposal ZIP (`Platform_Design_and_Claude_Prompt_v0_1.zip`). `PROMPT_TO_PASTE.txt` has the one message to paste.

## Read in this order
1. `START_HERE.md` -> `PROJECT_STATE.json` -> `ENVIRONMENT_AND_REPO.md` -> `PROJECT_LOG.md` -> `SESSION_HANDOFF.md` -> `CLAUDE.md`
2. `docs/CLAUDE_MASTER_PROMPT.md`, `PLATFORM_GDD.md` (the owner's originals), `DECISIONS.md`
3. `MILESTONE_01_REPORT.md` (section 11 = final validated results; sections 1-10 are chronological history), `ARCHITECTURE.md`, `ENGINE_EVALUATION.md`, `PROJECT_UNDERSTANDING.md` (open design questions)
4. `TODO.md`, `KNOWN_ISSUES.md`, `TESTING.md`, `DEVELOPMENT_PROGRESS.md`, `CHANGELOG.md`, `CONTINUITY_PROTOCOL.md`

## Build / run (phone-only workflow)
M02a (not yet run): `sh tools/RUN_M02A_TERMUX.sh` tests the core in Termux; push to GitHub to build `libplatform_bridge.so` (release `m02-android-so`). The block below is the verified Milestone 01 workflow.
See `CLAUDE.md` (commands) and `ENVIRONMENT_AND_REPO.md` (paths, accounts, exact sequences). Short version: run the core tests in Termux with `sh prototype/RUN_M01B_TERMUX.sh`; push to the private GitHub repo to build the Android library (published as release asset `m01-android-so`); copy `prototype/godot_proto/*` and the library into `Documents/platform_m01`; export the APK from the Godot Android editor 4.7.2.

## Layout (v0.1.2)
`core/` (engine-independent Rust: rng, clock, governor, wheel, snapshot/command, save, host + tests + fixtures), `bridge/` (gdext binding), `game/` (Godot project; currently a developer check scene), `tools/` (check_handoff.py, ref/platform_ref.py Python references, RUN_M02A_TERMUX.sh), `.github/workflows/ci.yml`. Older layout: `START_HERE.md` and the handoff files at the root; `docs/` (original proposal files, master prompt); `prototype/` (reference/ Python+C references and all raw results, core_proto/ Rust core + tests, bridge_proto/ gdext bridge, godot_proto/ test app); `tools/check_handoff.py`; `.github/workflows/`.

## Principle
Architecture and engine validation first. Particle mechanics only after the simulation core, clock, determinism and save skeleton are proven and the owner approves. Timeline: months, not days.
