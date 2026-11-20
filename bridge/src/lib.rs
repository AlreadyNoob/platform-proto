//! Thin Godot binding for platform_core. Contains NO game rules. Coarse-grained API (D-013).
//! STATUS (M03d): written in a sandbox without a Rust toolchain; compiled only by CI / on the owner's phone.
//! Only API shapes already proven to compile in Milestone 01/01b are used (GString, i64, f64, bool, Packed*Array::from(slice)).
use godot::prelude::*;
use platform_core::command::Command;
use platform_core::host::SimHost;
use platform_core::rng::RngStreams;
use platform_core::game::GameWorld;
use platform_core::save::{load_slot, load_slot_game, save_slot, save_slot_game, SaveData};
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
        let speed = speed.max(1) as u64;
        if burn_ns > 0 {
            // governor test tool (check scene): every tick burns CPU, no game rules
            let world = BurnWorld::new(Arc::clone(&self.burn_ns));
            self.host = Some(SimHost::start(world, 0, speed));
        } else {
            let world = GameWorld::new_dev_game(self.seed);
            self.host = Some(SimHost::start(world, 0, speed));
        }
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

    /// Places one particle (type `type_id`) at a grid cell. Paid by the economy; a failure changes nothing (see reject_seq / reject_code in
    /// the stats). `stroke` groups the placements of one tap or one drag for undo.
    #[func]
    fn place(&self, x: i64, y: i64, type_id: i64, stroke: i64) {
        if let Some(h) = &self.host {
            h.send(Command::Place { x: x as i32, y: y as i32, type_id: type_id.clamp(0, 65535) as u16, stroke: stroke as u32 });
        }
    }

    /// Takes back the most recent stroke.
    #[func]
    fn undo(&self) {
        if let Some(h) = &self.host {
            h.send(Command::Undo);
        }
    }

    /// The latest published particles as [x, y, type, x, y, type, ...]. Fetch it when the `view_rev` stat changes.
    #[func]
    fn cells(&self) -> PackedInt32Array {
        let v: Vec<i32> = match &self.host {
            Some(h) => h.view().1,
            None => Vec::new(),
        };
        PackedInt32Array::from(v.as_slice())
    }

    /// Collapse records since the last call as [tick, x, y, chain, depth, ...] (the feed for the wave animation).
    #[func]
    fn events(&self) -> PackedInt64Array {
        let v: Vec<i64> = match &self.host {
            Some(h) => h.take_events(),
            None => Vec::new(),
        };
        PackedInt64Array::from(v.as_slice())
    }

    /// Real save (M03d): header plus the whole world (particles, types, count, free placements, collapse settings).
    /// If the simulation does not answer in time nothing is written, so a good save is never replaced by a worse one.
    #[func]
    fn save_game(&self, path: GString, label: GString) -> GString {
        let h = match &self.host {
            Some(h) => h,
            None => return gs("save: simulation not running".to_string()),
        };
        let snap = h.snapshot();
        let (d, world) = match h.export_blocking(1500) {
            None => return gs("save FAILED: the simulation did not answer in time, nothing was written".to_string()),
            Some(Some(e)) => (
                SaveData { seed: self.seed, tick: e.tick, requested_speed: snap.requested_speed, rng_states: e.rng_states, label: label.to_string() },
                Some(e.state),
            ),
            Some(None) => (
                SaveData {
                    seed: self.seed,
                    tick: snap.tick,
                    requested_speed: snap.requested_speed,
                    rng_states: RngStreams::new(self.seed).states(),
                    label: label.to_string(),
                },
                None,
            ),
        };
        let particles = match &world {
            Some(w) => w.cells.len(),
            None => 0,
        };
        let p = path.to_string();
        match save_slot_game(Path::new(&p), &d, world.as_ref()) {
            Ok(()) => gs(format!("save ok: tick={} speed={} particles={} -> {}", d.tick, d.requested_speed, particles, p)),
            Err(e) => gs(format!("save FAILED: {}", e)),
        }
    }

    /// Real load (M03d): replaces the running simulation with the saved world. It starts PAUSED so the player decides when to continue.
    /// A file without a world (older saves) becomes a new world at the saved tick (D-040).
    #[func]
    fn load_game(&mut self, path: GString) -> GString {
        let p = path.to_string();
        match load_slot_game(Path::new(&p)) {
            Ok((d, ws, src)) => {
                let restored = match &ws {
                    Some(w) => GameWorld::restore(d.tick, &d.rng_states, w),
                    None => GameWorld::new_dev_game_at(d.seed, d.tick),
                };
                match restored {
                    Ok(world) => {
                        if let Some(mut h) = self.host.take() {
                            h.stop();
                        }
                        self.seed = d.seed;
                        let host = SimHost::start(world, d.tick, d.requested_speed.max(1));
                        host.send(Command::SetPaused(true));
                        self.host = Some(host);
                        let kind = if ws.is_some() { "with world" } else { "no world in file, new world" };
                        gs(format!("loaded from {:?}: tick={} speed={} ({}, paused)", src, d.tick, d.requested_speed, kind))
                    }
                    Err(e) => gs(format!("load FAILED: {}", e)),
                }
            }
            Err(e) => gs(format!("load FAILED: {}", e)),
        }
    }

    /// One-line description of a slot for the UI, or "empty" when nothing readable is there.
    #[func]
    fn slot_summary(&self, path: GString) -> GString {
        let p = path.to_string();
        match load_slot_game(Path::new(&p)) {
            Ok((d, Some(w), _)) => gs(format!("tick {}, {} particles, count {}, free {}", d.tick, w.cells.len(), w.count, w.free)),
            Ok((d, None, _)) => gs(format!("tick {}, speed {}x (no world)", d.tick, d.requested_speed)),
            Err(_) => gs("empty".to_string()),
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
