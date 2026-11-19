# START HERE — Platform project handoff (read this file first, completely)

You are a fresh Claude session continuing the owner's Android game project **Platform**. You have been given (1) this milestone ZIP — the complete, latest project snapshot — and (2) the owner's **original proposal ZIP** (`Platform_Design_and_Claude_Prompt_v0_1.zip`: the game design document, the master prompt and a README). Nothing else exists: you have no memory of earlier sessions. Everything you need is in these files. This document tells you what is true now and how to continue exactly where the last session stopped.

## 0. Do this first (in order)
1. Unzip both files (for example `unzip -q <zip> -d /home/claude/work`). Find this ZIP's top folder `Platform_v*_milestone-*`.
2. Run `python3 <project>/tools/check_handoff.py <project>` — it verifies the package is complete and that the original proposal files are byte-identical to the copies in the project. Report any FAIL to the owner. If the original proposal ZIP is available, also run `sha256sum` on its three files and compare with `docs/ORIGINAL_FILES_SHA256.txt`. If anything differs, the **owner's original proposal ZIP wins** for `PLATFORM_GDD.md` and the master prompt; tell the owner about the difference.
3. Read, in this order: this file → `PROJECT_STATE.json` → `ENVIRONMENT_AND_REPO.md` → `PROJECT_LOG.md` → `SESSION_HANDOFF.md` → `CLAUDE.md` → `docs/CLAUDE_MASTER_PROMPT.md` (the owner's standing workflow instructions, still authoritative) → `PLATFORM_GDD.md` (the design; Appendix A is the owner's original concept) → `DECISIONS.md` → `MILESTONE_01_REPORT.md` §11 (final validated results) → `ARCHITECTURE.md` → `TODO.md` → `KNOWN_ISSUES.md` → `TESTING.md`. Skim the rest as needed.
4. Do **not** redo finished work. Do **not** start particle mechanics (see rules below).
5. Send your first reply following section 9.

## 1. What Platform is (one paragraph)
A 2D top-down Android simulation game: the player places particles on a huge sparse grid; particles produce more particles each tick; connected groups become riskier (collapse chain reactions); Entropy rises with population/time; Convergence/Divergence timeline events follow; research, special particles, grid warping and endings come later. Speed 1x–10,000x, multiple save slots, millions of particles as a *measured ambition*, extremely polished 2D animation. The owner wants it **slow, impressive and maintainable (6–12 months is fine)**, not a rushed prototype. Design source of truth: `PLATFORM_GDD.md` (v0.1, unmodified).

## 2. Where the project stands (state at v0.1.3, 2026-11-19)
- **Milestones 00, 01, 01b** complete (engine validation, `prototype/`, report in `MILESTONE_01_REPORT.md` §11). **Milestone 02a core is TESTED on the owner's phone: first compile, 27/27 Rust tests, outputs identical to the Python references** (`core/results/phone_termux_02a/`). `bridge/`, `game/` and `ci.yml` have still never been built (the authoring sandbox has no Rust). **No game rules exist; do not start particle mechanics.**
- **Committed technology (owner, 2026-11-19):** Godot 4.7.2 + engine-independent Rust core via gdext 0.5.5 (D-002/D-003). The owner said "Whatever you think is good? We'll go with that." to my recommendation (commit + start Milestone 02).
- **What 02a contains** (see `ARCHITECTURE.md` §9a, `DECISIONS.md` D-019..D-023): `core/` (rng + named streams, clock with requested/effective speed, speed governor, timing wheel, command/snapshot interface, save container v1 with atomic write, simulation host on a worker thread, tests and golden fixtures produced by the independent Python reference `tools/ref/platform_ref.py`), `bridge/` (Godot class `PlatformCore`), `game/` (developer check scene with 5 test buttons, not the game shell), `.github/workflows/ci.yml`, `tools/RUN_M02A_TERMUX.sh`.
- **Immediate next step:** the owner overlays this project into the git working copy and pushes; `ci.yml` builds the core on x86 and the Android `.so` (release `m02-android-so`). Fix whatever the first CI/bridge compile finds (K-26). Then build the check APK from `game/` and run its buttons 1-5 (info, run, pause, overload/governor, save/load), paste results, record them.
- **Then:** 02b Godot game shell (layout, white/dark themes, touch pan/zoom, settings, audio foundation, debug overlay, "limited speed" indicator), 02c save/lifecycle wiring.
- **Still awaiting the owner (later, before core rules):** GDD Q-4 (tick length at 1x) and Q-2 (collapse propagation); then Q-1, Q-3, Q-5, Q-6, Q-7.
- **Validated by execution (Milestone 01/01b only):** see report §11 (event-driven scheduling equals per-tick rolling; bit-identical on arm64/x86_64/Python; 10-minute soak at 10,000x with ~1M simulated particles and 100,000 drawn instances, render ~119 fps). **Not validated:** everything in 02a, real rules, real visuals/touch, thermal behaviour, other devices, Android lifecycle, audio.

