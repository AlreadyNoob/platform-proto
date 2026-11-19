# CHANGELOG
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
