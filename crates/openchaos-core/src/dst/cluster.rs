//! Drives a set of [`Node`]s over a [`SimWorld`] with network and clock faults.

use super::fault::Fault;
use super::network::{MsgId, Network, NetworkConfig};
use super::node::{Ctx, Effect, Node, NodeId};
use super::node_clock::{ClockConfig, NodeClock};
use super::trace::{DropReason, Trace, TraceEntry};
use crate::sim::{Clock, EventId, InPast, RunSummary, Seed, SimWorld};

/// Every fault parameter of a run, in one value a property test can generate.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SimConfig {
    /// Latency, loss, duplication, and ordering.
    pub network: NetworkConfig,
    /// Per-node skew and drift bounds.
    pub clock: ClockConfig,
}

#[derive(Debug)]
enum Event<M> {
    Deliver {
        from: NodeId,
        to: NodeId,
        msg: MsgId,
        payload: M,
    },
    Timer {
        node: NodeId,
        tag: u64,
        incarnation: u64,
    },
    Request {
        to: NodeId,
        msg: MsgId,
        payload: M,
    },
    Fault(Fault),
}

#[derive(Debug)]
struct Slot<N> {
    node: N,
    clock: NodeClock,
    up: bool,
    incarnation: u64,
}

/// A deterministic cluster: same seed, config, nodes, and faults give the same [`Trace`].
#[derive(Debug)]
pub struct Cluster<N: Node> {
    world: SimWorld<Event<N::Msg>>,
    slots: Vec<Slot<N>>,
    network: Network,
    trace: Trace,
    next_msg: u64,
}

impl<N: Node> Cluster<N> {
    /// Builds the cluster, draws each node's clock, then calls [`Node::on_start`] in id order.
    #[must_use]
    pub fn new(seed: Seed, config: SimConfig, nodes: Vec<N>) -> Self {
        let mut world = SimWorld::new(seed);
        let slots = nodes
            .into_iter()
            .map(|node| Slot {
                node,
                clock: NodeClock::draw(config.clock, world.rng()),
                up: true,
                incarnation: 0,
            })
            .collect();
        let mut cluster = Self {
            world,
            slots,
            network: Network::new(config.network),
            trace: Trace::default(),
            next_msg: 0,
        };
        for id in cluster.ids() {
            cluster.invoke(id, N::on_start);
        }
        cluster
    }

    /// Queues `fault` to take effect at global time `at`.
    ///
    /// # Errors
    ///
    /// [`InPast`] if `at` is earlier than [`Cluster::now`].
    pub fn schedule_fault(&mut self, at: Clock, fault: Fault) -> Result<EventId, InPast> {
        self.world.schedule_at(at, Event::Fault(fault))
    }

    /// Queues a client request: [`Node::on_request`] runs on `to` at global time `at`.
    /// Clients sit outside the network, so partitions and loss never touch requests,
    /// but a request to a down node is dropped.
    ///
    /// # Errors
    ///
    /// [`InPast`] if `at` is earlier than [`Cluster::now`].
    pub fn inject(&mut self, at: Clock, to: NodeId, payload: N::Msg) -> Result<EventId, InPast> {
        let msg = MsgId::new(self.next_msg);
        let id = self
            .world
            .schedule_at(at, Event::Request { to, msg, payload })?;
        self.next_msg += 1;
        Ok(id)
    }

    /// Handles the next event. Returns false when nothing is pending.
    pub fn step(&mut self) -> bool {
        let Some(event) = self.world.step() else {
            return false;
        };
        let at = event.at;
        match event.payload {
            Event::Deliver {
                from,
                to,
                msg,
                payload,
            } => self.deliver(at, from, to, msg, payload),
            Event::Timer {
                node,
                tag,
                incarnation,
            } => self.fire(at, node, tag, incarnation),
            Event::Request { to, msg, payload } => self.request(at, to, msg, payload),
            Event::Fault(fault) => self.apply(at, fault),
        }
        true
    }

    /// Steps until no events remain or `max_steps` events are handled.
    pub fn run(&mut self, max_steps: u64) -> RunSummary {
        let mut delivered = 0;
        while delivered < max_steps && self.step() {
            delivered += 1;
        }
        RunSummary {
            delivered,
            drained: self.world.pending() == 0,
        }
    }

    /// Global logical time.
    #[must_use]
    pub fn now(&self) -> Clock {
        self.world.clock()
    }

    /// `id`'s local clock reading at the current global time.
    #[must_use]
    pub fn local_clock(&self, id: NodeId) -> Clock {
        self.slots[id.index()].clock.local(self.now())
    }

    /// The state machine for `id`.
    #[must_use]
    pub fn node(&self, id: NodeId) -> &N {
        &self.slots[id.index()].node
    }

