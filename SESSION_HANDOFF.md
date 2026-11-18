# SESSION_HANDOFF (v0.1.3, 2026-11-19) — rewritten at every milestone/checkpoint, never appended
**Start with `START_HERE.md`.** Short "what now".

**State:** Engine/language committed. Milestone 02a core (rng, clock, governor, wheel, snapshot/command, save v1, host) TESTED on the phone: compiled first time, 27/27 tests, identical to the Python references. `bridge/`, `game/` check scene and `ci.yml` never built. No game rules.

**Waiting for:** the owner to overlay the project into the git working copy, push, and report the CI result (then the check APK results).

**Next action:** read CI log; fix bridge/CI compile errors (K-26); download `m02-android-so`; guide building the check APK (new Godot folder `Documents/platform_m02`); record governor overload + save/load results on the phone; ship v0.1.4 checkpoint. Then 02b Godot shell.

**Do not:** start mechanics; claim bridge/game work before output says so; copy prototype code blindly; assume Rust/network in the sandbox.

**Unconfirmed:** whether M02 files are in the repo yet; NDK version used by CI.

**Owner's working loop:** Claude gives exact commands -> owner runs them on the phone -> pastes outputs -> Claude records real results and ships the next ZIP. The owner answers briefly ("whatever you think is good"): decide sensibly, state the decision, move on.
