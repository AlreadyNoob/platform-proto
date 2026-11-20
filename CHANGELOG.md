# CHANGELOG
## v0.2.11 — 2026-11-20 — Milestone 03e: the starter particle (WRITTEN, NOT COMPILED; Python balance simulation run)
Owner verified on the phone: paint-mode fix (v0.2.10), undo, ghost colours, collapse ring, high speed. Owner chose to work on particle design (option A): the starter makes some count AND has a small spawn chance; one starter should earn back its cost in about 1 minute at 1x. Core: `TypeConfig::starter` (cost 60, 1 count/tick, 1% spawn/tick, blocked spawn lost, free placements first); `GameWorld::new_dev_game` / `new_dev_game_at` (20 free + 120 count); the bridge starts the dev game with them. `GameSetup`/`new_game` keep the old placeholder type (cost 1, no spawning) because the 65 older tests rely on it. 5 new tests in `core/tests/game.rs` (70 expected). `tools/ref/balance_ref.py` (balance simulation, NOT a test) and `tools/RUN_M03E_TERMUX.sh`. D-046. All numbers are PLACEHOLDERS. Saves from before 03e keep their old type.
## v0.2.10 — 2026-11-20 — 03d paint-mode fix (GDScript only; core unchanged, 65/65 still valid)
Owner ran v0.2.9 on the phone: saves, placement and collapse work. Bug found: in paint mode a drag remembered every cell it had already sent, so a cell whose particle collapsed during the same drag could not be refilled until the finger was lifted. `world_view.gd`: a cell is re-sent once it is shown empty (and 250 ms have passed since it was last sent). The core already ignores occupied cells, so no double placement or charge is possible. Not yet re-tested on the phone.
## v0.2.9 — 2026-11-20 — Milestone 03d: placement UI and world wiring (WRITTEN, NOT COMPILED)
03c verified on the phone (52/52, fixtures identical). Core: `Command::Place/Undo/Export`, `GameWorld` (`core/src/game.rs`: placement via `place_typed`, undo as its exact inverse per stroke, reject counters), `ParticleWorld::unplace/revision/cell_triples`, `World` trait extended (commands, stats, view cells, collapse-record feed, export), `SimHost` publishes cells (change-driven, throttled, capped) and collapse records and answers `export_blocking`; snapshot carries 8 world counters. Bridge: `place`, `undo`, `cells`, `events`; `save_game`/`load_game`/`slot_summary` now save and restore the whole world (nothing is written if the simulation does not answer); `start` runs the real game world unless a burn load is requested. Game: tap placement (default) and drag paint mode, ghost preview, counters, undo, collapse-wave animation (skipped above 100x), Saves panel text. CI builds the Android library on every push again (repository is public). New `core/tests/game.rs` (13 tests expected). Owner decisions D-041; Claude's choices D-042..D-045.
## v0.2.8 — 2026-11-20 — Milestone 03c: save v2 with world state (WRITTEN, NOT COMPILED)
Save schema 2 = v1 header + world block (collapse model, count, free placements, chains, particle types, particles); `ParticleWorld::export_state` / `restore`; migration 1 -> 2; `save_slot_game` / `load_slot_game`. `tools/ref/save2_ref.py` + golden fixtures `save_v2_noworld_golden.hex`, `save_v2_world_golden.hex`; `core/tests/save_world.rs` (7 tests); `core/tests/save.rs` updated; `tools/RUN_M03C_TERMUX.sh`. Core crate 0.2.8. `ci.yml`: Android job now on request only (K-50: GitHub refused to start CI because of billing). D-039, D-040. Not wired to bridge/game.

## v0.2.7 — 2026-11-20 — Milestone 03b core tested on the phone
Owner ran `tools/RUN_M03B_TERMUX.sh`: first compile succeeded, 44/44 Rust tests passed, all 5 fixtures identical to the Python references. Raw outputs in `core/results/phone_termux_03b/`. Docs only.

