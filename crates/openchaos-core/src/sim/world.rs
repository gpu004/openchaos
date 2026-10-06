use super::clock::Clock;
use super::rng::SimRng;
use super::scheduler::{EventId, InPast, Scheduler, TimedEvent};
use super::seed::Seed;

/// Handles a delivered payload. Runs outside the queue and may schedule more events.
pub trait EventHandler<T> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSummary {
    pub delivered: u64,
    pub drained: bool,
}

/// Same seed and same schedule calls produce the same trace.
#[derive(Debug)]
pub struct SimWorld<T> {
    seed: Seed,
    rng: SimRng,
    scheduler: Scheduler<T>,
    trace: Vec<(Clock, EventId)>,
}

impl<T> SimWorld<T> {
    pub fn new(seed: Seed) -> Self {
        Self {
            seed,
            rng: SimRng::from_seed(seed),
            scheduler: Scheduler::new(),
            trace: Vec::new(),
        }
    }

    pub fn seed(&self) -> Seed {
        self.seed
    }

    pub fn clock(&self) -> Clock {
        self.scheduler.now()
    }

    pub fn rng(&mut self) -> &mut SimRng {
        &mut self.rng
    }

    /// `(time, id)` of every delivered event, in delivery order.
    pub fn trace(&self) -> &[(Clock, EventId)] {
        &self.trace
    }

    pub fn pending(&self) -> usize {
        self.scheduler.len()
    }

    pub fn peek_next(&self) -> Option<(Clock, EventId)> {
        self.scheduler.peek()
    }

    /// Fails with [`InPast`] if `at` is earlier than [`Self::clock`]; nothing is queued then.
    pub fn schedule_at(&mut self, at: Clock, payload: T) -> Result<EventId, InPast> {
        self.scheduler.schedule(at, payload)
    }

    /// Saturates at `u64::MAX` ticks.
    pub fn schedule_in(&mut self, delay: u64, payload: T) -> EventId {
        let at = self.clock().after(delay);
        self.scheduler
            .schedule(at, payload)
            .expect("now + delay is never in the past")
    }

    /// Delivers the earliest event and advances the clock to its time.
    pub fn step(&mut self) -> Option<TimedEvent<T>> {
        let event = self.scheduler.pop()?;
        self.trace.push((event.at, event.id));
        Some(event)
    }

    /// Steps until the queue drains or `max_steps` events are delivered.
    pub fn run(&mut self, max_steps: u64, handler: &mut impl EventHandler<T>) -> RunSummary {
        let mut delivered = 0;
        while delivered < max_steps {
            let Some(event) = self.step() else { break };
            handler.on_event(self, event.payload);
            delivered += 1;
        }
        RunSummary {
            delivered,
            drained: self.pending() == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_can_reschedule() {
        let mut w = SimWorld::new(Seed::new(1));
        w.schedule_in(1, 0u32);
        let mut seen = Vec::new();
        let summary = w.run(5, &mut |world: &mut SimWorld<u32>, n| {
            seen.push(n);
            if n < 3 {
                world.schedule_in(1, n + 1);
            }
        });
        assert_eq!(seen, vec![0, 1, 2, 3]);
        assert_eq!(
            summary,
            RunSummary {
                delivered: 4,
                drained: true
            }
        );
        assert_eq!(w.clock(), Clock::new(4));
    }

    #[test]
    fn run_stops_at_max_steps() {
        let mut w = SimWorld::new(Seed::new(2));
        for i in 0..5u32 {
            w.schedule_in(u64::from(i), i);
        }
        let summary = w.run(2, &mut |_: &mut SimWorld<u32>, _| {});
        assert_eq!(
            summary,
            RunSummary {
                delivered: 2,
                drained: false
            }
        );
        assert_eq!(w.pending(), 3);
    }

    #[test]
    fn past_schedule_is_rejected_and_clock_holds() {
        let mut w = SimWorld::new(Seed::new(3));
        w.schedule_at(Clock::new(10), 0u32).unwrap();
        w.step().unwrap();
        assert!(w.schedule_at(Clock::new(5), 1).is_err());
        assert_eq!(w.pending(), 0);
        assert!(w.step().is_none());
        assert_eq!(w.clock(), Clock::new(10));
    }

    #[test]
    fn peek_matches_next_step() {
        let mut w = SimWorld::new(Seed::new(3));
        let id = w.schedule_at(Clock::new(4), 7u32).unwrap();
        assert_eq!(w.peek_next(), Some((Clock::new(4), id)));
        let event = w.step().unwrap();
        assert_eq!((event.id, event.payload), (id, 7));
    }
}
