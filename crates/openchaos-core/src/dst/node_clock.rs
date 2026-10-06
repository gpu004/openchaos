//! Per-node clocks with constant skew and drift over global logical time.

use crate::sim::{Clock, SimRng};

const PPM: i128 = 1_000_000;

/// Largest drift magnitude accepted, so every local clock still moves forward.
pub const MAX_DRIFT_PPM: u32 = 999_999;

/// Bounds for per-node clock faults. Zero means a perfect clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClockConfig {
    /// Each node's offset is drawn uniformly from `-max_skew..=max_skew` ticks.
    pub max_skew: u64,
    /// Each node's rate error is drawn uniformly from `-max_drift_ppm..=max_drift_ppm`
    /// parts per million, capped at [`MAX_DRIFT_PPM`].
    pub max_drift_ppm: u32,
}

/// `local = max(0, floor(global * (1e6 + drift_ppm) / 1e6) + skew)`, monotonic in `global`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeClock {
    skew: i128,
    rate_ppm: i128,
}

impl NodeClock {
    pub(crate) fn draw(config: ClockConfig, rng: &mut SimRng) -> Self {
        let skew = draw_signed(rng, config.max_skew);
        let drift = draw_signed(rng, u64::from(config.max_drift_ppm.min(MAX_DRIFT_PPM)));
        Self {
            skew,
            rate_ppm: PPM + drift,
        }
    }

    /// Local reading at global time `global`.
    #[must_use]
    pub fn local(self, global: Clock) -> Clock {
        let scaled = (i128::from(global.ticks()) * self.rate_ppm).div_euclid(PPM);
        saturate(scaled + self.skew)
    }

    /// Earliest global time at or after `now` whose local reading is at least `target`.
    pub(crate) fn global_at(self, target: Clock, now: Clock) -> Clock {
        let needed = i128::from(target.ticks()) - self.skew;
        let global = if needed <= 0 {
            0
        } else {
            (needed * PPM + self.rate_ppm - 1).div_euclid(self.rate_ppm)
        };
        saturate(global).max(now)
    }
}

fn draw_signed(rng: &mut SimRng, max: u64) -> i128 {
    let span = max.saturating_mul(2).saturating_add(1);
    i128::from(rng.gen_range(span)) - i128::from(max)
}

fn saturate(ticks: i128) -> Clock {
    Clock::new(u64::try_from(ticks.max(0)).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(skew: i128, drift: i128) -> NodeClock {
        NodeClock {
            skew,
            rate_ppm: PPM + drift,
        }
    }

    #[test]
    fn perfect_clock_is_identity() {
        let c = clock(0, 0);
        assert_eq!(c.local(Clock::new(42)), Clock::new(42));
        assert_eq!(c.global_at(Clock::new(42), Clock::new(0)), Clock::new(42));
    }

    #[test]
    fn global_at_is_the_earliest_matching_time() {
        for (skew, drift) in [(-50, -400_000), (30, 250_000), (0, -999_999), (7, 3)] {
            let c = clock(skew, drift);
            for target in 0..300 {
                let g = c.global_at(Clock::new(target), Clock::new(0));
                assert!(c.local(g).ticks() >= target);
                if g.ticks() > 0 {
                    assert!(c.local(Clock::new(g.ticks() - 1)).ticks() < target);
                }
            }
        }
    }

    #[test]
    fn negative_skew_clamps_at_zero() {
        let c = clock(-10, 0);
        assert_eq!(c.local(Clock::new(3)), Clock::new(0));
        assert_eq!(c.local(Clock::new(15)), Clock::new(5));
    }
}
