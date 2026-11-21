# PROJECT LOG — chronological record of what happened (append new entries at the bottom, never rewrite history)

## ZIP history (every milestone ZIP is complete; the latest one supersedes all others)
| ZIP | What it was |
|---|---|
| v0.0.1 milestone-00 engine-validation-plan | Planning only: understanding, engine evaluation, architecture, decisions, prototype plan |
| v0.0.2 milestone-01 engine-validation-partial | First prototype code (Python equivalence test, C scaling bench, Rust core, bridge, Godot harness, CI) — written without a compiler |
| v0.0.3 phone-core-results | Rust core compiled and passed 5/5 tests on the owner's phone; hashes matched an x86 Python port; bench dead-code bug found |
| v0.0.4 phone-bench-valid | Valid phone benchmark; crossover finding (event heap vs rolling) |
| v0.0.5 android-app-validated | Godot 4.7.2 app with the Rust extension ran on the phone; render limits measured |
| v0.0.6 milestone-01b hybrid-clock-soak-ready | Hybrid scheduler (Python validated), clock, worker thread, soak harness — Rust unrun |
| v0.0.7 milestone-01b phone-core-results | 14/14 Rust tests on the phone; timing wheel beats heap; hybrid (0.3% threshold) loses |
| v0.0.8 milestone-01b hot-list-measured | High-hazard run: hot list wins above ~5% hazard; overhead fix |
| v0.1.0 milestone-01 engine-validated | Final results: CI x86 identical, snapshot cost, 10-minute soak, overload; Milestones 01 + 01b closed |
| v0.1.1 milestone-01 handoff-ready | This handoff layer (START_HERE, state JSON, log, protocol, checker). No technical change |

## 2026-11-16 — Milestone 00 (planning)
- The owner uploaded `Platform_Design_and_Claude_Prompt_v0_1.zip` (GDD v0.1, master prompt, README) and said: **do not jump into particle mechanics; establish the architecture and validate the engine first; they would rather wait 6-12 months for a genuinely impressive, maintainable simulation than get a rushed prototype.**
- Claude read the three files, researched current sources (Godot 4.7.2 stable 18 Aug 2026; libGDX 1.14.2; gdext supports stable Godot >= 4.0), and delivered v0.0.1: project understanding (GDD ambiguities Q-1..Q-7), `ENGINE_EVALUATION.md` (Godot vs libGDX vs native Android), `ARCHITECTURE.md` (engine-independent core, commands/snapshots, per-tick semantics, event-driven scheduling for the 10,000x target, save design), `DECISIONS.md`, `PROTOTYPE_PLAN.md` (acceptance criteria A1-A8).
- The owner **approved Godot 4.7.2 + an engine-independent simulation core, Rust the leading candidate**, explicitly "to investigate and prototype, not a final commitment". Milestone 01 asks: Godot-to-Rust on Android; test event-driven vs per-tick probabilities (changing probabilities, connected particles); benchmark stored/simulated/visible; separate sim and render performance; document limitations/alternatives; keep the prototype isolated; no full game development.

