# openchaos (Rust language package)

Thin Rust adapter over [`openchaos-core`](../../crates/openchaos-core).

This package does **not** embed a simulator. It re-exports the core seam for
in-process use and hosts a thin Hegel adapter (`openchaos::bind`) that draws
seeds / worlds via `hegeltest` and drives `openchaos-core`.

```bash
cargo test -p openchaos
```

```rust
use hegel::TestCase;
use openchaos::bind::draw_world_max;
use openchaos::Clock;

#[hegel::test]
fn example(tc: TestCase) {
    let mut world = draw_world_max::<u64>(&tc, 100);
    world.schedule_at(Clock::new(0), 1);
    assert!(world.step().is_some());
}
```
