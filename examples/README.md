# Examples (quarantined)

Demos and scenario sketches live **here** (and under `lang/*/examples`).
They are **not** core depth:

- Call `openchaos-core` only through the thin language package (`lang/rust`).
- Do not re-export demo worlds from crate roots.
- Safe to delete without changing sim/meter/seed locality.

```bash
cargo run -p openchaos --example sim_bench
cargo run -p openchaos --example lru_capacity
```
