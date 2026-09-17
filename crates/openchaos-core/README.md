# openchaos-core

Shared **deterministic discrete-event simulation** for openchaos.

This crate owns the deep sim modules: seed/RNG, logical clock, payload scheduler,
`SimWorld`, and CodSpeed-inspired logical meters. Language packages must bind to
this core — they must not embed a second simulator.

```bash
cargo test -p openchaos-core
```
