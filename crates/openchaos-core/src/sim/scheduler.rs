//! Priority queue of timed events.

use crate::sim::Clock;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Opaque handle assigned when an event is enqueued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(u64);

impl EventId {
    /// Raw id bits.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// An event stamped with logical time and stable insertion order.
#[derive(Debug, Clone)]
pub struct TimedEvent<T> {
    /// When the event fires.
    pub at: Clock,
    /// Stable id (lower = earlier for equal times).
    pub id: EventId,
    /// User payload.
    pub payload: T,
}

#[derive(Debug)]
struct HeapEntry<T> {
    at: Clock,
    id: EventId,
    payload: T,
}

impl<T> PartialEq for HeapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at && self.id == other.id
    }
}

impl<T> Eq for HeapEntry<T> {}

impl<T> PartialOrd for HeapEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for HeapEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap; reverse so earliest time / lowest id pops first.
        match other.at.cmp(&self.at) {
            Ordering::Equal => other.id.0.cmp(&self.id.0),
            ord => ord,
        }
    }
}

/// Deterministic event priority queue.
#[derive(Debug)]
pub struct Scheduler<T> {
    next_id: u64,
    heap: BinaryHeap<HeapEntry<T>>,
}

impl<T> Default for Scheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Scheduler<T> {
    /// Empty scheduler.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            heap: BinaryHeap::new(),
        }
    }

    /// Number of pending events.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Schedule `payload` at logical time `at`.
    pub fn schedule(&mut self, at: Clock, payload: T) -> EventId {
        let id = EventId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.heap.push(HeapEntry { at, id, payload });
        id
    }

    /// Peek at the next event without removing it.
    pub fn peek(&self) -> Option<(Clock, EventId)> {
        self.heap.peek().map(|e| (e.at, e.id))
    }

    /// Pop the next event.
    pub fn pop(&mut self) -> Option<TimedEvent<T>> {
        self.heap.pop().map(|e| TimedEvent {
            at: e.at,
            id: e.id,
            payload: e.payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_by_time_then_id() {
        let mut s = Scheduler::new();
        s.schedule(Clock::new(5), "late");
        let first = s.schedule(Clock::new(1), "a");
        let second = s.schedule(Clock::new(1), "b");
        let e1 = s.pop().unwrap();
        let e2 = s.pop().unwrap();
        let e3 = s.pop().unwrap();
        assert_eq!(e1.id, first);
        assert_eq!(e2.id, second);
        assert_eq!(e3.payload, "late");
    }
}
