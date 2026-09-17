# openchaos domain language

Living vocabulary for the codebase. Prefer these terms in modules, ADRs, and docs.

| Term | Meaning |
| --- | --- |
| **Core** | `openchaos-core` — the single deep simulation implementation. |
| **Language package** | Thin `lang/<x>` package: Hegel client for `x` + bindings to core. |
| **lang/rust** | Crate `openchaos` — thin in-process adapter over `openchaos-core` (not a second sim). |
| **bind** | `openchaos::bind` — maps Hegel `TestCase` draws onto core `Seed` / `SimWorld`. |
| **hegel / hegeltest** | Official Hegel Rust client (Cargo package `hegeltest`, crate name `hegel`). |
| **SimWorld** | Deterministic discrete-event world: seed, clock, payload scheduler, meters, trace. |
| **Seed** | `u64` that fully determines a sim (and, via Hegel, a property example). |
| **SimRng** | Core-owned PRNG stream derived from a seed (xoshiro128**-style). |
| **Clock** | Logical (not wall) time used by the scheduler. |
| **Scheduler** | Priority queue of timed **opaque payloads** (no host closures). |
| **EventHandler / RunLimit** | External handler + drive limits; handlers never live on the queue. |
| **TimedEvent** | `(at, id, payload)` delivered by `SimWorld::step`. |
| **run_until / run_with** | Drive seam: payloads leave the queue; handlers may reschedule. |
| **Meter / Span / bench_sim** | CodSpeed-inspired logical instruments owned by core. |
| **Seam** | Stable interface at a package or module boundary (core ↔ lang). |
| **Adapter** | Thin code in a language package that calls across a seam. |
| **Depth** | Non-shallow knowledge concentrated in one module (sim lives only on core). |
| **Leverage** | Reuse gained by deepening one module instead of duplicating across langs. |
| **Locality** | Putting related decisions together (entropy only in core seed/RNG). |

## Rules

1. Language packages never reimplement clock, scheduler, world, seed stream, or meters.
2. Core never depends on Hegel or on `lang/*`.
3. Demos/examples call the lang adapter + core; they are not core depth.
4. Do not re-export demo entrypoints from `lang/*/src/lib.rs`.
