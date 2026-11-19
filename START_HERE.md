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

## 2. Where the project stands (state at v0.2.0, 2026-11-19)
- **Milestones 00, 01, 01b** complete. **Milestone 02a is VERIFIED ON DEVICE:** core 27/27 Rust tests on the phone (identical to Python references); GitHub CI run 44406424568 green (core job + Android `.so`, release `m02-android-so`); the check APK (`com.example.platformm02`) ran on the TECNO CM7: extension loads, 1000x run, pause, overload governor, save/load with backup (see `TESTING.md`, `core/results/phone_godot_app_02a/`). **No game rules exist; do not start particle mechanics.**
- **Committed technology (owner, 2026-11-19):** Godot 4.7.2 + engine-independent Rust core via gdext 0.5.5 (D-002/D-003).
- **Open technical items:** governor hunts 899<->1123 under constant overload (K-34) and recovers slowly after load removal (K-32); governor numbers still PROVISIONAL (K-27). Not tested: Android lifecycle, thermal, long steady state.
- **02b shell is WRITTEN, NOT RUN (v0.2.0):** `game/shell.gd`, `world_view.gd`; owner must run it (TESTING.md checklist) and paste the result/errors. K-35.
- **After that:** 02b remaining polish, then (old text) Godot game shell (layout, white/dark themes, touch pan/zoom, settings, audio foundation, debug overlay, smoothed "limited speed" indicator), then 02c save/lifecycle wiring and governor review. Ask the owner to confirm before starting 02b.
- **Still awaiting the owner (before core rules):** GDD Q-4 (tick length at 1x) and Q-2 (collapse propagation); then Q-1, Q-3, Q-5, Q-6, Q-7.
- **Validated by execution:** Milestone 01/01b (report §11) and Milestone 02a above. **Not validated:** real rules, real visuals/touch, thermal behaviour, other devices, lifecycle, audio.

## 3. What the owner has done (so you do not ask again)
Everything is phone-only: Rust in Termux; unzipped milestone ZIPs and ran tests; private GitHub repo `platform-proto` (git working copy `~/platform/Platform_v0.0.4_milestone-01_phone-bench-valid`, M02a pushed as commit f7e2f85); CI runs; Godot Android editor 4.7.2 with export templates; built and ran the M01 and M02a check APKs (folders `Documents/platform_m01`, `Documents/platform_m02`). Paths and commands: `ENVIRONMENT_AND_REPO.md`, `PROJECT_LOG.md`.
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
