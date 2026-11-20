#!/usr/bin/env python3
"""Python reference for Milestone 03a: the collapse core (D-025, D-029, D-030, D-031). Independent of the Rust code.
Run: python3 tools/ref/collapse_ref.py [fixtures_dir]   -> self-tests, then writes collapse_scenarios.txt that the Rust tests must reproduce exactly.
Rules implemented (the Claude-proposed details are listed in DECISIONS D-032):
 - Sparse grid, orthogonal adjacency (D-007). Groups = connected components. Group size n is fixed at the START of each tick.
 - Risk per tick for every particle of a group of n: min(cap, base * 2^(n-1)), fixed point p32 = probability * 2^32 (integers only).
 - Each tick, particles roll in row-major order (y, then x), one draw each from the collapse RNG stream. A success collapses the particle
   and its chain resolves completely (breadth first; neighbour order up, right, down, left) before the next particle rolls.
 - In a chain every linked, still-present, not-yet-rolled neighbour rolls once at its own group risk (D-030, D-031); success joins the chain.
 - A particle is processed at most once per chain (visited set). Collapsed particles are removed at once."""
import sys, os, struct
from collections import deque
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import platform_ref as P

STREAM_COLLAPSE = 0
STREAM_GENESIS = 2
M64 = (1 << 64) - 1
NEIGH = ((-1, 0), (0, 1), (1, 0), (0, -1))  # (dy, dx): up, right, down, left

def p32_from_ppm(ppm): return (ppm << 32) // 1_000_000

class Model:
    def __init__(self, base_ppm=1000, cap_ppm=100_000):
        self.base = p32_from_ppm(base_ppm); self.cap = p32_from_ppm(cap_ppm)
    def risk(self, n):
        shift = min(max(n - 1, 0), 64)
        return min(self.base << shift, self.cap)

class TypeConfig:
    """Per-type settings (03b). ALL VALUES ARE PLACEHOLDERS until the owner tunes them (D-037)."""
    def __init__(self, id, cost=0, count_per_tick=0, spawn_p32=0, spawn_type=None, blocked=("skip",), free_first=True):
        self.id = id; self.cost = cost; self.count_per_tick = count_per_tick; self.spawn_p32 = spawn_p32
        self.spawn_type = id if spawn_type is None else spawn_type; self.blocked = blocked; self.free_first = free_first

class World:
    def __init__(self, seed, model, log_cap=100_000):
        self.tick = 0; self.cells = set(); self.model = model; self.rng = P.streams(seed)
        self.log = []; self.chains = 0; self.log_cap = log_cap; self.dropped = 0
        self.kinds = {}; self.types = {0: TypeConfig(0)}; self.count = 0; self.free = 0
    def place(self, x, y):  # debug placement of a plain (type 0) particle: free, ignores the economy
        k = (y, x)
        if k in self.cells: return False
        self.cells.add(k); self.kinds[k] = 0; return True
    def define_type(self, cfg): self.types[cfg.id] = cfg
    def add_count(self, n): self.count = min(self.count + n, M64)
    def grant_free_placements(self, n): self.free = min(self.free + n, M64)
    def place_typed(self, x, y, type_id):
        cfg = self.types.get(type_id)
        if cfg is None: return "unknown"
        k = (y, x)
        if k in self.cells: return "occupied"
        can_count = self.count >= cfg.cost; can_free = self.free > 0
        if cfg.free_first: pay = "free" if can_free else ("count" if can_count else None)
        else: pay = "count" if can_count else ("free" if can_free else None)
        if pay is None: return "insufficient"
        if pay == "free": self.free -= 1
        else: self.count -= cfg.cost
        self.cells.add(k); self.kinds[k] = type_id
        return pay
    def genesis_roll(self, p): return (self.rng[STREAM_GENESIS].next() >> 32) < p
    def generate(self):
        for (k, t) in sorted(self.kinds.items()):  # snapshot, row-major
            cfg = self.types.get(t)
            if cfg is None: continue
            self.add_count(cfg.count_per_tick)
            if cfg.spawn_p32 == 0: continue
            if not self.genesis_roll(cfg.spawn_p32): continue
            free_nb = []
            for dy, dx in NEIGH:
                n = (k[0] + dy, k[1] + dx)
                if n not in self.cells: free_nb.append(n)
            if not free_nb:
                if cfg.blocked[0] == "convert": self.add_count(cfg.blocked[1])
                continue
            i = self.rng[STREAM_GENESIS].below(len(free_nb))
            if cfg.spawn_type in self.types:
                n = free_nb[i]; self.cells.add(n); self.kinds[n] = cfg.spawn_type
    def full_hash(self):
        b = struct.pack("<QQQ", self.state_hash(), self.count, self.free)
        for (y, x), t in sorted(self.kinds.items()): b += struct.pack("<iiH", y, x, t)
        return P.fnv1a(b)
    def risks(self):
        risk = {}; seen = set()
        for k in sorted(self.cells):
            if k in seen: continue
            comp = [k]; seen.add(k); i = 0
            while i < len(comp):
                cy, cx = comp[i]; i += 1
                for dy, dx in NEIGH:
                    n = (cy + dy, cx + dx)
                    if n in self.cells and n not in seen:
                        seen.add(n); comp.append(n)
            r = self.model.risk(len(comp))
            for c in comp: risk[c] = r
        return risk
    def roll(self, p): return (self.rng[STREAM_COLLAPSE].next() >> 32) < p
    def push(self, rec):
        if len(self.log) < self.log_cap: self.log.append(rec)
        else: self.dropped += 1
    def run_chain(self, origin, risks):
        cid = self.chains; self.chains += 1
        visited = {origin}; q = deque([(origin, 0)])
        while q:
            k, depth = q.popleft()
            self.cells.discard(k); self.kinds.pop(k, None)
            self.push((self.tick, k[1], k[0], cid, depth))
            for dy, dx in NEIGH:
                n = (k[0] + dy, k[1] + dx)
                if n not in self.cells or n in visited: continue
                visited.add(n)
                if self.roll(risks.get(n, 0)): q.append((n, depth + 1))
    def step_tick(self):
        risks = self.risks()
        for k in sorted(self.cells):
            if k not in self.cells: continue
            if self.roll(risks[k]): self.run_chain(k, risks)
        self.generate()
        self.tick += 1
    def state_hash(self):
        b = struct.pack("<Q", self.tick)
        for (y, x) in sorted(self.cells): b += struct.pack("<ii", y, x)
        for r in self.rng: b += struct.pack("<4Q", *r.s)
        return P.fnv1a(b)

