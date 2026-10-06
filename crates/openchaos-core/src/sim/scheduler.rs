use crate::sim::Clock;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::fmt;

/// Insertion order of an event. Breaks ties between events due at the same time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(u64);

impl EventId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct TimedEvent<T> {
    pub at: Clock,
    pub id: EventId,
    pub payload: T,
}

/// Returned when an event is scheduled before the current time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InPast {
    pub at: Clock,
    pub now: Clock,
}

impl fmt::Display for InPast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot schedule at {} before now ({})",
            self.at, self.now
        )
    }
}

impl std::error::Error for InPast {}

#[derive(Debug)]
struct Entry<T>(TimedEvent<T>);

impl<T> Entry<T> {
    fn key(&self) -> (Clock, EventId) {
        (self.0.at, self.0.id)
    }
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl<T> Eq for Entry<T> {}

impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

#[derive(Debug)]
pub(crate) struct Scheduler<T> {
    now: Clock,
    next_id: u64,
    heap: BinaryHeap<Reverse<Entry<T>>>,
}

impl<T> Scheduler<T> {
    pub(crate) fn new() -> Self {
        Self {
            now: Clock::default(),
            next_id: 0,
            heap: BinaryHeap::new(),
        }
    }

    pub(crate) fn now(&self) -> Clock {
        self.now
    }

    pub(crate) fn len(&self) -> usize {
        self.heap.len()
    }

    pub(crate) fn schedule(&mut self, at: Clock, payload: T) -> Result<EventId, InPast> {
        if at < self.now {
            return Err(InPast { at, now: self.now });
        }
        let id = EventId(self.next_id);
        self.next_id += 1;
        self.heap
            .push(Reverse(Entry(TimedEvent { at, id, payload })));
        Ok(id)
    }

    pub(crate) fn peek(&self) -> Option<(Clock, EventId)> {
        self.heap.peek().map(|Reverse(e)| e.key())
    }

    pub(crate) fn pop(&mut self) -> Option<TimedEvent<T>> {
        let Reverse(Entry(event)) = self.heap.pop()?;
        self.now = event.at;
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_by_time_then_id() {
        let mut s = Scheduler::new();
        s.schedule(Clock::new(5), "late").unwrap();
        let a = s.schedule(Clock::new(1), "a").unwrap();
        let b = s.schedule(Clock::new(1), "b").unwrap();
        assert_eq!(s.pop().unwrap().id, a);
        assert_eq!(s.pop().unwrap().id, b);
        assert_eq!(s.pop().unwrap().payload, "late");
        assert_eq!(s.now(), Clock::new(5));
    }

    #[test]
    fn rejects_past_and_keeps_clock() {
        let mut s = Scheduler::new();
        s.schedule(Clock::new(10), ()).unwrap();
        s.pop().unwrap();
        let err = s.schedule(Clock::new(3), ()).unwrap_err();
        assert_eq!(
            err,
            InPast {
                at: Clock::new(3),
                now: Clock::new(10)
            }
        );
        assert_eq!(s.len(), 0);
        assert!(s.schedule(Clock::new(10), ()).is_ok());
    }
}
