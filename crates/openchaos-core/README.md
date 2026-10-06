# openchaos-core

Deterministic discrete-event simulation: `Seed`, `SimRng` (xoshiro128**),
`Clock`, and `SimWorld`, an event queue plus clock driven by `step` or `run`.
`Cluster` drives `Node` state machines over `SimWorld` with a simulated
network (latency, loss, duplication, reordering, partitions), per-node clock
skew and drift, and crash/restart faults. No dependencies.

```bash
cargo test -p openchaos-core
```
