# openchaos

**Status:** Private development. Public repo is a placeholder; open sourcing soon.

## What

Deterministic simulation testing (DST) for your codebase.

1. **Distributed deterministic simulation** — model nodes, clocks, and network behavior with reproducible execution.
2. **Simulation against your code** — drive real system code under that deterministic environment; same inputs → same trace.
3. **Property-based testing** — assert invariants over simulation runs (schedules, failures, message order), not hand-written scenario lists.

Goal: find concurrency, timing, and distributed bugs before production.

## To build

- [ ] Deterministic scheduler (virtual time, seeded RNG, ordered events)
- [ ] Network/process model (latency, partitions, drops, reorder)
- [ ] Harness to run your code inside the simulator
- [ ] Trace capture and replay (record run → replay for debug)
- [ ] Property layer (generators for schedules/topologies; shrink on failure)
- [ ] CLI: `simulate`, `replay`, `check` (properties)
- [ ] Minimal example (e.g. leader election or RPC client) proving the loop

## License

[MIT](./LICENSE)
