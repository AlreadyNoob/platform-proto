# ENVIRONMENT AND REPO — the owner's setup, paths, accounts and working commands
(Facts as of 2026-11-18. Items marked **UNCONFIRMED** were not verified; ask the owner or check before relying on them.)

## 1. The owner's device and tools (everything is phone-only; there is no PC)
- **Phone:** TECNO CAMON 40 Pro 5G (TECNO CM7), Android 16, MediaTek MT6878 (4+4 cores, 2.5 GHz), Mali-G615 MC2, about 7.3 GiB RAM, 1080x2436, display reported 120 Hz (some system-info runs said 60 Hz: record the refresh rate with every frame-time measurement). Battery is often low: ask the owner to plug in for long runs (the 10-minute soak was run plugged in).
- **Termux 0.118.3**, zsh. Rust **1.99.1** (`pkg install rust`), Python 3.14, `gh` (GitHub CLI, already authenticated), `git`, `unzip`. In non-interactive use `gh run view` needs an explicit run id; `export GH_PAGER=cat` stops the pager.
- **Godot editor for Android**, Play Store listing named **"Godot Engine 4"** (package `org.godotengine.editor.v4`, https://play.google.com/store/apps/details?id=org.godotengine.editor.v4), version **4.7.2.stable**. Export templates (4.7.2) were downloaded once inside the editor (Project > Export > Manage Export Templates). Non-Gradle export works and carries the Rust library; GABE (Gradle on device) has never been needed. The editor once showed an unexplained endless startup spinner and then started by itself.
- A phone file manager and a notes app (used to paste the clipboard log).

## 2. Files and folders (on the phone)
| Path | What it is |
|---|---|
| `/storage/emulated/0/Download/` | exchange folder: milestone ZIPs, patch scripts, result files are copied here from Termux and uploaded to Claude from here |
| `~/platform/` (Termux home) | all Platform work |
| `~/platform/Platform_v0.0.4_milestone-01_phone-bench-valid/` | **the git working copy** (remote `origin` = `https://github.com/AlreadyNoob/platform-proto.git`, branch `main`). Despite the old name, it was overlaid with the contents of v0.0.8 by `cp -r Platform_v0.0.8.../. <repo>/`. Last known pushed commit: `d0fb713` "M01b". It also accidentally tracks a 3 MB `libplatform_bridge_proto.so`; a cleanup (`.gitignore`, `git rm --cached`) was proposed — **UNCONFIRMED** whether done. v0.1.0/v0.1.1 have **not** been copied into the repo yet (**UNCONFIRMED**) |
| `~/platform/Platform_v0.0.2..., v0.0.6..., v0.0.7..., v0.0.8...` | unzipped older milestone folders (history; safe to ignore) |
| `~/platform/m01b_work/`, `~/platform/m01b_out/` | scratch workspace and result files produced by `prototype/RUN_M01B_TERMUX.sh` |
| `/storage/emulated/0/Documents/platform_m01/` | **the Godot project** the Android editor opens (`project.godot`, `main.gd` v3, `main.tscn`, `platform_bridge_proto.gdextension`, `bin/android_arm64/libplatform_bridge_proto.so` from CI run 44281130334, `export_presets.cfg` created by Godot). The Godot app cannot see Termux's private home, so the project must live in shared storage |
| `/storage/emulated/0/Download/platform_m01.apk` | the exported debug APK (package `com.example.platformm01`) |

## 3. GitHub
- **Private repo `platform-proto`** (owner account `AlreadyNoob`). Workflow `.github/workflows/prototype-m01.yml` (runs on every push and on demand) has two jobs: `core-desktop` (Rust tests, benchmarks, determinism hashes, Python references on `ubuntu-latest`, Rust 1.100.0) and `android-arm64-so` (cross-compiles the Godot bridge for `aarch64-linux-android` with the runner's default NDK — version not recorded — using godot crate **0.5.5**, pinned, and publishes `libplatform_bridge_proto.so` as a **release asset** of the pre-release tag `m01-android-so`; about 3 minutes 50 seconds).
- **Actions artifact storage quota is FULL** (the owner's other projects use it). Never rely on `upload-artifact` for anything needed; the upload steps are `continue-on-error`. Release assets are used instead.
- No tokens or secrets are stored in this project. Never ask the owner to paste a token.

## 4. Command sequences that are known to work (verified by the owner)
**A. Run the Rust core tests and benchmarks on the phone (core only; does not touch the repo)**
```
cd ~/platform
cp /storage/emulated/0/Download/<milestone>.zip . && unzip -q <milestone>.zip
cd <extracted-folder>/prototype
sh RUN_M01B_TERMUX.sh        # about 3-4 minutes; copies result .txt files to Download
```
**B. Push a new milestone to GitHub and build the Android library**
```
cd ~/platform
cp -r <extracted-new-milestone-folder>/. Platform_v0.0.4_milestone-01_phone-bench-valid/
cd Platform_v0.0.4_milestone-01_phone-bench-valid
git add -A && git commit -m "<message>" && git push
RUN=$(gh run list -L 1 --json databaseId --jq '.[0].databaseId'); gh run watch $RUN
gh release download m01-android-so --clobber        # puts libplatform_bridge_proto.so in the current folder
gh run view $RUN --log > ~/platform/ci_log.txt; cp ~/platform/ci_log.txt /storage/emulated/0/Download/
```
**C. Update the Godot project and export the APK**
```
D=/storage/emulated/0/Documents/platform_m01
cp libplatform_bridge_proto.so $D/bin/android_arm64/
cp prototype/godot_proto/main.gd $D/main.gd          # also main.tscn / project.godot if changed
```
Then in the Godot Android editor: quit to the project list, reopen `platform_m01`, Project > Export > Android (Use Gradle Build off, architecture arm64-v8a only, package `com.example.platformm01`) > Export Project (Export With Debug on), overwrite `Download/platform_m01.apk`, install it (uninstall first if Android refuses).
**D. In the app:** buttons `1 Integration`, `4 Snapshot cost`, `5a Soak 1 min`, `5c Overload 1 min`, `5b Soak 10 min` (buttons are at the top; the whole log is copied to the clipboard automatically after every line — paste into a notes app and send it).

## 5. Pitfalls already met (do not repeat)
- Termux scripts run with `/bin/sh`: no brace expansion, keep scripts POSIX.
- An earlier benchmark printed `inf`: the compiler deleted unused loops (use `std::hint::black_box`).
- `gh run view` without an id fails in a pipe; unattended `gh run watch` needs the id.
- Godot Android editor: opens projects only from shared storage; may need "all files access"; export templates are not bundled.
- GDScript test harness once pushed its own buttons off-screen (long log lines widened the layout); buttons are now first and the log wraps.
- Long soak: keep the screen on and the phone plugged in; the owner's display may drop to 60 Hz.

## 6. Claude-side environment seen in the previous session (may differ in yours — check)
Linux sandbox with bash, Python 3 (numpy, scipy), gcc, no Rust, **no network from bash** (web search/fetch tools existed), outputs go to `/mnt/user-data/outputs`, uploads arrive in `/mnt/user-data/uploads`. All Rust in this project was compiled only on the owner's phone or in GitHub CI.

## 5. Milestone 02 additions (v0.1.4: CI and check APK verified by the owner; repo at commit f7e2f85; Godot folder Documents/platform_m02, package com.example.platformm02; `RUN_M02A_TERMUX.sh` verified by the owner, the rest NOT yet)
- Owner's repo (from the pasted log): `libplatform_bridge_proto.so` was untracked and `.gitignore` committed (commit 201dceb). Whether M02 files are in the repo is still unconfirmed.
- New workflow `.github/workflows/ci.yml` (jobs `core`, `android-arm64-so`); `prototype-m01.yml` is now manual (`workflow_dispatch`) only.
- Android library: release tag `m02-android-so`, file `libplatform_bridge.so` (crate `platform_bridge`, Godot class `PlatformCore`).
- Suggested new Godot project folder: `/storage/emulated/0/Documents/platform_m02` (copy `game/*` plus `bin/android_arm64/libplatform_bridge.so`). Package name for the new check APK: choose one different from `com.example.platformm01` so both can be installed.
- Owner's repo: v0.1.x still UNCONFIRMED as copied in; the new layout (`core/ bridge/ game/ tools/`) should be overlaid the same way as before (`cp -r <new folder>/. <git working copy>/`), after which `git rm --cached` any tracked `.so` files.
- 2026-11-20: GitHub Actions refused to start (billing/spending limit message). `ci.yml` now runs only the `core` job on push; the Android library job runs on request: `gh workflow run ci` (then `gh run watch`). Check GitHub Settings > Billing and plans.
