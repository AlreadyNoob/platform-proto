# Project understanding and open questions (2026-11-16)

Confirmed present in the supplied files: GDD v0.1 with Appendix A (original concept transcript) and Appendix B (proposed additions); master prompt v0.1; README.

## One-paragraph understanding
Platform is an offline-less, simulation-first Android game: the player places particles on a large sparse 2D grid; particles produce more each tick; connected groups get riskier (collapse chain reactions); Entropy rises with population and time; at higher Entropy, Convergence/Divergence timeline events occur; research, special particles (Black Holes etc.), grid warping and multiple endings come later. Speed 1×–10,000×, multiple independent save slots, millions of particles as a *benchmarked ambition*, extremely polished 2D animation.

## Genuine ambiguities (GDD does not settle them)
Needed before the stated phase, not before Phase 0 ends.
- **Q-1 — PARTLY ANSWERED 2026-11-19 (D-027):** generation yields both a count and particles that spawn on the grid. Open: the split rule, where spawns appear, blocked spawns.
- **Q-2 — ANSWERED 2026-11-19 (D-025):** chains resolve immediately within the same tick; animated waves are presentation only. Sub-question moved to Q-3.
- **Q-3 — PARTLY ANSWERED 2026-11-19 (D-029): default = GDD example (doubling per linked particle, base 0.1%, cap 10%, configurable).** Second part ANSWERED 2026-11-19 (D-030): each connected neighbour rolls its own chance; the roll uses the neighbour's normal group risk (D-031). Q-3 is fully answered. (original text:)  The GDD example doubles risk per added particle (0.1%, 0.2%, 0.4%, 0.8% … = 2^(n-1) × base), which hits a 10% cap at about 8 linked particles. Confirm that is the intended feel, or choose another curve (configurable either way). Also: does each connected neighbour always collapse during a chain, or roll a chance? (left open by the Q-2 answer, which says a collapse \"may trigger\" connected particles)
- **Q-4 — ANSWERED 2026-11-19 (D-024):** 1 tick = 1 simulation second at 1x.
- **Q-5 (before Phase 6): Timelines.** Meaning of "previous timeline" after repeated Divergences; how the timeline value increments; what "the game starts with Convergence as its default event" means in practice.
- **Q-6 — PARTLY ANSWERED 2026-11-19 (D-028):** placing costs count, and there is also a separate starting budget. Open: what the starting budget is, whether it replenishes, how a placement is paid when both exist.
- **Q-7 — ANSWERED 2026-11-19 (D-026):** mid-range phones of the last 4-5 years, Android 11 minimum, scale quality and capacity to the device, millions of particles = long-term ambition.

## Decision needed now
Approve (or change) the provisional engine/architecture direction in `DECISIONS.md` D-002/D-003 so Milestone 01 can build the validation prototype.

**Update 2026-11-20:** all open GDD questions except Q-5 (timelines) are answered or delegated to per-type tuning (D-037). See `OWNER_DECISIONS.md`.
