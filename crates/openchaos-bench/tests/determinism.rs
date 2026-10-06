//! Runs the `sim_events` example under callgrind twice and expects identical counts.

use anyhow::{ensure, Context, Result};
use openchaos_bench::{valgrind_version, Report};
use std::path::{Path, PathBuf};
use std::process::Command;

const CLI: &str = env!("CARGO_BIN_EXE_openchaos-bench");

fn example() -> Result<PathBuf> {
    let profile_dir = Path::new(CLI)
        .parent()
        .context("openchaos-bench binary has no target profile directory")?;
    Ok(profile_dir.join("examples").join("sim_events"))
}

fn run(out_dir: &Path) -> Result<Report> {
    let status = Command::new(CLI)
        .arg("run")
        .arg("--out-dir")
        .arg(out_dir)
        .arg("--")
        .arg(example()?)
        .status()?;
    ensure!(status.success(), "openchaos-bench run failed: {status}");
    let text = std::fs::read_to_string(out_dir.join("report.json"))?;
    Ok(serde_json::from_str(&text)?)
}

#[test]
#[ignore = "needs valgrind; CI runs it with --ignored"]
fn sim_world_counts_repeat_exactly() -> Result<()> {
    valgrind_version()?;
    ensure!(
        example()?.exists(),
        "build the example first: cargo build -p openchaos-bench --example sim_events"
    );
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let first = run(&tmp.join("determinism-1"))?;
    let second = run(&tmp.join("determinism-2"))?;

    let names: Vec<_> = first.benchmarks.keys().cloned().collect();
    assert_eq!(names, ["sim_world/10000_events", "sim_world/1000_events"]);
    let small = first.benchmarks["sim_world/1000_events"].instructions;
    let large = first.benchmarks["sim_world/10000_events"].instructions;
    assert!(small > 0 && large > small);
    assert_eq!(first.benchmarks, second.benchmarks);
    Ok(())
}
