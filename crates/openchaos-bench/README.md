# openchaos-bench

Deterministic benchmarks for openchaos. `openchaos-bench` runs a program under
callgrind and reports the instructions executed inside each region the program
marks with `openchaos_bench::bench`. The same binary on the same machine gives
the same instruction count on every run.

This is a trimmed copy of the simulation mode of the
[CodSpeed runner](https://github.com/CodSpeedHQ/codspeed). It makes no network
calls and needs no account. See [NOTICE](NOTICE) for which files come from
CodSpeed.

## Usage

Install Valgrind (`sudo apt-get install valgrind`), then mark the code to
measure:

```rust
let delivered = openchaos_bench::bench(c"sim_world/10000_events", || run_events(10_000));
```

Outside Valgrind, `bench` only calls the closure. Run the program under the
runner:

```bash
cargo build --release --example sim_events -p openchaos-bench
cargo run -p openchaos-bench -- run --out-dir target/bench-head -- target/release/examples/sim_events
```

`run` prints a table and writes `<out-dir>/report.json` with, per benchmark,
`instructions`, `l1_hits`, `ll_hits`, `ram_hits` and `estimated_cycles`.
Child processes are traced too, so `run -- cargo bench` works, but running the
built binary directly is faster.

Compare two reports. The command exits with code 1 if any benchmark's
estimated cycles grew by more than `--threshold` percent (default 1):

```bash
cargo run -p openchaos-bench -- compare target/bench-base/report.json target/bench-head/report.json
```

## Metrics

Callgrind simulates a 32 KiB 8-way I1 and D1 cache and an 8 MiB 16-way LL
cache, the geometry CodSpeed uses, so the cache counts do not depend on the
host CPU. The cycle estimate is

```text
estimated_cycles = l1_hits + 5 * ll_hits + 35 * ram_hits
```

CodSpeed computes its cycle estimate inside its Valgrind fork
(`--cycle-estimation`) and on its servers, so this crate uses the cachegrind
weights above instead. Instruction counts are exact. Cache counts can shift by
a few hits when the environment or argument list changes, because those move
the stack, so compare reports produced with the same command.

## Valgrind choice

The runner uses the system `valgrind`. CodSpeed downloads a pinned
valgrind-codspeed `.deb` with `sudo` or builds it from source. The fork adds
`--cycle-estimation` and output for CodSpeed's servers; this crate needs
neither, and upstream callgrind counts the same instructions. The report
records `valgrind --version`, and `compare` warns when the two reports come
from different versions.

## Kept from CodSpeed

- The callgrind arguments: cache geometry, `--instr-atstart=no` so only marked
  regions count, `--trace-children=yes`, `--combine-dumps=yes`, and one
  profile and log file per process.
- `setarch --addr-no-randomize`, which disables ASLR for the benchmark.
- The instrument-hooks marker sequence: zero stats, start instrumentation, run,
  stop instrumentation, dump stats named after the benchmark.

## Removed

Everything else: the API client, login, upload, CI provider detection,
walltime and perf, memory tracking, samply, MCP and plugin files, Tracegrind,
the valgrind-codspeed download and source build, and Python and Node support.
The C instrument-hooks library is replaced by inline client requests for
x86_64 and aarch64 Linux.
