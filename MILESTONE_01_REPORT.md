# Milestone 01 report — engine validation (2026-11-17)

## Verdict: PARTIAL. Two of the six asks are answered with real measurements; the Android/Godot/Rust asks are prepared but NOT verified.
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
