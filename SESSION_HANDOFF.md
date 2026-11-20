# SESSION_HANDOFF (v0.2.7, 2026-11-20) — rewritten at every milestone/checkpoint, never appended
**Start with `START_HERE.md`.**

**State:** Milestone 02 complete. Milestones 03a (collapse core) and 03b (economy core: count ledger, free placements, per-type settings, generation with spawns) TESTED on the owner's phone: 44/44 tests, all fixtures identical to the independent Python references. All GDD questions answered or delegated except Q-5 timelines. Economy numbers are placeholders (`docs/TUNING_PLACEHOLDERS.md`). Nothing is wired to host/bridge/UI; world state is not saved (K-48).

**Waiting for:** the owner to overlay v0.2.7 on the repo (run `git status --short` first and paste it; the repo may still be at commit 63ee843/214d895) and push; CI should be green.

**Next action:** plan 03c: save v2 with world state + migration (K-48: cells, particle types, count, free placements, RNG states), then wiring `ParticleWorld` into host/bridge/game (placement UI with economy, wave presentation from the chain log); event-driven engine later (K-44); K-41/K-42/K-43 device-target follow-ups.

**Do not:** overlay an older ZIP onto the owner's repo (happened once, reverted in 214d895); present placeholders as decisions; start Q-5 timelines work before asking.
