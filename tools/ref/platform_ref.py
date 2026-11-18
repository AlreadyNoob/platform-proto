#!/usr/bin/env python3
"""Python reference ports for Milestone 02a (independent of the Rust code).
Run:  python3 tools/ref/platform_ref.py [fixtures_dir]
It (1) self-tests the algorithms, (2) writes golden fixtures that the Rust tests must reproduce exactly."""
import sys, os, random
M = (1 << 64) - 1
GOLD = 0x9e3779b97f4a7c15

def rotl(x, k): return ((x << k) | (x >> (64 - k))) & M

def mix(z):
    z = ((z ^ (z >> 30)) * 0xbf58476d1ce4e5b9) & M
    z = ((z ^ (z >> 27)) * 0x94d049bb133111eb) & M
    return z ^ (z >> 31)

class Rng:
    def __init__(self, seed=None, state=None):
        if state is not None:
            self.s = list(state); return
        z = seed & M; s = []
        for _ in range(4):
            z = (z + GOLD) & M
            s.append(mix(z))
        self.s = s
    def next(self):
        s = self.s
        r = (rotl((s[1] * 5) & M, 7) * 9) & M
        t = (s[1] << 17) & M
        s[2] ^= s[0]; s[3] ^= s[1]; s[1] ^= s[2]; s[0] ^= s[3]; s[2] ^= t; s[3] = rotl(s[3], 45)
        return r
    def below(self, n):
        t = ((-n) & M) % n
        while True:
            r = self.next()
            if r >= t: return r % n

def stream_seed(seed, sid):
    return mix((seed + ((sid + 1) * GOLD)) & M)

NUM_STREAMS = 3
def streams(seed): return [Rng(stream_seed(seed, i)) for i in range(NUM_STREAMS)]

# ---------------- Clock / Governor (mirror of core/src/clock.rs and governor.rs) ----------------
BASE = 1; MAX_SPEED = 10_000
class Clock:
    def __init__(self, req):
        req = min(max(req, 1), MAX_SPEED); self.tick = 0; self.req = req; self.eff = req
        self.paused = False; self.acc = 0; self.owed = 0
    def set_requested(self, v):
        v = min(max(v, 1), MAX_SPEED)
        if self.eff == self.req: self.eff = v
        else: self.eff = min(self.eff, v)
        self.req = v
    def set_effective(self, v): self.eff = min(max(v, 1), self.req)
    def advance(self, dt):
        if self.paused: return
        self.acc += dt * self.eff * BASE
        self.owed += self.acc // 1_000_000; self.acc %= 1_000_000
    def take(self, budget):
        if self.paused: return 0
        n = min(self.owed, budget); self.owed -= n; self.tick += n; return n
    def forgive(self): self.owed = 0

