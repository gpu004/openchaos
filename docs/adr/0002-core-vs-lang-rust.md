# ADR 0002: Split Rust façade into core vs thin lang/rust

## Status

Accepted

## Context

The salvage crate `openchaos` re-exported sim, meters, and hand-rolled PBT from
one package interface. That wide façade hid the seam between host core and
language support, so “openchaos = one Rust crate” blocked clean multi-language
bindings.

## Decision

- `crates/openchaos-core` — deep sim + meters only; no Hegel.
- `lang/rust` (crate name `openchaos`) — thin in-process adapter: re-exports
  core and owns Hegel wiring. Never owns clock/scheduler/world/meters.

## Consequences

- Package interface narrows: core is publishable without PBT; lang/rust can
  depend on hegeltest without contaminating core.
- Future `lang/ts`, `lang/go`, … mirror the same thinness.
