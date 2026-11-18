//! Thin Godot binding for platform_core. Contains NO game rules. Coarse-grained API (D-013).
//! STATUS (M02a): written in a sandbox without a Rust toolchain; compiled only by CI / on the owner's phone.
//! Only API shapes already proven to compile in Milestone 01/01b are used (GString, i64, f64, bool, Packed*Array::from(slice)).
use godot::prelude::*;
use platform_core::command::Command;
use platform_core::host::SimHost;
use platform_core::rng::RngStreams;
use platform_core::save::{load_slot, save_slot, SaveData};
use platform_core::snapshot::Snapshot;
use platform_core::world::BurnWorld;
use platform_core::CORE_VERSION;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn gs(s: String) -> GString {
    GString::from(s.as_str())
}

struct PlatformExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PlatformExtension {}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct PlatformCore {
    base: Base<Node>,
    host: Option<SimHost>,
    seed: u64,
    burn_ns: Arc<AtomicU64>,
}

#[godot_api]
impl INode for PlatformCore {
    fn init(base: Base<Node>) -> Self {
        PlatformCore { base, host: None, seed: 1, burn_ns: Arc::new(AtomicU64::new(0)) }
    }
}

#[godot_api]
impl PlatformCore {
    #[func]
    fn core_version(&self) -> GString {
        gs(format!(
            "platform_core {} arch={} os={} ptr_bits={}",
            CORE_VERSION,
            std::env::consts::ARCH,
            std::env::consts::OS,
            usize::BITS
        ))
    }

    /// Starts the simulation worker (replaces a running one). `burn_ns` > 0 makes every tick cost that much CPU:
    /// a test tool for the speed governor, not gameplay.
    #[func]
    fn start(&mut self, seed: i64, speed: i64, burn_ns: i64) {
        if let Some(mut h) = self.host.take() {
            h.stop();
        }
        self.seed = seed as u64;
        self.burn_ns.store(burn_ns.max(0) as u64, Ordering::Relaxed);
        let world = BurnWorld::new(Arc::clone(&self.burn_ns));
        self.host = Some(SimHost::start(world, 0, speed.max(1) as u64));
    }

    #[func]
    fn stop(&mut self) {
        if let Some(mut h) = self.host.take() {
            h.stop();
        }
    }

    #[func]
    fn set_speed(&mut self, speed: i64) {
        if let Some(h) = &self.host {
            h.send(Command::SetRequestedSpeed(speed.max(1) as u64));
        }
    }

    #[func]
    fn set_paused(&mut self, paused: bool) {
        if let Some(h) = &self.host {
            h.send(Command::SetPaused(paused));
        }
    }

    #[func]
    fn set_burn_ns(&mut self, burn_ns: i64) {
        self.burn_ns.store(burn_ns.max(0) as u64, Ordering::Relaxed);
    }

    /// Latest snapshot header as integers, in the order given by stats_fields().
    #[func]
    fn stats(&self) -> PackedInt64Array {
        let s = match &self.host {
            Some(h) => h.snapshot(),
            None => Snapshot::default(),
        };
        let a = s.to_array();
        PackedInt64Array::from(a.as_slice())
    }

    #[func]
    fn stats_fields(&self) -> GString {
        gs(Snapshot::FIELDS.join(","))
    }

    #[func]
    fn running(&self) -> bool {
        self.host.is_some()
    }

    /// Save-system check: writes the current tick/speed (RNG states are placeholders until a real world exists).
    #[func]
    fn save_check(&self, path: GString, label: GString) -> GString {
        let snap = match &self.host {
            Some(h) => h.snapshot(),
            None => return gs("save: simulation not running".to_string()),
        };
        let d = SaveData {
            seed: self.seed,
            tick: snap.tick,
            requested_speed: snap.requested_speed,
            rng_states: RngStreams::new(self.seed).states(),
            label: label.to_string(),
        };
        let p = path.to_string();
        match save_slot(Path::new(&p), &d) {
            Ok(()) => gs(format!("save ok: tick={} speed={} -> {}", d.tick, d.requested_speed, p)),
            Err(e) => gs(format!("save FAILED: {}", e)),
        }
    }

    #[func]
    fn load_check(&self, path: GString) -> GString {
        let p = path.to_string();
        match load_slot(Path::new(&p)) {
            Ok((d, src)) => gs(format!(
                "load ok from {:?}: tick={} speed={} seed={} streams={} label={}",
                src,
                d.tick,
                d.requested_speed,
                d.seed,
                d.rng_states.len(),
                d.label
            )),
            Err(e) => gs(format!("load FAILED: {}", e)),
        }
    }

    /// Resident memory of the whole app process in KB (Linux/Android), -1 if unavailable.
    #[func]
    fn rss_kb(&self) -> i64 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("VmRSS:"))
                    .and_then(|l| l.split_whitespace().nth(1).and_then(|v| v.parse::<i64>().ok()))
            })
            .unwrap_or(-1)
    }
}
