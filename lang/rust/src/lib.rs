//! Thin Rust language package for openchaos.
//!
//! # Role
//!
//! This crate is an **adapter**, not a second sim. Simulation depth lives in
//! [`openchaos_core`]. Here we only:
//!
//! - re-export the core seam for in-process Rust callers
//! - (next) host the Hegel client adapter that drives core worlds
//!
//! Deletion test: remove this package and `openchaos-core` still stands; remove
//! core and this package has nothing useful left.

#![deny(missing_docs)]

pub use openchaos_core::{
    bench_sim, Clock, EventId, Meter, Seed, SimReport, SimRng, SimWorld, Span, TimedEvent,
};

/// Prelude for Rust callers binding to core through this thin package.
pub mod prelude {
    //! Common core types re-exported for ergonomics.
    pub use openchaos_core::{
        bench_sim, Clock, EventId, Meter, Seed, SimReport, SimRng, SimWorld, Span, TimedEvent,
    };
}
