# SESSION_HANDOFF (v0.2.8, 2026-11-20) — rewritten at every milestone/checkpoint, never appended
**Start with `START_HERE.md`.**

**State:** Milestone 02 complete. 03a (collapse) and 03b (economy) TESTED on the owner's phone and pushed to the repo (commit 8b93e8d). 03c (save schema 2 with the whole world, export/restore, migration 1->2) WRITTEN; Python reference passes; Rust not yet compiled. GitHub Actions did not start for the last push (billing, K-50), so no CI result exists for 8b93e8d. All GDD questions answered or delegated except Q-5 timelines. Nothing is wired to host/bridge/UI yet.

**Waiting for:** the owner to run `sh tools/RUN_M03C_TERMUX.sh` (expect 52 tests, 7 fixtures identical) and upload `test_out_m03c.txt` + `python_ref_m03c.txt`; and to look at GitHub billing (K-50).

**Next action:** record results, fix failures; then 03d: wire `ParticleWorld` + save v2 into host/bridge/game (placement UI with the economy, chain-wave presentation, world save/load). The bridge/game need the Android `.so` rebuilt, which needs CI/billing (or another way to build it).

**Do not:** overlay an older ZIP onto the owner's repo (happened once, reverted in 214d895); claim 03c works before the phone output says so; present placeholders as decisions.
