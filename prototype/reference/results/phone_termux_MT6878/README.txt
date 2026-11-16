Run 2026-11-17 on TECNO CAMON 40 Pro 5G (MT6878, Android 16, Termux, Rust 1.99.1), cargo --release, bridge crate excluded.
test_out.txt  : all 5 Rust tests passed (2 detmath, 3 equivalence incl. negative controls).
bench_out.txt : first run. event-driven numbers valid; brute columns INVALID (compiler deleted the loops, F-5).
bench_out2.txt: re-run after the black_box fix. ALL columns valid. Use this one.
det_phone.txt : hashes; independently reproduced on x86 by reference/detport.py (see report section 7).
