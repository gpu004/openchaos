use hegel::generators::{self as gs, PrintableGenerator};
use hegel::TestCase;
use openchaos_core::{
    Clock, ClockConfig, Fault, Latency, NetworkConfig, NodeId, Seed, SimConfig, SimWorld,
    MAX_DRIFT_PPM,
};

use crate::scenario::Request;

const MAX_FAULTS: usize = 8;
const MAX_REQUESTS: usize = 16;
const MAX_TIME: u64 = 200;

pub fn draw_seed(tc: &TestCase) -> Seed {
    Seed::new(tc.draw(gs::integers::<u64>()))
}

pub fn draw_world<T>(tc: &TestCase) -> SimWorld<T> {
    SimWorld::new(draw_seed(tc))
}

/// Draws every [`SimConfig`] field, with link overrides between nodes `0..nodes`.
pub fn draw_sim_config(tc: &TestCase, nodes: u32) -> SimConfig {
    let mut network = NetworkConfig {
        latency: draw_latency(tc),
        drop_per_million: draw_per_million(tc),
        duplicate_per_million: draw_per_million(tc),
        reorder: tc.draw(gs::booleans()),
        ..NetworkConfig::default()
    };
    if nodes > 0 {
        let overrides =
            tc.draw(gs::vecs(gs::tuples!(draw_node(nodes), draw_node(nodes))).max_size(4));
        for (from, to) in overrides {
            network
                .links
                .insert((NodeId::new(from), NodeId::new(to)), draw_latency(tc));
        }
    }
    let clock = ClockConfig {
        max_skew: tc.draw(gs::integers::<u64>().min_value(0).max_value(1_000)),
        max_drift_ppm: tc.draw(gs::integers::<u32>().min_value(0).max_value(MAX_DRIFT_PPM)),
    };
    SimConfig { network, clock }
}

/// Draws up to 8 faults over nodes `0..nodes` at times `0..=200`.
/// Shrinks toward fewer faults, earlier times, lower node ids, and crashes.
pub fn draw_faults(tc: &TestCase, nodes: u32) -> Vec<(Clock, Fault)> {
    if nodes == 0 {
        return Vec::new();
    }
    let raw = tc.draw(
        gs::vecs(gs::tuples!(
            draw_time(),
            gs::integers::<u8>().min_value(0).max_value(3),
            draw_node(nodes),
            gs::vecs(draw_node(nodes)).max_size(4),
        ))
        .max_size(MAX_FAULTS),
    );
    raw.into_iter()
        .map(|(at, kind, node, side)| {
            let node = NodeId::new(node);
            let fault = match kind {
                0 => Fault::Crash(node),
                1 => Fault::Restart(node),
                2 => Fault::Partition(side.into_iter().map(NodeId::new).collect()),
                _ => Fault::Heal,
            };
            (Clock::new(at), fault)
        })
        .collect()
}

/// Draws up to 16 client requests to nodes `0..nodes` at times `0..=200`, each op from `ops`.
/// Shrinks toward fewer requests, earlier times, and lower node ids.
pub fn draw_workload<T>(
    tc: &TestCase,
    nodes: u32,
    ops: impl PrintableGenerator<T>,
) -> Vec<Request<T>> {
    if nodes == 0 {
        return Vec::new();
    }
    let raw =
        tc.draw(gs::vecs(gs::tuples!(draw_time(), draw_node(nodes), ops)).max_size(MAX_REQUESTS));
    raw.into_iter()
        .map(|(at, to, op)| Request {
            at: Clock::new(at),
            to: NodeId::new(to),
            op,
        })
        .collect()
}

fn draw_time() -> gs::IntegerGenerator<u64> {
    gs::integers::<u64>().min_value(0).max_value(MAX_TIME)
}

fn draw_node(nodes: u32) -> gs::IntegerGenerator<u32> {
    gs::integers::<u32>()
        .min_value(0)
        .max_value(nodes.saturating_sub(1))
}

fn draw_latency(tc: &TestCase) -> Latency {
    let min = tc.draw(gs::integers::<u64>().min_value(0).max_value(20));
    let max = tc.draw(gs::integers::<u64>().min_value(min).max_value(min + 50));
    Latency { min, max }
}

fn draw_per_million(tc: &TestCase) -> u32 {
    tc.draw(gs::integers::<u32>().min_value(0).max_value(500_000))
}
