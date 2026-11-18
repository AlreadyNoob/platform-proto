#!/usr/bin/env python3
"""Reference model for the hybrid scheduler (D-012). Geometry-free: C component slots, each with a per-tick
collapse probability q. On collapse the slot is respawned with a freshly drawn q (hot w.p. p_hot else cold) and
starts rolling at the NEXT tick. Strategies: brute (roll every slot every tick), heap, wheel (hashed timing wheel with
overflow), hybrid (slots with q >= QSTAR rolled per tick, others in the wheel). All must give the same process.
TEST VALUES only. Used to validate the algorithm before the Rust port."""
import sys, math, random, heapq, json, time
import numpy as np
from scipy.stats import chi2_contingency

def geo(rng, q):                      # failures before first success, success prob q per tick
    u = 1.0 - rng.random()
    return int(math.floor(math.log(u) / math.log(1.0 - q)))

class Sim:
    def __init__(s, strat, C, p_hot, q_hot, q_cold, qstar, bits, seed, bias=1.0, bug=None):
        s.strat, s.C, s.p_hot, s.qh, s.qc, s.qstar = strat, C, p_hot, q_hot, q_cold, qstar
        s.rng = random.Random(seed); s.bias, s.bug = bias, bug
        s.W = 1 << bits; s.mask = s.W - 1
        s.q = [s.draw() for _ in range(C)]
        s.events = []                  # (tick, slot)
        s.now = 0
        if strat == "heap":
            s.heap = [(geo(s.rng, s.q[c]), c) for c in range(C)]; heapq.heapify(s.heap)
        if strat in ("wheel", "hybrid"):
            s.head = [-1] * s.W; s.nxt = [-1] * C; s.due = [0] * C; s.over = []; s.hot = set()
            for c in range(C):
                if strat == "hybrid" and s.q[c] >= qstar: s.hot.add(c)
                else: s.schedule(c, geo(s.rng, s.q[c]))
    def draw(s): return s.qh if s.rng.random() < s.p_hot else s.qc
    def schedule(s, c, t):
        s.due[c] = t
        if t < s.now + s.W:
            b = t & s.mask; s.nxt[c] = s.head[b]; s.head[b] = c
        else: s.over.append(c)
    def respawn(s, c, t):              # collapse of slot c at tick t
        s.events.append((t, c)); s.q[c] = s.draw()
        if s.strat == "heap": heapq.heappush(s.heap, (t + 1 + geo(s.rng, s.q[c]), c))
        elif s.strat == "wheel": s.schedule(c, t + 1 + geo(s.rng, s.q[c]))
        elif s.strat == "hybrid":
            if s.q[c] >= s.qstar: s.hot.add(c)           # rolls from t+1
            else: s.schedule(c, t + 1 + geo(s.rng, s.q[c]))
    def run_until(s, T):
        if s.strat == "brute":
            for t in range(s.now, T):
                hit = [c for c in range(s.C) if s.rng.random() < s.q[c] * s.bias]
                for c in hit: s.events.append((t, c)); s.q[c] = s.draw()
            s.now = T; return
        if s.strat == "heap":
            while s.heap and s.heap[0][0] < T:
                t, c = heapq.heappop(s.heap); s.events.append((t, c)); s.q[c] = s.draw()
                heapq.heappush(s.heap, (t + 1 + geo(s.rng, s.q[c]), c))
            s.now = T; return
        while s.now < T:               # wheel / hybrid: visit every tick
            t = s.now
            if t & s.mask == 0 and s.over and s.bug != "no_rescan":
                keep = []
                for c in s.over:
                    if s.due[c] < t + s.W: b = s.due[c] & s.mask; s.nxt[c] = s.head[b]; s.head[b] = c
                    else: keep.append(c)
                s.over = keep
            b = t & s.mask; c = s.head[b]; s.head[b] = -1; due = []
            while c != -1: due.append(c); c = s.nxt[c]
            assert all(s.due[c] == t for c in due), "wheel invariant broken"
            hot_hits = [c for c in sorted(s.hot) if s.rng.random() < s.q[c]] if s.strat == "hybrid" else []
            for c in due + hot_hits:
                s.hot.discard(c)                  # a collapsing hot slot leaves the hot set; respawn re-adds if still hot
                s.respawn(c, t)
            s.now = t + 1

def observe(ev, T):
    tot = sum(1 for t, c in ev if t < T)
    s0 = [t for t, c in ev if c == 0 and t < T]
    return tot, len(s0), (s0[0] if s0 else -1)

def bins(vals, edges):
    return np.digitize(vals, edges)

def compare(xa, xb):
    a, b = np.array(xa), np.array(xb); lo = min(a.min(), b.min()); hi = max(a.max(), b.max())
    ha = np.bincount(a - lo, minlength=hi - lo + 1); hb = np.bincount(b - lo, minlength=hi - lo + 1)
    rows = []; acc = np.zeros(2, dtype=int)
    for r in np.vstack([ha, hb]).T:
        acc += r
        if acc.min() >= 5: rows.append(acc.copy()); acc[:] = 0
    if acc.sum() and rows: rows[-1] += acc
    chi, p, dof, _ = chi2_contingency(np.array(rows)); return float(chi), int(dof), float(p)

def run(strat, R, seed, T=150, C=8, bias=1.0, bug=None):
    out = []
    for i in range(R):
        s = Sim(strat, C, 0.5, 0.2, 0.01, 0.05, 5, seed + i, bias, bug); s.run_until(T); out.append(observe(s.events, T))
    return out

if __name__ == "__main__":
    R = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    t0 = time.time(); base = run("brute", R, 1_000_000)
    plan = [("heap", 1.0, None), ("wheel", 1.0, None), ("hybrid", 1.0, None),
            ("brute", 1.15, None), ("wheel", 1.0, "no_rescan")]
    res = []
    for strat, bias, bug in plan:
        other = run(strat, R, 5_000_000 if bias == 1.0 else 9_000_000, bias=bias, bug=bug)
        tests = []
        for name, idx in (("total_events", 0), ("slot0_events", 1), ("slot0_first_tick", 2)):
            edges = sorted(set(np.percentile([x[idx] for x in base], range(5, 100, 5)).tolist()))   # edges from the brute baseline
            chi, dof, p = compare(bins([x[idx] for x in base], edges), bins([x[idx] for x in other], edges))
            tests.append({"stat": name, "chi2": round(chi, 1), "dof": dof, "p": p})
        res.append({"strategy": strat, "bias": bias, "bug": bug, "runs": R, "tests": tests})
        print(json.dumps(res[-1]), flush=True)
    json.dump(res, open("hybrid_ref_results.json", "w"), indent=1); print("seconds", round(time.time() - t0))
