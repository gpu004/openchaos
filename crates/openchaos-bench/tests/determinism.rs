use openchaos_bench::{valgrind_version, Report};
use std::path::{Path, PathBuf};
use std::process::Command;

fn example() -> PathBuf {
    let deps = std::env::current_exe().unwrap();
    let profile_dir = deps.parent().unwrap().parent().unwrap();
    profile_dir.join("examples").join("sim_events")
}

fn run(out_dir: &Path) -> Report {
    let status = Command::new(env!("CARGO_BIN_EXE_openchaos-bench"))
        .arg("run")
        .arg("--out-dir")
        .arg(out_dir)
        .arg("--")
        .arg(example())
        .status()
        .unwrap();
    assert!(status.success());
    let text = std::fs::read_to_string(out_dir.join("report.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn sim_world_counts_repeat_exactly() {
    if valgrind_version().is_err() {
        assert!(std::env::var_os("CI").is_none(), "CI must install valgrind");
        eprintln!("skipping: valgrind is not installed");
        return;
    }
    assert!(
        example().exists(),
        "build the example first: cargo build -p openchaos-bench --example sim_events"
    );
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let first = run(&tmp.join("determinism-1"));
    let second = run(&tmp.join("determinism-2"));

    let names: Vec<_> = first.benchmarks.keys().cloned().collect();
    assert_eq!(names, ["sim_world/10000_events", "sim_world/1000_events"]);
    let small = first.benchmarks["sim_world/1000_events"].instructions;
    let large = first.benchmarks["sim_world/10000_events"].instructions;
    assert!(small > 0 && large > small);
    assert_eq!(first.benchmarks, second.benchmarks);
}