## v0.2.6 — 2026-11-20 — Milestone 03b: economy core (WRITTEN, NOT COMPILED) + owner answers
New: economy in `core/src/collapse.rs` (count ledger, free placements, per-type `TypeConfig`, `place_typed`, generation phase with spawns, `full_hash`), `tools/ref/economy_ref.py` + fixture `economy_scenarios.txt`, `core/tests/economy.rs` (8 tests), `tools/RUN_M03B_TERMUX.sh`, CI step, `docs/TUNING_PLACEHOLDERS.md`, D-038, K-47..K-49. The 03a collapse fixture is unchanged. Core crate 0.2.6. Owner answers recorded since v0.2.5 (docs): Owner confirmed K-45: one roll per neighbour per chain (D-033). K-45 closed. Q-1 follow-up answered: spawn placement is a per-particle-type property (D-034). Q-6 follow-up answered: the starting budget is a fixed number of free placements (D-035); free placements can be granted by upgrades or events (D-036). The remaining economy details become per-type settings with placeholder values (D-037).

## v0.2.5 — 2026-11-20 — Milestone 03a core tested on the phone
Owner ran `tools/RUN_M03_TERMUX.sh`: first compile succeeded, 36/36 Rust tests passed, collapse scenarios and all other fixtures identical to the Python references. Raw outputs in `core/results/phone_termux_03a/`. K-46 closed. Docs only.

