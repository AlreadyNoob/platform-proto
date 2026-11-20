//! Collapse core (03a) + economy core (03b): the EXECUTABLE SPECIFICATION of the rules decided so far (D-025, D-029..D-037).
//! Economy (03b): a count ledger, free placements, per-type settings (cost, count per tick, spawn chance, blocked-spawn policy, payment order).
//! EVERY economy number in here is a PLACEHOLDER (D-037) until the owner tunes it: see docs/TUNING_PLACEHOLDERS.md.
//! A straightforward per-tick engine: it rebuilds the groups every tick, so it is correct and deterministic but NOT the scalable
//! engine (that comes later and must reproduce this one). Integers only. Mirrors tools/ref/collapse_ref.py exactly.
//! Rules, in short: sparse grid, orthogonal adjacency; every particle of a group of n has risk min(cap, base * 2^(n-1)) per tick (n fixed at
//! the start of the tick); each tick particles roll in row-major order; a collapse resolves its whole chain at once (breadth first, neighbour
//! order up/right/down/left); every linked neighbour rolls once per chain at its own group risk; collapsed particles are removed.
use crate::rng::{RngStreams, NUM_STREAMS, STREAM_COLLAPSE, STREAM_GENESIS};
use crate::save::{fnv1a64, WorldState};
use crate::world::World;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Probabilities are fixed point: p32 = probability * 2^32.
pub const P32_ONE: u64 = 1u64 << 32;

pub fn p32_from_ppm(ppm: u64) -> u64 {
    (((ppm as u128) << 32) / 1_000_000u128) as u64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollapseModel {
    pub base_p32: u64,
    pub cap_p32: u64,
}

impl CollapseModel {
    pub fn from_ppm(base_ppm: u64, cap_ppm: u64) -> Self {
        CollapseModel { base_p32: p32_from_ppm(base_ppm), cap_p32: p32_from_ppm(cap_ppm) }
    }

    /// Risk per tick for each particle of a group of `group_size` (D-029): min(cap, base * 2^(n-1)); never overflows.
    pub fn risk_p32(&self, group_size: u32) -> u64 {
        let shift = group_size.saturating_sub(1).min(64);
        let v = (self.base_p32 as u128) << shift;
        v.min(self.cap_p32 as u128) as u64
    }
}

impl Default for CollapseModel {
    /// GDD example as the starting default (owner, Q-3): 0.1% base, doubling per linked particle, 10% cap.
    fn default() -> Self {
        CollapseModel::from_ppm(1000, 100_000)
    }
}

/// One collapsed particle, in the order the chain resolved it. This is what the presentation turns into an animated wave (D-025).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollapseRecord {
    pub tick: u64,
    pub x: i32,
    pub y: i32,
    pub chain: u64,
    pub depth: u32,
}

/// What happens when a particle's spawn finds no free neighbouring cell (placeholder behaviours, per type, D-037).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockedPolicy {
    /// The spawn is lost.
    Skip,
    /// The spawn is converted into this much count.
    ConvertToCount(u64),
}

/// Per-particle-type settings. ALL VALUES ARE PLACEHOLDERS until the owner tunes them (D-037, docs/TUNING_PLACEHOLDERS.md).
/// Spawn placement is `Adjacent` (a random free orthogonal neighbour) for every type for now; other rules per type come later (D-034).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeConfig {
    pub id: u16,
    /// Count paid to place one particle of this type when it is not paid with a free placement.
    pub cost: u64,
    /// Count added to the ledger every tick by each particle of this type (D-027).
    pub count_per_tick: u64,
    /// Chance per tick (p32) that the particle also spawns one particle on the grid (D-027). PLACEHOLDER split rule.
    pub spawn_p32: u64,
    pub spawn_type: u16,
    pub blocked: BlockedPolicy,
    /// true: spend a free placement before count (D-035); false: pay count first and use a free placement only if count is short.
    pub free_first: bool,
}

