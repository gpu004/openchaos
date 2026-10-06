#![forbid(unsafe_code)]

mod bind;

pub use bind::{draw_seed, draw_world};
pub use openchaos_core::{
    Clock, EventHandler, EventId, InPast, RunSummary, Seed, SimRng, SimWorld, TimedEvent,
};
