//! Nodes, a faulty network, and skewed clocks on top of [`crate::SimWorld`].

mod cluster;
mod fault;
mod network;
mod node;
mod node_clock;
mod trace;

pub use cluster::{Cluster, SimConfig};
pub use fault::Fault;
pub use network::{Latency, MsgId, NetworkConfig};
pub use node::{Ctx, Node, NodeId};
pub use node_clock::{ClockConfig, NodeClock, MAX_DRIFT_PPM};
pub use trace::{DropReason, Trace, TraceEntry};
