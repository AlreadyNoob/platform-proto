# Independent Python port of core_proto/src/hazard.rs (all four strategies) to reproduce the phone's --determinism-hazard hashes on x86.
import heapq, math
from detport import Rng, ln, ln_1m, M64
NONE = -1; NEVER = (1 << 64) // 4
class Sim:
    def __init__(s, strat, comps, p_hot, q_hot, q_cold, q_star, bits, seed, steady=True):
        s.strat = strat; s.n = comps; s.p_hot, s.qh, s.qc, s.qs = p_hot, q_hot, q_cold, q_star
        s.rng = Rng(seed); s.q = [0.0]*comps; s.thr = [0]*comps; s.now = 0; s.events = 0; s.hash = 0xcbf29ce484222325
        s.heap = []; s.w = 1 << bits; s.mask = s.w - 1; wheel = strat in ("wheel", "hybrid")
        s.head = [NONE]*s.w if wheel else []; s.nxt = [NONE]*comps if wheel else []; s.due = [0]*comps if wheel else []
        s.over = []; s.hot = []; s.hot_pos = [NONE]*comps if strat == "hybrid" else []
        h = p_hot / q_hot; c = (1.0 - p_hot) / q_cold
        p_first = (h / (h + c)) if steady else p_hot
        for i in range(comps):
            s.set_q(i, q_hot if s.rng.unit() < p_first else q_cold)
        for i in range(comps):
            q = s.q[i]
            if strat == "heap": heapq.heappush(s.heap, (s.geo(q), i))
            elif strat == "wheel": s.schedule(i, s.geo(q))
            elif strat == "hybrid":
                if q >= q_star: s.hot_add(i)
                else: s.schedule(i, s.geo(q))
    def set_q(s, c, q):
        s.q[c] = q; s.thr[c] = M64 if q >= 1.0 else int(q * 18446744073709551616.0)
    def draw_q(s): return s.qh if s.rng.unit() < s.p_hot else s.qc
    def geo(s, q):
        if q <= 0.0: return NEVER
        if q >= 1.0: return 0
        return int(math.floor(ln(s.rng.unit_open0()) / ln_1m(q)))
    def schedule(s, c, t):
        s.due[c] = t
        if t < s.now + s.w:
            b = t & s.mask; s.nxt[c] = s.head[b]; s.head[b] = c
        else: s.over.append(c)
    def rescan(s, t):
        over = s.over; s.over = []
        for c in over:
            d = s.due[c]
            if d < t + s.w:
                b = d & s.mask; s.nxt[c] = s.head[b]; s.head[b] = c
            else: s.over.append(c)
    def hot_add(s, c): s.hot_pos[c] = len(s.hot); s.hot.append(c)
    def hot_remove(s, c):
        p = s.hot_pos[c]
        if p == NONE: return
        last = s.hot.pop()
        if last != c: s.hot[p] = last; s.hot_pos[last] = p
        s.hot_pos[c] = NONE
    def record(s, t, c):
        s.events += 1
        s.hash ^= t; s.hash = (s.hash * 0x100000001b3) & M64
        s.hash ^= c; s.hash = (s.hash * 0x100000001b3) & M64
    def collapse(s, c, t):
        s.record(t, c)
        if s.strat == "hybrid" and s.q[c] >= s.qs: s.hot_remove(c)
        q = s.draw_q(); s.set_q(c, q)
        if s.strat == "heap": heapq.heappush(s.heap, (t + 1 + s.geo(q), c))
        elif s.strat == "wheel": s.schedule(c, t + 1 + s.geo(q))
        elif s.strat == "hybrid":
            if q >= s.qs: s.hot_add(c)
            else: s.schedule(c, t + 1 + s.geo(q))
    def tick_wheel(s):
        t = s.now
        if t & s.mask == 0 and s.over: s.rescan(t)
        b = t & s.mask; c = s.head[b]; s.head[b] = NONE; scratch = []
        while c != NONE: scratch.append(c); c = s.nxt[c]
        if s.strat == "hybrid":
            for i in range(len(s.hot)):
                h = s.hot[i]
                if s.rng.next() < s.thr[h]: scratch.append(h)
        for c in scratch: s.collapse(c, t)
        s.now = t + 1
    def run_until(s, target):
        if s.strat == "brute":
            while s.now < target:
                t = s.now
                for c in range(s.n):
                    if s.rng.next() < s.thr[c]: s.collapse(c, t)
                s.now = t + 1
        elif s.strat == "heap":
            while s.heap and s.heap[0][0] < target:
                t, c = heapq.heappop(s.heap); s.collapse(c, t)
            if target > s.now: s.now = target
        else:
            while s.now < target: s.tick_wheel()
phone = {"brute": (39051, "c6192bc0d4e6d6e2"), "heap": (37637, "c68b73b07b025628"),
         "wheel": (37637, "66bfb35d93919c91"), "hybrid": (37274, "101ce1486d4693e9")}
for st in ("brute", "heap", "wheel", "hybrid"):
    s = Sim(st, 2000, 0.9, 0.02, 1e-4, 0.003, 16, 1); s.run_until(20000)
    h = "%016x" % s.hash; pe, ph = phone[st]
    print(f"{st:7s} x86-python events={s.events} hash={h} | phone-rust events={pe} hash={ph} | {'MATCH' if (s.events, h) == (pe, ph) else 'DIFFERENT'}")
