# openchaos

Deterministic discrete-event simulation for property-based tests written with
[Hegel](https://hegel.dev/).

| Package | Path | Contents |
| --- | --- | --- |
| `openchaos-core` | `crates/openchaos-core` | Seed, RNG, clock, event queue, `SimWorld`, and `Cluster` (nodes, faulty network, skewed clocks). No dependencies. |
| `openchaos-bench` | `crates/openchaos-bench` | Runs benchmarks under callgrind and compares instruction-count reports. Trimmed from CodSpeed. |
| `openchaos` | `lang/rust` | Re-exports core and adds Hegel generators and `Scenario`, a shrinkable run of a `Cluster`. |

Requires Rust 1.86+.

```bash
cargo test --workspace
```

## Quick start

A `Scenario` holds a seed, a `SimConfig`, timed faults and client requests.
`check` runs it on your nodes, heals the network, restarts every node, runs
until no events remain, then calls your property. On failure it panics with
the shrunk scenario, the trace hash and the trace.

```rust
use hegel::{generators as gs, TestCase};
use openchaos::{Cluster, Scenario};

#[hegel::test]
fn no_acked_write_is_lost(tc: TestCase) {
    let scenario = Scenario::draw(&tc, 3, gs::integers::<u8>(), 50_000);
    scenario.check(my_nodes(), |cluster: &Cluster<MyNode>| {
        if lost_writes(cluster).is_empty() {
            Ok(())
        } else {
            Err("acknowledged write lost".to_owned())
        }
    });
}
```

`replay` converts each request op into the node's message type with `From`
and delivers it to `Node::on_request`. `scenario.replay(nodes)` returns the same
cluster, with the same `trace().hash()`, every time.

[`lang/rust/tests/replicated_kv.rs`](lang/rust/tests/replicated_kv.rs) is a
3-node replicated KV store. Its leader acknowledges writes either after a
follower stores them or immediately. Hegel shrinks the immediate version to
one write at time 0 and a leader crash at time 1:

```text
property failed: acknowledged write WriteId { epoch: 0, index: 0 } missing on node 0
Scenario { seed: Seed(0), config: SimConfig { .. }, faults: [(Clock(1), Crash(NodeId(0)))],
           workload: [Request { at: Clock(0), to: NodeId(0), op: Put { key: 0, value: 0 } }], .. }
t=0 request ->0 #0
t=1 crash 0
t=2 heal
...
```

## Design

- `openchaos-core` does not depend on Hegel. Hegel generates test inputs in the
  language packages, and the core only sees the resulting `Seed` and schedule
  calls. A package for another Hegel language can wrap the same core.
- Each `SimWorld` owns one RNG, seeded from its `Seed`. Model code draws
  randomness from `world.rng()` and nowhere else, so a seed replays a run
  exactly. `Seed::stream("net")` derives a separate seed for a subsystem.
- The event queue stores plain payloads, not callbacks. `step` returns the next
  payload and the caller handles it, or `run` passes each payload to an
  `EventHandler`. Handlers can schedule more events.
- The queue owns the clock. `schedule_at` returns `Err(InPast)` for a time
  earlier than `clock()`, so `step` can only move time forward.
- `Cluster<N: Node>` runs user state machines on a `SimWorld`. Handlers get a
  `Ctx` to send messages and set timers; the cluster applies those after the
  handler returns. `SimConfig` holds every fault parameter: per-link latency,
  drop and duplicate rates, reordering, and per-node clock skew and drift.
  Partitions, crashes and restarts are `Fault` values scheduled at a global
  time. A crash discards the node's pending timers and calls `on_crash`, where
  the node resets whatever state it treats as volatile.
- `Cluster::inject` queues a client request for `Node::on_request`. Clients
  are outside the network, so partitions and loss do not affect requests, but
  a request to a down node is dropped.
- `Cluster::trace()` records every send, delivery, request, drop, timer and
  fault, and `Display` prints one entry per line.
  The same seed, config, nodes and faults give the same `Trace::encode()`
  bytes and the same `Trace::hash()`.

## Not yet simulated

Disk I/O (latency, torn writes, fsync loss) is the planned next fault domain.

## License

MIT OR Apache-2.0. See [NOTICE](NOTICE) for attribution.
