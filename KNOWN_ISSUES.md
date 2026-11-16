# KNOWN_ISSUES
| ID | Severity | Issue | Status |
|---|---|---|---|
| K-1 | High | Engine choice unvalidated; recommendation based on documentation only | Open (Milestone 01) |
| K-2 | High | Event-driven acceleration for 10,000× at scale is unproven | Open |
| K-3 | Medium | GDD ambiguities Q-1..Q-7 unresolved | Open |
| K-4 | Medium | C# on Android in Godot 4.7 not verified (avoided) | Open |
| K-5 | High | Rust core COMPILED and tested on phone (5/5). Bridge, Godot harness and CI are still UNCOMPILED/never run | Partly closed |
| K-6 | High | Godot-to-Rust on Android not verified | Open |
| K-7 | Medium | Event-driven advantage vanishes when ~1% of components collapse per tick (F-2); real collapse handling cost not modelled (F-4) | Open |
| K-8 | Medium | gdext version/Godot 4.7.2 pairing and NDK version unverified; `godot = "*"` unpinned | Open |
| K-9 | Low | Equivalence test covers one small board and one multiplier window; chain semantics assumed (Q-2) | Open |
| K-10 | Medium | Rust bench brute-force columns were optimised away (F-5) | Fixed and re-run on phone (v0.0.4) |
| K-11 | Medium | Phone: no strategy reaches 10,000 ticks/s at 10M stored; at 1M stored with ~1% collapse rate per tick, event-driven (1,033) is slower than per-component rolling (2,310) (F-6; crossover ~0.25-0.45%/tick) | Open — hybrid scheduler D-012 to be designed and tested |
