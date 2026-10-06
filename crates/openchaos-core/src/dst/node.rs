//! Node identity, the user state machine trait, and its handler context.

use crate::sim::{Clock, SimRng};

/// Index of a node in a [`crate::Cluster`], assigned in construction order from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);

impl NodeId {
    /// Node with the given index.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Raw index.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    pub(crate) fn index(self) -> usize {
        usize::try_from(self.0).unwrap_or(usize::MAX)
    }
}

/// A user state machine. The cluster calls these handlers outside the event queue.
pub trait Node {
    /// Message type exchanged between nodes. Cloned when the network duplicates a message.
    type Msg: Clone;

    /// Called once per node when the cluster is built.
    fn on_start(&mut self, ctx: &mut Ctx<'_, Self::Msg>);

    /// Called when a message from `from` arrives and the link is not partitioned.
    fn on_message(&mut self, ctx: &mut Ctx<'_, Self::Msg>, from: NodeId, msg: Self::Msg);

    /// Called when a timer set with [`Ctx::set_timer`] fires on the node's local clock.
    fn on_timer(&mut self, ctx: &mut Ctx<'_, Self::Msg>, tag: u64);

    /// Called when the node crashes. Reset volatile state here; anything kept is durable.
    /// Pending timers are discarded and messages addressed to a down node are dropped.
    fn on_crash(&mut self) {}

    /// Called when a crashed node restarts. Defaults to [`Node::on_start`].
    fn on_restart(&mut self, ctx: &mut Ctx<'_, Self::Msg>) {
        self.on_start(ctx);
    }
}

pub(crate) enum Effect<M> {
    Send { to: NodeId, msg: M },
    Timer { delay: u64, tag: u64 },
}

/// What a handler may do: read its local time, draw randomness, send, and set timers.
#[derive(Debug)]
pub struct Ctx<'a, M> {
    id: NodeId,
    now: Clock,
    nodes: u32,
    rng: &'a mut SimRng,
    effects: Vec<Effect<M>>,
}

impl<M> core::fmt::Debug for Effect<M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Send { to, .. } => write!(f, "Send({})", to.0),
            Self::Timer { delay, tag } => write!(f, "Timer({delay}, {tag})"),
        }
    }
}

impl<'a, M> Ctx<'a, M> {
    pub(crate) fn new(id: NodeId, now: Clock, nodes: u32, rng: &'a mut SimRng) -> Self {
        Self {
            id,
            now,
            nodes,
            rng,
            effects: Vec::new(),
        }
    }

    pub(crate) fn into_effects(self) -> Vec<Effect<M>> {
        self.effects
    }

    /// The node this handler runs on.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// The node's local clock reading, including skew and drift.
    #[must_use]
    pub fn now(&self) -> Clock {
        self.now
    }

    /// All node ids in the cluster, in order.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> {
        (0..self.nodes).map(NodeId)
    }

    /// The world's RNG.
    pub fn rng(&mut self) -> &mut SimRng {
        self.rng
    }

    /// Hands `msg` to the network after the handler returns.
    pub fn send(&mut self, to: NodeId, msg: M) {
        self.effects.push(Effect::Send { to, msg });
    }

    /// Fires [`Node::on_timer`] with `tag` once the local clock has advanced by `delay` ticks.
    pub fn set_timer(&mut self, delay: u64, tag: u64) {
        self.effects.push(Effect::Timer { delay, tag });
    }
}
