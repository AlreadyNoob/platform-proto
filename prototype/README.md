# prototype/ — Milestone 01 validation code (ISOLATED, throwaway)
Nothing here is the game. Do not import it from future game code; if an idea survives, re-implement it under the real architecture.

## Milestone 01b (v0.0.6)
Run `sh RUN_M01B_TERMUX.sh` from this folder in Termux (core-only workspace in ~/platform/m01b_work; your repo is not touched). Then see `../TODO.md` for the CI/APK/soak steps.

## Status
| Part | Status |
|---|---|
| `reference/equivalence_test.py` | **ran** (Python 3, numpy, scipy) — results in `reference/results/` |
| `reference/bench_scaling.c` | **ran** on the authoring sandbox (x86) — `reference/results/bench_scaling_x86_sandbox.txt` |
| `core_proto/` (Rust, std only) | **written, UNCOMPILED, tests not run** |
| `bridge_proto/` (gdext) | **written, UNCOMPILED**; gdext version unpinned |
| `godot_proto/` (Godot 4.7.2 harness) | **written, never opened in Godot** |
| `../.github/workflows/prototype-m01.yml` | **never executed** |

## Run the Rust core (no Godot needed; works in Termux)
```
cd prototype
cargo test --release -p platform_core_proto -- --nocapture   # equivalence + negative controls + detmath
cargo run --release -p platform_core_proto --bin bench        # scaling table (compare with the C numbers)
cargo run --release -p platform_core_proto --bin bench -- --determinism   # run on PC and phone; hashes must match
```
If the tests fail to compile, send the compiler output back; the code was never compiled.

## Run the Python/C reference
```
cd prototype/reference && python3 equivalence_test.py 25000
gcc -O2 -o bench_scaling bench_scaling.c -lm && ./bench_scaling 1000000 4 1e-4 120000 1000
```

## Godot ↔ Rust on Android
1. Build the arm64 library: run the CI job `android-arm64-so` (or build locally with the NDK, see CI file) → `libplatform_bridge_proto.so`.
2. Put it at `godot_proto/bin/android_arm64/`.
3. Open `godot_proto` in Godot 4.7.2 (or GABE on the phone), add an Android export preset (arm64-v8a, `gdextension` libs included), export a debug APK.
4. In the app press: **1 Integration** (core loaded? call overhead, determinism hash), **2 Sim bench**, **3 Render N** buttons, then **Copy** and paste results into `TESTING.md`. `adb logcat -s godot` shows the same lines.
Pass criteria are in `../PROTOTYPE_PLAN.md`.
