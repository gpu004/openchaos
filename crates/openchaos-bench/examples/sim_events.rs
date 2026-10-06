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

fn main() {
    for events in [1_000, 10_000] {
        let delivered = bench(&format!("sim_world/{events}_events"), || run_events(events));
        println!("{events} initial events, {delivered} delivered");
    }
}