INTERVAL_US = 250_000
P = dict(high_lag_us=1_000_000, low_lag_us=100_000, margin_pct=90, up_pct=125, base_wait=6, backoff_max=80, ok_reset=40)
class Governor:
    """Drops speed when lag is high; probes upward carefully; a probe that makes lag build up is reverted at once
    and the wait before the next probe doubles (so a device that cannot keep up is not repeatedly overloaded)."""
    def __init__(self):
        self.calm = 0; self.wait = P['base_wait']; self.since_raise = None; self.pre_raise = 1
    def _failed_probe(self):
        self.wait = min(self.wait * 2, P['backoff_max']); self.since_raise = None; self.calm = 0
    def update(self, req, eff, owed, done, interval_us):
        lag_us = owed * 1_000_000 // (eff * BASE)
        if lag_us > P['high_lag_us']:
            achieved = done * 1_000_000 // interval_us
            new = min(achieved * P['margin_pct'] // 100, eff * P['margin_pct'] // 100) // BASE
            new = max(new, 1)
            if self.since_raise is not None: self._failed_probe()
            else: self.calm = 0
            return (new, True)
        if lag_us <= P['low_lag_us']:
            self.calm += 1
            if self.since_raise is not None:
                self.since_raise += 1
                if self.since_raise > P['ok_reset']:
                    self.since_raise = None; self.wait = P['base_wait']
            if eff < req and self.calm >= self.wait:
                new = min(req, max(eff + 1, eff * P['up_pct'] // 100))
                self.pre_raise = eff; self.calm = 0; self.since_raise = 0
                return (new, False)
        else:
            if self.since_raise is not None:
                back = self.pre_raise; self._failed_probe()
                return (back, True)
            self.calm = 0
        return None

UNLIMITED = 1_000_000_000
def run_scenario(req, caps):
    """caps: list of device capacities in ticks/s, one per 250 ms interval."""
    c = Clock(req); g = Governor(); effs = []; maxlag = 0
    for cap in caps:
        c.advance(INTERVAL_US)
        done = c.take(cap * INTERVAL_US // 1_000_000)
        out = g.update(c.req, c.eff, c.owed, done, INTERVAL_US)
        effs.append(c.eff)  # effective speed that was in force during this interval
        if out:
            c.set_effective(out[0])
            if out[1]: c.forgive()
    return effs

SCENARIOS = {
    "s1_const_1000_req10000": (10_000, [1000] * 400),
    "s2_unlimited_req10000": (10_000, [UNLIMITED] * 100),
    "s3_step_5000_500_8000_req10000": (10_000, [5000] * 100 + [500] * 100 + [8000] * 200),
    "s4_req100_cap50_then_1000": (100, [50] * 80 + [1000] * 200),
}

# ---------------- Timing wheel (mirror of core/src/wheel.rs) ----------------
class Wheel:
    def __init__(self, size, start=0):
        s = 2
        while s < size: s *= 2
        self.size = s; self.mask = s - 1; self.now = start
        self.ring = [[] for _ in range(s)]; self.over = []
    def schedule(self, due, item):
        due = max(due, self.now)
        if due < self.now + self.size: self.ring[due & self.mask].append((due, item))
        else: self.over.append((due, item))
    def pop_tick(self):
        b = self.ring[self.now & self.mask]; out = []
        for d, it in b:
            assert d == self.now, "item in wrong bucket"
            out.append(it)
        b.clear()
        self.now += 1
        if self.now & (self.size // 2 - 1) == 0:
            limit = self.now + self.size; i = 0
            while i < len(self.over):
                if self.over[i][0] < limit:
                    d, it = self.over[i]; self.over[i] = self.over[-1]; self.over.pop()
                    self.ring[d & self.mask].append((d, it))
                else: i += 1
        return out

def wheel_selftest(seed, size, ticks, ops):
    rnd = random.Random(seed); w = Wheel(size); model = {}; nid = 0
    for t in range(ticks):
        for _ in range(rnd.randint(0, ops)):
            horizon = rnd.choice([0, 1, 3, size - 1, size, size + 1, 3 * size, rnd.randint(0, 5 * size)])
            due = w.now + horizon
            w.schedule(due, nid); model.setdefault(max(due, w.now), []).append(nid); nid += 1
        got = sorted(w.pop_tick()); want = sorted(model.pop(t, []))
        assert got == want, (seed, size, t, got, want)
    return nid

# ---------------- Save container (mirror of core/src/save.rs) ----------------
import struct
MAGIC = b"PLTF"; SCHEMA = 1
def fnv1a(data):
    h = 0xcbf29ce484222325
    for b in data:
        h ^= b; h = (h * 0x100000001b3) & M
    return h
def encode_save(seed, tick, speed, rng_states, label):
    p = struct.pack("<QQQI", seed, tick, speed, len(rng_states))
    for st in rng_states: p += struct.pack("<4Q", *st)
    lb = label.encode("utf-8"); p += struct.pack("<I", len(lb)) + lb
    body = MAGIC + struct.pack("<II", SCHEMA, len(p)) + p
    return body + struct.pack("<Q", fnv1a(body))

def main(fx):
    os.makedirs(fx, exist_ok=True)
    # RNG vectors
    lines = []
    for seed in (0, 1, 0xdeadbeefcafef00d):
        r = Rng(seed); lines.append(f"rng seed={seed:016x} " + " ".join(f"{r.next():016x}" for _ in range(6)))
    r = Rng(7); lines.append("below seed=7 n=10 " + " ".join(str(r.below(10)) for _ in range(12)))
    r = Rng(7); lines.append("below seed=7 n=3 " + " ".join(str(r.below(3)) for _ in range(12)))
    for sid in range(NUM_STREAMS):
        lines.append(f"stream_seed seed=0123456789abcdef id={sid} {stream_seed(0x0123456789abcdef, sid):016x}")
    st = streams(0x0123456789abcdef)
    lines.append("stream_first seed=0123456789abcdef " + " ".join(f"{s.next():016x}" for s in st))
    open(os.path.join(fx, "rng_vectors.txt"), "w").write("\n".join(lines) + "\n")
    # governor traces + property checks
    out = []
    for name, (req, caps) in SCENARIOS.items():
        e = run_scenario(req, caps); out.append(name + " " + ",".join(map(str, e)))
        assert max(e) <= req and min(e) >= 1, name
        SC = {"s1_const_1000_req10000": e, "s2_unlimited_req10000": e}
    s1 = run_scenario(*SCENARIOS["s1_const_1000_req10000"])
    s2 = run_scenario(*SCENARIOS["s2_unlimited_req10000"])
    assert all(x == 10_000 for x in s2), "governor must not engage when the device keeps up"
    assert s1[40] <= 1000, "governor must settle at or below device capacity"
    tail = s1[100:]; assert sum(1 for x in tail if x > 1000) <= len(tail) // 6, "too many overload probes in steady state"
    s3 = run_scenario(*SCENARIOS["s3_step_5000_500_8000_req10000"])
    assert s3[199] <= 500 and s3[399] > 5000, "governor must recover when capacity returns"
    open(os.path.join(fx, "governor_traces.txt"), "w").write("\n".join(out) + "\n")
    # wheel selftests
    n = 0
    for seed in range(30):
        for size in (2, 4, 8, 64):
            n += wheel_selftest(seed, size, 600, 6)
    print("wheel selftest ok,", n, "events checked")
    # save golden
    seed = 0x0123456789abcdef
    blob = encode_save(seed, 42, 100, [s.s for s in streams(seed)], "slot-one \u00e9")
    open(os.path.join(fx, "save_v1_golden.hex"), "w").write(blob.hex() + "\n")
    print("all reference checks passed; fixtures written to", fx)
    for name, (req, caps) in SCENARIOS.items():
        e = run_scenario(req, caps)
        print(name, "min/max/last effective:", min(e), max(e), e[-1])

if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.join(here, "..", "..", "core", "tests", "fixtures"))
