# PROTOTYPE_PLAN — Milestone 01: engine validation (status: PARTIAL — see MILESTONE_01_REPORT.md)

Progress against acceptance criteria (updated v0.0.5, owner's phone):
| # | Status |
|---|---|
| A1 APK builds and installs on device | **PASS** (exported on-device with Godot Android editor 4.7.2; library built in CI) |
| A2 smooth with 100k visible instances | **PASS for static camera**: avg 8.75 ms, p95 8.33 ms at 120 Hz; pan/zoom under interaction not measured |
| A3 1M stored loads and idles | **Partial**: 1M stored ran inside the app; memory/load time not recorded |
| A4 core tick throughput | **PASS (measured)** in app and Termux |
| A5 sampling equivalence | **PASS** (Python, Rust on phone, CI) |
| A6 determinism desktop vs device | **PASS** (x86 CI + Python vs arm64 Termux + Android app) |
| A7 binding overhead | **Partial**: 233 ns/call measured; snapshot transfer cost not measured |
| A8 pause/speed correctness | **NOT RUN** (no clock in prototype) |
Decision gate result: A1 and A2 pass, A7 call overhead is small -> no libGDX spike triggered. Prototype code: `prototype/`.

Purpose: confirm or reject D-002/D-003 with evidence. It is throwaway-quality *presentation* code plus the real seed of the engine-independent core. **No Platform particle rules** (no collapse, formations, entropy, timelines).

## Scope
1. Godot 4.7.2 project, landscape, exports a debug APK that installs and runs on the creator's device (record model + Android version).
2. Sparse chunked grid drawn with pan and pinch-zoom (touch), viewport-culled.
3. Tap places a particle; placement animation (scale-in + ripple shader).
4. Particle rendering through batched instancing, with a synthetic dataset at 10k / 100k / 1M *stored* particles and a visible subset.
5. Core stub (engine-independent) with fixed-step clock, seeded RNG, sparse chunk store, and a *representative workload*: N particles, per-component geometric collapse sampling vs brute-force per-tick rolls (this also proves ARCHITECTURE §3 equivalence statistically).
6. Core ↔ engine binding measured: calls per frame, snapshot copy cost.
7. Determinism check: same seed → identical state hash on desktop CI and on the Android device.
8. Build path check: one reproducible CI workflow producing the APK (the creator already uses GitHub Actions) and the on-device GABE path noted as an alternative.

## Acceptance criteria (all objective)
| # | Criterion | Pass condition |
|---|---|---|
| A1 | APK builds in CI and installs on device | yes/no, steps documented |
| A2 | Pan/zoom smooth with 100k visible instances | frame time p95 ≤ 16.7 ms on the creator's device (target to be confirmed; record actuals) |
| A3 | 1M stored particles loads and idles | memory use and load time recorded; no crash |
| A4 | Core tick throughput | ticks/s and particles/s recorded at 3 sizes; compared to brute force |
| A5 | Sampling equivalence | collapse-count distribution from event-driven vs brute-force agrees (chi-square, seed printed) |
| A6 | Determinism | identical hash desktop vs device for 3 seeds |
| A7 | Binding overhead | per-frame snapshot cost < 10% of frame budget at 100k visible |
| A8 | Pause/speed correctness | pause stops ticks exactly; 100× run produces same state hash as 1× run for same tick count |

## Decision gate
Pass A1–A8 → confirm D-002/D-003. Fail A1, A2 or A7 → run libGDX spike (§4). Fail A5/A6 → fix the core design, not the engine.

## 4. Optional libGDX spike (time-boxed, only on trigger)
Same items 2–4 and 6–7 on libGDX 1.14.2/Kotlin, then compare.

## Results
Recorded in `TESTING.md`. Until then every cell there reads NOT RUN.
