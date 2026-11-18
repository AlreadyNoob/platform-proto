# PLATFORM
## Game Design Document (GDD)
**Document version:** 0.1 — Foundation specification  
**Status:** Pre-production / living document  
**Primary platform:** Android  
**Primary orientation:** Landscape  
**Visual format:** 2D top-down simulation

---

## 1. Product vision

**Platform** is a highly systemic 2D particle-and-universe simulation. The player begins with a small number of particles on a vast grid, develops increasingly intricate formations, manages instability and Entropy, experiences timeline events, researches new physical behaviors, and eventually manipulates gravity, space, and multiple timelines.

The game should feel like a scientific instrument for constructing and observing an evolving universe. Its complexity should come from interacting, understandable systems—not arbitrary menus or needless bookkeeping.

### Design pillars

1. **Simulation first:** Displayed numbers, events, probabilities, and animations must reflect the actual simulation state.
2. **Emergent complexity:** Systems should interact in surprising but explainable ways.
3. **Readable technical depth:** Advanced information is available on inspection, while the default play surface remains legible.
4. **Expressive 2D presentation:** A restrained interface with excellent animation, shader effects, motion design, and sound.
5. **Long-term experimentation:** Multiple saves, branching experiments, discoveries, milestones, and multiple endings.
6. **Engineering integrity:** Small, tested, documented increments are preferred over rushed feature volume.

## 2. Platform, camera, and presentation

- Android is the first target.
- Landscape is the primary orientation.
- Keep the architecture responsive enough to support portrait mode later; portrait is not a launch requirement.
- Gameplay is strictly 2D top-down. Do not build a 3D world or use 3D gameplay as a substitute for 2D effects.
- Main theme: white/off-white scientific workspace, subtle grid, dark text, restrained accent colors.
- Optional inverted/dark theme: near-black background and appropriately inverted UI.
- Use color sparingly and semantically. Color should communicate particle class, state, timeline, or event—not decorate everything.
- The board should support pan, zoom, selection, placement, and contextual inspection.
- UI panels must adapt to different Android aspect ratios and safe areas.

### Animation direction

Every meaningful interaction should have a designed response:
- placement: scale-in, soft ripple, brief grid response;
- generation: split, emerge, or transfer animation;
- linking: a pulse travels along the newly established connection;
- formation discovery: outline traces the shape, then settles into a subtle persistent indicator;
- collapse: a clear warning/trigger sequence followed by a readable propagation wave;
- timeline shift: layered ghosting, offset outlines, and a controlled visual transition;
- Convergence: layers draw together and resolve;
- Divergence: the active layer separates from the previous one;
- grid expansion: boundary grows outward with a spatial ripple;
- curvature: grid lines bend smoothly around the source;
- Black Hole: particles orbit, distort, and disappear into a 2D event-horizon effect;
- research/unlock: restrained, distinctive confirmation animation.

Animations must never obscure critical information or make the game unresponsive. Provide settings for animation intensity, reduced motion, visual effects quality, and sound volume.

## 3. Core gameplay loop

1. Start a universe with a limited placement budget.
2. Place normal particles on valid grid cells.
3. Particles produce additional particles on simulation ticks.
4. Adjacent particles form connected components.
5. Specific geometric patterns and structures grant bonuses.
6. Larger or more connected structures become increasingly unstable.
7. Global Entropy rises through particle population and elapsed in-game simulation time.
8. Entropy and other conditions influence anomalies and major events.
9. Earn resources and research to unlock particle types, structures, upgrades, and physical constants.
10. Expand and manipulate the grid, discover new interactions, and explore timeline mechanics.
11. Create, duplicate, export, and import independent universe saves.

This is not an offline idle game. The simulation progresses only while the game is actively open and running. Pausing stops simulation time. Returning to the game after it has been closed must not grant offline production.

## 4. Grid and spatial model

The grid begins as a manageable region and can expand, contract, and distort through upgrades and events.

### Required spatial capabilities

- Sparse representation: do not allocate an object for every empty cell in a huge or effectively unbounded grid.
- Chunk-based storage and loading.
- Stable integer or fixed-point logical coordinates, separate from screen coordinates.
- Camera pan and zoom with smooth interpolation.
- Grid expansion and contraction.
- Spatial distortion and curvature around special entities.
- Regions with distinct physical properties, introduced progressively.
- Multi-cell occupancy for special entities such as Black Holes.
- Clear rules for placement, overlap, occupancy, and collision between entities.

### Grid design rule

