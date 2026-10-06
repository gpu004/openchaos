use super::seed::Seed;

/// xoshiro128** seeded from a [`Seed`].
#[derive(Debug, Clone)]
pub struct SimRng {
    s: [u32; 4],
}

impl SimRng {
    #[must_use]
    pub fn from_seed(seed: Seed) -> Self {
        let mut s = [0u32; 4];
        let mut z = seed.get();
        for slot in &mut s {
            z = z
                .wrapping_add(0x9E37_79B9_7F4A_7C15)
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            *slot = low_u32(z ^ (z >> 32));
        }
        if s.iter().all(|&x| x == 0) {
            s[0] = 1;
        }
        Self { s }
    }

    pub fn next_u32(&mut self) -> u32 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 9;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(11);

        result
    }

    pub fn next_u64(&mut self) -> u64 {
        let hi = u64::from(self.next_u32());
        let lo = u64::from(self.next_u32());
        (hi << 32) | lo
    }

    /// Little-endian `u32` words; a partial last word is truncated.
    pub fn fill_bytes(&mut self, buf: &mut [u8]) {
        let mut i = 0;
        while i < buf.len() {
            let word = self.next_u32().to_le_bytes();
            let take = (buf.len() - i).min(4);
            buf[i..i + take].copy_from_slice(&word[..take]);
            i += take;
        }
    }

    /// Uniform in `0..bound`. Returns 0 when `bound == 0`.
    pub fn gen_range(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        if let Ok(bound32) = u32::try_from(bound) {
            let mut product = u64::from(self.next_u32()) * u64::from(bound32);
            let mut low = low_u32(product);
            if low < bound32 {
                let threshold = bound32.wrapping_neg() % bound32;
                while low < threshold {
                    product = u64::from(self.next_u32()) * u64::from(bound32);
                    low = low_u32(product);
                }
            }
            product >> 32
        } else {
            loop {
                let v = self.next_u64();
                if v < (u64::MAX - (u64::MAX % bound)) {
                    return v % bound;
                }
            }
        }
    }

    pub fn gen_bool(&mut self) -> bool {
        self.next_u32() & 1 == 1
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "keeping only the low 32 bits is the intended mixing step"
)]
const fn low_u32(value: u64) -> u32 {
    value as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = SimRng::from_seed(Seed::new(42));
        let mut b = SimRng::from_seed(Seed::new(42));
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = SimRng::from_seed(Seed::new(1));
        let mut b = SimRng::from_seed(Seed::new(2));
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn gen_range_respects_bound() {
        let mut rng = SimRng::from_seed(Seed::new(7));
        for _ in 0..1000 {
            assert!(rng.gen_range(10) < 10);
        }
    }

    #[test]
    fn golden_first_u64_for_seed_0() {
        let mut zero = SimRng::from_seed(Seed::new(0));
        assert_eq!(zero.next_u64(), 0xda23_aa7a_8b1f_aad2);
    }

    #[test]
    fn golden_gen_range_for_seed_1() {
        let mut rng = SimRng::from_seed(Seed::new(1));
        let small: Vec<u64> = (0..4).map(|_| rng.gen_range(1000)).collect();
        assert_eq!(small, [458, 777, 883, 317]);
        assert_eq!(
            SimRng::from_seed(Seed::new(1)).gen_range(1 << 40),
            480_080_490_810
        );
    }

    #[test]
    fn golden_first_u64_for_seed_1() {
        let mut rng = SimRng::from_seed(Seed::new(1));
        assert_eq!(rng.next_u64(), 0x7577_4b6f_c706_f13a);
    }

    #[test]
    fn fill_bytes_matches_u32_le_chunks() {
        let mut rng = SimRng::from_seed(Seed::new(3));
        let mut buf = [0u8; 6];
        rng.fill_bytes(&mut buf);
        let mut rng2 = SimRng::from_seed(Seed::new(3));
        let w0 = rng2.next_u32().to_le_bytes();
        let w1 = rng2.next_u32().to_le_bytes();
        assert_eq!(&buf[..4], &w0);
        assert_eq!(&buf[4..], &w1[..2]);
    }
}
