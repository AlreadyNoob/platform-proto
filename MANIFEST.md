# Release notes — Platform_v0.2.9_milestone-03d_placement.zip
Complete project snapshot (v0.2.9) = the repo at v0.2.8 (03a/03b/03c, tested on the phone) + Milestone 03d: `core/src/game.rs`, `core/tests/game.rs`, extended `command.rs`, `snapshot.rs`, `world.rs`, `host.rs`, `collapse.rs`, `bridge/src/lib.rs`, `game/shell.gd`, `game/world_view.gd`, `tools/RUN_M03D_TERMUX.sh`, `ci.yml`, docs. NOTHING in 03d has been compiled or run (no Rust or Godot in the authoring sandbox).
Run the tests from the unzipped folder first (`sh tools/RUN_M03D_TERMUX.sh`); overlay with `unzip -qo` then `cp -r <folder>/. <repo>/` and check `git status --short` (no deletions expected).
No `.so`, `.apk` or `target/`. Integrity check: `python3 tools/check_handoff.py <folder or zip>`.
