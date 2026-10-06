use hegel::generators as gs;
use hegel::TestCase;
use openchaos_core::{
    ClockConfig, Latency, NetworkConfig, NodeId, Seed, SimConfig, SimWorld, MAX_DRIFT_PPM,
};

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
        let node = || gs::integers::<u32>().min_value(0).max_value(nodes - 1);
        let overrides = tc.draw(gs::vecs(gs::tuples!(node(), node())).max_size(4));
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

fn draw_latency(tc: &TestCase) -> Latency {
    let min = tc.draw(gs::integers::<u64>().min_value(0).max_value(20));
    let max = tc.draw(gs::integers::<u64>().min_value(min).max_value(min + 50));
    Latency { min, max }
}

fn draw_per_million(tc: &TestCase) -> u32 {
    tc.draw(gs::integers::<u32>().min_value(0).max_value(500_000))
}
