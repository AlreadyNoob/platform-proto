#!/usr/bin/env python3
"""Python reference for Milestone 03b: the economy core (D-027, D-028, D-034..D-037). Independent of the Rust code.
Run: python3 tools/ref/economy_ref.py [fixtures_dir]  -> self-tests, then writes economy_scenarios.txt that the Rust tests must reproduce line by line.
ALL economy numbers below are PLACEHOLDERS (D-037); see docs/TUNING_PLACEHOLDERS.md. Tick order: collapse phase, then generation phase."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from collapse_ref import World, Model, TypeConfig, p32_from_ppm

def e1():
    w = World(11, Model(0, 0))
    w.define_type(TypeConfig(1, cost=3, count_per_tick=2))
    w.define_type(TypeConfig(4, cost=3, count_per_tick=1, free_first=False))
    w.define_type(TypeConfig(5, cost=1000))
    w.count = 10; w.free = 2
    ops = [(0, 0, 0, 1), (0, 1, 0, 1), (0, 2, 0, 1), (0, 0, 0, 1), (0, 3, 0, 9), (1, 4, 0, 4), (2, 5, 0, 5), (3, 6, 0, 4)]
    return w, 40, ops
def e2():
    w = World(22, Model(0, 0))
    w.define_type(TypeConfig(2, cost=0, count_per_tick=1, spawn_p32=p32_from_ppm(500_000)))
    w.define_type(TypeConfig(3, cost=0, count_per_tick=0, spawn_p32=p32_from_ppm(1_000_000), blocked=("convert", 5)))
    for (x, y) in [(10, 9), (11, 10), (10, 11), (9, 10)]: w.place(x, y)  # debug blockers around the particle at (10, 10)
    ops = [(0, 0, 0, 2), (0, 10, 10, 3)]
    return w, 40, ops
def e3():
    w = World(33, Model(8_000, 150_000))
    w.define_type(TypeConfig(6, cost=0, count_per_tick=1, spawn_p32=p32_from_ppm(100_000)))
    ops = [(0, x, y, 6) for y in range(3) for x in range(4)]
    return w, 60, ops
SCENARIOS = {"e1_costs_and_payment_order": e1, "e2_spawning_and_blocked_policy": e2, "e3_collapse_with_economy": e3}

def run(w, ticks, ops):
    out = []
    for t in range(ticks):
        for (ot, x, y, ty) in ops:
            if ot == t: out.append(f"op {t} {x} {y} {ty} {w.place_typed(x, y, ty)}")
        w.step_tick()
        for (rt, x, y, c, d) in w.log: out.append(f"rec {rt} {x} {y} {c} {d}")
        w.log = []
        if w.tick % 10 == 0: out.append(f"chk {w.tick} {w.count} {w.free} {len(w.cells)} {w.full_hash():016x}")
    out.append(f"end full={w.full_hash():016x} count={w.count} free={w.free} tick={w.tick} cells={len(w.cells)} chains={w.chains}")
    return out

def selftest():
    # payment order and failures
    w = World(1, Model(0, 0)); w.define_type(TypeConfig(1, cost=3)); w.define_type(TypeConfig(2, cost=3, free_first=False))
    w.count = 3; w.free = 1
    assert w.place_typed(0, 0, 9) == "unknown"
    assert w.place_typed(0, 0, 1) == "free" and (w.count, w.free) == (3, 0)      # free first
    assert w.place_typed(0, 0, 1) == "occupied" and (w.count, w.free) == (3, 0)   # nothing changes on failure
    assert w.place_typed(1, 0, 2) == "count" and (w.count, w.free) == (0, 0)      # count first
    assert w.place_typed(2, 0, 2) == "insufficient"
    w.grant_free_placements(2); assert w.free == 2
    assert w.place_typed(2, 0, 2) == "free" and w.free == 1                        # count short -> free placement
    # generation: one count per particle per tick, no spawns when spawn_p32 == 0, no genesis draws
    g = World(1, Model(0, 0)); g.define_type(TypeConfig(1, count_per_tick=2)); before = g.rng[2].s[:]
    g.place_typed(0, 0, 1); g.place_typed(1, 0, 1)
    for _ in range(5): g.step_tick()
    assert g.count == 20 and g.rng[2].s == before and len(g.cells) == 2
    # a blocked spawn converts to count; an unknown spawn type is skipped
    b = World(1, Model(0, 0)); b.define_type(TypeConfig(3, spawn_p32=p32_from_ppm(1_000_000), blocked=("convert", 5)))
    for (x, y) in [(0, -1), (1, 0), (0, 1), (-1, 0)]: b.place(x, y)
    b.place_typed(0, 0, 3); b.step_tick(); assert b.count == 5 and len(b.cells) == 5
    u = World(1, Model(0, 0)); u.define_type(TypeConfig(3, spawn_p32=p32_from_ppm(1_000_000), spawn_type=77))
    u.place_typed(5, 5, 3); u.step_tick(); assert len(u.cells) == 1
    # a collapsed particle stops producing and its type entry is removed
    c = World(1, Model(1_000_000, 1_000_000)); c.define_type(TypeConfig(1, count_per_tick=7)); c.place_typed(0, 0, 1)
    c.step_tick(); assert len(c.cells) == 0 and c.kinds == {} and c.count == 0
    # spawned particles do not generate in the tick they appear
    s = World(1, Model(0, 0)); s.define_type(TypeConfig(2, count_per_tick=1, spawn_p32=p32_from_ppm(1_000_000)))
    s.place_typed(0, 0, 2); s.step_tick(); assert len(s.cells) == 2 and s.count == 1
    s.step_tick(); assert s.count == 1 + 2
    assert set(s.kinds) == s.cells

def main(fx):
    selftest()
    out = []
    for name, f in SCENARIOS.items():
        w, ticks, ops = f()
        out.append("scenario " + name); out += run(w, ticks, ops)
        assert set(w.kinds) == w.cells
        print(name, "count:", w.count, "free:", w.free, "particles:", len(w.cells), "chains:", w.chains)
    os.makedirs(fx, exist_ok=True)
    open(os.path.join(fx, "economy_scenarios.txt"), "w").write("\n".join(out) + "\n")
    print("economy reference checks passed;", len(out), "lines written to", fx)

if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.join(here, "..", "..", "core", "tests", "fixtures"))
