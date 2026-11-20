# TODO (v0.2.7)
## Milestones 01 and 01b — DONE (engine and core design validated on the owner's phone; report §11).
## Owner decisions needed
- [x] Final commitment to Godot 4.7.x + engine-independent Rust core — DONE 2026-11-19
- [x] GDD Q-4 (tick length), Q-2 (collapse propagation), Q-7 (device targets) — answered 2026-11-19 (D-024, D-025, D-026)
- [x] GDD Q-3 (collapse formula) fully answered (D-029, D-030, D-031)
- [ ] Economy sub-questions, one at a time: Q-1 (split rule, spawn placement, blocked spawns; D-027), Q-6 (what the starting budget is, replenishment, how a placement is paid; D-028); Q-5 timelines later
## Milestone 02 — Phase 1 skeleton, NO particle mechanics — COMPLETE (v0.2.2)
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
- [ ] Then: owner answers the remaining GDD questions, then Milestone 03 (core rules) planning

## Device-target follow-ups (D-026)
- [ ] Check export preset min SDK = 30 and renderer choice (K-42); weaker-device test profile (K-41); save portability policy (K-43)

## Milestone 03a collapse core (WRITTEN v0.2.4; owner said "continue", Python reference passes)
- [x] Core tested on the phone: 36/36 (v0.2.5)
- [ ] OWNER: overlay v0.2.5 on the repo, push, CI green
- [x] Owner confirmed K-45 (one roll per neighbour per chain, D-033)
- [ ] 03b: wire `ParticleWorld` into host/bridge/game (debug placement tool, wave animation from the chain log); event-driven engine later (K-44)
### original 03a scope
- [ ] sparse grid, placement by debug command (free), orthogonal connectivity/components, collapse model D-029 (configurable), chains D-025/D-030/D-031, chain log for the presentation, deterministic RNG stream, Python reference + golden fixtures, tests; NO generation, economy, entropy or timelines

## Milestone 03b economy core (WRITTEN v0.2.6)
- [x] Core tested on the phone: 44/44 (v0.2.7)
- [ ] OWNER: overlay v0.2.7 on the repo, push, CI green
- [ ] Owner tunes placeholders later (`docs/TUNING_PLACEHOLDERS.md`)
## 03c (next): save v2 with world state + migration (K-48); wiring into host/bridge/game; chain-wave presentation; event-driven engine later (K-44)