The logical simulation grid must remain authoritative. Visual bending or distortion changes how the grid is drawn and how spatial rules are applied, but must not silently corrupt coordinates or save data. Any mechanic that changes logical adjacency must be explicit, deterministic, and testable.

## 5. Particles and connectivity

### Normal particle

The initial particle is the foundational generator. It has:
- production behavior per tick;
- a base collapse probability;
- adjacency and connection membership;
- a timeline;
- age and state;
- any relevant formation membership.

Do not expose internal database-style IDs as a prominent player-facing mechanic. Internal identifiers may be used for reliable save references, history, and simulation bookkeeping. Player-facing inspection should show meaningful properties such as type, position, timeline, age, production, stability, and recent events.

### Connected components

Adjacent particles form connected groups according to a documented adjacency rule. The initial default should be orthogonal adjacency (up, down, left, right), unless a later upgrade or particle explicitly enables diagonal or nonlocal links.

When a particle is added or removed:
- update the affected connected component(s);
- recalculate relevant component statistics;
- update collapse-risk rules;
- update any formations that depend on the changed neighborhood.

Avoid recalculating the entire universe after every placement. Use incremental connectivity updates where safe, with periodic validation tools.

### Collapse probability

The original concept proposes that a particle's collapse risk increases as more particles join its linked structure. The example doubling rule is illustrative, not yet a final formula.

Claude must not silently choose a balancing formula. Implement the rule through a configurable, documented probability model. The model must specify:
- base per-tick collapse probability;
- how component size and/or links modify risk;
- maximum risk cap;
- modifiers from particle type, upgrades, events, and timeline;
- whether a collapse check is per particle, per component, or another explicitly defined unit;
- how random outcomes are generated and saved.

Use numerically stable probability calculations. At high simulation speeds, do not incorrectly treat a per-tick probability as a per-frame probability.

### Collapse and chain reactions

A collapse may affect a particle and trigger further collapses among eligible linked particles. Define propagation order and event semantics carefully. The simulation must avoid accidental duplicate processing, infinite loops, and order-dependent bugs.

A collapse should be visually legible:
1. identify the triggering particle;
2. show a short warning or pulse if the speed setting permits;
3. propagate through the affected structure;
4. update the logical state;
5. update formations, production, and Entropy;
6. record the event in the simulation log.

At extreme speed, the simulation may batch visual events, but the underlying state transitions must remain correct.

## 6. Formations and structures

Specific patterns provide multipliers or unlocks. Initial candidates:
- line;
- square;
- triangle;
- rectangle;
- ring;
- hexagonal arrangement;
- nested/recursive structures;
- symmetric formations;
- repeated or mirrored formations.

Formations may overlap. A particle can participate in multiple structures. Removing one particle can invalidate several formations at once.

Each formation definition should specify:
- shape/topology;
- rotation and reflection handling;
- coordinate matching rules;
- minimum and maximum size;
- production multiplier;
- Entropy or instability effect;
- unlock or discovery condition;
- visual signature.

Do not implement every proposed formation at once. Start with a small, testable set and add more through milestones.

## 7. Entropy

Entropy is a global percentage-like meter beginning at 0.0%, displayed with decimal precision.

It rises from:
- particle population;
- elapsed in-game simulation time;
- selected particle behaviors;
- certain events and structures.

Its exact equation must be configurable and documented. Avoid an arbitrary formula hidden in UI code.

Entropy influences the probability or eligibility of anomalies and major events. Initial example bands may be used for prototyping, but final thresholds and probabilities require playtesting.

### Entropy Debt (proposed feature)

Some effects may temporarily suppress Entropy while creating a debt that returns over in-game time. This is a proposed advanced mechanic, not a launch requirement. Its rules must be clear and visible to the player.

## 8. Timelines: Convergence and Divergence

Timeline mechanics are a defining feature and must be implemented with explicit rules.

### Timeline representation

- The original timeline is represented by 0.00.
- Divergence creates or moves the active state to a new timeline value.
- The original concept uses a value between 0 and 1 with up to two decimal places.
- Display precision and internal identity should be separated: two distinct timelines must never accidentally become the same timeline merely because their display values round to the same number.
- Internally, use stable timeline references. In the UI, show readable timeline labels and values.

### Divergence

During Divergence:
- a particle whose collapse would normally occur may instead transfer to a previous timeline;
- the particle ceases participating in the current timeline's connected chain;
- the vacated cell becomes available in the active timeline, subject to occupancy rules;
- multiple timelines may contain particles at the same displayed coordinates;
- occupancy is evaluated per timeline and according to explicit cross-timeline interaction rules;
- collisions or conflicts when transferring into an occupied cell must have a defined outcome.

