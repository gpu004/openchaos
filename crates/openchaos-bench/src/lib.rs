//! Deterministic benchmarks: run a program under callgrind, collect the
//! regions it marks with [`bench()`], and compare reports.

mod callgrind;
mod hooks;
mod report;
mod valgrind;

pub use callgrind::Metrics;
pub use hooks::bench;
pub use report::{compare, Change, Comparison, Report};
pub use valgrind::{measure, valgrind_version};
