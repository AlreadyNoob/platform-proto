# DECISIONS log
Format: ID · date · decision · alternatives · rationale · consequences · reversible?

| ID | Date | Status |
|---|---|---|
| D-001 | 2026-11-16 | ACCEPTED (process) |
| D-002 | 2026-11-16 | APPROVED for investigation (owner); **supported by Milestone 01 evidence (v0.0.5)**; final commitment awaits owner decision after the gaps in report §9 |
| D-003 | 2026-11-16 | APPROVED for investigation (owner); **supported by evidence**: Rust core built and ran on Android arm64 via gdext 0.5.5; alternatives stay documented, none triggered |
| D-009..D-011 | 2026-11-17 | ACCEPTED (Milestone 01) |
| D-004..D-008 | 2026-11-16 | ACCEPTED as design rules (follow GDD/master prompt) |
| A-1 | 2026-11-16 | PROVISIONAL assumption |

**D-001 Work order.** Architecture and engine validation first; no particle mechanics until the validated skeleton exists (owner instruction, 2026-11-16: prefer 6–12 months and a maintainable simulation over a rushed prototype). Alternatives: feature-first prototype. Consequence: slower visible progress, lower rewrite risk. Reversible.

**D-002 Engine: Godot 4.7.x for presentation.** Alternatives: libGDX 1.14.x, native Android. Rationale: ENGINE_EVALUATION.md. Consequence: GDScript/scenes for UI and animation. Reversible *cheaply only if D-004 holds*; costly once many scenes exist.

**D-003 Simulation core is an engine-independent library, Rust preferred.** Alternatives: C++ GDExtension, GDScript, Kotlin. Rationale: speed, determinism, headless CI tests, engine swap-ability. Consequence: build pipeline needs Android cross-compilation; binding overhead must be measured. Reversible before Phase 2.

**D-004 Core never depends on engine/UI; UI uses Commands, renderer reads Snapshots.** (master prompt §5) Reversible: no.
**D-005 Per-tick probabilities; integer thresholds; seeded saved RNG streams.** Event-driven shortcuts only when proven equivalent and documented.
**D-006 Timeline identity is a stable id separate from the 2-decimal display value.**
**D-007 Orthogonal adjacency by default.**
**D-008 Save format is versioned with migrations and atomic writes from the first save feature.**
**A-1 PROVISIONAL: 1 tick = 1 sim-second at 1× speed.** Configurable constant; to be confirmed (Q-4).

**D-009 Deterministic math in the core.** Event scheduling must not depend on platform libm: use `detmath::ln` (IEEE basic ops only) and integer-only RNG. Evidence: finding F-3 in MILESTONE_01_REPORT.md. Status: designed and written, UNCOMPILED. Reversible: yes.
**D-010 Prototype isolation.** All validation code lives in `prototype/`; the game core is re-implemented, not copied. Reversible: yes.
**D-011 Event-driven scheduling is the baseline for LOW-hazard components (amended v0.0.4: not for all).** Evidence: Python equivalence results and C scaling results (MILESTONE_01_REPORT.md §1–2). It stays tick-exact; when the device cannot keep up, speed is reduced, never rules. Per-particle brute force is rejected above ~100k particles. Provisional until measured on a phone with real rules.
**D-012 (REVISED v0.0.7) Use a hashed timing wheel for all slots; a hybrid hot list is NOT adopted at current hazards.** Evidence: report §10.5 — wheel 2.4-3.1x faster than heap, hybrid with threshold 0.3% slower than wheel at every mix; break-even for per-tick rolling is about 3-5% hazard per slot. Algorithm validated in Python and Rust (14/14 tests on the phone); hybrid for hazards above ~3-5% remains an untested option (`bench_hybrid --hi`). Original text: Low-hazard components in an event queue (evaluate timing wheel vs binary heap), high-hazard components rolled per tick; component moves between modes as its hazard changes. Evidence: F-6 (phone crossover ~0.25-0.45% collapse/tick per component). Needs an equivalence test and a benchmark before acceptance. Not built.
**D-013 Coarse-grained binding API.** Measured 233 ns per GDScript→Rust call. Production API passes batches/snapshots per frame, never per-particle calls. Evidence: report §8.
**D-014 Draw budget and LOD.** Individually drawn instances capped around 100,000 on this phone class (300k marginal, 1M not viable); beyond that, aggregation/LOD. Evidence: report §8 (static quads, caveats apply). Provisional; re-measure with real visuals.
**D-015 gdext pinned to =0.5.5; Godot 4.7.2 (official) on Android loaded it.** Reversible.
**D-016 Clock is integer-only.** Accumulate microseconds x speed, no floats; lag kept as owed ticks; pause accumulates nothing. Evidence: tests written (unrun). Reversible: yes.
**D-017 Simulation runs on a worker thread; the UI reads atomic counters.** Prototype only; the real snapshot interface is a later milestone. Soak test (B5) decides whether this is sufficient.
**D-018 Capacity planning number.** On this phone one core handles about 6-7.5 million toy collapse events per second with the wheel (about 2.4 million with the heap) and up to about 19 million for hot slots on the hot list. Treat as an upper bound; re-measure with real collapse handling. Reversible: yes.
**D-012 (second revision, v0.0.8):** two tiers — timing wheel for slots with hazard below about 5% per tick, per-tick hot list at or above it (`q_star` = 0.05). Evidence: report §10.5-10.6 (hybrid loses at q=0.02, wins 1.2-2.6x at q=0.2). Validated statistically (Python, Rust); speed gain of the overhead fix not yet measured. Reversible: yes.
