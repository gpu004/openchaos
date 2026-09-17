# ADR 0007: Demos quarantine behind a thin example seam

## Status

Accepted

## Context

Prior slices re-exported demo worlds (TS ledger) or mixed examples with the
library façade, risking mistaking scenarios for core depth.

## Decision

- Demos live under `examples/` / `lang/*/examples` / optional `tests/demo_*`.
- They call only the thin language package + core; they are not re-exported
  from crate roots and do not justify a second sim.
- Deleting demos must not change seed/RNG/world/meter depth.

## Consequences

- Phase 2 can port the TS ledger against bindings without promoting it into core.
- README documents how to run examples without implying they are the product API.
