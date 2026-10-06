use hegel::generators as gs;
use hegel::TestCase;
use openchaos::{draw_seed, draw_world, Clock, EventId, Seed, SimWorld};

#[derive(Debug, Clone, Copy)]
enum Op {
    At(u64),
    In(u64),
    Step,
}

fn draw_ops(tc: &TestCase) -> Vec<Op> {
    let raw = tc.draw(gs::vecs(gs::tuples!(
        gs::integers::<u8>().min_value(0).max_value(2),
        gs::integers::<u64>().min_value(0).max_value(50),
    )));
    raw.into_iter()
        .map(|(kind, t)| match kind {
            0 => Op::At(t),
            1 => Op::In(t),
            _ => Op::Step,
        })
        .collect()
}

fn run_random_model(seed: Seed, initial: &[u64], max_steps: u64) -> Vec<(Clock, EventId)> {
    let mut world = SimWorld::new(seed);
    for &delay in initial {
        world.schedule_in(delay, 3u32);
    }
    world.run(max_steps, &mut |w: &mut SimWorld<u32>, fanout: u32| {
        for _ in 0..w.rng().gen_range(u64::from(fanout)) {
            let delay = w.rng().gen_range(10);
            let child = fanout - 1;
            w.schedule_in(delay, child);
        }
    });
    world.trace().to_vec()
}

#[hegel::test]
fn same_seed_same_trace(tc: TestCase) {
    let seed = draw_seed(&tc);
    let initial = tc.draw(gs::vecs(gs::integers::<u64>().min_value(0).max_value(20)).max_size(8));
    let max_steps = tc.draw(gs::integers::<u64>().min_value(0).max_value(200));
    assert_eq!(
        run_random_model(seed, &initial, max_steps),
        run_random_model(seed, &initial, max_steps)
    );
}

#[hegel::test]
fn time_never_goes_backwards(tc: TestCase) {
    let mut world = draw_world::<()>(&tc);
    let mut last_delivered = Clock::default();
    for op in draw_ops(&tc) {
        let before = world.clock();
        match op {
            Op::At(t) => {
                let at = Clock::new(t);
                let pending = world.pending();
                let result = world.schedule_at(at, ());
                assert_eq!(result.is_err(), at < before);
                if result.is_err() {
                    assert_eq!(world.pending(), pending);
                }
            }
            Op::In(d) => {
                world.schedule_in(d, ());
            }
            Op::Step => {
                if let Some(event) = world.step() {
                    assert!(event.at >= last_delivered);
                    assert_eq!(world.clock(), event.at);
                    last_delivered = event.at;
                }
            }
        }
        assert!(world.clock() >= before);
    }
    while world.step().is_some() {}
    assert!(world.trace().windows(2).all(|w| w[0].0 <= w[1].0));
}
