# TODO
## Milestone 01 — finish (REQUIRED, needs a machine with toolchains)
- [x] Rust core tests on phone: 5/5 pass (v0.0.3)
- [x] Determinism hashes phone vs x86 Python port: identical
- [x] Bench re-run on phone with fix (valid)
- [ ] Design + test hybrid scheduler (D-012): timing wheel vs heap, equivalence test for the hybrid, crossover re-measured
- [ ] Optional: run `--determinism` on a PC with Rust as a third data point
- [ ] Run CI `android-arm64-so`; load the .so in Godot 4.7.2 on desktop first, then Android (A1)
- [ ] Export APK (manual preset), run harness buttons 1/2/3, record results (A2, A3, A7, A8)
- [ ] Pin gdext version and NDK; commit Cargo.lock
- [ ] Decide D-002/D-003 from evidence; if Rust-on-Android fails, evaluate alternatives (MILESTONE_01_REPORT §5)
## Milestone 02 (planned, only after the above): Phase 1 skeleton (layout, themes, camera, settings, audio foundation)
## Optional
- [ ] libGDX spike (only on trigger)
- [ ] Resolve GDD questions Q-1..Q-7 (Q-2 collapse propagation affects event model)
## Deferred
Everything in GDD §§6-12 until the core is validated.
