#!/bin/sh
# Run from the prototype/ folder in Termux:   sh RUN_M01B_TERMUX.sh
# Builds a core-only workspace in ~/platform/m01b_work (your repo is not modified) and runs all Rust tests + benchmarks.
set -u
SRC="$(pwd)"
W="$HOME/platform/m01b_work"
OUT="$HOME/platform/m01b_out"
rm -rf "$W"; mkdir -p "$W" "$OUT"
cp -r "$SRC/core_proto" "$W/"
printf '[workspace]\nmembers = ["core_proto"]\nresolver = "2"\n\n[profile.release]\nopt-level = 3\nlto = true\ncodegen-units = 1\n' > "$W/Cargo.toml"
cd "$W" || exit 1
echo "== tests (takes a few minutes) =="
cargo test --release -- --nocapture 2>&1 | tee "$OUT/test_out_m01b.txt"
echo "== scheduler benchmark (about 1 minute) =="
cargo run --release --bin bench_hybrid 2>&1 | tee "$OUT/bench_hybrid.txt"
echo "== high-hazard scheduler benchmark (about 1 minute) =="
cargo run --release --bin bench_hybrid -- --hi 2>&1 | tee "$OUT/bench_hybrid_hi.txt"
echo "== determinism hashes =="
cargo run --release --bin bench_hybrid -- --determinism-hazard 2>&1 | tee "$OUT/det_hazard.txt"
cp "$OUT"/*.txt /storage/emulated/0/Download/ && echo "Copied 3 result files to Download."
