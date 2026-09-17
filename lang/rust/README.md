# openchaos (Rust language package)

Thin Rust adapter over [`openchaos-core`](../../crates/openchaos-core).

This package does **not** embed a simulator. It re-exports the core seam for
in-process use and (later) hosts the Hegel PBT adapter.

```bash
cargo test -p openchaos
```
