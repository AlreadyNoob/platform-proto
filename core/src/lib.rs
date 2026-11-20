//! Platform simulation core. std only; knows nothing about Godot, Android or rendering.
//! STATUS (Milestone 03a): foundations (rng, clock, speed governor, timing wheel, snapshot/command interface, save container v2 with world state, simulation host)
//! plus the collapse rules and the economy as a reference engine (`collapse`: count ledger, free placements, per-type settings, generation).
//! No entropy, timelines, upgrades or events yet. Every economy number is a placeholder (docs/TUNING_PLACEHOLDERS.md).
pub mod clock;
pub mod collapse;
pub mod command;
pub mod governor;
pub mod host;
pub mod rng;
pub mod save;
pub mod snapshot;
pub mod wheel;
pub mod world;

pub const CORE_VERSION: &str = "0.2.8";
