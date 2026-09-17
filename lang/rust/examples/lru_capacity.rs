//! Quarantined demo: LRU capacity property via Hegel + thin openchaos bind.
//!
//! Scenario only — not part of openchaos-core depth. Uses `#[hegel::main]` and
//! draws through Hegel; the sim is not involved (pure model property), which
//! shows demos can sit on the lang package without embedding a second sim.

use hegel::generators as gs;
use hegel::TestCase;
use std::collections::{HashMap, VecDeque};

struct LruCache {
    capacity: usize,
    map: HashMap<String, i64>,
    order: VecDeque<String>,
}

impl LruCache {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn put(&mut self, key: String, value: i64) {
        if self.capacity == 0 {
            return;
        }
        if self.map.contains_key(&key) {
            self.order.retain(|k| k != &key);
        } else if self.map.len() >= self.capacity {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
        self.order.push_back(key.clone());
        self.map.insert(key, value);
    }

    fn size(&self) -> usize {
        self.map.len()
    }
}

#[hegel::main]
fn main(tc: TestCase) {
    let capacity = tc.draw(gs::integers::<u64>().min_value(0).max_value(32)) as usize;
    let mut cache = LruCache::new(capacity);
    let entries = tc.draw(gs::vecs(gs::tuples!(
        gs::text().max_size(8),
        gs::integers::<i64>().min_value(0).max_value(10_000),
    )));
    for (key, value) in entries {
        cache.put(key, value);
    }
    assert!(
        cache.size() <= capacity,
        "cache size {} exceeds capacity {}",
        cache.size(),
        capacity
    );
}
