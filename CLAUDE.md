# CLAUDE.md — persistent instructions for future sessions

**Project:** Platform (Android, 2D top-down particle/universe simulation). Product owner: the creator (may be learning; explain decisions plainly, avoid dumping internals).

## Session start (always)
**Read `START_HERE.md` first** and follow its section 0 and 9 (run `tools/check_handoff.py`, read the listed files in order, send the specified first reply, wait for the owner's go). The master workflow is `docs/CLAUDE_MASTER_PROMPT.md` and the design is `PLATFORM_GDD.md` (both byte-identical to the owner's original proposal ZIP; the original wins on design/workflow, this project wins on current state). Then inspect real project files before editing.
If documents conflict: stop, state the conflict, propose a resolution.

## Rules
- One milestone (or bounded sub-milestone) at a time. Do not start a new feature while a previous milestone is broken.
- Never claim something is implemented/tested/packaged unless it is. Label: designed / planned / implemented / tested / verified on device.
- Simulation core has **no dependency** on the engine, UI or renderer. UI mutates state only through commands; renders only from snapshots.
- Per-tick probabilities stay per-tick. No frame-rate-dependent outcomes. Randomness is seeded and its state is saved.
- Timeline internal identity is a stable id, never a rounded display value.
- Do not invent balancing numbers silently; mark them PROVISIONAL in `DECISIONS.md` and get owner approval for significant ones.
- Do not add placeholder buttons that do nothing. Do not fake state in the renderer.
- Never claim particle counts or FPS without a reproducible benchmark on named hardware.

## Build/test commands (all verified on the owner's phone, Termux, Rust 1.99.1)
- Core tests/benchmarks: `cd prototype && sh RUN_M01B_TERMUX.sh` (builds a core-only workspace in ~/platform/m01b_work).
- Android library: push to GitHub; the workflow `.github/workflows/prototype-m01.yml` builds `libplatform_bridge_proto.so` and publishes it as release `m01-android-so` (artifact storage is full; do not use upload-artifact for it). `gh release download m01-android-so --clobber`.
- App: copy `prototype/godot_proto/*` and the .so (to `bin/android_arm64/`) into `/storage/emulated/0/Documents/platform_m01`, open in the Godot Android editor 4.7.2, export Android (non-Gradle, arm64-v8a, debug), install the APK.
- The owner works phone-only (no PC). Never assume a desktop is available.
- Termux scripts must use /bin/sh-compatible syntax; the sandbox here has no Rust, so Rust is only ever compiled on the owner's phone or in CI.

## Working style for this owner
Phone-only workflow; replies short, answer first; exact copy-paste commands (Termux, `/bin/sh`); answer exactly the question asked; send a checkpoint ZIP after any meaningful step; state predictions as hypotheses until measured; never claim results that were not run.

## Milestone / checkpoint close-out (mandatory — see `CONTINUITY_PROTOCOL.md`)
Update `PROJECT_STATE.json`, `PROJECT_LOG.md` (append), `SESSION_HANDOFF.md` (rewrite), `START_HERE.md` sections 2/3/5, progress, changelog, TODO, known issues, testing, decisions, `ENVIRONMENT_AND_REPO.md` if anything changed; run `python3 tools/check_handoff.py` on the folder and on the ZIP; package a complete ZIP named `Platform_vMAJOR.MINOR.PATCH_milestone-NN_short-description.zip` (never overwrite earlier ZIPs; no `.so`/`.apk`/`target/`); in the final message tell the owner which ZIP to drop next time together with the original proposal ZIP, and give the prompt from `PROMPT_TO_PASTE.txt`.

## Milestone 02 commands (written in v0.1.2, NOT yet verified)
- Core tests on the phone: `sh tools/RUN_M02A_TERMUX.sh` (builds a core-only workspace in ~/platform/m02a_work, runs `cargo test`, the Python references, compares fixtures, copies `test_out_m02a.txt` and `python_ref_m02a.txt` to Download).
- Python references / fixtures: `python3 tools/ref/platform_ref.py` (CI fails if the checked-in fixtures differ from what it generates).
- Android library: workflow `.github/workflows/ci.yml` publishes `libplatform_bridge.so` as release `m02-android-so` (`gh release download m02-android-so --clobber`). Put it in the Godot project at `bin/android_arm64/`.
- Godot project: `game/` (copy to `/storage/emulated/0/Documents/platform_m02` — a NEW folder, keep `platform_m01` untouched).
- 03a core tests on the phone: `sh tools/RUN_M03_TERMUX.sh` (uploads test_out_m03.txt, python_ref_m03.txt). Python references: `python3 tools/ref/platform_ref.py` and `python3 tools/ref/collapse_ref.py`.
- 03b core tests on the phone: `sh tools/RUN_M03B_TERMUX.sh` (uploads test_out_m03b.txt, python_ref_m03b.txt). Python references: `platform_ref.py`, `collapse_ref.py`, `economy_ref.py` in `tools/ref/`.
- 03c core tests on the phone: `sh tools/RUN_M03C_TERMUX.sh` (uploads test_out_m03c.txt, python_ref_m03c.txt). Python references now also include `tools/ref/save2_ref.py`.
