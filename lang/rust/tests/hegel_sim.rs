//! Hegel properties that drive openchaos-core through the thin lang adapter.

use hegel::generators as gs;
use hegel::TestCase;
use openchaos::bind::{draw_seed_max, draw_world_max};
use openchaos::{Clock, SimWorld};

#[hegel::test]
fn pbt_over_sim_schedule_is_deterministic(tc: TestCase) {
    let seed = draw_seed_max(&tc, 10_000);
    let delays =
        tc.draw(gs::vecs(gs::integers::<u64>().min_value(0).max_value(20)).max_size(8));

    let mut a = SimWorld::new(seed);
    let mut b = SimWorld::new(seed);
    for d in &delays {
        a.schedule_in(*d, *d);
        b.schedule_in(*d, *d);
    }
    while a.step().is_some() {}
    while b.step().is_some() {}
    assert_eq!(a.trace(), b.trace());
}

#[hegel::test]
fn hegel_drawn_world_meters_match_steps(tc: TestCase) {
    let mut world = draw_world_max::<u64>(&tc, 500);
    let n = tc.draw(gs::integers::<u64>().min_value(1).max_value(12));
    for i in 0..n {
        world.schedule_at(Clock::new(i), i);
    }
    while world.step().is_some() {}
    assert_eq!(world.meter().steps, n);
    assert_eq!(world.meter().events, n);
}
