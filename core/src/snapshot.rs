//! Snapshot: immutable, engine-neutral copy of what the presentation layer may read. The UI never touches live simulation state.
//! Since M03d it also carries the world's counters (count, free placements, particles, ...). The particle cells themselves travel
//! through a separate, change-driven channel (`SimHost::view`), never inside the snapshot.
use crate::world::WORLD_STATS;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// Increases every time the simulation host publishes a new snapshot.
    pub seq: u64,
    pub tick: u64,
    pub requested_speed: u64,
    pub effective_speed: u64,
    pub paused: bool,
    /// Ticks owed to the player's requested/effective speed that the device has not yet run.
    pub owed_ticks: u64,
    /// How many times the governor lowered / raised the effective speed since start.
    pub governor_drops: u64,
    pub governor_raises: u64,
    /// Simulation-thread busy time over the last governor interval, in permille (1000 = 100%).
    pub busy_permille: u64,
    /// World counters, in the order of `WORLD_FIELDS` (all zero for worlds without game rules).
    pub world: [i64; WORLD_STATS],
}

impl Snapshot {
    /// Number of values in `to_array`.
    pub const LEN: usize = 9 + WORLD_STATS;

    /// Stable field order used by the Godot bridge (PackedInt64Array). Append only; never reorder.
    pub const FIELDS: [&'static str; Snapshot::LEN] = [
        "seq",
        "tick",
        "requested_speed",
        "effective_speed",
        "paused",
        "owed_ticks",
        "governor_drops",
        "governor_raises",
        "busy_permille",
        // world counters (M03d)
        "count",
        "free_placements",
        "particles",
        "chains",
        "view_rev",
        "reject_seq",
        "reject_code",
        "undo_depth",
        "next_cost",
    ];

    pub fn to_array(&self) -> [i64; Snapshot::LEN] {
        let mut a = [0i64; Snapshot::LEN];
        a[0] = self.seq as i64;
        a[1] = self.tick as i64;
        a[2] = self.requested_speed as i64;
        a[3] = self.effective_speed as i64;
        a[4] = self.paused as i64;
        a[5] = self.owed_ticks as i64;
        a[6] = self.governor_drops as i64;
        a[7] = self.governor_raises as i64;
        a[8] = self.busy_permille as i64;
        for i in 0..WORLD_STATS {
            a[9 + i] = self.world[i];
        }
        a
    }
}