## 3. What the owner has done (so you do not ask again)
Everything is phone-only: installed Rust in Termux; unzipped each milestone ZIP and ran the tests/benchmarks; created the private GitHub repo `platform-proto`, pushed the code, ran the GitHub Actions workflow, downloaded the Android library from a release; installed the Godot editor from the Play Store, imported the project from `Documents/platform_m01`, downloaded export templates, exported and installed the APK, ran every test button and pasted the results back. Full list with paths and exact commands: `ENVIRONMENT_AND_REPO.md` and `PROJECT_LOG.md`.

- 2026-11-19: answered "Whatever you think is good? We'll go with that." (= commit to Godot + Rust, go for Milestone 02). 2026-11-19 (later): ran `RUN_M02A_TERMUX.sh`, uploaded results: 27/27 passed.

## 4. Rules (non-negotiable; summary of `CLAUDE.md` + master prompt)
- One milestone at a time. Architecture first; **no particle mechanics** until the owner approves the core milestone and answers the GDD questions.
- Never claim anything is implemented/tested/verified unless it was. Use the vocabulary **designed / planned / implemented / tested / verified on device**. No FPS/particle-count claims without a reproducible measurement on named hardware.
- The simulation core has no dependency on the engine/UI/renderer; the UI sends commands and reads snapshots. Per-tick probabilities stay per-tick; randomness is seeded and saved; timeline identity is a stable id.
- Do not invent balancing numbers silently; mark PROVISIONAL in `DECISIONS.md`.
- **The owner reads on a phone**: short replies (answer first), exact copy-paste commands, no desktop assumptions.
- Answer exactly what is asked (when the owner says "just cd to the directory", give only that).
- **Your sandbox probably has no Rust toolchain and no network** (check; it was true in the last session). Rust can only be compiled on the owner's phone or in CI, so any Rust you write is unverified until the owner runs it: keep it simple, add a Python reference port when the logic is non-trivial, and ask for the output.

## 5. Next step: finish Milestone 02a, then 02b (no mechanics)
See section 2 and `TODO.md`. Do not copy prototype code blindly; do not claim 02a works until the owner's phone output says so. Keep replies short; give exact copy-paste commands.

## 6. Where the code lives
The authoritative code is **inside this ZIP** (`prototype/`, `.github/workflows/`). The owner also keeps a private GitHub repo (`platform-proto`); you cannot read it without network. If the ZIP and repo disagree, the ZIP is canonical. See `ENVIRONMENT_AND_REPO.md`.

## 7. How the two ZIPs relate
The original proposal ZIP is the owner's untouched source: design (`PLATFORM_GDD.md`), your standing instructions (`CLAUDE_MASTER_PROMPT.md`), README. This milestone ZIP contains byte-identical copies plus everything built since. When they conflict about the *design or workflow*, the original wins; when they conflict about *current state*, this milestone ZIP wins (it is newer). If a design decision was changed by the owner later, it is recorded in `DECISIONS.md` — check there.

## 8. How to keep this handoff alive (you must do this at the end of every milestone or interim checkpoint)
Follow `CONTINUITY_PROTOCOL.md`: update the docs and `PROJECT_STATE.json`, append to `PROJECT_LOG.md`, regenerate `SESSION_HANDOFF.md`, run `tools/check_handoff.py`, zip as `Platform_vX.Y.Z_milestone-NN_short-description.zip` (never overwrite older ZIPs), and **tell the owner which ZIP to drop next time together with the original proposal ZIP**, plus the one-line prompt from `PROMPT_TO_PASTE.txt`. The owner should never have to rely on a single conversation.

## 9. Your first reply must contain (and then wait)
1. One short paragraph in your own words: what Platform is and where it stands (milestone, version, what is validated).
2. The result of `check_handoff.py` (PASS/FAIL) and the version of the milestone ZIP you were given.
3. The list of open owner decisions (section 2) and any ambiguity you found in the files.
4. A concrete proposal for the next action (Milestone 02, section 5) in 5 lines or fewer, and a question: "go?".
Do not start work before the owner answers. Do not ask for information that is already in these files.
