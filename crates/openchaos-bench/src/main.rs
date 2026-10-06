//! `openchaos-bench` CLI: `run` a command under callgrind, `compare` two reports.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use openchaos_bench::{compare, measure, Report};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version, about = "Deterministic benchmarks under callgrind")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a command under callgrind and write a JSON report.
    Run {
        /// Directory for callgrind profiles, Valgrind logs and the report.
        #[arg(long, default_value = "target/openchaos-bench")]
        out_dir: PathBuf,
        /// Command to benchmark.
        #[arg(last = true, required = true)]
        command: Vec<OsString>,
    },
    /// Compare two reports; fail if a benchmark regressed.
    Compare {
        /// Report of the baseline revision.
        base: PathBuf,
        /// Report of the revision under test.
        head: PathBuf,
        /// Maximum allowed growth in estimated cycles, in percent.
        #[arg(long, default_value_t = 1.0)]
        threshold: f64,
    },
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the CLI reports its results on the terminal"
)]
fn main() -> Result<()> {
    match Cli::parse().command {
        Cmd::Run { out_dir, command } => {
            let report = measure(&command, &out_dir)?;
            let path = out_dir.join("report.json");
            fs::write(&path, serde_json::to_string_pretty(&report)?)?;
            print!("{report}");
            println!("report: {}", path.display());
        }
        Cmd::Compare {
            base,
            head,
            threshold,
        } => {
            let base = read_report(&base)?;
            let head = read_report(&head)?;
            if base.valgrind != head.valgrind {
                eprintln!(
                    "warning: reports come from different Valgrind versions ({} vs {})",
                    base.valgrind, head.valgrind
                );
            }
            let comparison = compare(&base, &head);
            print!("{comparison}");
            let regressions = comparison.regressions(threshold);
            if !regressions.is_empty() {
                bail!(
                    "{} benchmark(s) regressed by more than {threshold}%",
                    regressions.len()
                );
            }
        }
    }
    Ok(())
}

fn read_report(path: &Path) -> Result<Report> {
    let text = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    serde_json::from_str(&text).with_context(|| path.display().to_string())
}
