# prototype/ — Milestone 01/01b validation code (ISOLATED, throwaway)
Nothing here is the game. Do not import it from future game code; re-implement surviving ideas under the real architecture.

## Status (all verified unless stated; see ../MILESTONE_01_REPORT.md section 11)
| Part | Status |
|---|---|
| `reference/equivalence_test.py`, `hybrid_ref.py`, `bench_scaling.c`, `detport.py`-style ports (`hazport.py`) | ran (Claude sandbox); raw results in `reference/results/` |
| `core_proto/` (Rust, std only): rng, detmath, model, scaling, hazard (brute/heap/wheel/hybrid), clock, worker, stats | **compiled and tested on the owner's phone (14/14) and in CI** |
| `bridge_proto/` (gdext 0.5.5) | **built in CI; loaded and ran in the Android app** |
| `godot_proto/` (Godot 4.7.2 harness v3) | **exported and run on the phone** |
| `../.github/workflows/prototype-m01.yml` | **runs green** |
| `patches/` | historical patch scripts, already applied; kept for the record |

## Run the Rust core tests/benchmarks (Termux, no Godot needed)
```
cd prototype
sh RUN_M01B_TERMUX.sh     # builds a core-only workspace in ~/platform/m01b_work; results copied to Download
```
## Run the references (any machine with Python; numpy+scipy for the first two)
```
cd prototype/reference && python3 equivalence_test.py 25000 && python3 hybrid_ref.py 20000
```
## Android app
See `../ENVIRONMENT_AND_REPO.md` sections 3-4 (CI builds the library; copy library + `godot_proto/*` to `Documents/platform_m01`; export from the Godot Android editor).
