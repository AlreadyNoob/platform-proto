#!/bin/sh
# Milestone 03d: test the Rust core (collapse rules, economy, save v2 and the 03d game world) on the phone. Core only; no Godot crate is built.
# Usage:  sh <project folder>/tools/RUN_M03D_TERMUX.sh
HERE="$(cd "$(dirname "$0")/.." && pwd)"
W="$HOME/platform/m03d_work"
OUT="$HOME/platform/m03d_out"
rm -rf "$W" "$OUT"
mkdir -p "$W" "$OUT"
cp -r "$HERE/core" "$W/core"
printf '[workspace]\nmembers = ["core"]\nresolver = "2"\n' > "$W/Cargo.toml"
cd "$W" || exit 1
cargo test --release -p platform_core > "$OUT/test_out_m03d.txt" 2>&1
echo "cargo exit code: $?" >> "$OUT/test_out_m03d.txt"
{ python3 "$HERE/tools/ref/platform_ref.py" "$OUT/fixtures_check"; python3 "$HERE/tools/ref/collapse_ref.py" "$OUT/fixtures_check"; python3 "$HERE/tools/ref/economy_ref.py" "$OUT/fixtures_check"; python3 "$HERE/tools/ref/save2_ref.py" "$OUT/fixtures_check"; } > "$OUT/python_ref_m03d.txt" 2>&1
for f in rng_vectors.txt governor_traces.txt save_v1_golden.hex collapse_scenarios.txt economy_scenarios.txt save_v2_noworld_golden.hex save_v2_world_golden.hex; do
  if cmp -s "$OUT/fixtures_check/$f" "$HERE/core/tests/fixtures/$f"; then echo "fixture identical: $f"; else echo "FIXTURE DIFFERS: $f"; fi
done >> "$OUT/python_ref_m03d.txt"
grep -E "^test result|^cargo exit|FAILED|panicked|error" "$OUT/test_out_m03d.txt" | head -30
cat "$OUT/python_ref_m03d.txt"
cp "$OUT/test_out_m03d.txt" "$OUT/python_ref_m03d.txt" /storage/emulated/0/Download/ 2>/dev/null
echo "Result files copied to Download: test_out_m03d.txt, python_ref_m03d.txt"