def run(w, ticks, placements):
    for t in range(ticks):
        for (pt, x, y) in placements:
            if pt == t: w.place(x, y)
        w.step_tick()
    return w

def s1():
    w = World(1, Model(20000, 300_000))
    for y in range(6):
        for x in range(6): w.place(x, y)
    for x in range(5): w.place(10 + x, 10)
    return w, 40, []
def s2():
    w = World(7, Model())  # default model: 0.1% base, doubling, 10% cap
    for x in range(4): w.place(x, 0)
    w.place(10, 10); w.place(11, 10)
    w.place(20, 20)
    w.place(30, 30); w.place(31, 30); w.place(30, 31); w.place(31, 31)
    return w, 20_000, []
def s3():
    w = World(99, Model(50_000, 500_000))
    pl = [(0, 0, 0), (0, 1, 0), (5, 2, 0), (5, 3, 0), (10, 0, 1), (10, 1, 1), (10, 2, 1), (10, 3, 1), (20, 5, 5), (20, 5, 6), (20, 6, 5), (20, 6, 6), (30, 0, 0), (30, 1, 0)]
    return w, 60, pl
SCENARIOS = {"s1_block_inflated": s1, "s2_default_model_small_groups": s2, "s3_placements_over_time": s3}

def selftest():
    m = Model()
    assert m.risk(1) == p32_from_ppm(1000) == 4294967
    assert m.risk(2) == 2 * m.risk(1) and m.risk(3) == 4 * m.risk(1)
    assert m.risk(8) == p32_from_ppm(100_000) and m.risk(1000) == p32_from_ppm(100_000)  # capped, no overflow
    assert Model(0, 100_000).risk(50) == 0
    w = World(5, Model()); assert w.place(1, 1) and not w.place(1, 1)
    w2 = World(5, Model()); before = [r.s[:] for r in w2.rng]
    for _ in range(100): w2.step_tick()
    assert [r.s for r in w2.rng] == before, "an empty world must not consume randomness"
    w3 = World(5, Model())
    for x in range(3): w3.place(x, 0)
    w3.place(10, 10)
    rk = w3.risks(); assert rk[(0, 0)] == rk[(0, 1)] == rk[(0, 2)] == w3.model.risk(3) and rk[(10, 10)] == w3.model.risk(1)
    w4 = World(5, Model()); w4.place(0, 0); w4.place(1, 1)  # diagonal is NOT adjacent (D-007)
    r4 = w4.risks(); assert r4[(0, 0)] == r4[(1, 1)] == w4.model.risk(1)

def main(fx):
    selftest()
    out = []; total = 0
    for name, f in SCENARIOS.items():
        w, ticks, pl = f(); run(w, ticks, pl)
        out.append("scenario " + name)
        for (t, x, y, c, d) in w.log: out.append(f"rec {t} {x} {y} {c} {d}")
        out.append(f"end hash={w.state_hash():016x} tick={w.tick} cells={len(w.cells)} chains={w.chains} dropped={w.dropped}")
        total += len(w.log)
        # properties on the log
        seen = set(); last_depth = {}
        for (t, x, y, c, d) in w.log:
            assert (t, x, y) not in seen, "a cell collapsed twice in one tick without being re-placed"
            seen.add((t, x, y))
            if c in last_depth: assert d >= last_depth[c], "breadth-first order violated"
            else: assert d == 0, "a chain must start at depth 0"
            last_depth[c] = d
        print(name, "records:", len(w.log), "chains:", w.chains, "left:", len(w.cells), "max chain size:", max([sum(1 for r in w.log if r[3] == c) for c in range(w.chains)] or [0]))
    os.makedirs(fx, exist_ok=True)
    open(os.path.join(fx, "collapse_scenarios.txt"), "w").write("\n".join(out) + "\n")
    print("collapse reference checks passed;", total, "records written to", fx)

if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.join(here, "..", "..", "core", "tests", "fixtures"))
