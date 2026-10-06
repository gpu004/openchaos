#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    /// SplitMix64 finalizer over `self + salt`.
    pub fn derive(self, salt: u64) -> Self {
        let mut z = self.0.wrapping_add(salt).wrapping_add(0x9E3779B97F4A7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        Self(z ^ (z >> 31))
    }

    /// `derive(fnv1a64(label))`, so each subsystem gets its own stream.
    pub fn stream(self, label: &str) -> Self {
        self.derive(fnv1a64(label.as_bytes()))
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_is_deterministic() {
        let s = Seed::new(42);
        assert_eq!(s.derive(1), s.derive(1));
        assert_ne!(s.derive(1), s.derive(2));
    }

    #[test]
    fn stream_labels_are_stable() {
        let s = Seed::new(1);
        assert_eq!(s.stream("net"), s.stream("net"));
        assert_ne!(s.stream("net"), s.stream("disk"));
        assert_eq!(s.stream("net").get(), 0xf3aa_bfdb_4a01_018e);
    }
}
