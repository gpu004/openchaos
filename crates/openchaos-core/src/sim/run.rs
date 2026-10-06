//! Portable world/run seam: opaque payloads in, handlers out.
//!
//! The scheduler stores **payloads only** — never host closures. Foreign
//! language adapters drive the same loop by calling [`crate::sim::SimWorld::step`]
//! (or [`crate::sim::SimWorld::run_with`]) and applying handlers outside the queue.
//! The TS closure-in-queue `EventHandler` shape is intentionally not the core model.

use crate::sim::world::SimWorld;

/// Limits for a driven run across the binding seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunLimit {
    /// Maximum events to deliver before returning.
    pub max_steps: u64,
}

impl RunLimit {
    /// Limit to at most `max_steps` deliveries.
    #[must_use]
    pub const fn steps(max_steps: u64) -> Self {
        Self { max_steps }
    }

    /// Effectively unbounded for practical scenario sizes.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self {
            max_steps: u64::MAX,
        }
    }
}

/// Handler invoked **outside** the event queue for each delivered payload.
///
/// Adapters implement this (or use a `FnMut`) so core never stores language
/// callables on the scheduler.
pub trait EventHandler<T> {
    /// Handle one payload; may schedule further opaque events on `world`.
    fn on_event(&mut self, world: &mut SimWorld<T>, payload: T);
}

impl<T, F> EventHandler<T> for F
where
    F: FnMut(&mut SimWorld<T>, T),
{
    fn on_event(&mut self, world: &mut SimWorld<T>, payload: T) {
        self(world, payload);
    }
}

/// Summary of a driven run (logical, seed-stable when the model is).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSummary {
    /// Events delivered during this drive call.
    pub delivered: u64,
    /// Whether the queue was empty when the drive stopped.
    pub drained: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Clock, Seed};

    struct Counting;

    impl EventHandler<u32> for Counting {
        fn on_event(&mut self, world: &mut SimWorld<u32>, payload: u32) {
            if payload < 2 {
                world.schedule_in(1, payload + 1);
            }
        }
    }

    #[test]
    fn trait_handler_runs_outside_queue() {
        let mut world = SimWorld::new(Seed::new(1));
        world.schedule_at(Clock::new(0), 0u32);
        let summary = world.run_with(RunLimit::steps(10), &mut Counting);
        assert_eq!(summary.delivered, 3);
        assert!(summary.drained);
    }
}
