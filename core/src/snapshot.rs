//! Snapshot: immutable, engine-neutral copy of what the presentation layer may read. The UI never touches live simulation state.
//! M02a carries only the header; viewport chunk data is added with the world model (later milestone).
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
}

impl Snapshot {
    /// Stable field order used by the Godot bridge (PackedInt64Array). Append only; never reorder.
    pub const FIELDS: [&'static str; 9] =
        ["seq", "tick", "requested_speed", "effective_speed", "paused", "owed_ticks", "governor_drops", "governor_raises", "busy_permille"];

    pub fn to_array(&self) -> [i64; 9] {
        [
            self.seq as i64,
            self.tick as i64,
            self.requested_speed as i64,
            self.effective_speed as i64,
            self.paused as i64,
            self.owed_ticks as i64,
            self.governor_drops as i64,
            self.governor_raises as i64,
            self.busy_permille as i64,
        ]
    }
}
