# ADR 0003: PBT via thin Hegel adapter, not hand-rolled engine

## Status

Accepted

## Context

The Rust salvage embedded a Hegel-*shaped* PBT engine (choice tape, generators,
shrink, `check`) inside the same crate as the sim. That duplicated Hegel’s
interface without adding sim depth, and would have forced every language to
either reimplement PBT or depend on a Rust-only lookalike.

## Decision

- `openchaos-core` has **zero** PBT / Hegel dependency.
- `lang/rust` depends on official `hegeltest` (crate lib name `hegel`) and exposes
  `openchaos::bind` that only maps draws → core `Seed` / `SimWorld`.
- Hand-rolled `pbt/*` from the salvage is deleted (never copied into core).

## Consequences

- Deletion test: remove the Hegel adapter and sim determinism is unchanged.
- Other language packages use their own Hegel clients the same way.
- Property authors write `#[hegel::test]` and drive core — never a second engine.
