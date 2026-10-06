use crate::callgrind::parse_profile;
use crate::report::Report;
use anyhow::{bail, ensure, Context, Result};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const VALGRIND_ARGS: &[&str] = &[
    "-q",
    "--tool=callgrind",
    "--trace-children=yes",
    "--cache-sim=yes",
    "--I1=32768,8,64",
    "--D1=32768,8,64",
    "--LL=8388608,16,64",
    "--read-inline-info=yes",
    "--instr-atstart=no",
    "--separate-threads=no",
    "--compress-strings=no",
    "--combine-dumps=yes",
    "--dump-line=no",
];

/// `valgrind --version`, or an error if Valgrind is not on `PATH`.
pub fn valgrind_version() -> Result<String> {
    let output = Command::new("valgrind")
        .arg("--version")
        .output()
        .context("valgrind not found on PATH; install it, e.g. `sudo apt-get install valgrind`")?;
    ensure!(output.status.success(), "`valgrind --version` failed");
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Run `command` under callgrind and collect the regions marked with
/// [`crate::bench`] from every traced process.
///
/// Profiles and Valgrind logs go to `out_dir`; files left there by a previous
/// run are deleted first. Fails if the command fails or if two regions share a
/// name.
pub fn measure(command: &[OsString], out_dir: &Path) -> Result<Report> {
    ensure!(!command.is_empty(), "no command to run");
    let valgrind = valgrind_version()?;
    fs::create_dir_all(out_dir)?;
    for path in files_with_extension(out_dir, "out")?
        .into_iter()
        .chain(files_with_extension(out_dir, "log")?)
    {
        fs::remove_file(path)?;
    }

    let status = Command::new("setarch")
        .arg(std::env::consts::ARCH)
        .arg("--addr-no-randomize")
        .arg("valgrind")
        .args(VALGRIND_ARGS)
        .arg(format!(
            "--callgrind-out-file={}",
            out_dir.join("%p.out").display()
        ))
        .arg(format!(
            "--log-file={}",
            out_dir.join("valgrind.%p.log").display()
        ))
        .args(command)
        .status()
        .context("failed to start setarch")?;
    if !status.success() {
        for path in files_with_extension(out_dir, "log")? {
            eprint!("{}", fs::read_to_string(&path).unwrap_or_default());
        }
        bail!("benchmark command failed: {status}");
    }

    let mut benchmarks = BTreeMap::new();
    for path in files_with_extension(out_dir, "out")? {
        let text = fs::read_to_string(&path)?;
        let regions = parse_profile(&text).with_context(|| path.display().to_string())?;
        for (name, metrics) in regions {
            ensure!(
                benchmarks.insert(name.clone(), metrics).is_none(),
                "benchmark {name:?} was measured more than once"
            );
        }
    }
    Ok(Report {
        valgrind,
        benchmarks,
    })
}

fn files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == extension) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
