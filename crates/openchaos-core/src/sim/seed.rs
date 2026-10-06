//! Authoritative simulation seed for openchaos-core.
//!
//! Entropy locality lives **here**: language packages adapt host integers
//! through this type and must not reimplement a divergent PRNG stream
//! (the old TS `xorshift32` / `number` seed is explicitly retired).

use std::fmt;

/// 64-bit seed that fully determines a simulation run.
///
/// Same [`Seed`] ⇒ same [`crate::sim::SimRng`] stream ⇒ same schedules and
/// logical meters when model code is deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    /// Construct from an explicit value (primary binding input).
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Raw seed bits (stable across languages for ABI / FFI).
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Mix two host integers into one seed (useful at language seams).
    #[must_use]
    pub fn mix(a: u64, b: u64) -> Self {
        Seed::new(a).derive(b)
    }

    /// Derive a child seed (for nested subsystems) without mutating this one.
    ///
    /// Uses a SplitMix64-style mix — deterministic and avalanche-friendly.
    #[must_use]
    pub fn derive(self, salt: u64) -> Self {
        let mut z = self
            .0
            .wrapping_add(salt)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Self(z ^ (z >> 31))
    }

    /// Domain-separated child seed from a UTF-8 label (stable across languages).
    ///
    /// The label is folded with FNV-1a 64 so bindings can request named streams
    /// (`"net"`, `"disk"`) without inventing their own mixers.
    #[must_use]
    pub fn stream(self, label: &str) -> Self {
        self.derive(fnv1a64(label.as_bytes()))
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

impl From<u64> for Seed {
    fn from(value: u64) -> Self {
        Self(value)
    }
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
    }

    #[test]
    fn net_stream_golden_is_the_cross_language_contract() {
        assert_eq!(Seed::new(1).stream("net").get(), 0xf3aa_bfdb_4a01_018e);
    }

    #[test]
    fn mix_matches_derive() {
        assert_eq!(Seed::mix(9, 3), Seed::new(9).derive(3));
    }
}
