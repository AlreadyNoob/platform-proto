//! Thin Godot binding for the prototype core. STATUS: NOT COMPILED, NOT RUN (no Rust/Godot in the
//! authoring sandbox). gdext API names may need small fixes for the version Cargo resolves.
//! All results are returned as plain strings / ints so the binding surface stays tiny.
use godot::prelude::*;
use platform_core_proto::{model, scaling};

fn gs(s: String) -> GString {
    GString::from(s.as_str())
}

struct ProtoExtension;

#[gdextension]
unsafe impl ExtensionLibrary for ProtoExtension {}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct CoreProbe {
    base: Base<Node>,
}

#[godot_api]
impl INode for CoreProbe {
    fn init(base: Base<Node>) -> Self {
        CoreProbe { base }
    }
}

#[godot_api]
impl CoreProbe {
    /// Proves the library loaded: returns architecture/OS the .so was compiled for.
    #[func]
    fn core_info(&self) -> GString {
        gs(format!(
            "platform_core_proto 0.0.1 arch={} os={} ptr_bits={}",
            std::env::consts::ARCH,
            std::env::consts::OS,
            usize::BITS
        ))
    }

    /// Cheapest possible call; GDScript times a loop of these to get per-call overhead.
    #[func]
    fn ping(&self) -> i64 {
        1
    }

    /// Acceptance A6: same hash on desktop and Android for the same seed.
    #[func]
    fn determinism_hash(&self, seed: i64) -> GString {
        gs(format!("{:016x}", model::determinism_hash(seed as u64, 200)))
    }

    /// SIMULATION benchmark only (no rendering): event-driven vs brute force, timed inside Rust.
    #[func]
    fn run_sim_bench(&self, stored: i64, comp_size: i64, q: f64) -> GString {
        let (n, s) = (stored as u64, comp_size as u32);
        let comps = n / s as u64;
        let ev_ticks = ((3.0e6 / (comps as f64 * q)) as u64).max(20_000);
        let br_ticks = (400_000_000u64 / n.max(1)).clamp(3, 20_000);
        let e = scaling::event_driven(n, s, q, ev_ticks, 12345);
        let c = scaling::brute_component(n, s, q, br_ticks, 12345);
        let p = scaling::brute_particle(n, s, q, br_ticks, 12345);
        gs(format!(
            "stored={} simulated/tick(event)={:.1} event={:.0} ticks/s ({:.2} ev/tick) brute_comp={:.0} ticks/s brute_particle={:.0} ticks/s",
            n, e.simulated_per_tick(), e.ticks_per_sec(), e.events_per_tick(), c.ticks_per_sec(), p.ticks_per_sec()
        ))
    }

    /// Binding data-transfer cost: build `count` positions in Rust and hand them to Godot in one call.
    #[func]
    fn make_positions(&self, count: i64, spacing: f32) -> PackedVector2Array {
        let side = (count as f64).sqrt().ceil() as i64;
        let v: Vec<Vector2> = (0..count)
            .map(|i| Vector2::new((i % side) as f32 * spacing, (i / side) as f32 * spacing))
            .collect();
        PackedVector2Array::from(v.as_slice())
    }
}
