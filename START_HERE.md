# START HERE — Platform project handoff (read this file first, completely)

You are a fresh Claude session continuing the owner's Android game project **Platform**. You have been given (1) this milestone ZIP — the complete, latest project snapshot — and (2) the owner's **original proposal ZIP** (`Platform_Design_and_Claude_Prompt_v0_1.zip`: the game design document, the master prompt and a README). Nothing else exists: you have no memory of earlier sessions. Everything you need is in these files. This document tells you what is true now and how to continue exactly where the last session stopped.

## 0. Do this first (in order)
1. Unzip both files (for example `unzip -q <zip> -d /home/claude/work`). Find this ZIP's top folder `Platform_v*_milestone-*`.
2. Run `python3 <project>/tools/check_handoff.py <project>` — it verifies the package is complete and that the original proposal files are byte-identical to the copies in the project. Report any FAIL to the owner. If the original proposal ZIP is available, also run `sha256sum` on its three files and compare with `docs/ORIGINAL_FILES_SHA256.txt`. If anything differs, the **owner's original proposal ZIP wins** for `PLATFORM_GDD.md` and the master prompt; tell the owner about the difference.
3. Read, in this order: this file → `PROJECT_STATE.json` → `ENVIRONMENT_AND_REPO.md` → `PROJECT_LOG.md` → `SESSION_HANDOFF.md` → `CLAUDE.md` → `docs/CLAUDE_MASTER_PROMPT.md` (the owner's standing workflow instructions, still authoritative) → `PLATFORM_GDD.md` (the design; Appendix A is the owner's original concept) → `DECISIONS.md` → `OWNER_DECISIONS.md` (owner's verbatim answers to GDD questions) → `MILESTONE_01_REPORT.md` §11 (final validated results) → `ARCHITECTURE.md` → `TODO.md` → `KNOWN_ISSUES.md` → `TESTING.md`. Skim the rest as needed.
4. Do **not** redo finished work. Do **not** start particle mechanics (see rules below).
5. Send your first reply following section 9.

## 1. What Platform is (one paragraph)
A 2D top-down Android simulation game: the player places particles on a huge sparse grid; particles produce more particles each tick; connected groups become riskier (collapse chain reactions); Entropy rises with population/time; Convergence/Divergence timeline events follow; research, special particles, grid warping and endings come later. Speed 1x–10,000x, multiple save slots, millions of particles as a *measured ambition*, extremely polished 2D animation. The owner wants it **slow, impressive and maintainable (6–12 months is fine)**, not a rushed prototype. Design source of truth: `PLATFORM_GDD.md` (v0.1, unmodified).

## 2. Where the project stands (state at v0.2.9, 2026-11-20)
- **Milestones 00, 01, 01b** complete. **Milestone 02a is VERIFIED ON DEVICE (02b and 02c too, see below):** core 27/27 Rust tests on the phone (identical to Python references); GitHub CI run 44406424568 green (core job + Android `.so`, release `m02-android-so`); the check APK (`com.example.platformm02`) ran on the TECNO CM7: extension loads, 1000x run, pause, overload governor, save/load with backup (see `TESTING.md`, `core/results/phone_godot_app_02a/`). **No game rules exist; do not start particle mechanics.**
- **Committed technology (owner, 2026-11-19):** Godot 4.7.2 + engine-independent Rust core via gdext 0.5.5 (D-002/D-003).
- **Open technical items:** governor hunts 899<->1123 under constant overload (K-34) and recovers slowly after load removal (K-32); governor numbers still PROVISIONAL (K-27). Not tested: Android lifecycle, thermal, long steady state.
- **Milestone 02 is COMPLETE (v0.2.2):** 02a core, 02b game shell and 02c saves + Android lifecycle are all verified on the owner's phone (TESTING.md). Saves hold tick, requested speed and seed only; no game world exists yet (K-28).
- **Owner design decisions 2026-11-19 (recorded):** Q-3 = default risk doubles per linked particle (base 0.1%, cap 10%), each linked neighbour rolls its own chance at its normal group risk (D-029..D-031); Q-1/Q-6 partly answered (D-027, D-028); Q-7 = mid-range phones of the last 4-5 years, Android 11 minimum, scale to device (D-026); Q-4 = 1 tick is 1 sim-second at 1x (D-024); Q-2 = collapse chains resolve instantly within the same tick, animated waves are presentation only (D-025). Both recorded verbatim in `OWNER_DECISIONS.md`.
- **Collapse rules are fully specified** (D-025, D-029, D-030, D-031; Q-3 answered). **Economy still open:** sub-questions of Q-1 (split rule, where spawns appear, blocked spawns; D-027) and Q-6 (what the starting budget is, replenishment, how a placement is paid; D-028); Q-5 timelines later.
- **Milestone 03a core is TESTED on the owner's phone (v0.2.5: first compile, 36/36 tests, identical to the Python reference); not wired to host/bridge/game:** `core/src/collapse.rs` (collapse rules as a per-tick reference engine, D-032), `tools/ref/collapse_ref.py` (independent Python reference + golden fixture), `core/tests/collapse.rs`. **Immediate step:** the owner pushes v0.2.5 to the repo (CI green); K-45 is answered (D-033); the economy details are per-type settings with placeholders (D-037); free placements come from the start, upgrades and events, D-035/D-036); then plan 03b (wiring + debug placement + wave presentation) and/or the economy.
- **Milestone 03c (save v2) TESTED on the phone: 52/52, fixtures identical. Milestone 03d (v0.2.9) WRITTEN, never compiled or run:** placement UI and wiring of the real game world into host, bridge and game (tap placement default, drag-paint mode, ghost preview, undo per stroke, counters, collapse-wave animation, world saves). The owner chose the hybrid placement (D-041). CI works since the repo became public (K-50 closed). See `SESSION_HANDOFF.md`.
- **Validated by execution:** Milestone 01/01b (report §11) and Milestone 02a above. **Not validated:** real rules, real visuals/touch, thermal behaviour, other devices, lifecycle, audio.

## 3. What the owner has done (so you do not ask again)
Everything is phone-only: Rust in Termux; unzipped milestone ZIPs and ran tests; private GitHub repo `platform-proto` (git working copy `~/platform/Platform_v0.0.4_milestone-01_phone-bench-valid`, M02a pushed as commit f7e2f85); CI runs; Godot Android editor 4.7.2 with export templates; built and ran the M01 and M02a check APKs (folders `Documents/platform_m01`, `Documents/platform_m02`). Paths and commands: `ENVIRONMENT_AND_REPO.md`, `PROJECT_LOG.md`.
- 2026-11-19 (evening): answered GDD Q-4 and Q-2 (see section 2); pushed M02c to the repo and CI was green; copied the new `.so` and 02c game files into `Documents/platform_m02` (02c checklist then passed, including background pause). Warning learned: overlaying an old milestone ZIP onto the repo overwrote newer files once (reverted, commit 214d895); always base a ZIP on the current repo.
- 2026-11-19: "Whatever you think is good? We'll go with that." (commit to Godot + Rust, Milestone 02). Same day: ran core tests (27/27), pushed to GitHub (CI green), built and ran the check APK and pasted the log.

## 4. Rules (non-negotiable; summary of `CLAUDE.md` + master prompt)
- One milestone at a time. Architecture first; **no particle mechanics** until the owner approves the core milestone and answers the GDD questions.
- Never claim anything is implemented/tested/verified unless it was. Use the vocabulary **designed / planned / implemented / tested / verified on device**. No FPS/particle-count claims without a reproducible measurement on named hardware.
- The simulation core has no dependency on the engine/UI/renderer; the UI sends commands and reads snapshots. Per-tick probabilities stay per-tick; randomness is seeded and saved; timeline identity is a stable id.
- Do not invent balancing numbers silently; mark PROVISIONAL in `DECISIONS.md`.
- **The owner reads on a phone**: short replies (answer first), exact copy-paste commands, no desktop assumptions.
- Answer exactly what is asked (when the owner says "just cd to the directory", give only that).
- **Your sandbox probably has no Rust toolchain and no network** (check; it was true in the last session). Rust can only be compiled on the owner's phone or in CI, so any Rust you write is unverified until the owner runs it: keep it simple, add a Python reference port when the logic is non-trivial, and ask for the output.

## 5. Next step: Milestone 02b (no mechanics)
See section 2 and `TODO.md`. Keep replies short; give exact copy-paste commands.

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
