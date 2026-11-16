#!/usr/bin/env python3
"""Statistical validation: event-driven collapse scheduling vs brute-force per-tick rolls.

PROTOTYPE MODEL ONLY. All numbers (base p, cap, 2^(n-1) curve, event multiplier) are TEST VALUES,
not game balance. 'Variant A' = each collapsing particle is removed (component may split).
'Variant B' = a collapse takes the particle's whole connected component (chain, instantaneous).
Per-tick semantics: at tick t every particle rolls once with p computed from the state at the START
of tick t (after scheduled actions). Orthogonal adjacency on a 6x6 board.
"""
import sys, json, math, time, random
from functools import lru_cache
import numpy as np
from scipy.stats import chi2_contingency

W = H = 6
INIT_CELLS = [(0,0),(1,0),(2,0),(2,1),(2,2),(1,2),   # 6-cell cluster
              (4,0),(5,0),(5,1),                      # 3-cell cluster
              (4,4),(0,5)]                            # two singles
PLACE_TICK, PLACE_CELL = 60, (3,1)                    # joins 6-cluster; also touches nothing else
BASE, CAP = 0.002, 0.25
M_WINDOWS = [(50,100,2.5)]                            # multiplier 2.5 on base p during [50,100)
T_MID, T_END = 75, 150

def bit(c): return 1 << (c[1]*W + c[0])
def m_at(t):
    for a,b,m in M_WINDOWS:
        if a <= t < b: return m
    return 1.0
M_BOUNDS = sorted({x for a,b,_ in M_WINDOWS for x in (a,b)})
INIT = sum(bit(c) for c in INIT_CELLS)

@lru_cache(maxsize=None)
def comps(mask):
    seen = 0; out = []
    for i in range(W*H):
        b = 1 << i
        if mask & b and not seen & b:
            comp = 0; stack = [i]
            while stack:
                j = stack.pop(); bj = 1 << j
                if comp & bj: continue
                comp |= bj
                x, y = j % W, j // W
                for nx, ny in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)):
                    if 0 <= nx < W and 0 <= ny < H:
                        k = ny*W+nx
                        if mask & (1<<k) and not comp & (1<<k): stack.append(k)
            seen |= comp; out.append(comp)
    return tuple(out)

@lru_cache(maxsize=None)
def cells_of(comp): return tuple(i for i in range(W*H) if comp >> i & 1)
def pc(x): return bin(x).count("1")
def p_of(n, m): return min(CAP, BASE * m * 2.0**(n-1))

def brute(rng, variant):
    mask = INIT; first = None; mid = None
    for t in range(T_END):
        if t == T_MID: mid = pc(mask)
        if t == PLACE_TICK: mask |= bit(PLACE_CELL)
        mt = m_at(t); dead = 0
        for comp in comps(mask):
            p = p_of(pc(comp), mt)
            hit = [i for i in cells_of(comp) if rng.random() < p]
            if hit: dead |= comp if variant == "B" else sum(1 << i for i in hit)
        if dead:
            mask &= ~dead
            if first is None: first = t
    return (-1 if first is None else first, mid, pc(mask))