Divergence lasts for a duration measured in simulation time, not wall-clock time. Simulation speed affects how quickly the duration elapses.

### Convergence

Convergence brings timelines or layers together. Define:
- which timelines are eligible;
- how coincident particles interact;
- how identical structures combine;
- what happens to incompatible particles;
- how occupied-cell conflicts resolve;
- whether the event can be interrupted or modified.

### Additional proposed timeline events

- Temporal Echo: a previous state briefly reappears.
- Timeline Fracture: one timeline branches into several.
- Reconciliation: two timelines begin merging.
- Multiversal Synchronization: structures in separate timelines temporarily interact.

These are future candidates, not all initial-release features.

## 9. Particle families and special entities

Proposed candidates, to be introduced progressively:

| Type | Proposed identity |
|---|---|
| Normal | Foundational particle generator |
| Neutrino | Stable, low-interaction particle; may stabilize neighbors |
| Plasma | High production and elevated local instability |
| Chronon | Alters local timing or tick behavior |
| Entropic | High production with increased Entropy contribution |
| Crystalline | Stabilizes or locks structures; may shatter |
| Black Hole | Multi-cell gravitational entity that consumes particles |
| Supermassive Black Hole | Advanced result of Black Hole interactions |

Each type needs a complete specification: placement cost, occupancy, production, adjacency behavior, collapse behavior, timeline behavior, visual treatment, audio cues, and upgrade interactions.

### Black Holes

Black Holes are special multi-cell entities, not ordinary particles. Candidate properties:
- consume particles within a defined radius or region;
- bend the displayed grid;
- affect local movement or production;
- possess an event-horizon boundary;
- interact with other Black Holes;
- potentially merge into a Supermassive Black Hole.

All spatial effects must remain 2D. Use shaders, curves, particle trails, and carefully designed sprite/mesh effects rather than 3D gameplay.

## 10. Research, upgrades, and universal constants

### Research

Research unlocks new mechanics and discoveries. Some research should require demonstrated formations, events, or interactions rather than only currency.

Candidate research:
- Quantum Entanglement;
- Dimensional Anchoring;
- Gravitational Manipulation;
- Temporal Mechanics;
- Entropy Suppression;
- Recursive Structures;
- Multiversal Synchronization.

### Universal Constants

Players may eventually modify selected rules of the simulated universe:
- gravitational strength;
- temporal behavior;
- Entropy coefficient;
- quantum stability;
- spatial curvature;
- causality;
- dimensional permeability.

Constants should create trade-offs. Increasing one value may strengthen one system while destabilizing another. All effects must be visible and testable.

## 11. Events and discovery

Candidate events:
- Vacuum Fluctuation: spontaneous particle appearance.
- Spatial Instability: a region temporarily changes spatial behavior.
- Particle Genesis: discovery or emergence of a new particle type.
- Void Incursion: a region becomes temporarily inaccessible.
- Resonance Cascade: multiple formations synchronize and gain temporary effects.
- Convergence and Divergence: core timeline events.

Events should have:
- eligibility conditions;
- probability or scheduling rules;
- simulation-time duration;
- clear player-facing explanation;
- deterministic event records for debugging;
- visual and audio treatment;
- interaction rules with other simultaneous events.

The Codex records discovered particles, formations, events, constants, anomalies, and research. Unknown entries can remain hidden until discovered.

## 12. Observer effects and particle memory

Advanced candidate systems:
- Observer Effect: selected particles may behave differently when inspected, if this is implemented as a clear game rule rather than a misleading visual trick.
- Particle Memory: particles accumulate a history of survival, collapse exposure, or unusual events and may develop traits.

These features must be introduced only after the foundational simulation is stable.

## 13. Time and simulation

- Simulation speed range: 1x to 10,000x.
- Pause must stop simulation time.
- All durations and timed events use simulation time.
- Closing the game stops progression; no offline production.
- Returning after closure resumes from the last saved simulation state.
- High speed must be practical and correct.

### High-speed requirements

The renderer must not attempt to draw every simulation tick. Separate simulation updates from visual frame updates. Use a fixed-step or otherwise deterministic simulation clock, bounded work per frame, event batching, and carefully designed catch-up processing.

Do not skip state transitions merely to make the speed control appear fast. If the device cannot keep up, expose honest performance feedback and safe speed limits rather than silently changing game rules.

## 14. Interface and information design

The interface is intentionally technical, but should not be needlessly difficult to operate.

