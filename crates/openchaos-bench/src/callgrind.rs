use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

const TRIGGER_PREFIX: &str = "desc: Trigger: Client Request: ";
const LL_HIT_CYCLES: u64 = 5;
const RAM_HIT_CYCLES: u64 = 35;

/// Counters for one benchmark region.
///
/// `estimated_cycles = l1_hits + 5 * ll_hits + 35 * ram_hits`, where hits are
/// derived from callgrind's simulated I1/D1/LL caches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metrics {
    /// Instructions executed (`Ir`).
    pub instructions: u64,
    /// Instruction and data accesses served by L1.
    pub l1_hits: u64,
    /// Accesses that missed L1 and hit LL.
    pub ll_hits: u64,
    /// Accesses that missed LL.
    pub ram_hits: u64,
    /// Cycle estimate from the hit counts above.
    pub estimated_cycles: u64,
}

impl Metrics {
    fn from_events(names: &[&str], values: &[u64]) -> Self {
        let get = |name: &str| {
            names
                .iter()
                .position(|n| *n == name)
                .and_then(|i| values.get(i).copied())
                .unwrap_or(0)
        };
        let instructions = get("Ir");
        let accesses = instructions + get("Dr") + get("Dw");
        let l1_misses = get("I1mr") + get("D1mr") + get("D1mw");
        let ram_hits = get("ILmr") + get("DLmr") + get("DLmw");
        let served_by_l1 = accesses.saturating_sub(l1_misses);
        let served_by_last_level = l1_misses.saturating_sub(ram_hits);
        Self {
            instructions,
            l1_hits: served_by_l1,
            ll_hits: served_by_last_level,
            ram_hits,
            estimated_cycles: served_by_l1
                + LL_HIT_CYCLES * served_by_last_level
                + RAM_HIT_CYCLES * ram_hits,
        }
    }
}

/// Extract the benchmark regions from a callgrind profile written with
/// `--combine-dumps=yes`.
///
/// Each part dumped by [`crate::bench()`] becomes one `(name, metrics)` entry,
/// in file order. Parts with any other trigger, such as program termination,
/// are skipped.
pub(crate) fn parse_profile(text: &str) -> Result<Vec<(String, Metrics)>> {
    let mut regions = Vec::new();
    let mut name: Option<&str> = None;
    let mut events: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.starts_with("part:") {
            name = None;
        } else if let Some(trigger) = line.strip_prefix(TRIGGER_PREFIX) {
            name = Some(trigger);
        } else if let Some(list) = line.strip_prefix("events:") {
            events = list.split_whitespace().collect();
        } else if let Some(totals) = line.strip_prefix("totals:") {
            let Some(name) = name.take() else { continue };
            if events.is_empty() {
                bail!("part `{name}` has totals before an events line");
            }
            let values = totals
                .split_whitespace()
                .map(str::parse)
                .collect::<Result<Vec<u64>, _>>()
                .with_context(|| format!("bad totals line for `{name}`: {totals}"))?;
            regions.push((name.to_owned(), Metrics::from_events(&events, &values)));
        }
    }
    Ok(regions)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: &str = "\
# callgrind format
version: 1
part: 1
desc: Trigger: Client Request: a/one
events: Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw
summary: 0
totals: 6011 2 2 2 0 1 2 0 1
part: 2
desc: Trigger: Client Request: b two
events: Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw
totals: 12011 20 2 2 4
part: 3
desc: Trigger: Program termination
events: Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw
totals: 0
";

    #[test]
    fn keeps_client_request_parts_and_pads_trailing_zeros() {
        let regions = parse_profile(PROFILE).unwrap();
        assert_eq!(
            regions,
            vec![
                (
                    "a/one".to_owned(),
                    Metrics {
                        instructions: 6011,
                        l1_hits: 6012,
                        ll_hits: 0,
                        ram_hits: 3,
                        estimated_cycles: 6012 + 35 * 3,
                    }
                ),
                (
                    "b two".to_owned(),
                    Metrics {
                        instructions: 12011,
                        l1_hits: 12027,
                        ll_hits: 6,
                        ram_hits: 0,
                        estimated_cycles: 12027 + 5 * 6,
                    }
                ),
            ]
        );
    }

    #[test]
    fn rejects_malformed_totals() {
        let text = "desc: Trigger: Client Request: x\nevents: Ir\ntotals: abc\n";
        assert!(parse_profile(text).is_err());
    }
}
