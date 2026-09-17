//! Deterministic discrete-event simulation primitives.
//!
//! Opaque payloads live on the scheduler; handlers run outside the queue so
//! foreign language adapters can drive the same world seam without embedding
//! host closures in core.

mod clock;
mod rng;
mod scheduler;
mod seed;
mod world;

pub use clock::Clock;
pub use rng::SimRng;
pub use scheduler::{EventId, Scheduler, TimedEvent};
pub use seed::Seed;
pub use world::SimWorld;
