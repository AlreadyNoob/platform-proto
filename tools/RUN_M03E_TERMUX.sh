#!/bin/sh
# Milestone 03e: test the Rust core (collapse rules, economy, save v2, game world, starter particle) on the phone. Core only; no Godot crate is built.
# Usage:  sh <project folder>/tools/RUN_M03E_TERMUX.sh
HERE="$(cd "$(dirname "$0")/.." && pwd)"
W="$HOME/platform/m03e_work"
OUT="$HOME/platform/m03e_out"
rm -rf "$W" "$OUT"
mkdir -p "$W" "$OUT"
cp -r "$HERE/core" "$W/core"
printf '[workspace]\nmembers = ["core"]\nresolver = "2"\n' > "$W/Cargo.toml"
cd "$W" || exit 1
cargo test --release -p platform_core > "$OUT/test_out_m03e.txt" 2>&1
echo "cargo exit code: $?" >> "$OUT/test_out_m03e.txt"
{ python3 "$HERE/tools/ref/platform_ref.py" "$OUT/fixtures_check"; python3 "$HERE/tools/ref/collapse_ref.py" "$OUT/fixtures_check"; python3 "$HERE/tools/ref/economy_ref.py" "$OUT/fixtures_check"; python3 "$HERE/tools/ref/save2_ref.py" "$OUT/fixtures_check"; } > "$OUT/python_ref_m03e.txt" 2>&1
for f in rng_vectors.txt governor_traces.txt save_v1_golden.hex collapse_scenarios.txt economy_scenarios.txt save_v2_noworld_golden.hex save_v2_world_golden.hex; do
  if cmp -s "$OUT/fixtures_check/$f" "$HERE/core/tests/fixtures/$f"; then echo "fixture identical: $f"; else echo "FIXTURE DIFFERS: $f"; fi
done >> "$OUT/python_ref_m03e.txt"
(cd "$OUT" && python3 "$HERE/tools/ref/balance_ref.py" 4 3600 > /dev/null 2>&1)
grep -E "^test result|^cargo exit|FAILED|panicked|error" "$OUT/test_out_m03e.txt" | head -30
cat "$OUT/python_ref_m03e.txt"
cat "$OUT/balance_report_m03e.txt"
cp "$OUT/test_out_m03e.txt" "$OUT/python_ref_m03e.txt" "$OUT/balance_report_m03e.txt" /storage/emulated/0/Download/ 2>/dev/null
echo "Result files copied to Download: test_out_m03e.txt, python_ref_m03e.txt, balance_report_m03e.txt"
