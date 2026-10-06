# openchaos

Deterministic discrete-event simulation for property-based tests written with
[Hegel](https://hegel.dev/).

| Package | Path | Contents |
| --- | --- | --- |
| `openchaos-core` | `crates/openchaos-core` | Seed, RNG, clock, event queue, `SimWorld`, and `Cluster` (nodes, faulty network, skewed clocks). No dependencies. |
| `openchaos` | `lang/rust` | Re-exports core and adds `draw_seed`, `draw_world` and `draw_sim_config` for Hegel tests. |

Requires Rust 1.86+.

```bash
cargo test --workspace
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
- `Cluster::trace()` records every send, delivery, drop, timer and fault.
  The same seed, config, nodes and faults give the same `Trace::encode()`
  bytes and the same `Trace::hash()`.

## Not yet simulated

Disk I/O (latency, torn writes, fsync loss) is the planned next fault domain.

## License

MIT OR Apache-2.0. See [NOTICE](NOTICE) for attribution.
