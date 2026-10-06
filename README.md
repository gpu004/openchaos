# openchaos

Deterministic discrete-event simulation for property-based tests written with
[Hegel](https://hegel.dev/).

| Package | Path | Contents |
| --- | --- | --- |
| `openchaos-core` | `crates/openchaos-core` | Seed, RNG, clock, event queue, `SimWorld`. No dependencies. |
| `openchaos-bench` | `crates/openchaos-bench` | Runs benchmarks under callgrind and compares instruction-count reports. Trimmed from CodSpeed. |
| `openchaos` | `lang/rust` | Re-exports core and adds `draw_seed` / `draw_world` for Hegel tests. |

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

## License

MIT OR Apache-2.0. See [NOTICE](NOTICE) for attribution.
