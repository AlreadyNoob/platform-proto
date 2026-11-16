# DECISIONS log
Format: ID · date · decision · alternatives · rationale · consequences · reversible?

| ID | Date | Status |
|---|---|---|
| D-001 | 2026-11-16 | ACCEPTED (process) |
| D-002 | 2026-11-16 | APPROVED for investigation (owner, 2026-11-17); NOT a final commitment |
| D-003 | 2026-11-16 | APPROVED for investigation (owner, 2026-11-17); Rust leading, alternatives open |
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
**D-012 PROPOSED — hybrid scheduler.** Low-hazard components in an event queue (evaluate timing wheel vs binary heap), high-hazard components rolled per tick; component moves between modes as its hazard changes. Evidence: F-6 (phone crossover ~0.25-0.45% collapse/tick per component). Needs an equivalence test and a benchmark before acceptance. Not built.
