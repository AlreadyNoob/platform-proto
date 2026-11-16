# SESSION_HANDOFF (Milestone 01, 2026-11-17)
**Read first:** `README.md` -> `MILESTONE_01_REPORT.md` -> `DECISIONS.md` -> `prototype/README.md` -> `ARCHITECTURE.md` -> `TODO.md`.
**Owner approval:** Godot 4.7.2 + engine-independent core, Rust leading, for investigation/prototyping only; not a final commitment. Alternatives must be evaluated if Rust-on-Android is unreliable.
**State:** Milestone 01 is PARTIAL. Verified: equivalence (Python and Rust on the phone), determinism across arm64/x86, event-driven scaling on phone. Unverified: gdext bridge, Godot loading the .so, Android app run, rendering, binding overhead.
**First thing to do next session:** (1) bench already re-run (valid; F-6 crossover found) — decide whether to prototype the hybrid scheduler (D-012) before or after integration; (2) build the bridge: CI job `android-arm64-so` or Termux if feasible, then the Godot harness; fix compile errors in `bridge_proto` (never compiled). Fill `TESTING.md` only with real numbers.
**Do not:** start game mechanics, quote performance numbers that were not measured on the named device, or treat the C/Python numbers as Android results.
**Key risks:** F-2/F-6 (event-driven gain depends on sparsity; phone crossover ~0.25-0.45% collapse/tick), F-4 (collapse handling cost unmodelled), K-6 (Android integration), Q-2 (collapse propagation rule).
