//! Replayable simulation world: clock + scheduler + seed.

use crate::instrument::Meter;
use super::clock::Clock;
use super::rng::SimRng;
use super::scheduler::{EventId, Scheduler, TimedEvent};
use super::seed::Seed;

/// A deterministic discrete-event world.
///
/// Same [`Seed`] and same schedule decisions produce the same event order and
/// meter totals — suitable for property tests and simulation benches.
#[derive(Debug)]
pub struct SimWorld<T> {
    seed: Seed,
    rng: SimRng,
    clock: Clock,
    scheduler: Scheduler<T>,
    meter: Meter,
    steps: u64,
    trace: Vec<(u64, EventId)>,
}

impl<T> SimWorld<T> {
    /// Create a world from an explicit seed.
    pub fn new(seed: Seed) -> Self {
        Self {
            seed,
            rng: SimRng::from_seed(seed),
            clock: Clock::new(0),
            scheduler: Scheduler::new(),
            meter: Meter::new(),
            steps: 0,
            trace: Vec::new(),
        }
    }

    /// Seed that created this world.
    pub fn seed(&self) -> Seed {
        self.seed
    }

    /// Current logical clock.
    pub fn clock(&self) -> Clock {
        self.clock
    }

    /// Mutable access to the world's RNG (for model code that needs entropy).
    pub fn rng(&mut self) -> &mut SimRng {
        &mut self.rng
    }

    /// Shared simulation meter (CodSpeed-inspired logical work counters).
    pub fn meter(&self) -> &Meter {
        &self.meter
    }

    /// Mutable meter for recording work inside handlers.
    pub fn meter_mut(&mut self) -> &mut Meter {
        &mut self.meter
    }

    /// Number of events delivered so far.
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// Compact trace of `(logical_time, event_id)` for equality checks.
    pub fn trace(&self) -> &[(u64, EventId)] {
        &self.trace
    }

    /// Schedule an event at an absolute logical time.
    pub fn schedule_at(&mut self, at: Clock, payload: T) -> EventId {
        self.meter.record_event();
        self.scheduler.schedule(at, payload)
    }

    /// Schedule an event `delay` ticks from now.
    pub fn schedule_in(&mut self, delay: u64, payload: T) -> EventId {
        let at = Clock::new(self.clock.ticks().saturating_add(delay));
        self.schedule_at(at, payload)
    }

    /// Pending event count.
    pub fn pending(&self) -> usize {
        self.scheduler.len()
    }

    /// Deliver the next event, advancing the clock. Returns `None` if idle.
    pub fn step(&mut self) -> Option<TimedEvent<T>> {
        let event = self.scheduler.pop()?;
        self.clock.set(event.at.ticks());
        self.steps += 1;
        self.meter.record_step();
        self.trace.push((event.at.ticks(), event.id));
        Some(event)
    }

    /// Run until the queue is empty or `max_steps` events have been delivered.
    ///
    /// `handler` receives each payload and may schedule more work.
    pub fn run_until<F>(&mut self, max_steps: u64, mut handler: F)
    where
        F: FnMut(&mut SimWorld<T>, T),
    {
        let mut delivered = 0u64;
        while delivered < max_steps {
            let Some(event) = self.step() else {
                break;
            };
            // Temporarily expose world without holding the payload borrow.
            handler(self, event.payload);
            delivered += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_trace() {
        fn run(seed: Seed) -> Vec<(u64, EventId)> {
            let mut w = SimWorld::new(seed);
            w.schedule_at(Clock::new(3), 30u32);
            w.schedule_at(Clock::new(1), 10u32);
            w.schedule_at(Clock::new(2), 20u32);
            while w.step().is_some() {}
            w.trace().to_vec()
        }
        assert_eq!(run(Seed::new(99)), run(Seed::new(99)));
    }

    #[test]
    fn handler_can_reschedule() {
        let mut w = SimWorld::new(Seed::new(1));
        w.schedule_in(1, 0u32);
        let mut seen = Vec::new();
        w.run_until(5, |world, n| {
            seen.push(n);
            if n < 3 {
                world.schedule_in(1, n + 1);
            }
        });
        assert_eq!(seen, vec![0, 1, 2, 3]);
    }
}
