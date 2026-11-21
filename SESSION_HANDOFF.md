# SESSION_HANDOFF (v0.2.12, 2026-11-20) — rewritten at every milestone/checkpoint, never appended
**Start with `START_HERE.md`.**

**State:** 03a-03d are tested and pushed (repo commit 118c842; the repo is public, CI works). Owner verified on the phone: saves, placement, collapse, the paint-mode fix, undo, ghost colours, collapse ring, high speed. Not seen: the "Not enough" message (game was too easy). Autosave is written on background but not loaded on start (K-60, by design so far). **03e (v0.2.12) is WRITTEN, NOT COMPILED:** v0.2.11 (starter particle: cost 60, 1 count/tick, 1% spawn/tick; dev start 20 free + 120 count) passed 70/70 on the phone, and the owner found it impossible to lose. v0.2.12 adds the owner-chosen rising cost (D-047: cost + cost * particles / 10, so the 21st placement costs 180), the `next_cost` stat, a Cost label and a corrected ghost colour; 71 tests expected. All numbers are PLACEHOLDERS.

**Waiting for:** the owner to run `sh tools/RUN_M03E_TERMUX.sh` (expect 71 tests) and upload `test_out_m03e.txt`, `python_ref_m03e.txt`, `balance_report_m03e.txt`; then push, wait for CI (the Rust and bridge changed, a new `.so` is needed), copy the `.so` AND `game/shell.gd` into the Godot project and run the 03e checklist (TESTING.md).

**Balance (K-61):** a lower spawn chance does not help; the count loop (income pays for more particles) is the cause. Rising cost slows growth in simulation (about 1,190 particles per hour for a spread-out player, 140 for a packed one) but nobody goes extinct. Other levers if still too easy: falling income per particle, harsher collapse (owner's D-029 numbers). Do not present placeholders as decisions.

**Next action:** fix whatever the first compile finds; record results and the owner's feel; tune placeholders with the owner. Open items: tuning of all placeholders, autosave on start (K-60), Black Hole placement (K-55), viewport chunks (K-52), event-driven collapse engine (K-44), entropy, timelines (Q-5), upgrades, events.

**Do not:** overlay an older ZIP onto the owner's repo; present placeholders as decisions; claim 03e compiles or plays well before the owner reports it.
