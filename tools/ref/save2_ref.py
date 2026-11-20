#!/usr/bin/env python3
"""Python reference for Milestone 03c: save schema 2 (header + world state) and world export/restore. Independent of the Rust code.
Run: python3 tools/ref/save2_ref.py [fixtures_dir] -> self-tests, then writes save_v2_noworld_golden.hex and save_v2_world_golden.hex.
Schema 2 payload = schema 1 payload (seed, tick, requested_speed, rng states, label) followed by a world block:
  u8 present (0 or 1); if 1: base_p32 u64, cap_p32 u64, count u64, free u64, chains u64,
  n_types u32, per type (ascending id): id u16, cost u64, count_per_tick u64, spawn_p32 u64, spawn_type u16, blocked_tag u8 (0 skip, 1 convert), blocked_n u64, free_first u8,
  n_cells u32, per cell (ascending y then x): y i32, x i32, kind u16.
A schema 1 file is migrated by appending present = 0."""
import sys, os, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import platform_ref as P
import collapse_ref as C
import economy_ref as E

def export_world(w):
    types = []
    for t in sorted(w.types.values(), key=lambda t: t.id):
        tag, n = (1, t.blocked[1]) if t.blocked[0] == "convert" else (0, 0)
        types.append((t.id, t.cost, t.count_per_tick, t.spawn_p32, t.spawn_type, tag, n, 1 if t.free_first else 0))
    cells = [(y, x, w.kinds[(y, x)]) for (y, x) in sorted(w.cells)]
    return dict(base=w.model.base, cap=w.model.cap, count=w.count, free=w.free, chains=w.chains, types=types, cells=cells)

def restore_world(seed, tick, rng_states, ws):
    w = C.World(seed, C.Model()); w.model.base = ws["base"]; w.model.cap = ws["cap"]
    w.tick = tick; w.rng = [P.Rng(state=s) for s in rng_states]
    w.count = ws["count"]; w.free = ws["free"]; w.chains = ws["chains"]
    w.types = {0: C.TypeConfig(0)}
    for (id, cost, cpt, sp, st, tag, n, ff) in ws["types"]:
        w.types[id] = C.TypeConfig(id, cost, cpt, sp, st, ("convert", n) if tag == 1 else ("skip",), ff == 1)
    for (y, x, k) in ws["cells"]:
        assert k in w.types; w.cells.add((y, x)); w.kinds[(y, x)] = k
    return w

def encode_world(ws):
    if ws is None: return b"\x00"
    b = b"\x01" + struct.pack("<5Q", ws["base"], ws["cap"], ws["count"], ws["free"], ws["chains"])
    b += struct.pack("<I", len(ws["types"]))
    for t in ws["types"]: b += struct.pack("<HQQQHBQB", *t)
    b += struct.pack("<I", len(ws["cells"]))
    for c in ws["cells"]: b += struct.pack("<iiH", *c)
    return b

def encode_v2(seed, tick, speed, rng_states, label, ws):
    p = struct.pack("<QQQI", seed, tick, speed, len(rng_states))
    for st in rng_states: p += struct.pack("<4Q", *st)
    lb = label.encode("utf-8"); p += struct.pack("<I", len(lb)) + lb
    p += encode_world(ws)
    body = P.MAGIC + struct.pack("<II", 2, len(p)) + p
    return body + struct.pack("<Q", P.fnv1a(body))

def e3_at(ticks):
    w, _, ops = E.e3()
    for t in range(ticks):
        for (ot, x, y, ty) in ops:
            if ot == t: w.place_typed(x, y, ty)
        w.step_tick()
    return w, ops

def selftest():
    # export -> restore -> continue equals running straight through
    a, ops = e3_at(30)
    ws = export_world(a); states = [r.s[:] for r in a.rng]
    r = restore_world(33, a.tick, states, ws)
    assert r.full_hash() == a.full_hash()
    b, _ = e3_at(30)
    for _ in range(30): b.step_tick(); r.step_tick()
    straight, _ = e3_at(60)
    assert r.full_hash() == straight.full_hash() == b.full_hash(), "restore must continue exactly like the original"
    assert r.log == [x for x in b.log if x[0] >= 30], "records after the restore must equal the original's"
    assert export_world(r) == export_world(straight)

def main(fx):
    selftest()
    seed = 0x0123456789abcdef
    noworld = encode_v2(seed, 42, 100, [s.s for s in P.streams(seed)], "slot-one \u00e9", None)
    w, _ = e3_at(30)
    world = encode_v2(33, w.tick, 1000, [r.s for r in w.rng], "e3 at tick 30", export_world(w))
    os.makedirs(fx, exist_ok=True)
    open(os.path.join(fx, "save_v2_noworld_golden.hex"), "w").write(noworld.hex() + "\n")
    open(os.path.join(fx, "save_v2_world_golden.hex"), "w").write(world.hex() + "\n")
    print("save v2 reference checks passed; world golden:", len(world), "bytes (", len(w.cells), "particles, tick", w.tick, ", count", w.count, ")")

if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.join(here, "..", "..", "core", "tests", "fixtures"))