## 2026-11-17 — Milestone 01
- **Sandbox limits:** Claude's sandbox had no Rust, Godot, Android toolchain or network. So Claude validated the *method* in Python (6 configurations x 25,000 runs, replicates, negative controls) and a C scaling benchmark, and wrote the Rust core, gdext bridge, Godot harness and a CI workflow without compiling them (v0.0.2).
- **Owner on the phone:** installed Rust in Termux, unzipped v0.0.2, removed the bridge crate from the workspace, ran `cargo test` (5/5 passed on first compile), the benchmark and the determinism hashes, and uploaded the outputs. The hashes were reproduced exactly by an independent x86 Python port (v0.0.3). Brute-force benchmark columns printed `inf` (compiler removed the loops, finding F-5); fixed and re-run (v0.0.4). Finding F-6: an event *heap* is slower than per-component rolling when >~0.3% of components collapse per tick.
- **GitHub CI:** the owner created the private repo `platform-proto` and pushed. The workflow built and tested, but `upload-artifact` failed (the account's artifact storage quota was full). Claude changed the workflow to publish the Android `.so` as a **release asset** (`m01-android-so`). The Android library built on the first try (gdext 0.5.5, 3.2 MB). The CI log reproduced the same hashes on x86_64 (Rust 1.100.0).
- **Godot on the phone:** the owner installed "Godot Engine 4" (Play Store, 4.7.2), copied the project to `Documents/platform_m01`, imported it (one unexplained endless startup spinner that resolved itself), downloaded the export templates, exported a non-Gradle arm64 APK, installed it and ran the harness. Results (v0.0.5): extension loaded; call overhead 233 ns; hashes identical; simulation inside the app within ~8% of native Termux speed; rendering 100,000 instances at the 120 Hz vsync, 300,000 about 70 fps, 1,000,000 about 34 fps.
- The owner said **"We'll go with what you think"** about what to do next; Claude chose Milestone 01b (close the gaps that shape the core design).

## 2026-11-17/18 — Milestone 01b
- Claude designed and validated in Python a **hybrid scheduler** (event wheel + per-tick hot list), wrote the Rust (hazard.rs, clock.rs, worker.rs, stats.rs, 14 tests, benchmark) and a harness v2 (v0.0.6). On the phone: **all 14 tests passed on first compile**; hazard hashes for all four strategies matched an independent x86 Python port bit for bit.
- Findings: **timing wheel is 2.4-3.1x faster than a binary heap** (about 6-7.5 million toy collapse events/s per core); the hybrid with threshold 0.3% was slower than the wheel (F-8); with hot slots at q = 0.2 the hybrid is 1.2-2.6x faster; cost model gives a **break-even of about 5% hazard per tick** (rolling a slot ~5.4 ns; hot collapse ~26 ns; wheel collapse ~136 ns). D-012 revised twice (v0.0.7, v0.0.8).
- **GitHub CI run 2026-11-18:** all steps green; x86 hashes identical to the phone; Android `.so` built with the new worker-thread bridge.
- **App run:** the first harness layout pushed its own buttons off-screen (long log lines widened the layout) — fixed in harness v3 (buttons first, wrapped log, clipboard auto-copy). Results (v0.1.0): snapshot 0.51-0.65 ms per 100,000 points; **10-minute soak at 10,000x with ~1M simulated particles + 100,000 drawn instances: simulation 99.8% of target with zero lag, render ~119 fps (p95 <= 9.09 ms), memory stable 498-507 MB, no crash**; overload test (33% hot slots): simulation reached 27% of the requested speed with lag growing without bound while rendering stayed at 119 fps (-> D-019 speed governor proposed).
- Claude closed Milestones 01 and 01b (v0.1.0) and recommended committing to Godot 4.7.x + Rust core and starting Milestone 02. **The owner has not yet answered** (the owner then asked for this handoff-ready package, v0.1.1).

## 2026-11-19 — Milestone 02a (core skeleton), session 3
- The owner dropped v0.1.1 + the original ZIP; check_handoff ALL PASS, original hashes identical. First reply followed START_HERE §9. The owner answered "Whatever you think is good? We'll go with that." -> engine/language committed, Milestone 02 started.
- Sandbox again had no Rust or network. Strategy: write simple Rust, and for every non-trivial piece write an independent Python reference first and generate golden fixtures from it (`tools/ref/platform_ref.py`): RNG vectors (also cross-checked against the validated prototype Python RNG), governor traces over 4 device scenarios, timing wheel vs a model (214,126 events), save container golden bytes.
- Design finding during the Python model: the first governor draft probed above capacity and only noticed after >1 s of lag (about 10 s per cycle, backoff never triggered). Fixed: a probe that makes lag exceed 0.1 s is reverted at once and the probe wait doubles (D-019 in DECISIONS). Backoff max reduced to 80 intervals so recovery after capacity returns takes about 36 s in the model.
- Written, NOT compiled: `core/`, `bridge/`, `game/`, `.github/workflows/ci.yml`, `tools/RUN_M02A_TERMUX.sh`. Only static brace/paren balance checks were possible.
- Result: ZIP v0.1.2; then the owner ran `RUN_M02A_TERMUX.sh` (v0.1.3 records it): the Rust core compiled on the first try and passed 27/27 tests (clock 5, governor 3, host 3, rng 4, save 10, wheel 2); RNG vectors, governor traces and save golden bytes identical to the Python references. Raw files: `core/results/phone_termux_02a/`.

## 2026-11-19 (evening) — Q-4/Q-2 decided; ZIP/repo mix-up
- Claude shipped a v0.1.3 ZIP without knowing the repo was already at M02c (another session had built 02b/02c). The owner overlaid it and pushed (b0f081e, 157 deletions); Claude spotted older CI run titles, the owner cancelled that CI run and `git revert` restored main (214d895); CI rebuilt green (44506926285). Lesson: before shipping an overlay ZIP, ask for / read the repo snapshot first (`git archive`).
- The owner answered GDD Q-4 and Q-2 (recorded as D-024, D-025). 02c checklist result still pending.

## 2026-11-19 (night) — Milestone 02 complete
- Owner ran the 02c checklist: all items passed; step 3 was first misread (the sim is expected to be running again after returning); a 1x tick-jump test settled it. Milestone 02 closed.

## 2026-11-19 (night) — design answers
- Owner answered, via tappable questions asked one at a time: Q-7 device targets, Q-1 (both a count and grid spawns), Q-6 (placing costs count + a separate starting budget), Q-3 (GDD-example curve as default, each neighbour rolls its own chance at its normal group risk). Recorded as D-026..D-031 and verbatim in `OWNER_DECISIONS.md`.
- Observation shared with the owner: with orthogonal adjacency and the 10% cap, chains are small on average (arithmetic in D-031).

## 2026-11-20 — Milestone 03a collapse core
- Owner said "continue" after the design answers; Claude treated it as the go for the fully specified part (collapse rules) and built only that: reference engine + Python reference + golden fixture + tests. No economy, no wiring.
- Python reference result: chains are small even with inflated probabilities (max 3-4 particles), matching the arithmetic in D-031.
- Owner ran it on the phone: 36/36 passed on the first compile; fixtures identical (v0.2.5). (Written without a compiler in the sandbox.) Claude-chosen details are listed in D-032; K-45 (one roll per neighbour per chain) needs the owner's confirmation.

## 2026-11-20 — Milestone 03b economy core
- The owner answered the remaining economy questions (D-034..D-037) and chose the economy core as the next milestone. Claude built it with a Python reference first (placement order, generation, spawning, blocked policy, collapse interplay) and a fixture the Rust must reproduce line by line; the 03a fixture stayed byte-identical. Placeholders are collected in `docs/TUNING_PLACEHOLDERS.md`.
- Owner ran it on the phone: 44/44 passed on the first compile; all 5 fixtures identical (v0.2.7). Written without a compiler in the sandbox. The owner has still not confirmed pushing v0.2.5 or reported `git status`.

## 2026-11-20 — pushed 03a/03b; Milestone 03c save v2
- The owner pushed v0.2.7 (commit 8b93e8d, `git status` showed only docs and new files). GitHub Actions refused to start (billing/spending limit message), so no CI result; the phone remains the test environment.
- 03c: save schema 2 with the whole world, restore that continues exactly, migration 1->2, written with a Python reference first (including an export/restore/continue self-test). Not compiled.

## Mistakes Claude made and fixed (so a new session avoids them)
00. (v0.2.1) Shipped an overlay ZIP built from a stale snapshot; it overwrote newer repo files (reverted). Always base a ZIP on the owner's current repo.
0. (v0.1.2) First governor draft let a failed probe run for ~10 s before reacting; caught by the Python device model, not by hardware. Lesson: model controllers before writing them in Rust.
1. Wrote `mkdir` with brace expansion in a `/bin/sh` script (no expansion) — use explicit paths or bash.
2. Benchmark loops without `black_box` were optimised away (F-5).
3. First GDScript harness: long log lines widened the layout and hid the buttons.
4. A scheduler loop in the first Python reference never advanced (infinite loop) — fixed with an explicit "next unprocessed tick" cursor.
5. Over-confident early statements about hybrid scheduling thresholds (0.3%) — measurement showed ~5%. Lesson: state predictions as hypotheses until measured.
6. Repeatedly gave multi-step command blocks when the owner asked a narrow question. Lesson: answer the literal question.

## Key numbers to remember (all measured; details and conditions in `MILESTONE_01_REPORT.md`)
Determinism hashes (model, seeds 1/2/3): `8fe431764a396449`, `05f5d4a4d9e575cf`, `6f401db9752f4ec1`. Hazard hashes (2000 slots, 20,000 ticks, seed 1, bench(0.9)): brute `c6192bc0d4e6d6e2`, heap `c68b73b07b025628`, wheel `66bfb35d93919c91`, hybrid `101ce1486d4693e9`. Call overhead 233-255 ns. Snapshot 0.51-0.65 ms per 100k points. Soak: 5,691,106 ticks, 1.36 billion events in 10 minutes.

## 2026-11-19 — Milestone 02a verification (v0.1.4), session 4
- Owner dropped v0.1.3 + original ZIP; check_handoff ALL PASS; hashes identical. Owner approved the next step, overlaid the project into the repo and pushed (f7e2f85). CI run 44406424568: both jobs green on first run (bridge/gdext 0.5.5, ci.yml, release `m02-android-so`).
- Owner built the check APK and ran it. Results in `TESTING.md`; raw log in `core/results/phone_godot_app_02a/`. Findings K-32/K-33/K-34. No code changes in v0.1.4.

## 2026-11-19 — Milestone 02b shell written (v0.1.5), session 4
- Owner said "Continue" (= go for 02b). Claude wrote the shell in GDScript (no Godot in the sandbox, so only static checks) and moved the 02a check scene behind Settings. Decisions: orientation sensor; UI scale from DPI; no placeholder buttons (audio = buses only); touch handled via ScreenTouch/ScreenDrag in `_input`, mouse only on desktop.

## 2026-11-19 — 02b first device run (v0.1.5) and display fix (v0.1.6)
- Owner confirmed the shell works from the installed APK; reported side space, 60 fps, letterbox. Claude wrongly guessed first that the owner used the editor Play button (it was the APK). Debug overlay: window (1080,2436), vp (1280,720). Mistake: edited project.godot on disk while the editor may keep settings in memory; v0.1.6 sets the scale mode in code instead.

## 2026-11-19 — v0.1.6 verified, v0.1.7
- Owner: 120 Hz and full-size window now work. Remaining visual problems: Menu cut off in portrait, grey strips at the insets, app should be landscape (the owner's phone has rotation lock on). Fixed in v0.1.7 (not yet run).

## 2026-11-19 — v0.1.7 verified, v0.1.8
- Owner: landscape works; space on the right remained (inset margin, my earlier fix only recoloured it); Back from Developer checks also quit the app (SceneTree quit_on_go_back). Fixed in v0.1.8 (not yet run).

## 2026-11-19 — v0.1.8 verified, v0.1.9
- Owner: Back works. Left space remained (91 px inset from the camera cutout) and asked for a portrait/landscape switch in the Menu. v0.1.9 (not yet run).

## 2026-11-19 — 02b closed, 02c written (v0.2.0)
- Owner confirmed v0.1.9 works (left gap and orientation switch). Claude closed 02b and wrote 02c: bridge save_game/load_game/slot_summary, Saves panel, lifecycle pause + autosave. Not run. Governor recovery (K-32) analysed on paper only.

## 2026-11-20 — v0.2.11, Milestone 03e written
- Owner confirmed on the phone: the paint fix works; undo, ghost colours, collapse ring and high speed work; the rejection message was never seen (game too easy); backgrounding autosaves but the app starts fresh (K-60). Owner chose option A (particle design): the starter makes count and has a small spawn chance (answer 1), pays itself back in about 1 minute (answer 2). Claude wrote the starter type (cost 60, 1 count/tick, 1% spawn), the dev game start (20 free + 120 count), 5 tests, a Python balance simulation (finding K-61: still very generous) and docs. Not compiled.

## 2026-11-20 — v0.2.12, rising cost written
- Core 70/70 passed on the phone for v0.2.11. Owner played it: impossible to lose, particles multiply far faster than count runs out. Claude's simulation: lowering spawn does not help, the count loop is the cause. Owner chose the rising-cost lever. Claude wrote it (D-047, divisor 10), the `next_cost` stat, the Cost label, the ghost fix and tests; simulation: spread player about 1,190 particles per hour, packed about 140, nobody extinct. Not compiled.