impl TypeConfig {
    /// The documented placeholder defaults: costs 1, makes 1 count per tick, never spawns, spawns its own type if enabled.
    pub fn placeholder(id: u16) -> Self {
        TypeConfig { id, cost: 1, count_per_tick: 1, spawn_p32: 0, spawn_type: id, blocked: BlockedPolicy::Skip, free_first: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceError {
    UnknownType,
    Occupied,
    Insufficient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paid {
    Free,
    Count,
}

// Cells are keyed (y, x) so that BTree order is row-major, which is the deterministic processing order.
type Key = (i32, i32);
const NEIGHBOURS: [(i32, i32); 4] = [(-1, 0), (0, 1), (1, 0), (0, -1)]; // (dy, dx): up, right, down, left

pub struct ParticleWorld {
    tick: u64,
    cells: BTreeSet<Key>,
    kinds: BTreeMap<Key, u16>,
    types: BTreeMap<u16, TypeConfig>,
    count: u64,
    free: u64,
    model: CollapseModel,
    rng: RngStreams,
    log: Vec<CollapseRecord>,
    log_cap: usize,
    dropped: u64,
    chains: u64,
    /// Bumped whenever the set of particles changes (placement, undo, collapse, spawn). Not saved, not part of any hash: it only
    /// tells the presentation that its copy of the cells is out of date.
    rev: u64,
}

impl ParticleWorld {
    pub fn new(seed: u64, model: CollapseModel) -> Self {
        // Type 0 is the "plain" debug particle: free to place by `place`, produces nothing, never spawns.
        let mut types = BTreeMap::new();
        types.insert(
            0u16,
            TypeConfig { id: 0, cost: 0, count_per_tick: 0, spawn_p32: 0, spawn_type: 0, blocked: BlockedPolicy::Skip, free_first: true },
        );
        ParticleWorld {
            tick: 0,
            cells: BTreeSet::new(),
            kinds: BTreeMap::new(),
            types,
            count: 0,
            free: 0,
            model,
            rng: RngStreams::new(seed),
            log: Vec::new(),
            log_cap: 100_000,
            dropped: 0,
            chains: 0,
            rev: 0,
        }
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.cells.contains(&(y, x))
    }

    pub fn chains_started(&self) -> u64 {
        self.chains
    }

    pub fn dropped_records(&self) -> u64 {
        self.dropped
    }

    pub fn rng_states(&self) -> Vec<[u64; 4]> {
        self.rng.states()
    }

    /// DEBUG placement of a plain (type 0) particle: free, ignores the economy. Returns false (and changes nothing) if occupied.
    pub fn place(&mut self, x: i32, y: i32) -> bool {
        let key = (y, x);
        if self.cells.insert(key) {
            self.kinds.insert(key, 0);
            self.rev += 1;
            true
        } else {
            false
        }
    }

    pub fn define_type(&mut self, cfg: TypeConfig) {
        self.types.insert(cfg.id, cfg);
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn free_placements(&self) -> u64 {
        self.free
    }

    pub fn set_count(&mut self, n: u64) {
        self.count = n;
    }

    pub fn add_count(&mut self, n: u64) {
        self.count = self.count.saturating_add(n);
    }

    /// Upgrades and events can grant free placements (D-036).
    pub fn grant_free_placements(&mut self, n: u64) {
        self.free = self.free.saturating_add(n);
    }

    pub fn kind_at(&self, x: i32, y: i32) -> Option<u16> {
        self.kinds.get(&(y, x)).copied()
    }

    /// Changes whenever the particles change (see the field comment). Presentation bookkeeping only.
    pub fn revision(&self) -> u64 {
        self.rev
    }

    /// Count a particle of this type costs when paid with count (None for an undefined type).
    pub fn type_cost(&self, type_id: u16) -> Option<u64> {
        self.types.get(&type_id).map(|c| c.cost)
    }

    /// Exact inverse of a successful `place_typed`: removes the particle at (x, y) if it is of type `type_id` and gives back what was paid
    /// (one free placement, or `cost` count). Returns false and changes nothing if no such particle is there (e.g. it collapsed).
    pub fn unplace(&mut self, x: i32, y: i32, type_id: u16, paid: Paid, cost: u64) -> bool {
        let key = (y, x);
        if self.kinds.get(&key).copied() != Some(type_id) {
            return false;
        }
        self.cells.remove(&key);
        self.kinds.remove(&key);
        match paid {
            Paid::Free => self.free = self.free.saturating_add(1),
            Paid::Count => self.count = self.count.saturating_add(cost),
        }
        self.rev += 1;
        true
    }

    /// Flat list `[x, y, type, x, y, type, ...]` in row-major order, at most `max_cells` particles. The bool is true if the list was cut.
    pub fn cell_triples(&self, max_cells: usize) -> (Vec<i32>, bool) {
        let mut v: Vec<i32> = Vec::with_capacity(self.kinds.len().min(max_cells) * 3);
        let mut cut = false;
        for ((y, x), t) in self.kinds.iter() {
            if v.len() / 3 >= max_cells {
                cut = true;
                break;
            }
            v.push(*x);
            v.push(*y);
            v.push(*t as i32);
        }
        (v, cut)
    }

    /// Economy placement. Checks in this order: unknown type, occupied cell, can-pay. Pays with a free placement or count according to
    /// the type's `free_first` setting. Changes nothing on failure.
    pub fn place_typed(&mut self, x: i32, y: i32, type_id: u16) -> Result<Paid, PlaceError> {
        let cfg = match self.types.get(&type_id) {
            Some(c) => *c,
            None => return Err(PlaceError::UnknownType),
        };
        let key = (y, x);
        if self.cells.contains(&key) {
            return Err(PlaceError::Occupied);
        }
        let can_count = self.count >= cfg.cost;
        let can_free = self.free > 0;
        let pay = if cfg.free_first {
            if can_free {
                Paid::Free
            } else if can_count {
                Paid::Count
            } else {
                return Err(PlaceError::Insufficient);
            }
        } else if can_count {
            Paid::Count
        } else if can_free {
            Paid::Free
        } else {
            return Err(PlaceError::Insufficient);
        };
        match pay {
            Paid::Free => self.free -= 1,
            Paid::Count => self.count -= cfg.cost,
        }
        self.cells.insert(key);
        self.kinds.insert(key, type_id);
        self.rev += 1;
        Ok(pay)
    }

    /// Records produced since the last call (bounded by the log capacity; overflow is counted in `dropped_records`).
    pub fn take_log(&mut self) -> Vec<CollapseRecord> {
        std::mem::take(&mut self.log)
    }

    /// Risk of every particle for the CURRENT state: groups are the orthogonally connected components.
    pub fn risks(&self) -> BTreeMap<(i32, i32), u64> {
        let mut risk: BTreeMap<Key, u64> = BTreeMap::new();
        let mut seen: BTreeSet<Key> = BTreeSet::new();
        for k in self.cells.iter() {
            if seen.contains(k) {
                continue;
            }
            let mut comp: Vec<Key> = vec![*k];
            seen.insert(*k);
            let mut i = 0;
            while i < comp.len() {
                let (cy, cx) = comp[i];
                i += 1;
                for (dy, dx) in NEIGHBOURS.iter() {
                    let n = (cy.wrapping_add(*dy), cx.wrapping_add(*dx));
                    if self.cells.contains(&n) && !seen.contains(&n) {
                        seen.insert(n);
                        comp.push(n);
                    }
                }
            }
            let r = self.model.risk_p32(comp.len() as u32);
            for c in comp.iter() {
                risk.insert(*c, r);
            }
        }
        risk
    }

    fn roll(&mut self, p32: u64) -> bool {
        (self.rng.stream(STREAM_COLLAPSE).next_u64() >> 32) < p32
    }

    fn push_log(&mut self, rec: CollapseRecord) {
        if self.log.len() < self.log_cap {
            self.log.push(rec);
        } else {
            self.dropped += 1;
        }
    }

    fn run_chain(&mut self, origin: Key, risks: &BTreeMap<Key, u64>) {
        let chain_id = self.chains;
        self.chains += 1;
        let mut visited: BTreeSet<Key> = BTreeSet::new();
        let mut queue: VecDeque<(Key, u32)> = VecDeque::new();
        visited.insert(origin);
        queue.push_back((origin, 0));
        while let Some((k, depth)) = queue.pop_front() {
            self.cells.remove(&k);
            self.kinds.remove(&k);
            self.rev += 1;
            let rec = CollapseRecord { tick: self.tick, x: k.1, y: k.0, chain: chain_id, depth };
            self.push_log(rec);
            for (dy, dx) in NEIGHBOURS.iter() {
                let n = (k.0.wrapping_add(*dy), k.1.wrapping_add(*dx));
                if !self.cells.contains(&n) || visited.contains(&n) {
                    continue;
                }
                visited.insert(n);
                let p = risks.get(&n).copied().unwrap_or(0);
                if self.roll(p) {
                    queue.push_back((n, depth + 1));
                }
            }
        }
    }

    fn genesis_roll(&mut self, p32: u64) -> bool {
        (self.rng.stream(STREAM_GENESIS).next_u64() >> 32) < p32
    }

    /// Generation phase (after the collapse phase): every surviving particle (as of the start of this phase, in row-major order) adds its
    /// count; if its type spawns, it rolls the spawn chance on the genesis stream and a success places one particle in a random free
    /// orthogonal neighbour (draw `below(k)` over the free neighbours in order up/right/down/left). New particles start generating next tick.
    fn generate(&mut self) {
        let order: Vec<(Key, u16)> = self.kinds.iter().map(|(k, t)| (*k, *t)).collect();
        for (k, t) in order {
            let cfg = match self.types.get(&t) {
                Some(c) => *c,
                None => continue,
            };
            self.count = self.count.saturating_add(cfg.count_per_tick);
            if cfg.spawn_p32 == 0 {
                continue;
            }
            if !self.genesis_roll(cfg.spawn_p32) {
                continue;
            }
            let mut free_nb: Vec<Key> = Vec::new();
            for (dy, dx) in NEIGHBOURS.iter() {
                let n = (k.0.wrapping_add(*dy), k.1.wrapping_add(*dx));
                if !self.cells.contains(&n) {
                    free_nb.push(n);
                }
            }
            if free_nb.is_empty() {
                if let BlockedPolicy::ConvertToCount(n) = cfg.blocked {
                    self.count = self.count.saturating_add(n);
                }
                continue;
            }
            let i = self.rng.stream(STREAM_GENESIS).below(free_nb.len() as u64) as usize;
            if self.types.contains_key(&cfg.spawn_type) {
                let n = free_nb[i];
                self.cells.insert(n);
                self.kinds.insert(n, cfg.spawn_type);
                self.rev += 1;
            }
        }
    }

    /// Processes the current tick (collapse phase, then generation phase) and moves on to the next one.
    pub fn step_tick(&mut self) {
        let risks = self.risks();
        let order: Vec<Key> = self.cells.iter().copied().collect();
        for k in order {
            if !self.cells.contains(&k) {
                continue; // already taken by an earlier chain this tick
            }
            let p = risks.get(&k).copied().unwrap_or(0);
            if self.roll(p) {
                self.run_chain(k, &risks);
            }
        }
        self.generate();
        self.tick += 1;
    }

    /// Everything needed to rebuild this world except the tick and the RNG states, which the save header carries (`rng_states()`).
    pub fn export_state(&self) -> WorldState {
        WorldState {
            base_p32: self.model.base_p32,
            cap_p32: self.model.cap_p32,
            count: self.count,
            free: self.free,
            chains: self.chains,
            types: self.types.values().copied().collect(),
            cells: self.kinds.iter().map(|((y, x), t)| (*y, *x, *t)).collect(),
        }
    }

    /// Rebuilds a world from a save. Continuing it gives exactly the same future as the original (same RNG states, same state).
    /// The presentation log and the dropped-record counter are not saved. Fails on inconsistent data instead of guessing.
    pub fn restore(tick: u64, rng_states: &[[u64; 4]], ws: &WorldState) -> Result<ParticleWorld, String> {
        if rng_states.len() != NUM_STREAMS {
            return Err("wrong number of RNG streams".to_string());
        }
        let mut types: BTreeMap<u16, TypeConfig> = BTreeMap::new();
        for t in ws.types.iter() {
            types.insert(t.id, *t);
        }
        types.entry(0u16).or_insert(TypeConfig { id: 0, cost: 0, count_per_tick: 0, spawn_p32: 0, spawn_type: 0, blocked: BlockedPolicy::Skip, free_first: true });
        let mut cells: BTreeSet<Key> = BTreeSet::new();
        let mut kinds: BTreeMap<Key, u16> = BTreeMap::new();
        for (y, x, k) in ws.cells.iter() {
            if !types.contains_key(k) {
                return Err("a particle has an undefined type".to_string());
            }
            if !cells.insert((*y, *x)) {
                return Err("duplicate cell".to_string());
            }
            kinds.insert((*y, *x), *k);
        }
        Ok(ParticleWorld {
            tick,
            cells,
            kinds,
            types,
            count: ws.count,
            free: ws.free,
            model: CollapseModel { base_p32: ws.base_p32, cap_p32: ws.cap_p32 },
            rng: RngStreams::from_states(rng_states),
            log: Vec::new(),
            log_cap: 100_000,
            dropped: 0,
            chains: ws.chains,
            rev: 0,
        })
    }

    /// Fingerprint of everything: `state_hash` plus count, free placements and every particle's type.
    pub fn full_hash(&self) -> u64 {
        let mut b: Vec<u8> = Vec::new();
        b.extend_from_slice(&self.state_hash().to_le_bytes());
        b.extend_from_slice(&self.count.to_le_bytes());
        b.extend_from_slice(&self.free.to_le_bytes());
        for ((y, x), t) in self.kinds.iter() {
            b.extend_from_slice(&y.to_le_bytes());
            b.extend_from_slice(&x.to_le_bytes());
            b.extend_from_slice(&t.to_le_bytes());
        }
        fnv1a64(&b)
    }

    /// FNV-1a over the tick, all cells (row-major, y then x) and all RNG states: a compact fingerprint of the whole state.
    pub fn state_hash(&self) -> u64 {
        let mut b: Vec<u8> = Vec::new();
        b.extend_from_slice(&self.tick.to_le_bytes());
        for (y, x) in self.cells.iter() {
            b.extend_from_slice(&y.to_le_bytes());
            b.extend_from_slice(&x.to_le_bytes());
        }
        for st in self.rng.states().iter() {
            for w in st.iter() {
                b.extend_from_slice(&w.to_le_bytes());
            }
        }
        fnv1a64(&b)
    }
}

impl World for ParticleWorld {
    fn step_to(&mut self, tick: u64) {
        while self.tick < tick {
            self.step_tick();
        }
    }
}
