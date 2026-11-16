# Independent Python port of core_proto (rng.rs, detmath.rs, model.rs::event_driven, determinism_hash).
# Runs on x86; compared with the hashes the Rust build printed on the arm64 phone.
import struct
M64 = (1 << 64) - 1
class Rng:
    def __init__(s, seed):
        z = seed; s.s = []
        for _ in range(4):
            z = (z + 0x9e3779b97f4a7c15) & M64; x = z
            x = ((x ^ (x >> 30)) * 0xbf58476d1ce4e5b9) & M64
            x = ((x ^ (x >> 27)) * 0x94d049bb133111eb) & M64
            s.s.append(x ^ (x >> 31))
    def rotl(s, x, k): return ((x << k) | (x >> (64 - k))) & M64
    def next(s):
        S = s.s
        r = (s.rotl((S[1] * 5) & M64, 7) * 9) & M64
        t = (S[1] << 17) & M64
        S[2] ^= S[0]; S[3] ^= S[1]; S[1] ^= S[2]; S[0] ^= S[3]; S[2] ^= t; S[3] = s.rotl(S[3], 45)
        return r
    def unit(s): return (s.next() >> 11) * (1.0 / 9007199254740992.0)
    def unit_open0(s): return ((s.next() >> 11) + 1) * (1.0 / 9007199254740992.0)
LN2 = 0.6931471805599453
def ln(x):
    bits = struct.unpack("<Q", struct.pack("<d", x))[0]
    e = ((bits >> 52) & 0x7ff) - 1023
    m = struct.unpack("<d", struct.pack("<Q", (bits & 0x000fffffffffffff) | 0x3ff0000000000000))[0]
    if m > 1.4142135623730951: m *= 0.5; e += 1
    z = (m - 1.0) / (m + 1.0); z2 = z * z; term = z; sm = 0.0; k = 1.0
    for _ in range(16):
        sm += term / k; term *= z2; k += 2.0
    return float(e) * LN2 + 2.0 * sm
def ln_1m(q):
    if q < 1e-3: return -(q + q*q/2.0 + q*q*q/3.0 + q*q*q*q/4.0)
    return ln(1.0 - q)
def powu(x, n):
    r = 1.0
    for _ in range(n): r *= x
    return r
import math
W = H = 6; BASE = 0.002; CAP = 0.25; INF = 1 << 70
def bit(x, y): return 1 << (y * W + x)
INIT = sum(bit(*c) for c in [(0,0),(1,0),(2,0),(2,1),(2,2),(1,2),(4,0),(5,0),(5,1),(4,4),(0,5)])
def m_at(t): return 2.5 if 50 <= t < 100 else 1.0
def p_of(n, m):
    p = BASE * m * powu(2.0, n - 1)
    return CAP if p > CAP else p
def components(mask):
    seen = 0; out = []
    for i in range(W * H):
        b = 1 << i
        if mask & b and not seen & b:
            comp = 0; st = [i]
            while st:
                j = st.pop(); bj = 1 << j
                if comp & bj: continue
                comp |= bj; x, y = j % W, j // W
                for nx, ny in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)):
                    if 0 <= nx < W and 0 <= ny < H:
                        k = ny * W + nx
                        if mask & (1 << k) and not comp & (1 << k): st.append(k)
            seen |= comp; out.append(comp)
    return out
def cells_of(c): return [i for i in range(W*H) if c >> i & 1]
def event_driven(rng, variant):
    mask = INIT; first = -1; mid = None; items = []
    def draw(comp, t0):
        n = bin(comp).count("1"); p = p_of(n, m_at(t0)); q = 1.0 - powu(1.0 - p, n)
        if q <= 0.0: t = INF
        elif q >= 1.0: t = t0
        else: t = t0 + int(math.floor(ln(rng.unit_open0()) / ln_1m(q)))
        items.append([comp, t])
    def refresh(t0, all_):
        nonlocal items
        cur = components(mask)
        items[:] = [it for it in items if (not all_) and it[0] in cur]
        for c in cur:
            if not any(it[0] == c for it in items): draw(c, t0)
    refresh(0, False)
    keys = [50, 60, 75, 100]; cur_t = 0
    while True:
        t = 150
        for _, tt in items:
            if tt >= cur_t and tt < INF and tt < t: t = tt
        for k in keys:
            if k >= cur_t and k < t: t = k
        if t >= 150: break
        if t == 75: mid = bin(mask).count("1")
        if t == 60: mask |= bit(3, 1); refresh(t, False)
        if t in (50, 100): refresh(t, True)
        due = [c for c, tt in items if tt == t]
        if due:
            dead = 0
            for comp in due:
                if variant == "B": dead |= comp; continue
                cl = cells_of(comp); n = len(cl); p = p_of(n, m_at(t)); q = 1.0 - powu(1.0 - p, n)
                u = rng.unit()
                k = int(math.ceil(ln(1.0 - u * q) / ln_1m(p))) - 1
                i = max(0, min(n - 1, k)); dead |= 1 << cl[i]
                for j in range(i + 1, n):
                    if rng.unit() < p: dead |= 1 << cl[j]
            mask &= ~dead
            if first < 0: first = t
            refresh(t + 1, False)
        cur_t = t + 1
    pcn = bin(mask).count("1")
    return first, (pcn if mid is None else mid), pcn
def det_hash(seed, runs):
    rng = Rng(seed); h = 0xcbf29ce484222325
    for v in ("A", "B"):
        for _ in range(runs):
            f, a, b = event_driven(rng, v)
            for x in (f & M64, a, b):
                h ^= x; h = (h * 0x100000001b3) & M64
    return h ^ rng.next()
if __name__ == "__main__":
    phone = {1: "8fe431764a396449", 2: "05f5d4a4d9e575cf", 3: "6f401db9752f4ec1"}
    for seed in (1, 2, 3):
        h = "%016x" % det_hash(seed, 200)
        print(f"seed={seed} x86-python={h} phone-rust={phone[seed]} {'MATCH' if h == phone[seed] else 'DIFFERENT'}")
