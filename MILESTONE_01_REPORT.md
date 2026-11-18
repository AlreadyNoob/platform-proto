# Milestone 01 report — engine validation (2026-11-17)

## Verdict (updated v0.0.5): engine feasibility VALIDATED on the owner's phone. Godot 4.7.2 + a Rust core loaded and ran in an Android app. Remaining gaps are listed in §9; the original sandbox-era text below is kept for history.
The authoring sandbox had **no Rust toolchain, no Godot, no Android SDK/NDK, no network and no device** (confirmed: no `cargo`, no `godot`, outbound requests blocked). So nothing below claims Rust or Android results. Code that could not be compiled is labelled **UNCOMPILED**.

| # | Your ask | Status | Evidence / what's missing |
|---|---|---|---|
| 1 | Godot↔Rust integration running on Android | **NOT VERIFIED** | Bridge crate, `.gdextension`, Godot harness and CI job written (UNCOMPILED). Must be built and run by you/CI. Steps in `prototype/README.md`. |
| 2 | Does event-driven scheduling preserve per-tick collapse distribution (changing p, connected particles)? | **YES in a Python reference model; Rust port untested** | §1 below |
| 3 | Progressively larger counts and speeds; stored vs simulated vs visible | **Partial** | Simulation scaling measured with a C reference on an x86 server (§2). Visible/render numbers: none. |
| 4 | Sim vs render measured separately | **Harness built, not run** | Sim timed inside Rust; render timed in Godot with sim idle (`godot_proto/main.gd`). |
| 5 | Limitations, build complications, alternatives | **Done** | §4–§5 |
| 6 | Prototype isolated | **Done** | Everything is under `prototype/`; no file in the root docs imports from it. |

## 1. Event-driven vs per-tick rolls (real results)
Reference: `prototype/reference/equivalence_test.py` (Python 3, numpy/scipy). 6×6 board, orthogonal adjacency, 11 particles in a 6-cluster, a 3-cluster and 2 singles; 150 ticks; test curve p = min(0.25, 0.002·m·2^(n−1)) per particle where n is the component size (**test values, not balance**).
- **Changing probabilities:** multiplier m = 2.5 during ticks 50–99 (a step change at two boundaries).
- **Connected interactions:** at tick 60 a placement merges a particle into the 6-cluster (component sizes and p change); in Variant A each collapse removes only the hit particles, so components split and p changes again; in Variant B a collapse takes the whole component.
- **Method under test:** per component, draw the first collapse tick from a geometric distribution with per-tick success 1−(1−p)^n; draw victims from the zero-truncated binomial; redraw schedules when a component's cell set changes (and for everyone when p changes).
- **Comparison:** 25,000 independent runs of each method per variant; chi-square homogeneity on (a) first-collapse tick, (b) particles at tick 75, (c) particles at tick 150.

| Config (25k runs each, seed) | worst-case p over 3 tests | Outcome |
|---|---|---|
| Variant A (1001) | 0.034 (survivors @75) | not rejected at α = 0.001 |
| Variant B (2002) | 0.074 | not rejected |
| Replicates A/B with new seeds 11, 22, 33, 44 | ≥ 0.164 | not rejected |
| **Negative control** A + stale p after multiplier change | survivors @75: χ²=2885, p≈0 | rejected (as it must be) |
| Negative control B + stale p | χ²=1698, p≈0 | rejected |
| Negative control A/B "forgot n" (used p instead of 1−(1−p)^n) | χ²>3900, p≈0 | rejected |

Reading: the method matches brute force in these scenarios, and the test is able to detect plausible bugs. Speed in this toy: brute 10.5 s vs event 1.7 s (A), 5.2 s vs 0.6 s (B) for 25k runs.
**Limits:** one small board, one multiplier window, one placement; 6 tests per method at α=0.001 can still show a chance low p (A@75 gave 0.034 once, replicates were fine); chain semantics were *assumed* (instant) because Q-2 is unresolved; first-collapse test cannot see stale-p bugs because first collapses happen before tick 50. The Rust port (`core_proto/tests/equivalence.rs`) mirrors this and must be run (UNCOMPILED).

