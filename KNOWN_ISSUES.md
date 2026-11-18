# KNOWN_ISSUES
| ID | Severity | Issue | Status |
|---|---|---|---|
| K-1 | High | Engine choice unvalidated; recommendation based on documentation only | Open (Milestone 01) |
| K-2 | High | Event-driven acceleration for 10,000× at scale is unproven | Open |
| K-3 | Medium | GDD ambiguities Q-1..Q-7 unresolved | Open |
| K-4 | Medium | C# on Android in Godot 4.7 not verified (avoided) | Open |
| K-5 | High | Rust core, bridge, Godot harness, CI all built and run | **Closed** |
| K-6 | High | Godot-to-Rust on Android | **Closed** (v0.0.5, verified on phone) |
| K-7 | Medium | Event-driven advantage vanishes when ~1% of components collapse per tick (F-2); real collapse handling cost not modelled (F-4) | Open |
| K-8 | Medium | gdext 0.5.5 + Godot 4.7.2 pairing verified and pinned; NDK version used by CI not recorded | Mostly closed |
| K-9 | Low | Equivalence test covers one small board and one multiplier window; chain semantics assumed (Q-2) | Open |
| K-10 | Medium | Rust bench brute-force columns were optimised away (F-5) | Fixed and re-run on phone (v0.0.4) |
| K-11 | Medium | Phone: no strategy reaches 10,000 ticks/s at 10M stored; at 1M stored with ~1% collapse rate per tick, event-driven (1,033) is slower than per-component rolling (2,310) (F-6; crossover ~0.25-0.45%/tick) | Open — hybrid scheduler D-012 to be designed and tested |
| K-12 | Medium | Render test only measured static camera, simulation idle, plain quads; worst-frame spikes (33-130 ms) not explained | Open |
| K-13 | Medium | Simulation ran on the main thread in the prototype (UI froze for seconds); worker-thread design untested | Open |
| K-14 | Low | Only one device tested; no thermal/soak data; Android editor showed an unexplained one-time startup spinner | Open |
| K-15 | Medium | 01b core Rust compiled and passed on the phone; the worker thread's use from the bridge and the GDScript harness v2 are still unbuilt/unrun | Partly closed |
| K-16 | Medium | The Rust hybrid equivalence test uses fixed seeds, so it passes or fails deterministically; chance-failure risk about 1%. A failure needs a seed check before blaming the code | Open |
| K-17 | Medium | Soak uses plain quads and an automated camera, not real touch input or real game visuals | Open |
| K-18 | Medium | Real collapse handling will cost more than the toy respawn, so measured event capacity (6-7.5M/s) is an upper bound | Open |
| K-19 | Low | Phone display refresh varied (120 Hz earlier, 60 Hz later); soak frame times must record it | Open |
| K-20 | Low | Phone benchmark noise up to ~20% between runs in the fastest cases; need repeated runs for small differences | Open |
| K-21 | Low | Hybrid no-hot-slot overhead fix (v0.0.8) is behaviour-preserving but its speed effect is unmeasured | Open |
