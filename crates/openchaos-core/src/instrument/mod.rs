//! CodSpeed-inspired simulation instrumentation (core-only depth).
//!
//! Language packages adapt start/stop/report through the binding seam; they
//! must not embed a second meter (the TS `Instrument` region map is retired as
//! product shape — its *ideas* live here as named [`RegionStats`]).
//!
//! Logical counters are hardware-independent. Wall duration on [`SimReport`] is
//! an optional side channel only — never the simulation metric.

use core::fmt;
use core::time::Duration;
use std::collections::BTreeMap;

/// Per-region logical totals (CodSpeed-like named instrument).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegionStats {
    /// How many times this region was closed (completed entries).
    pub entries: u64,
    /// Steps attributed while this region was the innermost open span.
    pub steps: u64,
    /// Bytes attributed while this region was the innermost open span.
    pub bytes: u64,
}

/// Logical work counters accumulated during a simulation or bench.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Meter {
    /// Discrete sim steps (events delivered).
    pub steps: u64,
    /// Events scheduled.
    pub events: u64,
    /// Estimated bytes touched / processed (user-defined).
    pub bytes: u64,
    /// Named span open/close pairs recorded (total across regions).
    pub span_entries: u64,
    regions: BTreeMap<String, RegionStats>,
    open: Vec<String>,
}

impl Meter {
    /// Empty meter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one delivered step (also attributes to the innermost open region).
    pub fn record_step(&mut self) {
        self.steps = self.steps.saturating_add(1);
        if let Some(name) = self.open.last() {
            let region = self.regions.entry(name.clone()).or_default();
            region.steps = region.steps.saturating_add(1);
        }
    }

    /// Record one scheduled event.
    pub fn record_event(&mut self) {
        self.events = self.events.saturating_add(1);
    }

    /// Record estimated bytes of work (also attributes to innermost open region).
    pub fn record_bytes(&mut self, n: u64) {
        self.bytes = self.bytes.saturating_add(n);
        if let Some(name) = self.open.last() {
            let region = self.regions.entry(name.clone()).or_default();
            region.bytes = region.bytes.saturating_add(n);
        }
    }

    /// Begin a named region (FFI-friendly; pairs with [`Self::end`]).
    pub fn begin(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.regions.entry(name.clone()).or_default();
        self.open.push(name);
    }

    /// End the innermost open region.
    ///
    /// Returns `false` if no region was open.
    pub fn end(&mut self) -> bool {
        let Some(name) = self.open.pop() else {
            return false;
        };
        let region = self.regions.entry(name).or_default();
        region.entries = region.entries.saturating_add(1);
        self.span_entries = self.span_entries.saturating_add(1);
        true
    }

    /// Run `f` inside a named region (RAII-equivalent without shared ownership).
    pub fn with_span<R>(&mut self, name: impl Into<String>, f: impl FnOnce(&mut Meter) -> R) -> R {
        self.begin(name);
        let result = f(self);
        let _: bool = self.end();
        result
    }

    /// Snapshot of named region stats (sorted by name via [`BTreeMap`]).
    #[must_use]
    pub fn regions(&self) -> &BTreeMap<String, RegionStats> {
        &self.regions
    }

    /// Lookup one region by name.
    #[must_use]
    pub fn region(&self, name: &str) -> Option<&RegionStats> {
        self.regions.get(name)
    }

    /// Merge another meter's totals into this one (regions included).
    pub fn merge(&mut self, other: &Meter) {
        self.steps = self.steps.saturating_add(other.steps);
        self.events = self.events.saturating_add(other.events);
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.span_entries = self.span_entries.saturating_add(other.span_entries);
        for (name, stats) in &other.regions {
            let dest = self.regions.entry(name.clone()).or_default();
            dest.entries = dest.entries.saturating_add(stats.entries);
            dest.steps = dest.steps.saturating_add(stats.steps);
            dest.bytes = dest.bytes.saturating_add(stats.bytes);
        }
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
            "bench_sim[{}]: steps={} events={} bytes={} spans={} regions={} (wall {}us)",
            self.name,
            self.meter.steps,
            self.meter.events,
            self.meter.bytes,
            self.meter.span_entries,
            self.meter.regions.len(),
            self.wall.as_micros()
        )
    }
}

/// Guard that ends a region on drop when created via [`Span::enter`].
#[derive(Debug)]
pub struct Span<'a> {
    meter: &'a mut Meter,
    active: bool,
}

impl<'a> Span<'a> {
    /// Begin `name` on `meter`; ends automatically on drop.
    pub fn enter(meter: &'a mut Meter, name: impl Into<String>) -> Self {
        meter.begin(name);
        Self {
            meter,
            active: true,
        }
    }

    /// End early (idempotent with [`Drop`]).
    pub fn end(mut self) {
        if self.active {
            let _: bool = self.meter.end();
            self.active = false;
        }
    }
}

impl Drop for Span<'_> {
    fn drop(&mut self) {
        if self.active {
            let _: bool = self.meter.end();
            self.active = false;
        }
    }
}

/// Run `f` once under simulation metering and return a [`SimReport`].
///
/// Analogous to `CodSpeed` simulation mode: a single deterministic pass yields
/// stable logical metrics rather than noisy wall-clock samples.
pub fn bench_sim<F>(name: impl Into<String>, mut f: F) -> SimReport
where
    F: FnMut(&mut Meter),
{
    let name = name.into();
    let mut meter = Meter::new();
    #[expect(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        reason = "wall time is an informational side channel on SimReport, never a sim input"
    )]
    let start = std::time::Instant::now();
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
    fn named_regions_attribute_inner_work() {
        let mut meter = Meter::new();
        meter.with_span("parse", |m| {
            m.record_step();
            m.record_bytes(10);
            m.with_span("decode", |m| {
                m.record_step();
                m.record_bytes(5);
            });
        });
        assert_eq!(meter.span_entries, 2);
        assert_eq!(meter.region("parse").unwrap().steps, 1);
        assert_eq!(meter.region("parse").unwrap().bytes, 10);
        assert_eq!(meter.region("decode").unwrap().steps, 1);
        assert_eq!(meter.region("decode").unwrap().bytes, 5);
        assert_eq!(meter.region("decode").unwrap().entries, 1);
    }

    #[test]
    fn span_guard_increments_on_drop() {
        let mut meter = Meter::new();
        {
            let _span = Span::enter(&mut meter, "work");
        }
        assert_eq!(meter.span_entries, 1);
        assert_eq!(meter.region("work").unwrap().entries, 1);
    }

    #[test]
    fn merge_folds_regions() {
        let mut a = Meter::new();
        a.with_span("x", |m| m.record_bytes(3));
        let mut b = Meter::new();
        b.with_span("x", |m| m.record_bytes(4));
        a.merge(&b);
        assert_eq!(a.region("x").unwrap().bytes, 7);
        assert_eq!(a.region("x").unwrap().entries, 2);
    }
}
