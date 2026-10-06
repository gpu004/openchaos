//! Quarantined demo property: Hegel draws drive a core `SimWorld` schedule.
//!
//! Lives under tests/ as a scenario — not re-exported from the lang package root.

use hegel::generators as gs;
use hegel::TestCase;
use openchaos::bind::draw_seed_max;
use openchaos::SimWorld;

#[hegel::test]
fn demo_schedule_trace_is_seed_stable(tc: TestCase) {
    let seed = draw_seed_max(&tc, 5_000);
    let n = tc.draw(gs::integers::<u64>().min_value(1).max_value(10));

    let mut a = SimWorld::new(seed);
    let mut b = SimWorld::new(seed);
    for i in 0..n {
        let delay = tc.draw(gs::integers::<u64>().min_value(0).max_value(15));
        a.schedule_in(delay, i);
        b.schedule_in(delay, i);
    }
    while a.step().is_some() {}
    while b.step().is_some() {}
    assert_eq!(a.trace(), b.trace());
    assert_eq!(a.meter().steps, b.meter().steps);
}