INF = 10**9
def event_driven(rng, variant, bug=None):
    mask = INIT; first = None; mid = None
    sched = {}
    def draw(comp, t0):
        n = pc(comp); p = p_of(n, m_at(t0))
        q = p if bug == "forget_n" else 1 - (1-p)**n
        if q <= 0: sched[comp] = INF; return
        if q >= 1: sched[comp] = t0; return
        u = 1.0 - rng.random()
        sched[comp] = t0 + int(math.floor(math.log(u) / math.log(1-q)))
    def refresh(t0, redraw_all=False):
        cur = set(comps(mask))
        for c in list(sched):
            if c not in cur or redraw_all: del sched[c]
        for c in cur:
            if c not in sched: draw(c, t0)
    refresh(0)
    keys = sorted(set(M_BOUNDS + [PLACE_TICK, T_MID]))
    cur = 0                                  # earliest tick not yet processed
    while True:
        pending = [T for T in sched.values() if cur <= T < INF]
        nk = [k for k in keys if k >= cur]
        t = min(pending + nk + [T_END])
        if t >= T_END: break
        if t == T_MID: mid = pc(mask)
        if t == PLACE_TICK:
            mask |= bit(PLACE_CELL)
            refresh(t)                       # only components whose cell-set changed are redrawn
        if t in M_BOUNDS and bug != "stale_p":
            refresh(t, redraw_all=True)      # p changed for everyone: redraw (memoryless => exact)
        due = [c for c, T in sched.items() if T == t]
        if due:
            dead = 0
            for comp in due:
                if variant == "B": dead |= comp; continue
                cl = cells_of(comp); n = len(cl); p = p_of(n, m_at(t)); q = 1 - (1-p)**n
                u = rng.random()
                i = max(0, min(n-1, int(math.ceil(math.log(1 - u*q) / math.log(1-p))) - 1))
                dead |= 1 << cl[i]
                for j in range(i+1, n):
                    if rng.random() < p: dead |= 1 << cl[j]
            mask &= ~dead
            if first is None: first = t
            refresh(t + 1)
        cur = t + 1
    if mid is None: mid = pc(mask)
    return (-1 if first is None else first, mid, pc(mask))

def hist(vals, edges=None):
    a = np.array(vals)
    if edges is not None: a = np.digitize(a, edges)
    return np.bincount(a - a.min() if edges is None else a, minlength=0), (a.min() if edges is None else 0)

def compare(name, xa, xb, edges=None):
    a = np.array(xa); b = np.array(xb)
    if edges is not None: a = np.digitize(a, edges); b = np.digitize(b, edges)
    lo = min(a.min(), b.min()); hi = max(a.max(), b.max())
    ha = np.bincount(a - lo, minlength=hi-lo+1); hb = np.bincount(b - lo, minlength=hi-lo+1)
    # merge sparse bins left-to-right until expected count >= 5 in both
    tab = np.vstack([ha, hb]).T; rows = []; acc = np.zeros(2, dtype=int)
    for r in tab:
        acc += r
        if acc.min() >= 5: rows.append(acc.copy()); acc[:] = 0
    if acc.sum() and rows: rows[-1] += acc
    tab = np.array(rows)
    chi, p, dof, _ = chi2_contingency(tab)
    return {"stat": name, "bins": int(len(tab)), "chi2": round(float(chi),2), "dof": int(dof), "p": float(p),
            "mean_brute": round(float(a.mean()),4), "mean_event": round(float(b.mean()),4)}

def run(variant, bug, R, seed):
    rb = random.Random(seed); re_ = random.Random(seed + 7919)
    t0 = time.time(); B = [brute(rb, variant) for _ in range(R)]; tb = time.time() - t0
    t0 = time.time(); E = [event_driven(re_, variant, bug) for _ in range(R)]; te = time.time() - t0
    fb = [x[0] for x in B]; fe = [x[0] for x in E]
    edges = list(range(0, 20)) + list(range(20, T_END+1, 10))
    res = [compare("first_collapse_tick(1-tick bins to 20 then 10-tick bins; -1=none)", fb, fe, edges),
           compare("survivors_at_tick_75", [x[1] for x in B], [x[1] for x in E]),
           compare("survivors_at_tick_150", [x[2] for x in B], [x[2] for x in E])]
    return {"variant": variant, "bug": bug, "runs": R, "seed": seed, "sec_brute": round(tb,1), "sec_event": round(te,1), "tests": res}

if __name__ == "__main__":
    R = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    out = []
    plan = [("A", None, 1001), ("B", None, 2002), ("A", "stale_p", 3003), ("B", "stale_p", 4004),
            ("A", "forget_n", 5005), ("B", "forget_n", 6006)]
    for v, bug, seed in plan:
        r = run(v, bug, R, seed); out.append(r)
        print(json.dumps(r), flush=True)
    json.dump(out, open("results/equivalence_results.json", "w"), indent=1)
