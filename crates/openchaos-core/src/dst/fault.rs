//! Faults a test can inject into a [`crate::Cluster`] at a chosen time.

use super::node::NodeId;

/// A scheduled fault. Crashing a down node or restarting an up node does nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// Cuts the listed nodes off from all other nodes, replacing any active partition.
    /// `Partition(vec![n])` isolates `n`; listing half the nodes splits the cluster in two.
    Partition(Vec<NodeId>),
    /// Removes the active partition.
    Heal,
    /// Stops the node and calls [`crate::Node::on_crash`].
    Crash(NodeId),
    /// Brings a crashed node back and calls [`crate::Node::on_restart`].
    Restart(NodeId),
}
