//! `cargo run --release --bin bench_hybrid` - heap vs timing wheel vs hybrid at several hot/cold mixes.
//! `bench_hybrid --hi` repeats it with much hotter hot slots (q=0.2, hybrid threshold 3%).
//! `bench_hybrid --determinism-hazard` prints state hashes to compare across devices.
use platform_core_proto::hazard::{HazardParams, HazardSim, Strategy};
use std::time::{Duration, Instant};

/// returns (ticks per second, events per tick, hot fraction at the end)
fn measure(strat: Strategy, comps: usize, params: HazardParams, secs: f64) -> (f64, f64, f64) {
    let mut sim = HazardSim::new(strat, comps, params, 12345);
    let (t0, e0) = (sim.now(), sim.events);
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs_f64(secs) {
        let target = sim.now() + 200;
        sim.run_until(target);
    }
    let el = start.elapsed().as_secs_f64();
    let ticks = (sim.now() - t0) as f64;
    (ticks / el, (sim.events - e0) as f64 / ticks, sim.hot_count() as f64 / comps as f64)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--determinism-hazard") {
        for strat in [Strategy::Brute, Strategy::Heap, Strategy::Wheel, Strategy::Hybrid] {
            let mut s = HazardSim::new(strat, 2000, HazardParams::bench(0.9), 1);
            s.run_until(20_000);
            println!("hazard {} events={} hash={:016x}", strat.name(), s.events, s.hash);
        }
        return;
    }
    let hi = args.iter().any(|a| a == "--hi");
    let comps = 250_000;
    if hi {
        println!("# HIGH-HAZARD run: comps={comps} (= 1M particles), q_hot=0.2 q_cold=1e-4 q_star=0.03, steady-state start, single thread");
    } else {
        println!("# comps={comps} (= 1M particles at S=4), q_hot=0.02 q_cold=1e-4 q_star=0.003, steady-state start, single thread");
    }
    println!("# stored = 1M particles; need 10,000 ticks/s at 10,000x speed");
    let mixes: [f64; 5] = if hi { [0.0, 0.9, 0.99, 0.999, 0.9999] } else { [0.0, 0.5, 0.9, 0.99, 0.999] };
    if !hi {
        let (tps, evt, _) = measure(Strategy::Brute, comps, HazardParams::bench(0.5), 2.0);
        println!("brute (cost independent of mix): {tps:.0} ticks/s ({evt:.1} ev/tick)");
    }
    for p in mixes {
        let params = if hi { HazardParams::bench_hi(p) } else { HazardParams::bench(p) };
        let h = params.steady_hot_fraction();
        for strat in [Strategy::Heap, Strategy::Wheel, Strategy::Hybrid] {
            let (tps, evt, hotf) = measure(strat, comps, params, 1.5);
            println!("p_hot={p} steady_hot={h:.3} {:<6} {tps:>10.0} ticks/s  {evt:>8.1} ev/tick  hot_list={:.3}", strat.name(), hotf);
        }
    }
}
