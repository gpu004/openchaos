use crate::callgrind::Metrics;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Results of one `openchaos-bench run`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// `valgrind --version` output of the run.
    pub valgrind: String,
    /// Metrics per benchmark name.
    pub benchmarks: BTreeMap<String, Metrics>,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = name_width(self.benchmarks.keys());
        writeln!(
            f,
            "{:width$}  {:>14}  {:>14}  {:>14}  {:>10}  {:>10}",
            "benchmark", "instructions", "est. cycles", "L1 hits", "LL hits", "RAM hits"
        )?;
        for (name, m) in &self.benchmarks {
            writeln!(
                f,
                "{name:width$}  {:>14}  {:>14}  {:>14}  {:>10}  {:>10}",
                m.instructions, m.estimated_cycles, m.l1_hits, m.ll_hits, m.ram_hits
            )?;
        }
        Ok(())
    }
}

/// Estimated cycles of one benchmark in two reports.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// Benchmark name.
    pub name: String,
    /// Estimated cycles in the base report.
    pub base: u64,
    /// Estimated cycles in the head report.
    pub head: u64,
}

impl Change {
    /// Relative change from base to head, in percent.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        clippy::as_conversions,
        reason = "cycle counts stay far below 2^52 and the percentage is only compared to a threshold"
    )]
    pub fn percent(&self) -> f64 {
        if self.base == self.head {
            return 0.0;
        }
        (self.head as f64 - self.base as f64) / self.base as f64 * 100.0
    }
}

/// Benchmark-by-benchmark comparison of two reports.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    /// Benchmarks present in both reports.
    pub changes: Vec<Change>,
    /// Benchmarks only in the base report.
    pub removed: Vec<String>,
    /// Benchmarks only in the head report.
    pub added: Vec<String>,
}

impl Comparison {
    /// Changes whose estimated cycles grew by more than `threshold_percent`.
    #[must_use]
    pub fn regressions(&self, threshold_percent: f64) -> Vec<&Change> {
        self.changes
            .iter()
            .filter(|c| c.percent() > threshold_percent)
            .collect()
    }
}

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = name_width(self.changes.iter().map(|c| &c.name));
        writeln!(
            f,
            "{:width$}  {:>14}  {:>14}  {:>9}",
            "benchmark", "base cycles", "head cycles", "change"
        )?;
        for c in &self.changes {
            writeln!(
                f,
                "{:width$}  {:>14}  {:>14}  {:>+8.2}%",
                c.name,
                c.base,
                c.head,
                c.percent()
            )?;
        }
        for name in &self.removed {
            writeln!(f, "removed: {name}")?;
        }
        for name in &self.added {
            writeln!(f, "added: {name}")?;
        }
        Ok(())
    }
}

/// Pair up the benchmarks of `base` and `head` by name.
#[must_use]
pub fn compare(base: &Report, head: &Report) -> Comparison {
    let changes = base
        .benchmarks
        .iter()
        .filter_map(|(name, b)| {
            head.benchmarks.get(name).map(|h| Change {
                name: name.clone(),
                base: b.estimated_cycles,
                head: h.estimated_cycles,
            })
        })
        .collect();
    let missing_from = |from: &Report, other: &Report| {
        from.benchmarks
            .keys()
            .filter(|name| !other.benchmarks.contains_key(*name))
            .cloned()
            .collect()
    };
    Comparison {
        changes,
        removed: missing_from(base, head),
        added: missing_from(head, base),
    }
}

fn name_width<'a>(names: impl Iterator<Item = &'a String>) -> usize {
    names
        .map(String::len)
        .max()
        .unwrap_or(0)
        .max("benchmark".len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(entries: &[(&str, u64)]) -> Report {
        Report {
            valgrind: "valgrind-test".to_owned(),
            benchmarks: entries
                .iter()
                .map(|&(name, cycles)| {
                    let metrics = Metrics {
                        instructions: cycles,
                        l1_hits: cycles,
                        ll_hits: 0,
                        ram_hits: 0,
                        estimated_cycles: cycles,
                    };
                    (name.to_owned(), metrics)
                })
                .collect(),
        }
    }

    #[test]
    fn flags_only_changes_over_threshold() {
        let base = report(&[("fast", 1000), ("slow", 1000), ("gone", 5)]);
        let head = report(&[("fast", 900), ("slow", 1011), ("new", 5)]);
        let cmp = compare(&base, &head);
        assert_eq!(cmp.removed, ["gone"]);
        assert_eq!(cmp.added, ["new"]);
        let names = |t| -> Vec<String> {
            cmp.regressions(t)
                .into_iter()
                .map(|c| c.name.clone())
                .collect()
        };
        assert_eq!(names(1.0), ["slow"]);
        assert!(names(1.1).is_empty());
    }

    #[test]
    fn growth_from_zero_is_a_regression() {
        let cmp = compare(&report(&[("x", 0)]), &report(&[("x", 1)]));
        assert_eq!(cmp.regressions(1000.0).len(), 1);
    }
}
