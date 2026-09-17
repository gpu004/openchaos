# ADR 0001: Sim depth lives only in openchaos-core

## Status

Accepted

## Context

Two prior first-slice branches each owned a simulator inside a language package
(TypeScript embedded sim; Rust façade mixing sim + hand-rolled PBT). That
inverted the dependency we need for multi-language Hegel support: language
support must not own the loop.

## Decision

Carve one deep `SimWorld` module cluster into `crates/openchaos-core`
(seed/RNG, clock, payload scheduler, world, meters). Language packages are
thin adapters over that seam. Language-owned sims (including the TS
`Simulator` / `SeededRng` / `Instrument` tree) are retired as product shape.

## Consequences

- Same-seed determinism and logical metrics have a single locality.
- Adding a language means bindings + Hegel client, never a forked sim.
- Hand-rolled PBT and demos must not re-enter core as depth.
