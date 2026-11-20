//! Commands: the ONLY way the UI changes simulation state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    SetRequestedSpeed(u64),
    SetPaused(bool),
    /// Place a particle of `type_id` at cell (x, y), paid by the economy (free placement or count). `stroke` groups all placements of one
    /// drag (or one tap) so that a single Undo takes the whole stroke back. A failed placement changes nothing.
    Place { x: i32, y: i32, type_id: u16, stroke: u32 },
    /// Takes back the most recent stroke (every placement of it that is still on the grid), refunding exactly what was paid.
    Undo,
    /// Asks the worker for a copy of everything that has to be saved; answered under this request id (see `SimHost::export_blocking`).
    Export(u64),
}
