use hegel::generators as gs;
use hegel::TestCase;
use openchaos::{
    draw_seed, draw_sim_config, Clock, Cluster, Ctx, Fault, MsgId, NetworkConfig, Node, NodeId,
    Seed, SimConfig, TraceEntry,
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_STEPS: u64 = 100_000;
const TIMER_ROUNDS: u64 = 3;

#[derive(Debug, Clone, Copy)]
struct Hops(u8);

#[derive(Debug, Default)]
struct Gossip {
    fanout: u8,
    deadlines: BTreeMap<u64, Clock>,
    early_timers: u64,
}

impl Gossip {
    fn new(fanout: u8) -> Self {
        Self {
            fanout,
            ..Self::default()
        }
    }

    fn send_random(ctx: &mut Ctx<'_, Hops>, hops: u8) {
        let count = u64::from(ctx.node_count());
        let index = ctx.rng().gen_range(count);
        ctx.send(NodeId::new(u32::try_from(index).unwrap_or(0)), Hops(hops));
    }

    fn arm(&mut self, ctx: &mut Ctx<'_, Hops>, tag: u64) {
        let delay = ctx.rng().gen_range(30);
        self.deadlines.insert(tag, ctx.now().after(delay));
        ctx.set_timer(delay, tag);
    }
}

impl Node for Gossip {
    type Msg = Hops;

    fn on_start(&mut self, ctx: &mut Ctx<'_, Hops>) {
        for _ in 0..self.fanout {
            Self::send_random(ctx, 3);
        }
        self.arm(ctx, 0);
    }

    fn on_message(&mut self, ctx: &mut Ctx<'_, Hops>, _from: NodeId, msg: Hops) {
        if let Some(hops) = msg.0.checked_sub(1) {
            Self::send_random(ctx, hops);
        }
    }

    fn on_timer(&mut self, ctx: &mut Ctx<'_, Hops>, tag: u64) {
        if self.deadlines.remove(&tag).is_some_and(|d| ctx.now() < d) {
            self.early_timers += 1;
        }
        Self::send_random(ctx, 1);
        if tag + 1 < TIMER_ROUNDS {
            self.arm(ctx, tag + 1);
        }
    }

    fn on_crash(&mut self) {
        self.deadlines.clear();
    }
}

fn draw_fanouts(tc: &TestCase) -> (u32, Vec<u8>) {
    let count = tc.draw(gs::integers::<u32>().min_value(1).max_value(6));
    let fanouts = (0..count)
        .map(|_| tc.draw(gs::integers::<u8>().min_value(0).max_value(3)))
        .collect();
    (count, fanouts)
}

fn gossips(fanouts: &[u8]) -> Vec<Gossip> {
    fanouts.iter().copied().map(Gossip::new).collect()
}

fn draw_faults(tc: &TestCase, nodes: u32) -> Vec<(Clock, Fault)> {
    let node = || gs::integers::<u32>().min_value(0).max_value(nodes - 1);
    let raw = tc.draw(
        gs::vecs(gs::tuples!(
            gs::integers::<u64>().min_value(0).max_value(200),
            gs::integers::<u8>().min_value(0).max_value(3),
            node(),
            gs::vecs(node()).max_size(6),
        ))
        .max_size(8),
    );
    raw.into_iter()
        .map(|(at, kind, n, side)| {
            let fault = match kind {
                0 => Fault::Partition(side.into_iter().map(NodeId::new).collect()),
                1 => Fault::Heal,
                2 => Fault::Crash(NodeId::new(n)),
                _ => Fault::Restart(NodeId::new(n)),
            };
            (Clock::new(at), fault)
        })
        .collect()
}

fn build(
    seed: Seed,
    config: SimConfig,
    nodes: Vec<Gossip>,
    faults: &[(Clock, Fault)],
) -> Cluster<Gossip> {
    let mut cluster = Cluster::new(seed, config, nodes);
    for (at, fault) in faults {
        let scheduled = cluster.schedule_fault(*at, fault.clone());
        assert!(scheduled.is_ok());
    }
    cluster
}

#[hegel::test]
fn same_seed_and_config_give_same_trace(tc: TestCase) {
    let seed = draw_seed(&tc);
    let (n, fanouts) = draw_fanouts(&tc);
    let config = draw_sim_config(&tc, n);
    let faults = draw_faults(&tc, n);
    let run = || {
        let mut cluster = build(seed, config.clone(), gossips(&fanouts), &faults);
        cluster.run(MAX_STEPS);
        cluster
    };
    let (a, b) = (run(), run());
    assert_eq!(a.trace().encode(), b.trace().encode());
    assert_eq!(a.trace().hash(), b.trace().hash());
}

#[hegel::test]
fn no_delivery_across_an_active_partition(tc: TestCase) {
    let (n, fanouts) = draw_fanouts(&tc);
    let config = draw_sim_config(&tc, n);
    let faults = draw_faults(&tc, n);
    let mut cluster = build(draw_seed(&tc), config, gossips(&fanouts), &faults);
    cluster.run(MAX_STEPS);

    let mut cut_off: Option<BTreeSet<NodeId>> = None;
    for entry in cluster.trace().entries() {
        match entry {
            TraceEntry::Fault {
                fault: Fault::Partition(side),
                ..
            } => {
                cut_off = Some(side.iter().copied().collect());
            }
            TraceEntry::Fault {
                fault: Fault::Heal, ..
            } => cut_off = None,
            TraceEntry::Deliver { from, to, .. } => {
                if let Some(side) = &cut_off {
                    assert_eq!(side.contains(from), side.contains(to));
                }
            }
            _ => {}
        }
    }
}

#[hegel::test]
fn lossless_network_delivers_every_message(tc: TestCase) {
    let (n, fanouts) = draw_fanouts(&tc);
    let mut config = draw_sim_config(&tc, n);
    config.network = NetworkConfig {
        drop_per_million: 0,
        ..config.network
    };
    let duplicates = config.network.duplicate_per_million > 0;
    let mut cluster = Cluster::new(draw_seed(&tc), config, gossips(&fanouts));
    assert!(cluster.run(MAX_STEPS).drained);

    let mut deliveries: BTreeMap<MsgId, u32> = BTreeMap::new();
    for entry in cluster.trace().entries() {
        match entry {
            TraceEntry::Send { msg, .. } => {
                deliveries.insert(*msg, 0);
            }
            TraceEntry::Deliver { msg, .. } => *deliveries.get_mut(msg).unwrap() += 1,
            TraceEntry::Drop { .. } => panic!("lossless network dropped {entry:?}"),
            _ => {}
        }
    }
    for count in deliveries.values() {
        if duplicates {
            assert!((1..=2).contains(count));
        } else {
            assert_eq!(*count, 1);
        }
    }
}

#[hegel::test]
fn node_clocks_are_monotonic_and_timers_never_fire_early(tc: TestCase) {
    let (n, fanouts) = draw_fanouts(&tc);
    let config = draw_sim_config(&tc, n);
    let faults = draw_faults(&tc, n);
    let mut cluster = build(draw_seed(&tc), config, gossips(&fanouts), &faults);
    let ids: Vec<NodeId> = cluster.ids().collect();
    let mut last: Vec<Clock> = ids.iter().map(|&id| cluster.local_clock(id)).collect();
    for _ in 0..MAX_STEPS {
        if !cluster.step() {
            break;
        }
        for (&id, prev) in ids.iter().zip(&mut last) {
            let now = cluster.local_clock(id);
            assert!(now >= *prev);
            *prev = now;
        }
    }
    for &id in &ids {
        assert_eq!(cluster.node(id).early_timers, 0);
    }
}
