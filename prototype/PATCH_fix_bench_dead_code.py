#!/usr/bin/env python3
# Run from the prototype/ folder:  python3 PATCH_fix_bench_dead_code.py
p = "core_proto/src/scaling.rs"
s = open(p).read()
old = "    ScalingResult { stored: n, components: comps, ticks, events, simulated_particles: n * ticks, seconds: start.elapsed().as_secs_f64() }"
if "black_box(events)" in s:
    print("already patched")
else:
    assert s.count(old) == 2, "unexpected file content"
    s = s.replace(old, "    let events = std::hint::black_box(events);\n" + old)
    open(p, "w").write(s)
    print("patched 2 places")
