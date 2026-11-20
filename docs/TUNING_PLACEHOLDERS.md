# TUNING_PLACEHOLDERS.md — every number that is NOT yet a decision (D-037)
Rule: a value is either **DECIDED** (owner), **Claude's reversible choice** (listed in DECISIONS) or a **PLACEHOLDER** (the owner will tune it after seeing the game run). Nothing here is balancing. Test scenarios use their own values (core/tests, tools/ref); those are test data, not tuning.

| Parameter | Current value | Status | Where |
|---|---|---|---|
| Collapse base risk per tick | 0.1% (1000 ppm) | DECIDED (owner, Q-3, D-029): default, adjustable | `CollapseModel::default` |
| Collapse risk growth | doubles per linked particle | DECIDED (owner, D-029) | `CollapseModel::risk_p32` |
| Collapse risk cap | 10% (100000 ppm) | DECIDED (owner, D-029): default, adjustable | `CollapseModel::default` |
| Chain neighbour roll | its normal group risk; one roll per chain | DECIDED (owner, D-031, D-033) | `run_chain` |
| Adjacency | orthogonal (4 neighbours) | DECIDED (D-007) | `NEIGHBOURS` |
| Tick length | 1 sim-second at 1x | DECIDED (owner, D-024) | `clock.rs` |
| Group size timing | fixed at the start of the tick | Claude's reversible choice (D-032) | `step_tick` |
| Placement cost, STARTER (count) | 60 (`TypeConfig::starter`, D-046); the older `placeholder` type keeps 1 for tests | PLACEHOLDER | `TypeConfig.cost` |
| Count produced per particle per tick, STARTER | 1 (pays back in 60 s at 1x; owner: about 1 minute, D-046) | PLACEHOLDER | `TypeConfig.count_per_tick` |
| Count / grid-spawn split rule | each tick: always add `count_per_tick`, plus an independent roll of `spawn_p32` for one spawn | PLACEHOLDER rule (owner: "both", D-027) | `generate` |
| Spawn chance per tick, STARTER | 1% (owner: small chance, D-046); the older `placeholder` type: 0 | PLACEHOLDER | `TypeConfig.spawn_p32` |
| Spawn placement rule | random free orthogonal neighbour, every type | PLACEHOLDER (per type later, D-034) | `generate` |
| Spawned particle's type | the spawner's own type | PLACEHOLDER | `TypeConfig.spawn_type` |
| Blocked spawn (no free neighbour) | spawn is lost (`Skip`); alternative `ConvertToCount(n)` | PLACEHOLDER (per type, D-037) | `TypeConfig.blocked` |
| Payment order | free placement first, then count | PLACEHOLDER (per type, D-037) | `TypeConfig.free_first` |
| Starting free placements | core default 0; the DEV GAME starts with 20 so something can be placed (D-035 leaves the number to the owner) | PLACEHOLDER | `game::DEV_START_FREE` |
| Starting count | 0 in tests; the DEV GAME starts with 120 (03e, `game::DEV_START_COUNT`) | PLACEHOLDER | `ParticleWorld::set_count` |
| Free placements granted by upgrades/events | mechanism exists (`grant_free_placements`); no upgrades or events yet | DECIDED in principle (D-036) | `ParticleWorld` |
| Refund when a particle collapses | none | NOT decided | — |
| Undo history length / rules | 256 placements, per stroke, exact refund, no refund for collapsed particles | Claude's reversible choice (D-043) | `game::UNDO_LIMIT` |
| Cell feed cap / refresh | 20,000 particles, every 30 ms at most | PLACEHOLDER (until viewport chunks) | `host::MAX_VIEW_CELLS`, `VIEW_MIN_INTERVAL_MS` |
| Wave animation | skipped above 100x; 0.07 s per depth, 0.55 s life, 400 effects max | PLACEHOLDER (presentation only) | `shell.gd`, `world_view.gd` |
