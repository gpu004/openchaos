//! Message transit: per-link latency, loss, duplication, ordering, and partitions.

use super::node::NodeId;
use super::trace::DropReason;
use crate::sim::{Clock, SimRng};
use std::collections::{BTreeMap, BTreeSet};

const PER_MILLION: u64 = 1_000_000;

/// Uniform latency in `min..=max` ticks. A `max` below `min` means exactly `min`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Latency {
    /// Shortest delay.
    pub min: u64,
    /// Longest delay.
    pub max: u64,
}

impl Default for Latency {
    fn default() -> Self {
        Self { min: 1, max: 1 }
    }
}

impl Latency {
    fn draw(self, rng: &mut SimRng) -> u64 {
        let spread = self.max.saturating_sub(self.min);
        self.min
            .saturating_add(rng.gen_range(spread.saturating_add(1)))
    }
}

/// Network fault parameters. The default is a reliable FIFO network with latency 1.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkConfig {
    /// Latency of every directed link not listed in `links`.
    pub latency: Latency,
    /// Per directed link `(from, to)` latency overrides.
    pub links: BTreeMap<(NodeId, NodeId), Latency>,
    /// Chance in a million that a sent message is lost.
    pub drop_per_million: u32,
    /// Chance in a million that a delivered message arrives a second time.
    pub duplicate_per_million: u32,
    /// When false, each link delivers in send order. When true, latency alone decides.
    pub reorder: bool,
}

/// Identity of a sent message. Duplicates share the id of the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MsgId(u64);

impl MsgId {
    pub(crate) const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Raw id, in send order from 0.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub(crate) struct Network {
    config: NetworkConfig,
    last_arrival: BTreeMap<(NodeId, NodeId), Clock>,
    cut_off: Option<BTreeSet<NodeId>>,
}

impl Network {
    pub(crate) fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            last_arrival: BTreeMap::new(),
            cut_off: None,
        }
    }

    pub(crate) fn partition(&mut self, nodes: &[NodeId]) {
        self.cut_off = Some(nodes.iter().copied().collect());
    }

    pub(crate) fn heal(&mut self) {
        self.cut_off = None;
    }

    pub(crate) fn separates(&self, a: NodeId, b: NodeId) -> bool {
        self.cut_off
            .as_ref()
            .is_some_and(|side| side.contains(&a) != side.contains(&b))
    }

    /// Arrival times of the message and its duplicate, if any.
    pub(crate) fn transit(
        &mut self,
        from: NodeId,
        to: NodeId,
        now: Clock,
        rng: &mut SimRng,
    ) -> Result<(Clock, Option<Clock>), DropReason> {
        if self.separates(from, to) {
            return Err(DropReason::Partitioned);
        }
        if chance(rng, self.config.drop_per_million) {
            return Err(DropReason::Lost);
        }
        let first = self.arrival(from, to, now, rng);
        let duplicate =
            chance(rng, self.config.duplicate_per_million).then(|| self.arrival(from, to, now, rng));
        Ok((first, duplicate))
    }

    fn arrival(&mut self, from: NodeId, to: NodeId, now: Clock, rng: &mut SimRng) -> Clock {
        let link = (from, to);
        let latency = self
            .config
            .links
            .get(&link)
            .copied()
            .unwrap_or(self.config.latency);
        let at = now.after(latency.draw(rng));
        if self.config.reorder {
            return at;
        }
        let at = self.last_arrival.get(&link).map_or(at, |&last| at.max(last));
        self.last_arrival.insert(link, at);
        at
    }
}

fn chance(rng: &mut SimRng, per_million: u32) -> bool {
    per_million > 0 && rng.gen_range(PER_MILLION) < u64::from(per_million)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::Seed;

    #[test]
    fn partition_separates_only_across_sides() {
        let (a, b, c) = (NodeId::new(0), NodeId::new(1), NodeId::new(2));
        let mut net = Network::new(NetworkConfig::default());
        net.partition(&[a]);
        assert!(net.separates(a, b));
        assert!(net.separates(c, a));
        assert!(!net.separates(b, c));
        net.heal();
        assert!(!net.separates(a, b));
    }

    #[test]
    fn fifo_links_never_reorder() {
        let (a, b) = (NodeId::new(0), NodeId::new(1));
        let mut net = Network::new(NetworkConfig {
            latency: Latency { min: 0, max: 50 },
            ..NetworkConfig::default()
        });
        let mut rng = SimRng::from_seed(Seed::new(9));
        let mut last = Clock::default();
        for t in 0..100 {
            let (at, _) = net.transit(a, b, Clock::new(t), &mut rng).unwrap();
            assert!(at >= last);
            last = at;
        }
    }
}
