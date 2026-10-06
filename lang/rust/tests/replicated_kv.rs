//! A 3-node replicated KV store. Node 0 leads, nodes 1 and 2 follow. The leader keeps its
//! log in memory and rebuilds it from the followers after a crash, so a write is safe once
//! one follower has it. Followers forward client writes to the leader,
//! which replicates in batches every 20 ticks. `Ack::Early` acknowledges writes before that and loses them.

use hegel::generators as gs;
use hegel::{Generator, TestCase};
use openchaos::{Cluster, Ctx, Node, NodeId, Scenario};
use std::collections::BTreeMap;

const NODES: u32 = 3;
const LEADER: NodeId = NodeId::new(0);
const FOLLOWERS: [NodeId; 2] = [NodeId::new(1), NodeId::new(2)];
const BATCH: u64 = 20;
const MAX_STEPS: u64 = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Put {
    key: u8,
    value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct WriteId {
    epoch: u64,
    index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Write {
    id: WriteId,
    put: Put,
}

#[derive(Debug, Clone)]
enum Msg {
    Put(Put),
    Recover {
        epoch: u64,
    },
    Log {
        epoch: u64,
        log_epoch: u64,
        log: Vec<Write>,
    },
    Append {
        epoch: u64,
        log: Vec<Write>,
    },
    Stored {
        epoch: u64,
        len: usize,
    },
}

impl From<Put> for Msg {
    fn from(put: Put) -> Self {
        Self::Put(put)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ack {
    Early,
    Replicated,
}

#[derive(Debug)]
struct Leader {
    ack: Ack,
    epoch: u64,
    log: Vec<Write>,
    recovering: Option<BTreeMap<NodeId, (u64, Vec<Write>)>>,
    stored: BTreeMap<NodeId, usize>,
    pending: Vec<WriteId>,
    ticking: bool,
    acked: Vec<WriteId>,
}

#[derive(Debug, Default)]
struct Follower {
    promised: u64,
    log_epoch: u64,
    log: Vec<Write>,
}

#[derive(Debug)]
enum Replica {
    Leader(Box<Leader>),
    Follower(Follower),
}

impl Leader {
    fn new(ack: Ack) -> Self {
        Self {
            ack,
            epoch: 0,
            log: Vec::new(),
            recovering: None,
            stored: BTreeMap::new(),
            pending: Vec::new(),
            ticking: false,
            acked: Vec::new(),
        }
    }

    fn put(&mut self, ctx: &mut Ctx<'_, Msg>, put: Put) {
        if self.recovering.is_some() {
            return;
        }
        let id = WriteId {
            epoch: self.epoch,
            index: self.log.len(),
        };
        self.log.push(Write { id, put });
        match self.ack {
            Ack::Early => self.acked.push(id),
            Ack::Replicated => self.pending.push(id),
        }
        self.arm(ctx);
    }

    fn broadcast(&mut self, ctx: &mut Ctx<'_, Msg>) {
        let epoch = self.epoch;
        for to in FOLLOWERS {
            if self.recovering.is_some() {
                ctx.send(to, Msg::Recover { epoch });
            } else if self.stored.get(&to).copied().unwrap_or(0) < self.log.len() {
                let log = self.log.clone();
                ctx.send(to, Msg::Append { epoch, log });
            }
        }
        self.arm(ctx);
    }

    fn arm(&mut self, ctx: &mut Ctx<'_, Msg>) {
        if !self.ticking {
            self.ticking = true;
            ctx.set_timer(BATCH, 0);
        }
    }

    fn tick(&mut self, ctx: &mut Ctx<'_, Msg>) {
        self.ticking = false;
        let behind = FOLLOWERS
            .iter()
            .any(|f| self.stored.get(f).copied().unwrap_or(0) < self.log.len());
        if self.recovering.is_some() || behind {
            self.broadcast(ctx);
        }
    }

    fn on_log(&mut self, ctx: &mut Ctx<'_, Msg>, from: NodeId, log_epoch: u64, log: Vec<Write>) {
        let Some(replies) = &mut self.recovering else {
            return;
        };
        replies.insert(from, (log_epoch, log));
        if replies.len() < FOLLOWERS.len() {
            return;
        }
        let newest = replies
            .values()
            .max_by_key(|(log_epoch, log)| (*log_epoch, log.len()));
        self.log = newest.map(|(_, log)| log.clone()).unwrap_or_default();
        self.recovering = None;
        self.broadcast(ctx);
    }

    fn on_stored(&mut self, from: NodeId, len: usize) {
        self.stored.insert(from, len);
        let (safe, waiting): (Vec<WriteId>, Vec<WriteId>) =
            self.pending.iter().partition(|id| id.index < len);
        self.pending = waiting;
        self.acked.extend(safe);
    }
}

impl Follower {
    fn on_message(&mut self, ctx: &mut Ctx<'_, Msg>, from: NodeId, msg: Msg) {
        match msg {
            Msg::Recover { epoch } if epoch >= self.promised => {
                self.promised = epoch;
                let (log_epoch, log) = (self.log_epoch, self.log.clone());
                ctx.send(
                    from,
                    Msg::Log {
                        epoch,
                        log_epoch,
                        log,
                    },
                );
            }
            Msg::Append { epoch, log } if epoch >= self.promised => {
                if (epoch, log.len()) > (self.log_epoch, self.log.len()) {
                    self.promised = epoch;
                    self.log_epoch = epoch;
                    self.log = log;
                }
                if self.log_epoch == epoch {
                    let len = self.log.len();
                    ctx.send(from, Msg::Stored { epoch, len });
                }
            }
            _ => {}
        }
    }
}

impl Node for Replica {
    type Msg = Msg;

    fn on_start(&mut self, _ctx: &mut Ctx<'_, Msg>) {}

    fn on_request(&mut self, ctx: &mut Ctx<'_, Msg>, msg: Msg) {
        match (self, msg) {
            (Self::Leader(leader), Msg::Put(put)) => leader.put(ctx, put),
            (Self::Follower(_), msg) => ctx.send(LEADER, msg),
            (Self::Leader(_), _) => {}
        }
    }

    fn on_message(&mut self, ctx: &mut Ctx<'_, Msg>, from: NodeId, msg: Msg) {
        match self {
            Self::Follower(follower) => follower.on_message(ctx, from, msg),
            Self::Leader(leader) => match msg {
                Msg::Put(put) => leader.put(ctx, put),
                Msg::Log {
                    epoch,
                    log_epoch,
                    log,
                } if epoch == leader.epoch => leader.on_log(ctx, from, log_epoch, log),
                Msg::Stored { epoch, len } if epoch == leader.epoch => leader.on_stored(from, len),
                _ => {}
            },
        }
    }

    fn on_timer(&mut self, ctx: &mut Ctx<'_, Msg>, _tag: u64) {
        if let Self::Leader(leader) = self {
            leader.tick(ctx);
        }
    }

    fn on_crash(&mut self) {
        if let Self::Leader(leader) = self {
            **leader = Leader {
                epoch: leader.epoch + 1,
                recovering: Some(BTreeMap::new()),
                acked: core::mem::take(&mut leader.acked),
                ..Leader::new(leader.ack)
            };
        }
    }

    fn on_restart(&mut self, ctx: &mut Ctx<'_, Msg>) {
        if let Self::Leader(leader) = self {
            leader.broadcast(ctx);
        }
    }
}

fn replicas(ack: Ack) -> Vec<Replica> {
    let followers = FOLLOWERS.map(|_| Replica::Follower(Follower::default()));
    core::iter::once(Replica::Leader(Box::new(Leader::new(ack))))
        .chain(followers)
        .collect()
}

fn draw_scenario(tc: &TestCase) -> Scenario<Put> {
    let put = gs::tuples!(gs::integers::<u8>().max_value(3), gs::integers::<u8>())
        .map(|(key, value)| Put { key, value });
    Scenario::draw(tc, NODES, put.print_as_debug(), MAX_STEPS)
}

fn no_acked_write_lost(cluster: &Cluster<Replica>) -> Result<(), String> {
    let mut nodes = cluster.ids().map(|id| cluster.node(id));
    let Some(Replica::Leader(leader)) = nodes.next() else {
        return Err("node 0 is not the leader".to_owned());
    };
    let logs: Vec<&[Write]> = core::iter::once(leader.log.as_slice())
        .chain(nodes.filter_map(|node| match node {
            Replica::Follower(f) => Some(f.log.as_slice()),
            Replica::Leader(_) => None,
        }))
        .collect();
    for id in &leader.acked {
        for (node, log) in logs.iter().enumerate() {
            if !log.iter().any(|w| w.id == *id) {
                return Err(format!("acknowledged write {id:?} missing on node {node}"));
            }
        }
    }
    Ok(())
}

#[hegel::test(test_cases = 1000)]
fn replicated_acks_survive_crashes_and_partitions(tc: TestCase) {
    draw_scenario(&tc).check(replicas(Ack::Replicated), no_acked_write_lost);
}

#[hegel::test(test_cases = 1000)]
#[should_panic(expected = "property failed: acknowledged write")]
fn early_acks_lose_writes(tc: TestCase) {
    draw_scenario(&tc).check(replicas(Ack::Early), no_acked_write_lost);
}

#[hegel::test]
fn replay_reproduces_the_trace_hash(tc: TestCase) {
    let scenario = draw_scenario(&tc);
    let first = scenario.replay(replicas(Ack::Replicated)).trace().hash();
    let again = scenario
        .clone()
        .replay(replicas(Ack::Replicated))
        .trace()
        .hash();
    assert_eq!(first, again);
}
