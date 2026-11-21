#!/usr/bin/env python3
"""Balance simulation for Milestone 03e (the starter particle). NOT a test and NOT a decision: it only shows what the PLACEHOLDER numbers do,
so the owner can tune them. Uses the independent Python reference world (collapse_ref.py), same rules as the Rust core.
Run: python3 tools/ref/balance_ref.py [seeds] [ticks]   -> prints a short report (also written to balance_report_m03e.txt next to the script's cwd)."""
import sys, os, random, statistics
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from collapse_ref import World, Model, TypeConfig, p32_from_ppm, NEIGH

STARTER = dict(cost=60, count_per_tick=1, spawn_ppm=10_000, cost_div=10)   # 60 count, 1 count/tick, 1% spawn per tick (PLACEHOLDERS)
START_FREE, START_COUNT = 20, 120

def make(seed):
    w = World(seed, Model())   # default collapse model: 0.1% base, doubling per linked particle, 10% cap (owner, D-029)
    w.define_type(TypeConfig(1, cost=STARTER["cost"], count_per_tick=STARTER["count_per_tick"], spawn_p32=p32_from_ppm(STARTER["spawn_ppm"])))
    w.cost_div = STARTER["cost_div"]   # rising cost: cost + cost * particles // 10 (D-047)
    w.grant_free_placements(START_FREE); w.add_count(START_COUNT)
    return w

def free_neighbours(w, k): return [(k[0] + dy, k[1] + dx) for dy, dx in NEIGH if (k[0] + dy, k[1] + dx) not in w.cells]

def place_spread(w, rnd, area=60):
    """Player who keeps particles apart: a random cell with no occupied orthogonal neighbour."""
    for _ in range(50):
        y, x = rnd.randrange(area), rnd.randrange(area)
        if (y, x) not in w.cells and len(free_neighbours(w, (y, x))) == 4:
            return w.place_typed(x, y, 1)
    return "none"

def place_pack(w, rnd, area=60):
    """Player who clusters: a free cell next to a random existing particle (or the middle if there is none)."""
    if not w.cells: return w.place_typed(area // 2, area // 2, 1)
    k = rnd.choice(sorted(w.cells)); nb = free_neighbours(w, k)
    return w.place_typed(nb[0][1], nb[0][0], 1) if nb else "none"

def biggest_group(w):
    seen = set(); best = 0
    for k in w.cells:
        if k in seen: continue
        comp = [k]; seen.add(k); i = 0
        while i < len(comp):
            cy, cx = comp[i]; i += 1
            for dy, dx in NEIGH:
                n = (cy + dy, cx + dx)
                if n in w.cells and n not in seen: seen.add(n); comp.append(n)
        best = max(best, len(comp))
    return best

def run(policy, seed, ticks):
    w = make(seed); rnd = random.Random(seed * 7919 + 1); rows = []; extinct_at = None; placed = 0; refused = 0
    for t in range(ticks):
        while w.free > 0 or w.count >= w.effective_cost(w.types[1]):   # the player spends everything they can, every tick
            r = policy(w, rnd)
            if r in ("free", "count"): placed += 1
            else: break
        w.step_tick(); w.log = []
        if not w.cells and extinct_at is None and w.free == 0 and w.count < w.effective_cost(w.types[1]): extinct_at = t
        if (t + 1) % (ticks // 6) == 0: rows.append((t + 1, len(w.cells), w.count, biggest_group(w)))
    return dict(rows=rows, extinct=extinct_at, placed=placed, chains=w.chains, final=len(w.cells), count=w.count)

def report(name, policy, seeds, ticks):
    out = [f"== policy: {name} | {seeds} seeds x {ticks} ticks (1 tick = 1 s at 1x) =="]
    res = [run(policy, s + 1, ticks) for s in range(seeds)]
    for i in range(len(res[0]["rows"])):
        t = res[0]["rows"][i][0]
        pop = sorted(r["rows"][i][1] for r in res); grp = [r["rows"][i][3] for r in res]; cnt = [r["rows"][i][2] for r in res]
        out.append(f"t={t:6d}s particles median {statistics.median(pop):7.1f} (min {pop[0]}, max {pop[-1]}) | biggest group median {statistics.median(grp):5.1f} | count median {statistics.median(cnt):9.1f}")
    ext = [r["extinct"] for r in res if r["extinct"] is not None]
    out.append(f"runs that ended with nothing left and no way to place: {len(ext)} of {seeds}" + (f" (first at t={min(ext)}s, median {statistics.median(ext)}s)" if ext else ""))
    out.append(f"placements made per run: median {statistics.median(r['placed'] for r in res)}; collapse chains per run: median {statistics.median(r['chains'] for r in res)}")
    return out

if __name__ == "__main__":
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 12
    ticks = int(sys.argv[2]) if len(sys.argv) > 2 else 3600
    lines = [f"Balance report M03e: starter cost {STARTER['cost']}, {STARTER['count_per_tick']} count/tick, spawn {STARTER['spawn_ppm']/10000}%/tick, cost rises: cost + cost*particles/{STARTER['cost_div']}, start {START_FREE} free + {START_COUNT} count (ALL PLACEHOLDERS)"]
    lines += report("spread (keep particles apart)", place_spread, seeds, ticks)
    lines += report("pack (always next to another particle)", place_pack, seeds, ticks)
    print("\n".join(lines))
    open("balance_report_m03e.txt", "w").write("\n".join(lines) + "\n")
