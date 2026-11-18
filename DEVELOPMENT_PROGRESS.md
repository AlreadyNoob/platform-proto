# DEVELOPMENT_PROGRESS
**Current milestone:** 01b (v0.0.7) — scheduler, clock, worker thread, soak. Core Rust verified on the phone (14/14 tests, benchmark, determinism). App-side snapshot/soak runs pending.
**Last verified on device:** Milestone 01 results (Godot 4.7.2 + Rust on the phone, report §8).
## Verified by execution in 01b
Python reference: heap, timing wheel and hybrid scheduler match per-tick rolling and exact theory; negative controls detected.
## Written, not verified
Rust hazard/clock/worker/bench/tests, bridge additions, GDScript harness v2, Termux runner, CI steps.
## Blockers
None. Needs the owner to run the steps in TODO.

## Verified on the phone in 01b (v0.0.7)
14/14 Rust tests; hazard hashes identical to an independent x86 Python port; timing wheel 2.4-3.1x faster than heap; hybrid (threshold 0.3%) slower than wheel (decision D-012 revised).
