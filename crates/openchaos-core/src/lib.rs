#![forbid(unsafe_code)]

mod sim;

pub use sim::{
    Clock, EventHandler, EventId, InPast, RunSummary, Seed, SimRng, SimWorld, TimedEvent,
};
