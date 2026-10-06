//! openchaos-core — shared deterministic discrete-event simulation.
//!
//! # Depth
//!
//! This crate is the single deep sim module for openchaos. Language packages
//! (`lang/*`) are thin adapters over this seam; they must not own a second
//! clock, scheduler, world loop, or meter implementation.
//!
//! # Modules
//!
//! - [`sim`] — seed, RNG, logical clock, payload scheduler, [`SimWorld`]
//! - [`instrument`] — CodSpeed-inspired logical meters and [`bench_sim`]
//!
//! Inspiration: [Hegel](https://hegel.dev/) (core/client split) and
//! [CodSpeed](https://github.com/CodSpeedHQ/codspeed) (simulation-mode meters).
//! This crate has **no** Hegel dependency.

#![forbid(unsafe_code)]

pub mod instrument;
pub mod sim;

pub use instrument::{bench_sim, Meter, RegionStats, SimReport, Span};
pub use sim::{
    Clock, EventHandler, EventId, RunLimit, RunSummary, Seed, SimRng, SimWorld, TimedEvent,
};
