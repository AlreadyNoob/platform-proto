# TODO
## Milestone 01 — DONE for engine feasibility (v0.0.5).
## Milestone 01b — core part DONE on the phone (v0.0.7); app part pending
- [x] Phone: Rust tests 14/14, scheduler benchmark, determinism hashes (matched to x86 Python port)
- [x] `bench_hybrid --hi` on phone: hot list pays off above ~5% hazard (report §10.6)
- [ ] (optional) re-run `sh RUN_M01B_TERMUX.sh` with v0.0.8 to see the overhead fix
- [ ] Push v0.0.8 to GitHub; read the CI log (x86 Rust results); download the new .so (release asset)
- [ ] Copy new main.gd + .so into Documents/platform_m01, re-export the APK; run buttons 1, 4, 5a, 5c, 5b (10 min), then Copy; note the display Hz
- [ ] Record real results in report §10 and TESTING; note NDK version used by CI
## Then: owner decision on D-002/D-003 final commitment; Milestone 02 (Phase 1 skeleton, real repo layout, no mechanics)
## Pending design questions: GDD Q-1..Q-7 (Q-2 collapse propagation, Q-4 tick length first)
