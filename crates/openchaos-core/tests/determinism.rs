//! Core-only integration: same seed ⇒ same trace and meters.

use openchaos_core::{Clock, Seed, SimWorld};

#[test]
fn same_seed_same_trace_and_meters() {
    fn run(seed: Seed) -> (Vec<(u64, openchaos_core::EventId)>, openchaos_core::Meter) {
        let mut world = SimWorld::new(seed);
        world.schedule_at(Clock::new(3), 30u32);
        world.schedule_at(Clock::new(1), 10u32);
        world.schedule_at(Clock::new(2), 20u32);
        while world.step().is_some() {}
        (world.trace().to_vec(), world.meter().clone())
    }

    let (t1, m1) = run(Seed::new(99));
    let (t2, m2) = run(Seed::new(99));
    assert_eq!(t1, t2);
    assert_eq!(m1, m2);
    assert_eq!(m1.steps, 3);
    assert_eq!(m1.events, 3);
}

#[test]
fn payload_handler_runs_outside_queue() {
    let mut world = SimWorld::new(Seed::new(1));
    world.schedule_in(1, 0u32);
    let mut seen = Vec::new();
    world.run_until(8, |w, n| {
        seen.push(n);
        if n < 3 {
            w.schedule_in(1, n + 1);
        }
    });
    assert_eq!(seen, vec![0, 1, 2, 3]);
}
