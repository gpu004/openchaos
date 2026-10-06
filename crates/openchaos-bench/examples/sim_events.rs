//! Benchmarks `SimWorld` delivering a fixed number of events.

use core::ffi::CStr;
use openchaos_bench::bench;
use openchaos_core::{Seed, SimWorld};

fn run_events(initial: u64) -> u64 {
    let mut world = SimWorld::new(Seed::new(42));
    for i in 0..initial {
        world.schedule_in(i % 97, i);
    }
    world
        .run_until(u64::MAX, |w, n| {
            if n % 4 == 0 {
                let delay = 1 + w.rng().gen_range(8);
                w.schedule_in(delay, n + 1);
            }
        })
        .delivered
}

const CASES: [(&CStr, u64); 2] = [
    (c"sim_world/1000_events", 1_000),
    (c"sim_world/10000_events", 10_000),
];

fn main() {
    for (name, events) in CASES {
        bench(name, || run_events(events));
    }
}
