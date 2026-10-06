# openchaos (Rust)

Re-exports [`openchaos-core`](../../crates/openchaos-core) and adds
`draw_seed` / `draw_world`, which draw a seed from a Hegel test case, and
`draw_sim_config`, which draws a `SimConfig` for a `Cluster`. `draw_faults`
and `draw_workload` draw timed faults and client requests. `Scenario` bundles
all of these; see the quick start in the [top-level README](../../README.md).

```rust
use hegel::TestCase;
use openchaos::{draw_world, Clock};

#[hegel::test]
fn example(tc: TestCase) {
    let mut world = draw_world::<u64>(&tc);
    world.schedule_at(Clock::new(0), 1).unwrap();
    assert!(world.step().is_some());
}
```
