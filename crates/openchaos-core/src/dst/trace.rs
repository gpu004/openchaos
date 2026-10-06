//! Ordered record of everything a [`crate::Cluster`] did, with a stable hash.

use super::fault::Fault;
use super::network::MsgId;
use super::node::NodeId;
use crate::sim::{fnv1a64, Clock};
use core::fmt;

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
    /// A client request reached [`crate::Node::on_request`].
    Request {
        /// Global time.
        at: Clock,
        /// Receiver.
        to: NodeId,
        /// Message id.
        msg: MsgId,
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

impl fmt::Display for DropReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lost => "lost",
            Self::Partitioned => "partitioned",
            Self::NodeDown => "node-down",
        })
    }
}

/// One line per entry, such as `t=3 send 0->1 #4`.
impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            writeln!(f, "{entry}")?;
        }
        Ok(())
    }
}

impl fmt::Display for TraceEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send { at, from, to, msg } => {
                write!(
                    f,
                    "t={} send {}->{} #{}",
                    at.ticks(),
                    from.get(),
                    to.get(),
                    msg.get()
                )
            }
            Self::Deliver { at, from, to, msg } => {
                write!(
                    f,
                    "t={} deliver {}->{} #{}",
                    at.ticks(),
                    from.get(),
                    to.get(),
                    msg.get()
                )
            }
            Self::Drop { at, msg, reason } => {
                write!(f, "t={} drop #{} {reason}", at.ticks(), msg.get())
            }
            Self::Request { at, to, msg } => {
                write!(f, "t={} request ->{} #{}", at.ticks(), to.get(), msg.get())
            }
            Self::Timer { at, node, tag } => {
                write!(f, "t={} timer {} tag={tag}", at.ticks(), node.get())
            }
            Self::Fault { at, fault } => write!(f, "t={} {fault}", at.ticks()),
        }
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
        TraceEntry::Request { at, to, msg } => {
            put(5);
            put(at.ticks());
            put(to.get().into());
            put(msg.get());
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