Main screen should provide:
- large central grid;
- particle placement tools;
- current resource/particle totals;
- Entropy meter with decimal precision;
- active timeline indicator;
- simulation speed and pause controls;
- access to research, upgrades, Codex, logs, and saves.

Advanced inspection should be contextual. Tapping a particle or structure opens a panel with relevant information, not a wall of irrelevant data.

### Simulation log

A searchable/filterable log should explain important state changes:
- particle placed;
- structure discovered;
- production event;
- collapse and chain reaction;
- timeline transfer;
- event triggered or ended;
- research unlocked;
- grid changed.

The log is a debugging aid and a player-facing explanation system. It must report actual events, not invented narrative.

## 15. Audio

Include interaction-focused sound design:
- placement click/soft tone;
- generation chime;
- connection pulse;
- formation discovery;
- collapse and chain reaction;
- timeline transition;
- Black Hole ambience;
- research unlock;
- subtle ambient soundtrack.

Provide independent controls for master volume, music, sound effects, and ambient audio. Avoid repetitive or fatiguing sounds during high-speed simulation. Audio can be summarized or throttled while the simulation is accelerated.

## 16. Save system

Save slots represent independent universes. Each universe has its own:
- simulation state;
- particles and structures;
- timeline state;
- research and upgrades;
- grid state;
- relevant settings;
- history and discoveries, as defined by game rules.

Required features:
- autosave enabled by default;
- manual save;
- create unlimited save slots, subject to available device storage;
- duplicate a save to create a safe experimental branch;
- rename and delete saves;
- import and export backups;
- validate imported data before loading;
- preserve save compatibility through versioned schemas and migrations;
- never overwrite an original save during import or duplication without explicit confirmation.

Use atomic or recoverable save writes. Handle app interruption, low storage, corrupt files, and failed imports safely. Export format should be documented and versioned. Consider compression for large universes.

## 17. Performance and scale

Long-term aspiration: support universes containing millions of stored particles across multiple timelines on capable devices. This is an ambition, not a guaranteed launch specification.

Separate:
- total stored particles;
- active simulation particles;
- visible particles;
- currently animated particles.

Potential techniques:
- sparse/chunked grid storage;
- compact data-oriented particle arrays;
- stable references rather than one heavyweight object per particle;
- incremental connectivity and formation updates;
- event-driven scheduling where mathematically valid;
- deterministic random number generation;
- background workers for independent tasks;
- GPU batching and instancing;
- viewport culling;
- level-of-detail rendering;
- animation throttling for distant/hidden particles;
- memory budgets and graceful degradation.

Never claim a particle count or frame rate without a reproducible benchmark on specified Android hardware. Establish performance tiers and test on lower-end as well as stronger devices.

## 18. Multiple endings and continued play

The game should have multiple possible endings or universe-changing outcomes. Endings should emerge from meaningful player decisions and simulation conditions, not just arbitrary timers.

After an ending, allow the player to:
- review the universe's history;
- preserve/export the completed universe;
- continue where appropriate;
- start a new universe;
- unlock discoveries or alternate starting conditions if supported by the design.

## 19. Accessibility and quality settings

Include:
- white and inverted dark themes;
- animation intensity;
- reduced-motion option;
- visual-effects quality;
- sound/music/effects volume;
- text scaling;
- touch target sizing;
- optional confirmation for destructive actions;
- performance mode;
- clear pause and speed indicators.

## 20. Development principles

1. Do not implement the whole game in one pass.
2. Build in small, independently testable milestones.
3. Do not add placeholder buttons that look functional but do nothing.
4. Do not fake simulation outcomes in the renderer.
5. Do not silently invent balancing rules; document provisional assumptions.
6. Keep simulation logic independent of UI and rendering.
7. Preserve working systems; refactor only with a clear reason and tests.
8. Keep save compatibility in mind from the beginning.
9. Keep project documentation current.
10. Deliver a complete project ZIP at every milestone.
11. Clearly state what was built, tested, not tested, and remains incomplete.
12. The user is the product owner. Ask only questions that block a significant decision; otherwise document a reversible assumption and proceed.

---

# APPENDIX A — Original concept reference

The following is the original concept supplied by the creator. It is preserved as a reference and should not be treated as a complete technical specification. Where the expanded GDD clarifies or evolves a rule, the implementation team should preserve the original intent while documenting the chosen formal rule.

Give me an Android game idea prompt to Claude I'm going to make. I have a foundation for it. It's going to be called "Platform".

In the screen, you have a massive white space with grids where you place "particles". 

