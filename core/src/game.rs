//! The game world as the simulation host sees it (Milestone 03d): the particle world (collapse rules + economy, 03a/03b) plus the
//! player's commands (placement, undo) and the data feeds for the presentation (cells, collapse records).
//! No new rules live here: placing is `ParticleWorld::place_typed`, undo is its exact inverse.
use crate::collapse::{CollapseModel, Paid, ParticleWorld, PlaceError, TypeConfig};
use crate::command::Command;
use crate::rng::RngStreams;
use crate::world::{World, WorldExport, WORLD_STATS};

/// The one particle type the game offers so far (PLACEHOLDER type; real types come with the design, D-034/D-037).
pub const BASIC_TYPE: u16 = 1;
/// Free placements a new dev game starts with. PLACEHOLDER: the real starting number is the owner's to pick (D-035).
pub const DEV_START_FREE: u64 = 20;
/// Result codes of a rejected command (in the snapshot as `reject_code`).
pub const REJECT_OCCUPIED: i64 = 1;
pub const REJECT_INSUFFICIENT: i64 = 2;
pub const REJECT_UNKNOWN_TYPE: i64 = 3;
pub const REJECT_NOTHING_TO_UNDO: i64 = 4;
/// Undo remembers this many placements (oldest forgotten first). The undo history is not saved.
pub const UNDO_LIMIT: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameSetup {
    pub seed: u64,
    pub start_free: u64,
    pub start_count: u64,
}

impl GameSetup {
    pub fn dev_default(seed: u64) -> Self {
        GameSetup { seed, start_free: DEV_START_FREE, start_count: 0 }
    }
}

#[derive(Clone, Copy, Debug)]
struct UndoEntry {
    x: i32,
    y: i32,
    type_id: u16,
    paid: Paid,
    cost: u64,
    stroke: u32,
}

pub struct GameWorld {
    inner: ParticleWorld,
    undo: Vec<UndoEntry>,
    /// Counts rejections that matter to the player (not enough to pay, unknown type, nothing to undo). Occupied cells are ignored quietly.
    reject_seq: u64,
    reject_code: i64,
}

impl GameWorld {
    fn wrap(inner: ParticleWorld) -> GameWorld {
        GameWorld { inner, undo: Vec::new(), reject_seq: 0, reject_code: 0 }
    }

    /// A new game: the basic type, the starting free placements and count of the setup.
    pub fn new_game(setup: &GameSetup) -> GameWorld {
        let mut w = ParticleWorld::new(setup.seed, CollapseModel::default());
        w.define_type(TypeConfig::placeholder(BASIC_TYPE));
        w.grant_free_placements(setup.start_free);
        w.add_count(setup.start_count);
        GameWorld::wrap(w)
    }

    /// A new game that stands at `tick` (used for saves that have no world, D-040: the caller decides, and the decision is "new world").
    pub fn new_game_at(setup: &GameSetup, tick: u64) -> Result<GameWorld, String> {
        let fresh = GameWorld::new_game(setup);
        let ws = fresh.inner.export_state();
        let rng = RngStreams::new(setup.seed).states();
        ParticleWorld::restore(tick, &rng, &ws).map(GameWorld::wrap)
    }

    /// Wraps an existing particle world (tests, restored saves). The undo history starts empty.
    pub fn from_world(w: ParticleWorld) -> GameWorld {
        GameWorld::wrap(w)
    }

    /// Rebuilds the game from a save. Refuses inconsistent data.
    pub fn restore(tick: u64, rng_states: &[[u64; 4]], ws: &crate::save::WorldState) -> Result<GameWorld, String> {
        ParticleWorld::restore(tick, rng_states, ws).map(GameWorld::wrap)
    }

    pub fn inner(&self) -> &ParticleWorld {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut ParticleWorld {
        &mut self.inner
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    fn reject(&mut self, code: i64) {
        self.reject_seq += 1;
        self.reject_code = code;
    }

    fn place(&mut self, x: i32, y: i32, type_id: u16, stroke: u32) {
        let cost = self.inner.type_cost(type_id).unwrap_or(0);
        match self.inner.place_typed(x, y, type_id) {
            Ok(paid) => {
                self.undo.push(UndoEntry { x, y, type_id, paid, cost, stroke });
                if self.undo.len() > UNDO_LIMIT {
                    self.undo.remove(0);
                }
            }
            Err(PlaceError::Occupied) => {} // a drag may cross occupied cells: nothing happens, nothing is overwritten
            Err(PlaceError::Insufficient) => self.reject(REJECT_INSUFFICIENT),
            Err(PlaceError::UnknownType) => self.reject(REJECT_UNKNOWN_TYPE),
        }
    }

    /// Takes back the most recent stroke that still has a particle on the grid. Placements whose particle has collapsed since are
    /// forgotten without a refund (a collapse never refunds, D-032/D-036).
    fn undo_last(&mut self) {
        let mut done = 0u32;
        let mut stroke: Option<u32> = None;
        loop {
            let top = match self.undo.last() {
                Some(e) => *e,
                None => break,
            };
            if let Some(s) = stroke {
                if top.stroke != s {
                    break;
                }
            }
            self.undo.pop();
            if self.inner.unplace(top.x, top.y, top.type_id, top.paid, top.cost) {
                done += 1;
                stroke = Some(top.stroke);
            }
        }
        if done == 0 {
            self.reject(REJECT_NOTHING_TO_UNDO);
        }
    }
}

impl World for GameWorld {
    fn step_to(&mut self, tick: u64) {
        self.inner.step_to(tick);
    }

    fn command(&mut self, c: Command) {
        match c {
            Command::Place { x, y, type_id, stroke } => self.place(x, y, type_id, stroke),
            Command::Undo => self.undo_last(),
            _ => {}
        }
    }

    fn stats(&self) -> [i64; WORLD_STATS] {
        [
            self.inner.count() as i64,
            self.inner.free_placements() as i64,
            self.inner.len() as i64,
            self.inner.chains_started() as i64,
            0, // view revision: filled in by the host with the revision it has published
            self.reject_seq as i64,
            self.reject_code,
            self.undo.len() as i64,
        ]
    }

    fn view_rev(&self) -> u64 {
        self.inner.revision()
    }

    fn view_cells(&self, max_cells: usize) -> Vec<i32> {
        self.inner.cell_triples(max_cells).0
    }

    fn drain_events(&mut self, out: &mut Vec<i64>) {
        for r in self.inner.take_log() {
            out.push(r.tick as i64);
            out.push(r.x as i64);
            out.push(r.y as i64);
            out.push(r.chain as i64);
            out.push(r.depth as i64);
        }
    }

    fn export(&self) -> Option<WorldExport> {
        Some(WorldExport { tick: self.inner.tick(), rng_states: self.inner.rng_states(), state: self.inner.export_state() })
    }
}
