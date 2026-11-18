#!/bin/sh
# Milestone 02a: test the Rust core on the phone (core only; the Godot crate is not needed or built).
# Usage (from anywhere):  sh <project folder>/tools/RUN_M02A_TERMUX.sh
HERE="$(cd "$(dirname "$0")/.." && pwd)"
W="$HOME/platform/m02a_work"
OUT="$HOME/platform/m02a_out"
rm -rf "$W" "$OUT"
mkdir -p "$W" "$OUT"
cp -r "$HERE/core" "$W/core"
printf '[workspace]\nmembers = ["core"]\nresolver = "2"\n' > "$W/Cargo.toml"
cd "$W" || exit 1
cargo test --release -p platform_core > "$OUT/test_out_m02a.txt" 2>&1
echo "cargo exit code: $?" >> "$OUT/test_out_m02a.txt"
python3 "$HERE/tools/ref/platform_ref.py" "$OUT/fixtures_check" > "$OUT/python_ref_m02a.txt" 2>&1
for f in rng_vectors.txt governor_traces.txt save_v1_golden.hex; do
  if cmp -s "$OUT/fixtures_check/$f" "$HERE/core/tests/fixtures/$f"; then echo "fixture identical: $f"; else echo "FIXTURE DIFFERS: $f"; fi
done >> "$OUT/python_ref_m02a.txt"
tail -n 25 "$OUT/test_out_m02a.txt"
cat "$OUT/python_ref_m02a.txt"
cp "$OUT/test_out_m02a.txt" "$OUT/python_ref_m02a.txt" /storage/emulated/0/Download/ 2>/dev/null
echo "Result files copied to Download: test_out_m02a.txt, python_ref_m02a.txt"
