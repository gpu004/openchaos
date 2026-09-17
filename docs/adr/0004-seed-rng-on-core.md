# ADR 0004: Seed and PRNG depth live only on core

## Status

Accepted

## Context

Prior slices used incompatible seed/RNG modules (Rust `u64` + xoshiro128** vs
TypeScript `number` + xorshift32). Same-seed replay across languages was fake
because entropy locality was duplicated.

## Decision

- `Seed` is a first-class core module (`sim/seed.rs`) with `derive` / `stream` /
  `mix` for domain separation at binding seams.
- `SimRng` is the sole PRNG stream (xoshiro128**). Golden vectors lock the
  first outputs for adapter verification.
- Language packages adapt host integers into `Seed`; they never reimplement the
  stream.

## Consequences

- Phase 1 same-seed success criterion is meaningful.
- Future FFI exports the same `u64` seed and stream contract.
