//! Thin lang/rust must not embed a second sim — only drive core.

use openchaos::{Clock, Seed, SimWorld};

#[test]
fn lang_package_drives_core_world() {
    let mut world = SimWorld::new(Seed::new(11));
    world.schedule_at(Clock::new(0), "ping");
    let event = world.step().expect("scheduled");
    assert_eq!(event.payload, "ping");
    assert_eq!(world.meter().steps, 1);
}
