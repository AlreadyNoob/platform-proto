# ARCHITECTURE (proposed — status: DESIGNED, nothing implemented)

Engine-neutral. Terminology maps onto any engine. See `DECISIONS.md` for which choices are provisional.

## 1. Layers and the one rule
```
Presentation (engine: renderer, camera, UI, audio, animation, shaders)
   │ reads Snapshots (immutable, read-only)         ▲
   │ sends Commands (place, set_speed, pause, ...)  │ receives Events (batched, for effects/log)
   ▼                                                │
Simulation Core (no engine dependency)  ── Save System ── Diagnostics
   clock · rng · world/grid · particles · connectivity · timelines · entropy/events · research
```
**Rule:** the Core never imports engine types; Presentation never mutates Core state except through Commands. Animations are driven by Events *after* the state change is final, so a renderer can never fake or delay game state.

## 2. Simulation Core modules
| Module | Responsibility | Notes |
|---|---|---|
| clock | fixed-step `SimTick` (u64), speed, pause | 1 tick = fixed sim-time quantum (ASSUMPTION A-1, see PROJECT_UNDERSTANDING). Speed = ticks per real second budget, never "1 frame = 1 tick". |
| rng | seeded deterministic generator, state saved | Separate named streams (collapse, events, genesis) so adding a feature does not reshuffle others. |
| world | sparse chunked grid, per timeline | Key = `(TimelineId, chunk_x, chunk_y)`; chunk = fixed array (e.g. 32×32 cells). Logical coords are i32/i64; screen coords never stored. |
| particles | data-oriented storage | Structure-of-arrays per chunk (type, age, state, component ref). No object-per-particle. Stable `ParticleRef` for saves/log only, never shown to players. |
| connectivity | orthogonal adjacency, components | Incremental union on placement; bounded local re-scan on removal; periodic full validation tool (debug). |
| timelines | `TimelineId` (u32, stable) + separate display label | Display label = hundredths (0.00–1.00) and may repeat; identity never derived from it. Occupancy is per `TimelineId`. |
| entropy/events | configurable Entropy function, event scheduler | Events carry sim-time durations and are scheduled in ticks. |
| research | resources, upgrades, constants | Later phase. |

## 3. Time and probability (the hardest part)
- **Per-tick semantics are law.** A collapse probability `p` is per tick. Stored as an integer threshold (e.g. `u64`, `p·2^64`) compared to RNG output — no floating-point in outcome decisions, so results match across devices and CI.
- **The scale problem.** 10,000× speed with large populations means up to ~10,000 ticks per real second; naive per-particle per-tick rolls would be N×10⁴ draws/s. This is infeasible at millions of particles, so the Core must be **event-driven where mathematically valid**, designed in from day one:
  - All particles in one component share the same `p`. The first collapse in a component of `n` particles over any tick span is geometric with per-tick probability `1-(1-p)^n`; the victim is uniform among the `n`. Same distribution as per-particle rolls (to be proven and unit-tested against brute force before use).
  - Geometric next-event times are memoryless, so when `p` changes (component merge/split, upgrade, event) the pending sample is discarded and redrawn — still exact.
  - Deterministic production over an event-free span can be applied analytically (rate × ticks) rather than looped.
  - Anything that cannot be proven equivalent stays tick-by-tick, and the device throttles speed honestly instead of changing rules.
  - **Milestone 01 evidence:** equivalence held in a Python reference model (changing p, merges, splits); scaling advantage depends on sparsity of events and vanishes when ~1% of components collapse every tick (MILESTONE_01_REPORT.md). Geometric sampling uses a deterministic ln (`detmath`) so devices cannot disagree (D-009). **Phone data (v0.0.4):** the event heap costs ~0.4-0.7 us/event vs ~1.7 ns per brute-force component roll, so it only wins below ~0.25-0.45% collapses per component per tick; high-hazard components need per-tick rolling (hybrid scheduler, D-012).
- **Catch-up:** each frame the Core is given a bounded work budget; leftover ticks stay owed (shown as "falling behind"), never dropped.
- **Pause / background:** no offline progress. On app close, state is saved; resume restores the same tick.

### 3a. Scheduler design (D-012 revised: wheel for all slots; hot list only above ~3-5% hazard per tick — see report §10.5)
Per component (slot) hazard q per tick. **Cold** slots (q below a threshold `q_star`) live in a hashed timing wheel (power-of-two ring of buckets as intrusive linked lists, plus an overflow list rescanned once per revolution); their next collapse tick is drawn geometrically. **Hot** slots (q >= q_star) sit in a flat list and are rolled every tick with an integer threshold. A slot changes mode only when it is respawned or its q changes, at which point its pending sample is discarded and redrawn (memoryless, so exact). All strategies must agree with per-tick rolling (tests). Expected benefit: O(1) schedule/pop instead of O(log N) heap cost and cache misses; per-tick overhead is constant. Unmeasured on the phone until B2 is run.
### 3b. Clock and worker thread
`Clock` (integer microseconds x speed) decides how many ticks are owed; the simulation runs on a worker thread in small chunks and publishes counters through atomics. The renderer never touches simulation state; it will read snapshots (not yet built, cost measured by button 4).

## 4. Collapse and chains (semantics to be fixed before implementing — see open question Q-2)
Whatever rule is chosen must: process each particle at most once per event, use a deterministic order (sorted by stable key), terminate, update occupancy/components/formations/entropy consistently, and write a log entry.

## 5. Timelines
Occupancy key `(TimelineId, x, y)`. Transfer on Divergence, conflict outcome, and Convergence resolution are specified and tested **before** the visuals (Phase 6). Event durations are in ticks.

## 6. Presentation
Reads `Snapshot` of visible chunks (viewport-culled, LOD by zoom). Total stored ≠ visible ≠ animated; animation has a per-frame budget. High-speed: Events are aggregated into summaries (counts per chunk, throttled audio). Themes via tokens (white default, inverted dark).

## 7. Save system (designed)
Versioned container: header (schema version, seed, tick, RNG states, metadata summary) + per-timeline chunk blocks (compressed) + research/event state. Writes are atomic (temp file → flush → rename), previous generation kept for recovery. Import validates fully into a staging slot before becoming selectable; never overwrites silently. Migrations are pure functions `vN → vN+1`, each with fixture tests.

## 8. Diagnostics
Structured log, tick/throughput metrics, debug overlay, seed + command-log replay tool. Every randomized test prints its seed.

## 9. Repository layout (IMPLEMENTED in v0.1.2; D-003 committed)
```
core/    engine-independent library + tests (headless, runs in CI)
bridge/  thin engine binding (no game rules)
game/    engine project: scenes, scripts (presentation only), themes, audio
tools/   benchmark harness, replay, save inspector
docs/
```
### 9a. Milestone 02a modules (written, not yet compiled)
`core/src`: rng (xoshiro256** + named streams), clock (requested vs effective speed), governor (D-019), wheel (timing wheel), command, snapshot (header; field order append-only), world (World trait, NullWorld, BurnWorld test world), host (worker thread, command channel, snapshot under a short lock), save (container v1). `bridge/src/lib.rs`: Godot class `PlatformCore`, coarse API only. `game/`: 02b shell (`shell.gd`, `world_view.gd`, written not run) and developer checks (`dev_check.gd`).

## 10. Known architectural risks
R1 events-vs-ticks equivalence must be proven; R2 millions-of-particles on mid/low Android devices unproven; R3 binding overhead between Core and engine unmeasured; R4 collapse/timeline rules still ambiguous in the GDD.
