# TODO (v0.2.0)
## Milestones 01 and 01b — DONE (engine and core design validated on the owner's phone; report §11).
## Owner decisions needed
- [x] Final commitment to Godot 4.7.x + engine-independent Rust core — DONE 2026-11-19
- [ ] GDD questions before core rules: Q-4 (tick length) and Q-2 (collapse propagation) first; then Q-1, Q-3, Q-5, Q-6, Q-7
## Milestone 02 — Phase 1 skeleton, NO particle mechanics (IN PROGRESS)
### 02a core skeleton (core TESTED v0.2.0; bridge/game/CI next)
- [x] Repo layout core/ bridge/ game/ tools/ + CI (written)
- [x] rng + named streams, clock (requested/effective), governor D-019, timing wheel, snapshot/command interface, save container v1, host, bridge, dev check scene, Python references (written; Python passes)
- [x] Core tested on the phone: 27/27 (v0.2.0)
- [ ] Push to GitHub, CI green, download `m02-android-so`, build the check APK, run buttons 1-5 (governor overload + save/load on the phone)
### 02b Godot game shell
- [ ] layout, themes (white default, inverted dark), real touch pan/zoom, settings, audio foundation, debug overlay (frame time, ticks/s, lag, memory), "limited speed / falling behind" indicator (K-30)
### 02c wiring
- [ ] save/load through the bridge with a real slot UI skeleton; Android lifecycle (pause/background/resume)
## Later: Milestone 03 core rules (needs the GDD answers), then formations, timelines, special particles, research, polish
## Open measurements (optional, cheap)
- [ ] Soak on a hot day / with a thermometer; a second, lower-end device
- [ ] Why in-app tick cost is higher than the benchmark (K-23)

- [x] CI green (core + Android .so), check APK verified on the phone (v0.2.0)
- [ ] Next: 02b Godot game shell (see START_HERE); smooth the speed indicator (K-34); review governor recovery (K-32) in 02c

- [x] 02b shell written (v0.2.0, not run)
- [ ] Owner runs the shell (checklist in TESTING.md), fixes from the log, then 02b verified
- [ ] 02c lifecycle (pause on background), governor review

- [x] 02b VERIFIED on device (v0.2.0)
- [x] 02c written (v0.2.0, not run): saves + lifecycle
- [ ] Push (CI builds new .so), update Godot project, run the 02c checklist (TESTING.md)
- [ ] Then: owner answers GDD Q-4 and Q-2, then Milestone 03 (core rules) planning