## 2. Scaling of the scheduling strategy (real results, C reference, **not Rust, not Android, not rendering**)
Machine: sandbox Intel Xeon @ 2.1 GHz, 1 core. Model: N stored particles in chain-collapse components of size S, per-component collapse prob q per tick, respawn on collapse so the workload is steady (`reference/bench_scaling.c`). Raw output: `prototype/reference/results/bench_scaling_x86_sandbox.txt`.
Terms: **stored** = N. **Simulated** = particles the tick loop evaluates per tick (brute: all N; event-driven: only those in components that collapse). **Visible** = 0 (no renderer here).
Required throughput under provisional assumption A-1 (1 tick = 1 sim-second at 1×): **10,000 ticks/s at 10,000× speed.**

| Stored N | S | q/tick | Event-driven ticks/s (simulated/tick) | Brute per-component | Brute per-particle |
|---|---|---|---|---|---|
| 10,000 | 4 | 1e-4 | 43,439,000 (1.0) | 325,607 | 66,405 |
| 100,000 | 4 | 1e-4 | 4,033,000 (10) | 33,580 | 8,577 |
| 1,000,000 | 4 | 1e-4 | 228,181 (100) | 3,222 | 812 |
| 10,000,000 | 4 | 1e-4 | 11,448 (1,000) | 326 | 80 |
| 1,000,000 | 4 | **1e-2** | **3,214 (10,000)** | 3,153 | 759 |
| 1,000,000 | 16 | 1e-3 | 134,712 (1,000) | 13,388 | 822 |

Findings:
- **F-1 Naive per-particle rolling cannot reach 10,000×** beyond ~100k stored particles even on a server core (812 ticks/s at 1M).
- **F-2 Event-driven scheduling is what makes high speed plausible, but only when collapses are rare per tick.** At 10M stored with rare collapses it just reaches 11k ticks/s on a desktop-class core. When 1% of components collapse every tick, it is **no better than** per-component brute force (3,214 vs 3,153). The advantage is proportional to sparsity.
- **F-4 This benchmark is optimistic:** handling a collapse here is "respawn". Real collapses will remove cells, update components, entropy, logs and animation events. A phone core will also be slower than this CPU (unmeasured).
- Memory (minimal 5 bytes/particle + heap): 88 MB RSS at 10M stored. The real particle record will be larger.
- **What to conclude:** the architecture's event-driven design is justified; the "millions at 10,000×" claim stays an *unproven ambition* until real rules and a phone measurement exist. Speed will need an adaptive cap (reduce speed rather than change rules).

## 3. Determinism finding (F-3)
Event-driven sampling floors `ln(u)/ln(1−q)`. Platform libm can differ by 1 ulp, which could flip a collapse tick between devices and break save/replay. Mitigation written (UNCOMPILED, unit tests included): `core_proto/src/detmath.rs` computes ln with only IEEE basic operations. RNG is integer-only. Acceptance A6 (hash identical desktop vs Android) is **NOT RUN**.

## 4. Limitations and build complications to expect
- Rust on Android needs the NDK arm64 linker per target triple and an environment variable per triple; godot-rust docs list Android SDK command-line tools and JDK 17, and one known NDK/libunwind workaround (applied in CI, harmless otherwise). The NDK version Godot 4.7.2 expects was not verified.
- godot-rust docs state support for stable Godot ≥ 4.0; the specific crate version ↔ Godot 4.7.2 pairing is unverified.
- `.so` must be built per architecture (arm64 phones; x86_64 only for emulators) and placed where the `.gdextension` file points.
- The Android APK export is **not automated** in CI (needs an export preset); the manual path is Godot editor or the GABE on-device exporter.
- The bridge uses strings for results to keep the surface small; gdext API names may need small fixes once the crate resolves.
- Core calls are measured per call from GDScript; copying large arrays to the renderer is a separate cost (`make_positions`).

