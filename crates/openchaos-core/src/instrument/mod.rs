//! CodSpeed-inspired simulation instrumentation.
//!
//! Instead of Valgrind CPU simulation, openchaos meters **logical work** inside
//! the deterministic sim: steps, events, and byte estimates. Benchmarks run
//! once under simulation (stable, hardware-independent), mirroring CodSpeed's
//! simulation-mode philosophy.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Logical work counters accumulated during a simulation or bench.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Meter {
    /// Discrete sim steps (events delivered).
    pub steps: u64,
    /// Events scheduled.
    pub events: u64,
    /// Estimated bytes touched / processed (user-defined).
    pub bytes: u64,
    /// Named span open/close pairs recorded.
    pub span_entries: u64,
}

impl Meter {
    /// Empty meter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one delivered step.
    pub fn record_step(&mut self) {
        self.steps = self.steps.saturating_add(1);
    }

    /// Record one scheduled event.
    pub fn record_event(&mut self) {
        self.events = self.events.saturating_add(1);
    }

    /// Record estimated bytes of work.
    pub fn record_bytes(&mut self, n: u64) {
        self.bytes = self.bytes.saturating_add(n);
    }

    /// Merge another meter's totals into this one.
    pub fn merge(&mut self, other: &Meter) {
        self.steps = self.steps.saturating_add(other.steps);
        self.events = self.events.saturating_add(other.events);
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.span_entries = self.span_entries.saturating_add(other.span_entries);
    }
}

/// Report produced by [`bench_sim`].
#[derive(Debug, Clone)]
pub struct SimReport {
    /// Benchmark name.
    pub name: String,
    /// Logical meters from the run.
    pub meter: Meter,
    /// Optional wall duration (informational only — not the simulation metric).
    pub wall: Duration,
}

impl fmt::Display for SimReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bench_sim[{}]: steps={} events={} bytes={} (wall {:?})",
            self.name, self.meter.steps, self.meter.events, self.meter.bytes, self.wall
        )
    }
}

/// RAII span that increments `span_entries` on drop (CodSpeed-like region marker).
#[derive(Debug)]
pub struct Span {
    meter: Rc<RefCell<Meter>>,
    name: &'static str,
    active: bool,
}

impl Span {
    /// Begin a named span bound to `meter`.
    pub fn new(meter: Rc<RefCell<Meter>>, name: &'static str) -> Self {
        Self {
            meter,
            name,
            active: true,
        }
    }

    /// Span name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if self.active {
            let mut m = self.meter.borrow_mut();
            m.span_entries = m.span_entries.saturating_add(1);
            self.active = false;
        }
    }
}

/// Run `f` once under simulation metering and return a [`SimReport`].
///
/// Analogous to CodSpeed simulation mode: a single deterministic pass yields
/// stable logical metrics rather than noisy wall-clock samples.
pub fn bench_sim<F>(name: impl Into<String>, mut f: F) -> SimReport
where
    F: FnMut(&mut Meter),
{
    let name = name.into();
    let mut meter = Meter::new();
    let start = Instant::now();
    f(&mut meter);
    let wall = start.elapsed();
    SimReport { name, meter, wall }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_sim_records_logical_work() {
        let report = bench_sim("toy", |m| {
            m.record_step();
            m.record_step();
            m.record_bytes(64);
        });
        assert_eq!(report.meter.steps, 2);
        assert_eq!(report.meter.bytes, 64);
        assert_eq!(report.name, "toy");
    }

    #[test]
    fn span_increments_on_drop() {
        let meter = Rc::new(RefCell::new(Meter::new()));
        {
            let _span = Span::new(Rc::clone(&meter), "work");
        }
        assert_eq!(meter.borrow().span_entries, 1);
    }
}