At the very start, you have a set amount of particles you can place on the grid. When placed, it generates more particles every "tick". When placing on very specific patterns/links, you get multipliers.

But each particle on the grid has a risk (very small chance) of collapsing in that space and basically destroying itself. Any adjacent particles to it will also begin to collapse, which can cause chain reactions to patterned/linked particles.

This collapse chance increased the more particles are linked together.

For example, let's assume a 4x4 grid JUST as an example:

Suppose you have:

a1, b1, c1, d1
a2, b2, c2, d2
a3, b3, c3, d3
a4, b4, c4, d4

(The numbers and letters are just representations for this example, and all numbers/percentages/multipliers below are also just representations.)

Let's say you place a particle at c3, and it has a collapse chance of 0.1% every tick. If you place an adjacent particle, at b3. Now both c3 and b3 has a collapse chance of 0.2% (doubled). Place another adjacent to EITHER c3 OR b3, and you increase the chances of ALL of them since they're all linked together.

So if you have:

a1, b1, p, d1
a2, b2, p, d2
a3, p, p, d3
a4, b4, c4, d4

The occupied ones (c1, c2, b3, and c3) now ALL have 0.8% chance of collapse (going by our earlier doubling logic). If you continue linking, the chance of collapse just keeps increasing until 10% or another cap (which can be increased by certain events).

There's an "Entropy Meter", it defines a percentage. It starts at 0.0% (have this with decimals), and increases the more particles you have, but also very slightly as you play the game and in-game time passes.

Time can be sped up 1x to 10,000x. A high speed up should be practical for this game (this is that type of game where you really need to focus on the long term).

As Entropy increases, certain events have a chance of occuring. Notably, Convergence and Divergence. The "default" event the game starts with is actually Convergence. On the other hand, Divergence causes particles to prevent collapsing (when the collapse chance triggers). Instead of collapsing (they can no longer cause chain reactions to linked particles; all particles are independent now), they are now in the previous timeline and the timeline indicator changes.

There's a timeline indicator below the Entropy meter, which is a number between 0 to 1 (allowing up to two decimal places). Whenever Divergence occurs, the timeline changes immediately. 0 is the original timeline, and the timeline indicator increments slightly.

For example you were at 0 and now Divergence happens and now you're at 0.19. That means you're in a different timeline. Whenever particles are in a different timeline, you can place a particle on an already occupied, different timeline particle, which also has a chance of "collapsing", which, since we're in Divergence, they'll now be in the previous timeline (original 0).

Let's say you're at timeline 0 and placed a particle at X. Divergence happens and now you're in 0.19. Then, the particle at X "collapsed". But since we're in Divergence, the particle at X is now at timeline 0 (previous timeline). Now the grid space there becomes unoccupied IN timeline 0.19, so you can place a particle there. So, now there's two particles in one grid space, from differing timelines. If you place a particle at X and it collapsed WHILE Divergence is still happening, then it goes to timeline 0, but now there can't be two particles on the same grid space on timeline 0, can't there? So now, the particles at X immediately collapses and all linked chains AT timeline 0 linked to X.

Divergence only lasts for a while (in-game time). Everything should happen in-game time so you can speed up time significantly.

Now, what's the goal of the game? Well, for now, you'd have to accumulate particles and also buy upgrades. You could place different particles which could have different interactions. Or you could place a "Black Hole" that occupies a large space and consumes other particles and bends the space-time grid. Two "Black Hole"s could then collide with each other to form a Supermassive Black Hole and that could have certain interactions.

For now, I want you to give me more ideas first because I'm running out of time and I can't think anymore.

For some personal background, I actually just want to make the game very complex to play, like I'm the only one who can understand it. I just wanna show off to my classmates or the public and maybe someone will ask me: "Yo cool game what game is it?" and I can say I made it and they won't believe it. I want the game with the coolest animations possible.

---

# APPENDIX B — Proposed ideas added during design

The following are expansions, not claims that they were part of the original concept:
- Neutrino, Plasma, Chronon, Entropic, and Crystalline particles.
- Square, triangle, hexagonal, and recursive formations.
- Entropy Debt.
- Reconciliation, Temporal Echo, and Timeline Fracture.
- Black Hole upgrade progression and Supermassive Black Hole behavior.
- Theoretical Research and Universal Constants.
- Vacuum Fluctuation, Spatial Instability, Particle Genesis, Void Incursion, and Resonance Cascade.
- Observer Effect and Particle Memory.
- Codex and detailed simulation log.
- Multiple endings and post-ending continuation.

All of these are candidates subject to staged design, testing, and balancing.
