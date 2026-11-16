//! `cargo run --release --bin bench` (desktop/Termux) — prints one line per measurement.
//! `bench --determinism` prints the state hash to compare across devices (acceptance A6).
use platform_core_proto::{model, scaling};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--determinism") {
        for seed in [1u64, 2, 3] {
            println!("determinism seed={seed} hash={:016x}", model::determinism_hash(seed, 200));
        }
        return;
    }
    println!("# stored = N total particles; simulated = particles the tick loop evaluates; visible = n/a (no rendering)");
    let cases: [(u64, u32, f64); 6] = [
        (10_000, 4, 1e-4), (100_000, 4, 1e-4), (1_000_000, 4, 1e-4),
        (10_000_000, 4, 1e-4), (1_000_000, 4, 1e-2), (1_000_000, 16, 1e-3),
    ];
    for (n, s, q) in cases {
        let comps = n / s as u64;
        let ev_ticks = ((3.0e6 / (comps as f64 * q)) as u64).max(20_000);
        let br_ticks = (1_000_000_000 / n).clamp(3, 20_000);
        let e = scaling::event_driven(n, s, q, ev_ticks, 12345);
        let c = scaling::brute_component(n, s, q, br_ticks, 12345);
        let p = scaling::brute_particle(n, s, q, br_ticks, 12345);
        println!(
            "N={n} S={s} q={q:e} | event {:.0} ticks/s ({:.2} ev/tick, simulated/tick {:.1}) | brute_comp {:.0} ticks/s | brute_particle {:.0} ticks/s",
            e.ticks_per_sec(), e.events_per_tick(), e.simulated_per_tick(), c.ticks_per_sec(), p.ticks_per_sec()
        );
    }
}
