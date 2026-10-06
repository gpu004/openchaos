//! Deterministic benchmarks: run a program under callgrind, collect the
//! regions it marks with [`bench`], and compare reports.

#![deny(missing_docs)]

mod callgrind;
mod hooks;
mod report;
mod valgrind;

pub use callgrind::{parse_profile, Metrics};
pub use hooks::{bench, running_on_valgrind};
pub use report::{compare, Comparison, Report};
pub use valgrind::{measure, valgrind_version};
