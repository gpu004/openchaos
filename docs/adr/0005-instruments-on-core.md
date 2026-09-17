# ADR 0005: Instruments deepen only on core

## Status

Accepted

## Context

Rust salvage had flat `Meter`/`Span`/`bench_sim`; the TS slice had a nested
`Instrument` region map. Two shallow interfaces blocked shared logical-work
leverage across languages.

## Decision

- Named regions, flat logical counters, and `bench_sim` live only in
  `openchaos-core` (`RegionStats` + `Meter::begin`/`end`/`with_span`).
- Language packages expose thin helpers over that seam; they do not embed meters.
- Wall-clock duration on `SimReport` remains an optional side channel, not the
  product metric.

## Consequences

- Same seed ⇒ identical logical metrics for every language binding.
- TS `Instrument` is a dead-end duplicate relative to this core module.
