# openchaos

Deterministic discrete-event simulation with first-class property-based testing
across Hegel languages.

## Architecture

- **`crates/openchaos-core`** — deep shared sim (seed, clock, scheduler, world,
  logical meters). No Hegel dependency.
- **`lang/<language>`** — thin packages: that language’s Hegel client + bindings
  to core. No embedded simulator.

Phase 1 ships the Rust language package. Other Hegel languages follow the same
thin-adapter pattern.

## Requirements

- Rust 1.86+ (`rustc` / `cargo`)

## Quick start

```bash
cargo test
cargo test -p openchaos-core
cargo test -p openchaos
cargo run -p openchaos --example sim_bench
cargo run -p openchaos --example lru_capacity
```

| Package | Path | Owns |
| --- | --- | --- |
| `openchaos-core` | `crates/openchaos-core` | Sim + meters (deep) |
| `openchaos` | `lang/rust` | Thin Rust bindings + Hegel `bind` adapter |
| examples | `examples/`, `lang/rust/examples/` | Quarantined demos (not core depth) |

## License

MIT OR Apache-2.0. See [NOTICE](NOTICE) for Hegel and CodSpeed attribution.
