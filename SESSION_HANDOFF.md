# SESSION_HANDOFF (v0.2.11, 2026-11-20) — rewritten at every milestone/checkpoint, never appended
**Start with `START_HERE.md`.**

**State:** 03a-03d are tested and pushed (repo commit 118c842; the repo is public, CI works). Owner verified on the phone: saves, placement, collapse, the paint-mode fix, undo, ghost colours, collapse ring, high speed. Not seen: the "Not enough" message (game was too easy). Autosave is written on background but not loaded on start (K-60, by design so far). **03e (v0.2.11) is WRITTEN, NOT COMPILED:** the starter particle (cost 60, 1 count/tick, 1% spawn/tick, all PLACEHOLDERS) and the dev game start (20 free + 120 count), 5 new tests (70 expected), `tools/ref/balance_ref.py`. Owner answers behind it: the starter makes count and can spawn; it should earn back its cost in about 1 minute at 1x (D-046).

**Waiting for:** the owner to run `sh tools/RUN_M03E_TERMUX.sh` and upload `test_out_m03e.txt`, `python_ref_m03e.txt`, `balance_report_m03e.txt`; then push, wait for CI (the Rust and bridge changed, a new `.so` is needed), update the Godot project and run the 03e checklist (TESTING.md).

**Balance finding (K-61):** a player who reinvests everything doubles income about every minute; the collapse rule hardly limits a spread-out player. Do not present this as a decision; the owner tunes after playing.

**Next action:** fix whatever the first compile finds; record results and the owner's feel; tune placeholders with the owner. Open items: tuning of all placeholders, autosave on start (K-60), Black Hole placement (K-55), viewport chunks (K-52), event-driven collapse engine (K-44), entropy, timelines (Q-5), upgrades, events.

**Do not:** overlay an older ZIP onto the owner's repo; present placeholders as decisions; claim 03e compiles or plays well before the owner reports it.