    /// False between a [`Fault::Crash`] and the matching [`Fault::Restart`].
    #[must_use]
    pub fn is_up(&self, id: NodeId) -> bool {
        self.slots[id.index()].up
    }

    /// Every node id, in order.
    pub fn ids(&self) -> impl Iterator<Item = NodeId> {
        (0u32..).map(NodeId::new).take(self.slots.len())
    }

    /// Everything that happened so far.
    #[must_use]
    pub fn trace(&self) -> &Trace {
        &self.trace
    }

    fn deliver(&mut self, at: Clock, from: NodeId, to: NodeId, msg: MsgId, payload: N::Msg) {
        let reason = if !self.is_up(to) {
            Some(DropReason::NodeDown)
        } else if self.network.separates(from, to) {
            Some(DropReason::Partitioned)
        } else {
            None
        };
        if let Some(reason) = reason {
            self.trace.push(TraceEntry::Drop { at, msg, reason });
            return;
        }
        self.trace.push(TraceEntry::Deliver { at, from, to, msg });
        self.invoke(to, |node, ctx| node.on_message(ctx, from, payload));
    }

    fn request(&mut self, at: Clock, to: NodeId, msg: MsgId, payload: N::Msg) {
        if !self.is_up(to) {
            let reason = DropReason::NodeDown;
            self.trace.push(TraceEntry::Drop { at, msg, reason });
            return;
        }
        self.trace.push(TraceEntry::Request { at, to, msg });
        self.invoke(to, |node, ctx| node.on_request(ctx, payload));
    }

    fn fire(&mut self, at: Clock, node: NodeId, tag: u64, incarnation: u64) {
        let slot = &self.slots[node.index()];
        if !slot.up || slot.incarnation != incarnation {
            return;
        }
        self.trace.push(TraceEntry::Timer { at, node, tag });
        self.invoke(node, |n, ctx| n.on_timer(ctx, tag));
    }

    fn apply(&mut self, at: Clock, fault: Fault) {
        self.trace.push(TraceEntry::Fault {
            at,
            fault: fault.clone(),
        });
        match fault {
            Fault::Partition(nodes) => self.network.partition(&nodes),
            Fault::Heal => self.network.heal(),
            Fault::Crash(id) => {
                let slot = &mut self.slots[id.index()];
                if slot.up {
                    slot.up = false;
                    slot.incarnation += 1;
                    slot.node.on_crash();
                }
            }
            Fault::Restart(id) => {
                if !self.is_up(id) {
                    self.slots[id.index()].up = true;
                    self.invoke(id, N::on_restart);
                }
            }
        }
    }

    fn invoke(&mut self, id: NodeId, handler: impl FnOnce(&mut N, &mut Ctx<'_, N::Msg>)) {
        let now = self.now();
        let nodes = u32::try_from(self.slots.len()).unwrap_or(u32::MAX);
        let slot = &mut self.slots[id.index()];
        let mut ctx = Ctx::new(id, slot.clock.local(now), nodes, self.world.rng());
        handler(&mut slot.node, &mut ctx);
        for effect in ctx.into_effects() {
            match effect {
                Effect::Send { to, msg } => self.send(id, to, msg),
                Effect::Timer { delay, tag } => self.set_timer(id, delay, tag),
            }
        }
    }

    fn send(&mut self, from: NodeId, to: NodeId, payload: N::Msg) {
        let at = self.now();
        let msg = MsgId::new(self.next_msg);
        self.next_msg += 1;
        self.trace.push(TraceEntry::Send { at, from, to, msg });
        match self.network.transit(from, to, at, self.world.rng()) {
            Err(reason) => self.trace.push(TraceEntry::Drop { at, msg, reason }),
            Ok((first, duplicate)) => {
                if let Some(again) = duplicate {
                    self.queue(
                        again,
                        Event::Deliver {
                            from,
                            to,
                            msg,
                            payload: payload.clone(),
                        },
                    );
                }
                self.queue(
                    first,
                    Event::Deliver {
                        from,
                        to,
                        msg,
                        payload,
                    },
                );
            }
        }
    }

    fn set_timer(&mut self, node: NodeId, delay: u64, tag: u64) {
        let now = self.now();
        let slot = &self.slots[node.index()];
        let target = slot.clock.local(now).after(delay);
        let at = slot.clock.global_at(target, now);
        let incarnation = slot.incarnation;
        self.queue(
            at,
            Event::Timer {
                node,
                tag,
                incarnation,
            },
        );
    }

    fn queue(&mut self, at: Clock, event: Event<N::Msg>) {
        let delay = at.ticks().saturating_sub(self.now().ticks());
        self.world.schedule_in(delay, event);
    }
}
