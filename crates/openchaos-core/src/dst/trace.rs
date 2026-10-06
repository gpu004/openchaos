//! Ordered record of everything a [`crate::Cluster`] did, with a stable hash.

use super::fault::Fault;
use super::network::MsgId;
use super::node::NodeId;
use crate::sim::{fnv1a64, Clock};

/// Why the network dropped a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    /// Random loss from [`crate::NetworkConfig::drop_per_million`].
    Lost,
    /// Sender and receiver were on different sides of a partition at send or arrival.
    Partitioned,
    /// The receiver was crashed when the message arrived.
    NodeDown,
}

/// One trace record. Times are global.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEntry {
    /// A node sent a message.
    Send {
        /// Global time.
        at: Clock,
        /// Sender.
        from: NodeId,
        /// Receiver.
        to: NodeId,
        /// Message id.
        msg: MsgId,
    },
    /// A message reached its receiver's handler.
    Deliver {
        /// Global time.
        at: Clock,
        /// Sender.
        from: NodeId,
        /// Receiver.
        to: NodeId,
        /// Message id.
        msg: MsgId,
    },
    /// The network dropped a message, at send or at arrival.
    Drop {
        /// Global time.
        at: Clock,
        /// Message id.
        msg: MsgId,
        /// Cause.
        reason: DropReason,
    },
    /// A timer fired.
    Timer {
        /// Global time.
        at: Clock,
        /// Node whose timer fired.
        node: NodeId,
        /// Tag passed to [`crate::Ctx::set_timer`].
        tag: u64,
    },
    /// A scheduled fault took effect.
    Fault {
        /// Global time.
        at: Clock,
        /// The fault.
        fault: Fault,
    },
}

/// Same seed, config, nodes, and faults produce byte-identical [`Trace::encode`] output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trace {
    entries: Vec<TraceEntry>,
}

impl Trace {
    pub(crate) fn push(&mut self, entry: TraceEntry) {
        self.entries.push(entry);
    }

    /// Entries in the order they happened.
    #[must_use]
    pub fn entries(&self) -> &[TraceEntry] {
        &self.entries
    }

    /// Little-endian binary encoding. Message payloads are not included.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for entry in &self.entries {
            encode_entry(entry, &mut out);
        }
        out
    }

    /// FNV-1a 64 of [`Trace::encode`].
    #[must_use]
    pub fn hash(&self) -> u64 {
        fnv1a64(&self.encode())
    }
}

fn encode_entry(entry: &TraceEntry, out: &mut Vec<u8>) {
    let mut put = |word: u64| out.extend_from_slice(&word.to_le_bytes());
    match entry {
        TraceEntry::Send { at, from, to, msg } => {
            put(0);
            put(at.ticks());
            put(from.get().into());
            put(to.get().into());
            put(msg.get());
        }
        TraceEntry::Deliver { at, from, to, msg } => {
            put(1);
            put(at.ticks());
            put(from.get().into());
            put(to.get().into());
            put(msg.get());
        }
        TraceEntry::Drop { at, msg, reason } => {
            put(2);
            put(at.ticks());
            put(msg.get());
            put(match reason {
                DropReason::Lost => 0,
                DropReason::Partitioned => 1,
                DropReason::NodeDown => 2,
            });
        }
        TraceEntry::Timer { at, node, tag } => {
            put(3);
            put(at.ticks());
            put(node.get().into());
            put(*tag);
        }
        TraceEntry::Fault { at, fault } => {
            put(4);
            put(at.ticks());
            match fault {
                Fault::Partition(nodes) => {
                    put(0);
                    put(u64::try_from(nodes.len()).unwrap_or(u64::MAX));
                    for node in nodes {
                        put(node.get().into());
                    }
                }
                Fault::Heal => put(1),
                Fault::Crash(node) => {
                    put(2);
                    put(node.get().into());
                }
                Fault::Restart(node) => {
                    put(3);
                    put(node.get().into());
                }
            }
        }
    }
}
