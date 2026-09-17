//! Quarantined demo: CodSpeed-inspired sim bench over a tiny work queue.
//!
//! Scenario only — not part of openchaos-core depth. Drives core through the
//! thin `openchaos` language package.

use openchaos::{bench_sim, Clock, Seed, SimWorld};

#[derive(Debug)]
enum Work {
    Process { bytes: u64 },
    Spawn { children: u32 },
}

fn main() {
    let report = bench_sim("work-queue", |meter| {
        let mut world = SimWorld::new(Seed::new(42));
        world.schedule_at(Clock::new(0), Work::Spawn { children: 8 });

        world.run_until(10_000, |w, event| match event {
            Work::Spawn { children } => {
                for i in 0..children {
                    let delay = w.rng().gen_range(5) + 1;
                    w.schedule_in(
                        delay,
                        Work::Process {
                            bytes: 16 * u64::from(i + 1),
                        },
                    );
                }
            }
            Work::Process { bytes } => {
                w.meter_mut().record_bytes(bytes);
            }
        });

        meter.merge(world.meter());
    });

    println!("{report}");
}
