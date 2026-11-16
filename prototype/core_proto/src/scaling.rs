//! Scaling workload (geometry-free chain model), mirrors reference/bench_scaling.c.
//! N stored particles in components of size S; each component collapses per tick with prob q and is
//! respawned so the population stays constant. "Simulated" = particles the tick loop evaluates.
use crate::detmath::geometric_failures;
use crate::rng::Rng;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ScalingResult {
    pub stored: u64,
    pub components: u64,
    pub ticks: u64,
    pub events: u64,
    pub simulated_particles: u64, // particles evaluated by the strategy over all ticks
    pub seconds: f64,
}

impl ScalingResult {
    pub fn ticks_per_sec(&self) -> f64 { self.ticks as f64 / self.seconds }
    pub fn events_per_tick(&self) -> f64 { self.events as f64 / self.ticks as f64 }
    pub fn simulated_per_tick(&self) -> f64 { self.simulated_particles as f64 / self.ticks as f64 }
}

pub fn event_driven(n: u64, s: u32, q: f64, ticks: u64, seed: u64) -> ScalingResult {
    let comps = n / s as u64;
    let mut rng = Rng::new(seed);
    // per-particle storage analogue (component id + type) so memory/touch costs are not ignored
    let part_comp: Vec<u32> = (0..n).map(|i| (i / s as u64) as u32).collect();
    let part_type: Vec<u8> = vec![1u8; n as usize];
    let mut heap: BinaryHeap<Reverse<(u64, u32)>> = BinaryHeap::with_capacity(comps as usize);
    for c in 0..comps {
        heap.push(Reverse((geometric_failures(rng.unit_open0(), q), c as u32)));
    }
    let start = Instant::now();
    let (mut events, mut touched) = (0u64, 0u64);
    while let Some(&Reverse((t, c))) = heap.peek() {
        if t >= ticks { break; }
        heap.pop();
        events += 1;
        touched += s as u64;
        heap.push(Reverse((t + 1 + geometric_failures(rng.unit_open0(), q), c)));
    }
    let seconds = start.elapsed().as_secs_f64();
    std::hint::black_box((&part_comp, &part_type));
    ScalingResult { stored: n, components: comps, ticks, events, simulated_particles: touched, seconds }
}

pub fn brute_particle(n: u64, s: u32, q: f64, ticks: u64, seed: u64) -> ScalingResult {
    let comps = n / s as u64;
    let p = 1.0 - (1.0 - q).powf(1.0 / s as f64);
    let thr = (p * 18446744073709551616.0) as u64;
    let mut rng = Rng::new(seed);
    let start = Instant::now();
    let mut events = 0u64;
    for _ in 0..ticks {
        for _ in 0..comps {
            let mut hit = false;
            for _ in 0..s {
                if rng.next_u64() < thr { hit = true; }
            }
            if hit { events += 1; }
        }
    }
    let events = std::hint::black_box(events); // F-5: without this LLVM deletes the loop and reports inf ticks/s
    ScalingResult { stored: n, components: comps, ticks, events, simulated_particles: n * ticks, seconds: start.elapsed().as_secs_f64() }
}

pub fn brute_component(n: u64, s: u32, q: f64, ticks: u64, seed: u64) -> ScalingResult {
    let comps = n / s as u64;
    let thr = (q * 18446744073709551616.0) as u64;
    let mut rng = Rng::new(seed);
    let start = Instant::now();
    let mut events = 0u64;
    for _ in 0..ticks {
        for _ in 0..comps {
            if rng.next_u64() < thr { events += 1; }
        }
    }
    let events = std::hint::black_box(events); // F-5: without this LLVM deletes the loop and reports inf ticks/s
    ScalingResult { stored: n, components: comps, ticks, events, simulated_particles: n * ticks, seconds: start.elapsed().as_secs_f64() }
}
