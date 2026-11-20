# OWNER_DECISIONS.md — the owner's verbatim answers to GDD questions (authoritative)
`PLATFORM_GDD.md` and `docs/original/` stay byte-identical to the owner's original files (the integrity check enforces it). Answers to the GDD's open questions are recorded here and in `DECISIONS.md`. They take precedence over ambiguity in the GDD.
### Q-4 — Tick duration (decided 2026-11-19)
One tick equals one second of simulation time at 1x speed. Higher speeds multiply simulation time accordingly, up to 10,000x. The simulation must use accumulated simulation time and efficient event processing, not depend on rendering frame rate.
### Q-2 — Collapse propagation (decided 2026-11-19)
Collapse chains resolve immediately within the same simulation tick. When a particle collapses, its collapse may trigger connected particles, which can trigger further connected particles. The chain continues until all resulting reactions have been resolved. The simulation must prevent duplicate processing and infinite reaction loops within a single chain event.
Visually, chain reactions should be presented as animated waves where practical. Animation timing must never determine the actual simulation outcome. At high simulation speeds, visual effects may be simplified or skipped without changing simulation results.

### Q-7 — Device targets (decided 2026-11-19)
Target mid-range Android phones from the last 4–5 years, with Android 11 as the minimum supported version. Optimize for a range of hardware rather than assuming flagship-level performance. Scale particle capacity, rendering quality and visual effects according to device capabilities. Millions of particles remain a long-term performance ambition, not a minimum requirement for every supported device.

### Q-1 — What a particle's per-tick generation produces (answered 2026-11-19, partly)
Question asked: the GDD says a placed particle "generates more particles every tick"; what does that produce? Options offered: new particles on the grid / a count spent to place particles / both / go with a recommendation.
Owner's answer: **Both: it makes a count, and some of it also spawns on the grid.**
Not yet specified (open sub-questions): what share or rule decides which part becomes a count and which spawns on the grid; where grid spawns appear (adjacent cell? random free cell?); whether spawning can be blocked (full grid, occupied cells).

### Q-6 — Placement budget (answered 2026-11-19, partly)
Question asked: is the generated count the same thing as the placement budget (what you spend to place particles by hand)? Options offered: yes / no, separate / partly / go with a recommendation.
Owner's answer: **Partly: placing costs count, but there is also a separate starting budget.**
Not yet specified (open sub-questions): what the separate starting budget is exactly (a number of free placements at the start? a starting amount of count?); whether it is ever replenished; whether placing spends only count once the starting budget is used up, or always both.

### Q-3 — Collapse probability defaults (answered 2026-11-19, first part)
Question asked: the GDD example doubles collapse risk with each linked particle (0.1%, 0.2%, 0.4%, 0.8%... capped at 10%) and calls it illustrative; what should be the starting default? (Everything stays adjustable in the model's settings.)
Owner's answer: **Use it as the default: doubling per linked particle, base 0.1%, cap 10%.**
Still open from Q-3: whether in a chain every connected neighbour always collapses or rolls a chance (asked next).

### Q-3 (second part) — Do linked neighbours always collapse in a chain? (answered 2026-11-19)
Question asked: when a particle collapses, does every linked neighbour always collapse with it, or does each neighbour roll a chance? Options offered: always / each rolls its own chance / depends on type / go with a recommendation.
Owner's answer: **No: each linked neighbour rolls its own chance.**
Not yet specified: which probability that roll uses (asked next).

### Q-3 (third part) — Chance a neighbour rolls in a chain (answered 2026-11-19)
Question asked: when a collapse spreads to a linked neighbour, which chance does that neighbour roll? Options offered: its normal group risk / a separate higher propagation chance / a separate lower one / go with a recommendation.
Owner's answer: **Its normal group risk (the same 0.1% x doubling, capped at 10%).**

### K-45 — Rolls per neighbour in a chain (answered 2026-11-20)
Question asked: in a chain, if a particle touches two collapsing neighbours, how many rolls does it get? (The code did one roll per chain.) Options offered: one roll per chain / one roll per exposure / go with a recommendation.
Owner's answer: **One roll per chain: a neighbour rolls once, however many collapsed neighbours it touches.**

### Q-1 follow-up — Where generated particles spawn (answered 2026-11-20)
Question asked: when a particle spawns new particles on the grid, where do they appear? Options offered: next to the generating particle in a random free neighbouring cell / anywhere on the grid in a random free cell / depends on particle type / go with a recommendation.
Owner's answer: **Depends on particle type (decide per type later).**

### Q-6 follow-up — What the separate starting budget is (answered 2026-11-20)
Question asked: what is the separate starting budget? Options offered: a fixed number of free placements at the start of a run / a starting amount of count in the same currency / free placements of one starter particle type only / go with a recommendation.
Owner's answer: **A fixed number of free placements at the start of a run.**

### Q-6 follow-up — Do free placements come back? (answered 2026-11-20)
Question asked: do free placements ever come back after you've used them? Options offered: no, only at the start of a run / yes, refunded when a particle collapses / yes, upgrades or events can grant more / go with a recommendation.
Owner's answer: **Yes: upgrades or events can grant more.**
(Refunds on collapse were not selected; nothing is decided about them.)

### Economy leftovers — how to handle them (answered 2026-11-20)
Open details: the split between count and grid spawns, what happens when a spawn is blocked, and whether free placements or count are used first. Question asked: decide each now, or build them as adjustable settings?
Owner's answer: **Build them as per-type settings with placeholders; I'll tune later.**

### Milestone 03d — how should particles be placed? (answered 2026-11-20)
Question asked: how should you place particles in 03d? Options offered: tap places one particle / tap and drag paints several / your recommendation.
Owner's answer (verbatim): **Option 3 — Hybrid placement. Support both single-tap placement and tap-and-drag continuous placement. Single tap places one particle. Dragging places particles across eligible grid cells, without duplicating placements or overwriting occupied cells. Provide clear visual previews, immediate particle-counter updates and undo support. Special entities such as Black Holes should have dedicated placement interactions appropriate to their size and behavior. Make tap placement the default, with drag placement available as a selectable mode.**
