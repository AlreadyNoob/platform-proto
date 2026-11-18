//! Platform simulation core. std only; knows nothing about Godot, Android or rendering.
//! STATUS (Milestone 02a): skeleton. There are NO game rules here, only the foundations:
//! rng, clock, speed governor, timing wheel, snapshot/command interface, save container, simulation host.
pub mod clock;
pub mod command;
pub mod governor;
pub mod host;
pub mod rng;
pub mod save;
pub mod snapshot;
pub mod wheel;
pub mod world;

pub const CORE_VERSION: &str = "0.1.2";