## v0.2.4 — 2026-11-20 — Milestone 03a: collapse core (WRITTEN, NOT COMPILED)
Added `core/src/collapse.rs`: sparse grid, orthogonal groups, collapse model D-029 (fixed-point, configurable), chains D-025/D-030/D-031, chain log for the presentation; a per-tick reference engine, no economy. Added `tools/ref/collapse_ref.py` (independent Python reference, golden fixture `core/tests/fixtures/collapse_scenarios.txt`), `core/tests/collapse.rs` (9 tests), `tools/RUN_M03_TERMUX.sh`, CI step. Core crate version 0.2.4. Python reference passes; Rust uncompiled. New: D-032 (Claude's implementation details), K-44..K-46. Not wired to host/bridge/game.

## v0.2.3 — 2026-11-19 — Owner design answers recorded (docs only)
Owner answered GDD Q-7: D-026 device targets (mid-range phones of the last 4-5 years, Android 11 minimum, scale capacity/quality to the device). Q-1 partly answered (D-027: generation yields both a count and grid particles). Q-6 partly answered (D-028: placing costs count; a separate starting budget also exists). Q-3 first part answered (D-029: default risk doubles per linked particle, base 0.1%, cap 10%). Q-3 second part answered (D-030: each linked neighbour rolls its own chance in a chain). Q-3 third part answered (D-031: it rolls its normal group risk). The collapse rules are now fully specified and can be built without the economy. New known issues K-41 (single test phone), K-42 (Android 11 / renderer not checked), K-43 (save portability across device capacity).

## v0.2.2 — 2026-11-19 — Milestone 02 complete
Owner verified 02c on the phone: new `.so` from CI, Saves panel with 4 rows, save/load slot 1 returns to the saved tick, Home and return shows no tick jump (background pause, no offline progress), Autosave filled after backgrounding, Back closes Saves first. K-38 closed. Docs only; no code changes.

## v0.2.1 — 2026-11-19 — Owner decisions Q-4 and Q-2 recorded (docs + one comment)
Q-4: 1 tick = 1 sim-second at 1x (D-024, A-1 confirmed). Q-2: collapse chains resolve completely within the same tick; animated waves are presentation only and never decide outcomes (D-025; consequences: deterministic chain order, ordered chain log for the presentation, atomic-tick cost K-39, snapshot chain log K-40). Verbatim answers recorded in `OWNER_DECISIONS.md` (the GDD itself stays unchanged). Only code change: a comment in `core/src/clock.rs`. Based on repo state v0.2.0 (commit 63ee843).

## v0.2.0 — 2026-11-19 — 02b verified; 02c saves + lifecycle WRITTEN, NOT RUN
02b closed: owner verified v0.1.9 on the phone (see PROJECT_STATE). 02c: `bridge/` gets `save_game`, `load_game` (restarts the host from the saved tick and requested speed, starts PAUSED) and `slot_summary` (Rust, not yet compiled: CI builds it; a new `.so` is needed). `game/shell.gd`: Menu > Saves (Autosave read-only + Slot 1-3 with Save/Load), autosave when the app goes to the background and on quit, background pauses the simulation and resume continues it unless the player had paused (no offline progress, D-023). Saved state is tick, requested speed and seed only; there is no game world yet (K-28).

## v0.1.9 — 2026-11-19 — 02b: cutout-aware bar, orientation switch
Owner verified v0.1.8: Back works (no exit). Overlay showed `safe insets l 91 t 0 r 0 b 0`: the left inset is the camera hole in landscape, which does not touch the top bar, yet the bar was inset by it (the "left space"). Now the bar is inset only on sides where a cutout from `DisplayServer.get_display_cutouts()` overlaps the bar's band; the grid stays edge to edge; the debug label keeps the full inset. New Menu toggle "Portrait mode (off = landscape)" (sensor portrait/landscape, saved in settings; default landscape).

## v0.1.8 — 2026-11-19 — 02b edge-to-edge + Back fix
Owner verified v0.1.7 (landscape, bar fits, no grey). Remaining: (1) empty space on the right: it was the safe-area inset (margin) around the whole UI; now the grid and bar background run edge to edge and only the controls are inset (overlay shows `safe insets l t r b`). (2) One Back press closed Developer checks and then also quit the app: Godot's SceneTree has `quit_on_go_back` on by default and quit in addition to our handler; now `set_quit_on_go_back(false)` (intended behaviour: Back closes Developer checks, then Menu, then quits only from the main screen).

## v0.1.7 — 2026-11-19 — 02b layout + landscape
Owner verified v0.1.6 on the phone: window (1080x2436) = viewport, fps 120, display 120 Hz (K-36, K-37 fixed by forcing `Window.content_scale_mode = DISABLED`; the 60 fps was a consequence of the scaled/letterboxed window, root cause of the scaling not found). Remaining: (1) Menu button cut off in portrait (top-bar minimum widths too large; reduced), (2) grey strips at the notch/status-bar insets (clear colour now set to the theme background via `RenderingServer.set_default_clear_color`), (3) app should start landscape even with the phone's rotation lock on: orientation = sensor landscape (project.godot 4 and `DisplayServer.screen_set_orientation` in code). Portrait is no longer offered.

## v0.1.6 — 2026-11-19 — 02b display fix attempt
Owner ran v0.1.5 (APK): shell works (theme, speed buttons, pause/play, grid, debug overlay). Problems: (1) content drawn in a 16:9 strip: overlay showed window (1080,2436) but viewport (1280,720) even with `window/stretch/mode="disabled"` in project.godot; (2) fps 60 on a 120 Hz phone, also while dragging. Changes: shell.gd forces content scale mode disabled / expand / factor 1 in `_ready`, `Engine.max_fps=0`, vsync enabled; overlay shows scale mode, aspect, display refresh (Hz), vsync; edge margins 0 (left/right/bottom); project.godot stretch disabled. Causes NOT yet known (hypotheses: editor kept old project settings in memory; Android export preset; OEM compatibility mode; Android refresh policy). K-36/K-37.

## v0.1.5 — 2026-11-19 — Milestone 02b game shell (WRITTEN, NOT RUN)
New in `game/`: `shell.tscn/shell.gd` (main scene), `world_view.gd`. Top bar (Pause/Play, speed 1x/10x/100x/1k/10k via `set_speed`/`set_paused`), status line with a smoothed "Limited speed" indicator (EMA of effective speed, 1.5 s hysteresis; addresses K-34), pan/zoom grid (one finger pan, two finger pinch; mouse drag/wheel on desktop), white and dark themes from palette tokens (saved in `user://settings.cfg`), Settings panel (theme, debug overlay, Developer checks, Close), debug overlay (fps, view, tick, speeds, busy, rss), safe-area margins, Android back button, audio buses `Music`/`Sfx` (buses only, no sounds), orientation: sensor. The 02a check scene moved to `dev_check.gd/.tscn` (opened from Settings, has Close). `project.godot` main scene = shell. Rust, bridge and CI unchanged. Godot was not available in the sandbox: **none of this has been run** (only bracket/indent checks).

## v0.1.4 — 2026-11-19 — Milestone 02a check app verified on device
No code changes. CI run 44406424568 (commit f7e2f85): `core` and `android-arm64-so` green on first run (bridge/gdext, ci.yml, release `m02-android-so` OK). Owner built the check APK (`com.example.platformm02`) from `game/` and ran it on the TECNO CM7: extension loaded, 1000x run ~1000 ticks/s, pause/resume, overload governor, save/load with backup. Raw log in `core/results/phone_godot_app_02a/`. Findings: governor hunts 899<->1123 around the 1000 ticks/s capacity (K-34); recovery after load removal slow (eff 1753 after 15 s; target 10000 not reached) (K-32). One harness artifact (K-33).
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
