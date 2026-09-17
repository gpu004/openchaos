# ADR 0006: World/run seam is payload queue + external handler

## Status

Accepted

## Context

The TypeScript first slice stored labeled closures on the event queue, tying the
sim interface to one runtime’s call model. Multi-language bindings need a world
seam that does not assume host functions live inside the scheduler.

## Decision

- Core `Scheduler` / `SimWorld` store **opaque payloads** only.
- Handlers run outside the queue via `step`, `run_until`, or `run_with` +
  `EventHandler` / `RunLimit` / `RunSummary`.
- Closure-in-queue scheduling is not part of core; foreign packages adapt this
  seam instead of forking the loop.

## Consequences

- FFI and future language packages can drive the same determinism contract.
- Demo scenarios schedule payloads; they do not justify embedding callables in core.
