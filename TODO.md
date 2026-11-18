# TODO (v0.1.3)
## Milestones 01 and 01b — DONE (engine and core design validated on the owner's phone; report §11).
## Owner decisions needed
- [x] Final commitment to Godot 4.7.x + engine-independent Rust core — DONE 2026-11-19
- [ ] GDD questions before core rules: Q-4 (tick length) and Q-2 (collapse propagation) first; then Q-1, Q-3, Q-5, Q-6, Q-7
## Milestone 02 — Phase 1 skeleton, NO particle mechanics (IN PROGRESS)
### 02a core skeleton (core TESTED v0.1.3; bridge/game/CI next)
- [x] Repo layout core/ bridge/ game/ tools/ + CI (written)
- [x] rng + named streams, clock (requested/effective), governor D-019, timing wheel, snapshot/command interface, save container v1, host, bridge, dev check scene, Python references (written; Python passes)
- [x] Core tested on the phone: 27/27 (v0.1.3)
- [ ] Push to GitHub, CI green, download `m02-android-so`, build the check APK, run buttons 1-5 (governor overload + save/load on the phone)
### 02b Godot game shell
- [ ] layout, themes (white default, inverted dark), real touch pan/zoom, settings, audio foundation, debug overlay (frame time, ticks/s, lag, memory), "limited speed / falling behind" indicator (K-30)
### 02c wiring
- [ ] save/load through the bridge with a real slot UI skeleton; Android lifecycle (pause/background/resume)
## Later: Milestone 03 core rules (needs the GDD answers), then formations, timelines, special particles, research, polish
## Open measurements (optional, cheap)
- [ ] Soak on a hot day / with a thermometer; a second, lower-end device
- [ ] Why in-app tick cost is higher than the benchmark (K-23)