## 5. Alternatives if Rust-on-Android proves unreliable (evaluate before committing)
1. **C++ GDExtension** with the same engine-independent core (godot-cpp is Godot's reference binding).
2. **Core as a standalone native library (Rust or C) called from a thin GDExtension in C++** to keep Godot-specific code minimal.
3. **libGDX/Kotlin** with a JVM core (primitive arrays); trade-off: much more hand-written animation/UI code.
4. **GDScript + compute shaders** is *not* recommended for the core (determinism and logic control).
No alternative was built or benchmarked in this milestone.

## 6. What you need to run (≈1 hour)
See `prototype/README.md`. Minimum: on your phone in Termux (`pkg install rust`), `cd prototype && cargo test --release -p platform_core_proto` and `cargo run --release -p platform_core_proto --bin bench`; this gives Rust correctness plus arm64 CPU numbers without Godot. Then the CI workflow `.github/workflows/prototype-m01.yml` for desktop results and the Android `.so`.

## 7. Phone results — Rust core on TECNO CAMON 40 Pro 5G (added 2026-11-17, v0.0.3)
Device: MT6878 (4+4), 7.3 GiB RAM, Android 16, Termux, Rust 1.99.1, `cargo --release`. Raw files: `prototype/reference/results/phone_termux_MT6878/`.

**Verified by execution on the phone**
- The Rust core **compiled first try** and **all 5 tests passed**: deterministic-ln accuracy (2), event-driven vs brute-force equivalence for both variants, negative controls detected, same-seed hash (3).
  Equivalence ratios (chi² / critical value at α=0.001, 20,000 runs): variant A ≤ 0.56, variant B ≤ 0.41 (all < 1 = not rejected). Negative controls: stale-p ratios 25–95 (survivor stats), forget-n ratios 137–403: all rejected. (The stale-p *first-collapse* stat sits at 0.21/0.70, expected, because first collapses happen before tick 50.)
- **Determinism across architectures (A6, partial):** the phone's three state hashes (seeds 1–3, 400 runs each: `8fe431764a396449`, `05f5d4a4d9e575cf`, `6f401db9752f4ec1`) were reproduced **bit-for-bit** by an independent Python port on x86 (`prototype/reference/detport.py`). Same RNG, same deterministic ln, same scheduling order, two languages, two architectures. Not yet compared to an Android *app* build (different compiler flags/libc), but the algorithm is platform-independent in practice.

**Phone scaling, all three strategies (valid; `bench_out2.txt`, after the F-5 fix).** Ticks/s; need 10,000 at 10,000× speed (A-1).
| Stored N | S | q/tick | Event-driven | Brute per-component | Brute per-particle | Meets 10,000? |
|---|---|---|---|---|---|---|
| 10,000 | 4 | 1e-4 | 31,142,256 | 228,214 | 51,264 | all |
| 100,000 | 4 | 1e-4 | 2,384,679 | 22,923 | 5,146 | event, brute-comp |
| 1,000,000 | 4 | 1e-4 | **101,067** | 2,297 | 515 | only event |
| 10,000,000 | 4 | 1e-4 | 5,661 | 229 | 51 | **none** |
| 1,000,000 | 4 | **1e-2** | 1,033 | **2,310** | 516 | **none** (event is 2.2x *slower* than brute-comp) |
| 1,000,000 | 16 | 1e-3 | 78,744 | 9,236 | 527 | only event |
Phone vs x86 sandbox: brute loops run at ~0.70x (compute-bound), event-driven at 0.32–0.72x and gets relatively worse as the heap grows (memory-latency-bound: cache misses on a big heap).

**F-6 Crossover (new, important).** Cost per event on the phone is about 0.4 µs (1M components) to 0.7 µs (10M); cost per brute-force component roll is about 1.7 ns. So event-driven scheduling only beats per-component rolling when fewer than about **0.25–0.45% of components collapse per tick**. Above that it loses (measured: 1M components at 1%/tick). The GDD's high-risk clusters (per-particle risk up to the 10% cap, component risk far higher) would be in that regime, but they also collapse almost at once, so what matters is the steady-state mix of low- and high-hazard components, which only real rules can tell us.
Consequence: a pure event-heap design is not enough. Candidate design (D-012, PROPOSED, not built): **hybrid scheduler** — low-hazard components in an event queue (try a timing wheel/bucket queue instead of a binary heap to cut the 0.4–0.7 µs/event and cache misses), high-hazard components rolled per tick directly. Requires its own equivalence test (same method as §1).
Other conclusions from valid data: per-particle rolling is ruled out above ~10k stored particles on this phone (51k ticks/s at 10k, 5k at 100k); 10M stored at 10,000× is not reachable on one thread with this model; 1M stored at 10,000× is reachable only when collapses are rare.
Caveats unchanged: respawn-only collapse handling (F-4), single thread, plain Termux process, no rendering, thermal state not recorded. Raw first run (`bench_out.txt`, brute columns invalid because of F-5) is kept for the record.

**Status of the six asks after this run:** (1) Godot↔Rust on Android: still NOT VERIFIED (bridge/Godot not built). (2) YES, now confirmed in Rust on arm64. (3) Simulated particles: measured on phone; render/visible: not measured. (4) Harness ready; render side not run. (5) updated here. (6) unchanged.

## 8. Android app results — Godot 4.7.2 + Rust on the phone (added 2026-11-17, v0.0.5)
Build path actually used: Rust bridge (gdext **0.5.5**) cross-compiled to aarch64-linux-android by GitHub Actions and published as a release asset; project copied to shared storage; **APK exported on the phone** with the Godot Android editor 4.7.2.stable (non-Gradle export, arm64-v8a, debug); installed and run. Raw output: `prototype/reference/results/phone_godot_app_4.7.2/app_output.txt`.

| Check | Result |
|---|---|
| Extension loads in the Android app | **YES** — `CoreProbe class FOUND`, `arch=aarch64 os=android` |
| Determinism hashes (seeds 1-3) in the app | **identical** to Termux (arm64), GitHub CI (x86_64, Rust 1.100.0) and the Python port: `8fe431764a396449`, `05f5d4a4d9e575cf`, `6f401db9752f4ec1` |
| Call overhead GDScript → Rust | **233 ns per call** (includes the GDScript loop). About 7,000 calls fit in 10% of a 60 fps frame; the API must be coarse-grained (batch, snapshots), not per-particle calls. |
| Sim speed inside the app vs plain Termux | Within ~8% (event 33.0M / 2.53M / 98k ticks/s at 10k / 100k / 1M stored vs Termux 31.1M / 2.38M / 101k). The binding and Godot add no visible cost to simulation. |
| Event vs brute at 1M stored, rare collapses (app) | 97,956 vs 2,469 (per-component) vs 552 (per-particle) ticks/s — consistent with §7 |

**Rendering (all instances in view, static camera, simulation idle; 180 frames each; display is 120 Hz so ~8.33 ms = vsync):**
| Instances drawn | Setup (GDScript loop) | Avg frame | p95 | Worst | Reading |
|---|---|---|---|---|---|
| 10,000 | 14 ms | 8.43 ms | 8.33 | 14.8 | at vsync (120 fps) |
| 100,000 | 47 ms | 8.75 ms | 8.33 | 33.3 | at vsync; true cost < 8.3 ms |
| 300,000 | 99 ms | 14.12 ms | 19.21 | 106 | ~70 fps avg, p95 ≈ 52 fps |
| 1,000,000 | 325 ms | 29.35 ms | 33.33 | 130 | ~34 fps |
Caveats: frame interval equals draw cost only once it exceeds vsync (so 10k/100k only show an upper bound); worst frames of 33-130 ms are probably one-off hitches after creating the MultiMesh (not separated from steady state); plain colored quads, no textures/shaders/animation; at 1M instances the quads overlap heavily on screen; no thermal data; no panning or zooming during measurement.
Reading for design: **up to roughly 100,000 individually drawn instances is comfortably inside a 60 fps budget on this phone; 300,000 is marginal; 1,000,000 is not**. So "millions of particles" must be stored and simulated but *drawn* through LOD/aggregation (density layers when zoomed out), as ARCHITECTURE §6 already assumes.

**Build complications actually met (for the record):**
1. GitHub Actions artifact storage quota was full → .so published as a **release asset** instead.
2. `gh` needs explicit run IDs in non-interactive use.
3. Android editor showed a startup spinner once; it resolved by itself, cause not identified.
4. Android Godot editor cannot see Termux's private folder → project must live in shared storage (`Documents/`).
5. Export templates (4.7.2) are not bundled; the editor downloads them once.
6. Non-Gradle export carried the extension `.so` correctly (no GABE/Gradle needed).
7. NDK version used by CI was not recorded (runner default).

## 9. What is still NOT validated (honest list)
- Simulation running **while** rendering (combined load, thermal throttling over ~10 minutes).
- Simulation on a worker thread (prototype ran it on the main thread, freezing the UI for seconds).
- Cost of moving a snapshot Rust → Godot (`make_positions` exists but has no button).
- Interactive pan/zoom frame times; multi-touch.
- A8 (pause/speed correctness) — the prototype has no clock/pause.
- Hybrid scheduler (D-012) — proposed, not built or tested.
- Memory use of the app at 1M stored; Android lifecycle (background/kill/resume).
- Lower-end devices (only one phone tested).

## 10. Milestone 01b — hybrid scheduler, clock, worker thread, soak (2026-11-17, v0.0.6)
Goal: close the gaps in §9 that shape the core design. **Status (v0.0.7): the Rust core part ran on the owner's phone (14/14 tests, scheduler benchmark, determinism). The Godot app part (snapshot cost, soak) is written but not yet built or run.**

### 10.1 What was built (all under `prototype/`, still isolated)
| Piece | Files | State |
|---|---|---|
| Hybrid scheduler: brute / heap / timing wheel / hybrid (hot slots rolled per tick, cold slots in a wheel with overflow list) | `reference/hybrid_ref.py` (ran), `core_proto/src/hazard.rs` | Python **validated**; Rust UNCOMPILED |
| Clock with pause, speed, exact integer time accounting, lag kept (not dropped) | `core_proto/src/clock.rs`, `tests/clock.rs` | UNCOMPILED |
| A8: results independent of frame size, speed changes, pauses, batching | `tests/hybrid.rs` (`speed_pause_and_batching_do_not_change_state_a8`) | UNCOMPILED |
| Simulation on a worker thread driven by the clock; UI reads atomics only | `core_proto/src/worker.rs`, `bridge_proto/src/lib.rs` | UNCOMPILED |
| Scheduler benchmark at ~1M particles with several hot/cold mixes | `core_proto/src/bin/bench_hybrid.rs` | UNCOMPILED, unrun |
| App harness v2: snapshot cost, 1-min and 10-min soak (render 100k + camera motion + sim at 10,000× on the worker thread, windowed frame stats, sim rate, lag, busy %, RSS) | `godot_proto/main.gd` | never run |
| Termux runner and CI steps for all of the above | `prototype/RUN_M01B_TERMUX.sh`, `.github/workflows/prototype-m01.yml` | never run |

### 10.2 Validated now (Python reference, 20,000 runs per comparison)
Model: 8 slots; hot q=0.2, cold q=0.01 (test values), a collapsed slot respawns with a freshly drawn q and rolls again from the next tick; wheel horizon 32 ticks so the overflow logic is exercised.
- Heap vs brute force: p = 0.52 / 0.74 / 0.003*; wheel: 0.52 / 0.80 / 0.003*; **hybrid: 0.31 / 0.75 / 0.29** (three statistics: total events, slot-0 events, slot-0 first-collapse tick).
- \*The 0.003 (first-collapse tick) is not below α = 0.001 and is a chance outlier: re-tested with 40,000 fresh-seed runs against the **exact** theoretical distribution, brute / heap / wheel / hybrid give p = 0.23 / 0.28 / 0.13 / 0.27. Raw: `reference/results/hybrid_ref_results.json`, `hybrid_ref_theory_check.txt`.
- Negative controls: a 15% biased brute force (chi² 1526, p≈0) and a wheel with the overflow rescan removed (chi² 24,224, p≈0) are rejected. The wheel's internal invariant (every bucket entry is due now) never fired.
- Conclusion: the heap, the wheel and the hybrid are the same stochastic process as per-tick rolling; mode switching at respawn is exact.

### 10.3 Acceptance criteria for 01b (to be filled from your runs)
| # | Check | Pass condition | Status |
|---|---|---|---|
| B1 | Rust tests (14: detmath 2, equivalence 3, clock 5, hybrid 4) | all pass, on phone and CI | **PASS on phone (14/14, first compile)**; CI pending |
| B2 | `bench_hybrid` at 250k slots (≈1M particles) | hybrid within 10% of the best strategy at every mix; report ticks/s vs the 10,000 needed | **FAILED as a design test — important finding F-7/F-8 (below): the wheel alone is best; the hybrid as specified is slower** |
| B3 | `--determinism-hazard` hashes phone vs x86 | identical | **PASS vs an independent x86 Python port (all 4 strategies match bit-for-bit: events and hash)**; CI x86 Rust pending |
| B4 | Snapshot transfer (button 4) | 100k points per frame ≤ 1.7 ms (10% of a 60 fps frame), idle and with sim running | NOT RUN |
| B5 | 10-minute soak (button 5b) | sim ≥ 95% of 10,000 ticks/s with lag not growing; render p95 ≤ 16.7 ms; last window within 20% of first (no throttling collapse); RSS growth < 10%; no crash/ANR | NOT RUN |
Anything that fails is a finding, not an error: it changes D-012/D-014 and is recorded here.

### 10.4 Design notes worth remembering
- Clock arithmetic is integer microseconds × speed; splitting a second into any frame sizes yields exactly the same tick count (tested). Pause accumulates nothing, so resuming cannot burst. Lag stays owed and is reported.
- The worker thread works in 100-tick chunks and yields after 8 ms, so Stop is responsive even with the brute strategy.
- The hybrid wheel costs a fixed O(1) per tick plus work per event, while heap cost grows with log(slots) and cache misses (F-6). The benchmark exists to find out whether that matters on the phone.

### 10.5 Results from the owner's phone (Rust core, Termux, MT6878) — added in v0.0.7
Raw files: `prototype/reference/results/phone_termux_01b/`.
**Tests:** 14 of 14 pass, compiled without a single error: clock (5: pause stops ticks and resume has no burst; any frame splitting gives the same tick count; exact speed changes; lag kept), A8 (a run driven through odd frame sizes, 10,000× steps, random speeds and pauses equals one direct run, for all four strategies), equivalence (heap, wheel, hybrid vs brute force: worst chi² ratio 0.58 where 1.0 would be rejection), negative controls rejected (15% biased brute force: ratio 34; wheel without overflow rescan: ratio 545).
**Determinism:** phone hashes for all four strategies (`brute c6192bc0d4e6d6e2`, `heap c68b73b07b025628`, `wheel 66bfb35d93919c91`, `hybrid 101ce1486d4693e9`; 2000 slots, 20,000 ticks) are reproduced **exactly** by an independent Python port on x86 (`prototype/reference/hazport.py`), including the order of events inside the wheel's linked lists.

**Scheduler benchmark (250,000 slots = 1M particles; single thread; ticks per second; need 10,000 at 10,000× speed):**
| Hot/cold mix (steady hot share) | Events per tick | Heap | **Wheel** | Hybrid (q* = 0.3%) | Brute force |
|---|---|---|---|---|---|
| all cold (0%) | 25 | 91,134 | **215,305** | 178,383 | 1,346 (same for every mix) |
| 0.5% hot | 50 | 49,251 | **151,685** | 84,240 | |
| 4.3% hot | 239 | 9,800 | **30,828** | 13,334 | |
| 33% hot | 1,672 | 1,384 | **3,764** | 943 | |
| 83% hot | 4,172 | 560 | **1,634** | 823 | |
(Hot slots here collapse with probability 0.02 per tick; cold 0.0001.)
- **F-7 The timing wheel beats the binary heap by 2.4–3.1× at every mix** (about 6–7.5 million collapse events per second per core vs about 2.3–2.5 million). Capacity at 10,000 ticks/s: the wheel handles roughly 600–750 events per tick, the heap about 230. The earlier worry (F-6) that event scheduling loses to plain rolling at high event rates was mostly a cost of the heap's memory access, not of event scheduling itself.
- **F-8 The hybrid, as specified, is slower than the wheel alone at every mix** (and 17% slower even with no hot slots, from bookkeeping on each respawn). Rolling a slot every tick costs on the order of 4–7 ns; handling one wheel event costs about 130 ns. The break-even per-slot hazard is therefore around 3–5% per tick (derived, not directly measured), about ten times higher than my threshold of 0.3%, and my hot slots (2%) are below it. A hybrid can only pay off for much hotter slots. This is a design correction: **D-012 is revised to "timing wheel for all slots"**; a hot list is kept only as an option for hazards above ~3–5%, to be tested with `bench_hybrid --hi` (q = 0.2).
- At the stated target the wheel sustains 1M particles at 10,000× up to about 4% hot slots (239 events/tick, 30.8k ticks/s); at 33% hot slots (1,672 events/tick) it reaches 3.8k ticks/s, i.e. 0.38× real-time speed at 10,000×. Brute force reaches 1.3k ticks/s. Caveat: collapse handling here is a toy (draw a new q, schedule); real collapses cost more, so real capacity will be lower.
- Display refresh matters for soak frame times: the phone reported 120 Hz earlier and 60 Hz in this session's system info; record it with each soak.

### 10.6 High-hazard run on the phone (v0.0.8): where the hot list pays off
`bench_hybrid --hi`: hot slots collapse with probability 0.2 per tick (about every 5 ticks), hybrid threshold 3%, 250,000 slots (1M particles). Ticks per second:
| Steady hot share | Events per tick | Heap | Wheel | **Hybrid** | Hybrid vs wheel |
|---|---|---|---|---|---|
| 0% | 25 | 87,043 | 228,834 | 156,194 | 0.68x (bookkeeping overhead) |
| 0.4% | 249 | 12,234 | 38,591 | **50,441** | 1.31x |
| 4.7% | 2,373 | 954 | 2,852 | **3,314** | 1.16x |
| 33% | 16,658 | 164 | 419 | **598** | 1.43x |
| 83% | 41,679 | 81 | 176 | **453** | 2.57x |
**F-9 The hot list pays off when slots are hot enough (q = 0.2), and not when they are not (q = 0.02, §10.5).** Cost model fitted from both runs (derived, not directly measured): rolling one slot per tick costs about **5.4 ns**; handling one collapse of a hot slot costs about **26 ns**; handling one collapse through the wheel costs about **136 ns**. Break-even per-slot hazard = 5.4 / (136 − 26) ≈ **5% per tick**: slots above it should roll every tick, slots below it belong in the wheel. Hot-event throughput reached about 19 million events per second (83% hot), against about 7 million for the wheel.
Other observations:
- **Run-to-run noise:** the same configuration varied by up to about 20% between two runs on the phone (wheel, all cold: 264k vs 215k ticks/s); other cases varied 2-5%. Treat differences below ~20% as noise at the fast end.
- **Hybrid overhead with no hot slots** was 32-35% below the wheel in both runs. Cause identified by reading the code: every collapse touched the hot-list index even for cold slots. Fixed in v0.0.8 (only slots that were hot are removed); the change is behaviour-preserving (an independent Python port gives the same four hashes), but the speed gain is **not yet measured**.
- **D-012 (second revision):** two tiers with the threshold at about 5% per tick (`q_star` = 0.05): wheel below, hot list above. Whether real Platform slots reach hazards that high in large numbers depends on the (undecided) collapse rules: a component's risk is 1-(1-p)^n, so large clusters will be far above 5% but also collapse within a few ticks.
