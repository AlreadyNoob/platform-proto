//! Commands: the ONLY way the UI changes simulation state. More variants arrive with the game rules (placing particles etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    SetRequestedSpeed(u64),
    SetPaused(bool),
}
