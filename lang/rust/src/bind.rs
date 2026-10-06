use hegel::generators as gs;
use hegel::TestCase;
use openchaos_core::{Seed, SimWorld};

pub fn draw_seed(tc: &TestCase) -> Seed {
    Seed::new(tc.draw(gs::integers::<u64>()))
}

pub fn draw_world<T>(tc: &TestCase) -> SimWorld<T> {
    SimWorld::new(draw_seed(tc))
}
