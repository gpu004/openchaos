use std::fmt;

/// Logical simulation time in ticks. Only the scheduler advances the world's clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Clock(u64);

impl Clock {
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    pub const fn ticks(self) -> u64 {
        self.0
    }

    pub const fn after(self, delay: u64) -> Self {
        Self(self.0.saturating_add(delay))
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t={}", self.0)
    }
}
