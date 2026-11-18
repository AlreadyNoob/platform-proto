//! Thin Godot binding for the prototype core (Milestone 01b adds the simulation worker, stats and RSS).
//! STATUS: the Milestone 01 part compiled and ran on Android; the 01b additions are NOT yet compiled.
use godot::prelude::*;
use platform_core_proto::hazard::{HazardParams, Strategy};
use platform_core_proto::worker::SimWorker;
use platform_core_proto::{model, scaling};
use std::time::Instant;

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
    sim: Option<SimWorker>,
    last: Option<(Instant, u64, u64)>, // (time, tick, busy_us) of the previous sim_stats call
}

#[godot_api]
impl INode for CoreProbe {
    fn init(base: Base<Node>) -> Self {
        CoreProbe { base, sim: None, last: None }
    }
}

#[godot_api]
impl CoreProbe {
    #[func]
    fn core_info(&self) -> GString {
        gs(format!(
            "platform_core_proto 0.0.2 arch={} os={} ptr_bits={}",
            std::env::consts::ARCH,
            std::env::consts::OS,
            usize::BITS
        ))
    }

    #[func]
    fn ping(&self) -> i64 {
        1
    }

    #[func]
    fn determinism_hash(&self, seed: i64) -> GString {
        gs(format!("{:016x}", model::determinism_hash(seed as u64, 200)))
    }

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

    #[func]
    fn make_positions(&self, count: i64, spacing: f32) -> PackedVector2Array {
        let side = (count as f64).sqrt().ceil() as i64;
        let v: Vec<Vector2> = (0..count)
            .map(|i| Vector2::new((i % side) as f32 * spacing, (i / side) as f32 * spacing))
            .collect();
        PackedVector2Array::from(v.as_slice())
    }

    /// Start the simulation on a worker thread. strategy: 0 brute, 1 heap, 2 wheel, 3 hybrid.
    #[func]
    fn sim_start(&mut self, strategy: i64, comps: i64, p_hot: f64, speed: i64) -> GString {
        if let Some(mut w) = self.sim.take() {
            w.stop();
        }
        let strat = Strategy::from_index(strategy);
        let params = HazardParams::bench(p_hot);
        let w = SimWorker::start(strat, comps.max(1) as usize, params, 12345, speed.max(1) as u64);
        self.sim = Some(w);
        self.last = Some((Instant::now(), 0, 0));
        gs(format!(
            "sim started: strategy={} comps={} (~{} particles at 4 per comp) p_hot={} steady hot fraction={:.3} speed={}x",
            strat.name(), comps, comps * 4, p_hot, params.steady_hot_fraction(), speed
        ))
    }

    #[func]
    fn sim_set_speed(&mut self, speed: i64) {
        if let Some(w) = &self.sim {
            w.set_speed(speed.max(1) as u64);
        }
    }

    #[func]
    fn sim_set_paused(&mut self, paused: bool) {
        if let Some(w) = &self.sim {
            w.set_paused(paused);
        }
    }

    #[func]
    fn sim_stop(&mut self) {
        if let Some(mut w) = self.sim.take() {
            w.stop();
        }
        self.last = None;
    }

    /// Rates are measured between two consecutive calls.
    #[func]
    fn sim_stats(&mut self) -> GString {
        let s = match &self.sim {
            Some(w) => w.stats(),
            None => return gs("sim: not running".to_string()),
        };
        let now = Instant::now();
        let (rate, duty) = match self.last {
            Some((t0, tick0, busy0)) => {
                let dt = now.duration_since(t0).as_secs_f64().max(1e-6);
                ((s.tick - tick0) as f64 / dt, (s.busy_us - busy0) as f64 / (dt * 1e6) * 100.0)
            }
            None => (0.0, 0.0),
        };
        self.last = Some((now, s.tick, s.busy_us));
        gs(format!(
            "tick={} achieved={:.0} ticks/s target={} owed(lag)={} events={} hot={} sim-thread busy={:.0}%",
            s.tick, rate, s.target_ticks_per_sec, s.owed, s.events, s.hot, duty
        ))
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
