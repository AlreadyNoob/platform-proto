# Project understanding and open questions (2026-11-16)

Confirmed present in the supplied files: GDD v0.1 with Appendix A (original concept transcript) and Appendix B (proposed additions); master prompt v0.1; README.

## One-paragraph understanding
Platform is an offline-less, simulation-first Android game: the player places particles on a large sparse 2D grid; particles produce more each tick; connected groups get riskier (collapse chain reactions); Entropy rises with population and time; at higher Entropy, Convergence/Divergence timeline events occur; research, special particles (Black Holes etc.), grid warping and multiple endings come later. Speed 1×–10,000×, multiple independent save slots, millions of particles as a *benchmarked ambition*, extremely polished 2D animation.

## Genuine ambiguities (GDD does not settle them)
Needed before the stated phase, not before Phase 0 ends.
- **Q-1 (before Phase 2): What does "generates more particles every tick" mean?** New placed-on-grid particles (where?) or a resource count used to place particles? This changes the whole data model.
- **Q-2 (before Phase 3): Collapse propagation.** "Adjacent particles begin to collapse": instant flood-fill, or a wave spreading over several ticks? Does each neighbor always collapse or roll a chance?
- **Q-3 (before Phase 3): Collapse unit and formula.** The GDD example doubles risk per added particle (0.1%, 0.2%, 0.4%, 0.8% … = 2^(n-1) × base), which hits a 10% cap at about 8 linked particles. Confirm that is the intended feel, or choose another curve (configurable either way).
- **Q-4 (before Phase 2): Tick length.** What is 1 tick at 1× in seconds? At 10,000× this sets the workload (see ARCHITECTURE §3). Working assumption A-1: 1 tick = 1 sim-second at 1× (PROVISIONAL).
- **Q-5 (before Phase 6): Timelines.** Meaning of "previous timeline" after repeated Divergences; how the timeline value increments; what "the game starts with Convergence as its default event" means in practice.
- **Q-6 (before Phase 2): Placement budget.** How the starting budget is replenished and whether it is the same thing as the particle count.
- **Q-7 (before Phase 1): Device targets.** Minimum Android version and the lowest-end device the game should still run on.

## Decision needed now
Approve (or change) the provisional engine/architecture direction in `DECISIONS.md` D-002/D-003 so Milestone 01 can build the validation prototype.
