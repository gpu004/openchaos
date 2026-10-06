//! A whole simulated run as one value: Hegel draws it, shrinks it, and prints it on failure.

use hegel::generators::PrintableGenerator;
use hegel::TestCase;
use openchaos_core::{Clock, Cluster, Fault, Node, NodeId, Seed, SimConfig};

use crate::bind::{draw_faults, draw_seed, draw_sim_config, draw_workload};

/// A client operation `op` sent to node `to` at global time `at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request<T> {
    /// Global time.
    pub at: Clock,
    /// Receiving node.
    pub to: NodeId,
    /// The operation, converted into the node's message type on injection.
    pub op: T,
}

/// Everything that determines a run. [`Scenario::replay`] of an equal value gives an equal trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario<T> {
    /// World seed.
    pub seed: Seed,
    /// Network and clock fault parameters.
    pub config: SimConfig,
    /// Faults at global times.
    pub faults: Vec<(Clock, Fault)>,
    /// Client requests.
    pub workload: Vec<Request<T>>,
    /// Upper bound on handled events.
    pub max_steps: u64,
}

impl<T> Scenario<T> {
    /// Draws a scenario for `nodes` nodes whose client operations come from `ops`.
    pub fn draw(
        tc: &TestCase,
        nodes: u32,
        ops: impl PrintableGenerator<T>,
        max_steps: u64,
    ) -> Self {
        Self {
            seed: draw_seed(tc),
            config: draw_sim_config(tc, nodes),
            faults: draw_faults(tc, nodes),
            workload: draw_workload(tc, nodes, ops),
            max_steps,
        }
    }
}

impl<T: Clone> Scenario<T> {
    /// Global time after the last fault and request.
    #[must_use]
    pub fn end(&self) -> Clock {
        let faults = self.faults.iter().map(|(at, _)| *at);
        let requests = self.workload.iter().map(|r| r.at);
        faults.chain(requests).max().unwrap_or_default().after(1)
    }

    /// Runs the scenario on `nodes`, then heals the network and restarts every node at
    /// [`Scenario::end`] and steps until no events remain or `max_steps` is reached.
    pub fn replay<N>(&self, nodes: Vec<N>) -> Cluster<N>
    where
        N: Node,
        N::Msg: From<T>,
    {
        let mut cluster = Cluster::new(self.seed, self.config.clone(), nodes);
        let end = self.end();
        let settle: Vec<Fault> = core::iter::once(Fault::Heal)
            .chain(cluster.ids().map(Fault::Restart))
            .collect();
        let faults = self.faults.iter().cloned();
        for (at, fault) in faults.chain(settle.into_iter().map(|f| (end, f))) {
            let queued = cluster.schedule_fault(at, fault);
            debug_assert!(queued.is_ok(), "a fresh cluster starts at time 0");
        }
        for request in &self.workload {
            let queued = cluster.inject(request.at, request.to, request.op.clone().into());
            debug_assert!(queued.is_ok(), "a fresh cluster starts at time 0");
        }
        cluster.run(self.max_steps);
        cluster
    }

    /// Replays the scenario and checks `property` on the final cluster.
    ///
    /// # Panics
    ///
    /// With the scenario, trace hash, and trace when `property` returns an error.
    /// Hegel shrinks the scenario that reaches the panic.
    #[expect(
        clippy::panic,
        reason = "a panic is how a Hegel property reports failure"
    )]
    pub fn check<N>(&self, nodes: Vec<N>, property: impl FnOnce(&Cluster<N>) -> Result<(), String>)
    where
        N: Node,
        N::Msg: From<T>,
        T: core::fmt::Debug,
    {
        let cluster = self.replay(nodes);
        if let Err(why) = property(&cluster) {
            let trace = cluster.trace();
            panic!(
                "property failed: {why}\n{self:?}\ntrace hash {:#018x}\n{trace}",
                trace.hash()
            );
        }
    }
}
