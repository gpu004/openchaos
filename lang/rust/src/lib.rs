//! Thin Rust language package for openchaos.
//!
//! # Role
//!
//! This crate is an **adapter**, not a second sim. Simulation depth lives in
//! [`openchaos_core`]. Here we only:
//!
//! - re-export the core seam for in-process Rust callers
//! - host a thin [`bind`] adapter that maps Hegel draws onto core worlds
//!
//! Deletion test: remove this package and `openchaos-core` still stands; remove
//! core and this package has nothing useful left. Hand-rolled PBT modules are
//! intentionally absent — Hegel (`hegeltest` / lib `hegel`) is the PBT interface.

pub mod bind;

pub use openchaos_core::{
    bench_sim, Clock, EventHandler, EventId, Meter, RegionStats, RunLimit, RunSummary, Seed,
    SimReport, SimRng, SimWorld, Span, TimedEvent,
};

/// Prelude for Rust callers binding to core through this thin package.
pub mod prelude {
    pub use crate::bind::{draw_seed, draw_seed_max, draw_world, draw_world_max};
    pub use openchaos_core::{
        bench_sim, Clock, EventHandler, EventId, Meter, RegionStats, RunLimit, RunSummary, Seed,
        SimReport, SimRng, SimWorld, Span, TimedEvent,
    };
}
