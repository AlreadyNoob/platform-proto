# DEVELOPMENT_PROGRESS
**Current milestone:** 01 — engine validation, PARTIAL (2026-11-17)
**Last known build status:** nothing compiled; no engine project exported.

## Completed and verified (real runs)
- Event-driven collapse scheduling vs per-tick rolls: statistically equivalent in the Python reference model, incl. changing p, merge and split; negative controls detected.
- Strategy scaling measured with a C reference on an x86 server core.
- Rust core compiled and all 5 tests pass on the creator's phone (MT6878, Termux); determinism hashes identical to an independent x86 Python port.
- Event-driven AND brute-force scaling measured on the phone (valid after fix); crossover finding F-6 (report §7).
## Written, not verified
gdext bridge, Godot harness, CI workflow (in `prototype/`); brute-force bench columns need a re-run after a fix.
## Not done
Godot-to-Rust loading, Android run, rendering benchmarks, determinism across devices, binding overhead.
## Blockers
Authoring sandbox had no Rust/Godot/Android toolchain or network. Needs the creator's phone/PC/CI.
## Next exact steps
See `TODO.md` (Milestone 01 — finish) and `prototype/README.md`.
