//! Logical simulation clock (not wall time).

/// Monotonic logical time used by the scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Clock(u64);

impl Clock {
    /// Create a clock at the given logical tick.
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Current logical time.
    pub const fn ticks(self) -> u64 {
        self.0
    }

    /// Advance by `delta` ticks.
    pub fn advance(&mut self, delta: u64) {
        self.0 = self.0.saturating_add(delta);
    }

    /// Jump to an absolute time (must be >= current).
    pub fn set(&mut self, ticks: u64) {
        debug_assert!(ticks >= self.0, "clock must not move backwards");
        self.0 = ticks;
    }
}

impl From<u64> for Clock {
    fn from(ticks: u64) -> Self {
        Self(ticks)
    }
}

impl std::fmt::Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t={}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_monotonically() {
        let mut c = Clock::new(0);
        c.advance(10);
        assert_eq!(c.ticks(), 10);
        c.set(20);
        assert_eq!(c.ticks(), 20);
    }
}
