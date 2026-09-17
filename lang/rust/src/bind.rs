//! Thin Hegel adapter over openchaos-core.
//!
//! Hegel owns draws, shrinking, and the property runner (`#[hegel::test]`).
//! This module only bridges Hegel [`hegel::TestCase`] values into core types
//! such as [`Seed`] and [`SimWorld`]. It deliberately does **not** reimplement a
//! choice tape, generators, or shrinker inside openchaos.
//!
//! The Cargo package is `hegeltest`; the Rust crate name is `hegel`.

use hegel::generators as gs;
use hegel::TestCase;
use openchaos_core::{Seed, SimWorld};

/// Draw a core [`Seed`] from the current Hegel test case.
///
/// Cross-language same-seed replay uses this core seed type — never a
/// language-local PRNG.
pub fn draw_seed(tc: &TestCase) -> Seed {
    Seed::new(tc.draw(gs::integers::<u64>()))
}

/// Draw a [`Seed`] within an inclusive upper bound (useful for smaller examples).
pub fn draw_seed_max(tc: &TestCase, max: u64) -> Seed {
    Seed::new(tc.draw(gs::integers::<u64>().min_value(0).max_value(max)))
}

/// Construct a fresh [`SimWorld`] whose seed was drawn from Hegel.
pub fn draw_world<T>(tc: &TestCase) -> SimWorld<T> {
    SimWorld::new(draw_seed(tc))
}

/// Construct a [`SimWorld`] from a bounded Hegel seed draw.
pub fn draw_world_max<T>(tc: &TestCase, max: u64) -> SimWorld<T> {
    SimWorld::new(draw_seed_max(tc, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use openchaos_core::Clock;

    #[hegel::test]
    fn drawn_seed_replays_identical_traces(tc: TestCase) {
        let seed = draw_seed_max(&tc, 10_000);
        let delays = tc.draw(
            gs::vecs(gs::integers::<u64>().min_value(0).max_value(20)).max_size(8),
        );

        let mut a = SimWorld::new(seed);
        let mut b = SimWorld::new(seed);
        for d in &delays {
            a.schedule_in(*d, *d);
            b.schedule_in(*d, *d);
        }
        while a.step().is_some() {}
        while b.step().is_some() {}
        assert_eq!(a.trace(), b.trace());
        assert_eq!(a.meter(), b.meter());
    }

    #[hegel::test]
    fn draw_world_schedules_from_hegel(tc: TestCase) {
        let mut world = draw_world_max::<u64>(&tc, 1_000);
        let at = tc.draw(gs::integers::<u64>().min_value(0).max_value(50));
        world.schedule_at(Clock::new(at), at);
        let event = world.step().expect("one event");
        assert_eq!(event.payload, at);
        assert_eq!(event.at.ticks(), at);
    }
}
