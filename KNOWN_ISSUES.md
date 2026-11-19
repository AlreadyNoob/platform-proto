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
| K-22 | Medium | Simulation lag grows without bound when the device cannot keep up (no speed governor yet) — D-019 | Open |
| K-23 | Low | In-app simulation cost per tick was ~1.4-2x the Termux benchmark (cause untested; CPU clock scaling suspected) | Open |
| K-24 | Low | NDK version used by CI not recorded in the log | Open |
| K-25 | Info | K-15, K-6, K-5, K-16 (fixed seeds passed) resolved: the 01b Rust code compiled and passed on the phone and in CI; the harness v3 ran | Closed |
| K-26 | Medium | M02a `core/` compiled and passed 27/27 tests on the phone (v0.1.3). v0.1.4: CI green on first run; Android `.so` built; check APK ran on the phone | Closed |
| K-27 | Medium | Governor numbers (1 s / 0.1 s lag, 90%, x1.25, backoff) are PROVISIONAL and validated only against a Python device model; real devices have noise and thermal throttling | Open (host test showed the governor reacts on a real thread; steady-state not measured on hardware) |
| K-28 | Low | Save v1: RNG states are placeholders until a real world exists; no directory fsync; no migration yet (schema 1 only); not tested on Android storage | Open |
| K-29 | Medium | `core/tests/host.rs` is timing based; passed once on the phone (3/3 in 1.32 s). A failure needs a re-run before blaming the code | Open |
| K-30 | Low | Under overload the governor makes game time run slower than the wall clock; the UI must show a clear "limited speed / falling behind" indicator (planned 02b) | Open |
| K-31 | Info | The bridge uses `BurnWorld` (a CPU-burning test world) because no real world exists | Open until Milestone 03 |

| K-32 | Medium | Governor recovery after load removal is slow: from 899 it reached only 1753 after ~15 s (target 10000; the Python model predicted ~36 s for a full recovery, so not yet a proven deviation). Measured once, one device | Open. Analysis (hypothesis, not measured): observed raise steps were ~6 s apart because the probe wait had backed off; the wait resets after 40 calm intervals (10 s), so full recovery from ~900 to 10000 is expected in roughly 1 minute. Not a bug by design; revisit with a longer on-device run |
| K-33 | Info | Check-scene harness artifact: pressing button 4 during the button-3 pause check restarts the run, so that line shows a tick drop (40393 -> 293). Not a core bug; avoid overlapping buttons | Open (cosmetic) |
| K-34 | Low | Governor hunts between 899 and 1123 ticks/s under constant overload (busy 90-100%); bounded, but the UI speed indicator will flicker unless smoothed | Open (02b/02c) |
| K-35 | Medium | 02b shell (GDScript) never run: possible parse/runtime errors, touch input (`ScreenTouch`/`ScreenDrag` plus emulated mouse), safe-area values, UI scale from DPI are all unverified | Open until the owner runs v0.1.5 |
| K-36 | High | v0.1.5 APK: viewport 1280x720 inside a 1080x2436 window although stretch is disabled in project.godot (content letterboxed 16:9). Fixed in v0.1.6 by forcing the Window scale mode in code (owner verified: vp = window); original cause still unknown | Closed (workaround) |
| K-37 | Medium | fps reads 60 on a 120 Hz phone (also while dragging). M01 app reached ~119 fps in landscape with canvas_items. Hypotheses: battery/refresh policy, Swappy frame pacing, compat mode. v0.1.6 owner screenshot: fps 120, display 120 Hz | Closed |
| K-38 | Medium | 02c Rust (`save_game`/`load_game`/`slot_summary`) and the Saves UI / lifecycle code were never compiled or run | Open until CI + owner run v0.2.0 |
