//! Closed owner snapshots for receive/correlation diagnosis, never work authority.
use bitaxe_stratum::v1::production_session::ProductionSessionSnapshot;
use std::sync::atomic::{AtomicU32, Ordering};
const COUNT: usize = 21;
const WORDS: usize = 1 + 2 * (COUNT + 1);
struct Cache {
    sequence: AtomicU32,
    words: [AtomicU32; WORDS],
}
impl Cache {
    const fn new() -> Self {
        Self {
            sequence: AtomicU32::new(0),
            words: [const { AtomicU32::new(0) }; WORDS],
        }
    }
    fn publish(&self, generation: u32, observed: u64, counts: [u64; COUNT]) {
        if generation == 0 {
            return;
        }
        let current = self.sequence.load(Ordering::SeqCst);
        let Some(next) = current.checked_add(2) else {
            return;
        };
        if current % 2 != 0
            || self
                .sequence
                .compare_exchange(current, next - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
        {
            return;
        }
        self.words[0].store(generation, Ordering::SeqCst);
        for (index, value) in std::iter::once(observed).chain(counts).enumerate() {
            self.words[1 + index * 2].store(value as u32, Ordering::SeqCst);
            self.words[2 + index * 2].store((value >> 32) as u32, Ordering::SeqCst);
        }
        self.sequence.store(next, Ordering::SeqCst);
    }
    fn read(&self, generation: u32) -> Option<(u64, [u64; COUNT])> {
        let before = self.sequence.load(Ordering::SeqCst);
        if before == 0 || before % 2 != 0 {
            return None;
        }
        let words = self
            .words
            .each_ref()
            .map(|word| word.load(Ordering::SeqCst));
        if before != self.sequence.load(Ordering::SeqCst)
            || generation == 0
            || words[0] != generation
        {
            return None;
        }
        let read = |index: usize| {
            u64::from(words[1 + index * 2]) | (u64::from(words[2 + index * 2]) << 32)
        };
        Some((read(0), std::array::from_fn(|index| read(index + 1))))
    }
}
static CACHE: Cache = Cache::new();
/// Called only while the sole production owner publishes a snapshot.
pub(crate) fn capture(generation: u32, snapshot: &ProductionSessionSnapshot) {
    CACHE.publish(
        generation,
        crate::runtime_uptime::millis(),
        counts(snapshot),
    );
}
fn counts(snapshot: &ProductionSessionSnapshot) -> [u64; COUNT] {
    let a = &snapshot.asic_bridge;
    let c = &snapshot.mining.counters;
    let d = &a.discards;
    let b = &a.blocked_correlations;
    [
        a.poll_request_count,
        a.idle_completion_count,
        a.nonce_completion_count,
        a.register_read_count,
        a.stale_completion_count,
        c.qualified_candidates,
        c.below_pool_target,
        c.duplicate_candidates,
        d.invalid_length,
        d.invalid_preamble,
        d.invalid_crc,
        d.job_lookup,
        d.core,
        d.address_interval,
        d.register_response,
        d.parser_invariant,
        b.wrong_session,
        b.job_lookup,
        b.work_stale,
        b.target_mismatch,
        b.other,
    ]
}
/// No freshness gate: retained terminal evidence carries its original capture time.
pub(crate) fn observation(generation: u32) -> Option<serde_json::Value> {
    let (observed, counts) = CACHE.read(generation)?;
    Some(project(generation, observed, counts))
}
fn project(generation: u32, observed: u64, c: [u64; COUNT]) -> serde_json::Value {
    let v = |index: usize| c[index].to_string();
    serde_json::json!({"schema":"worker-mining-progress-v1","generation":generation,"observed_at_ms":observed.to_string(),
 "poll_requested":v(0),"poll_idle":v(1),"poll_nonce":v(2),"poll_register":v(3),"stale_completion":v(4),"qualified_candidates":v(5),"below_pool_target":v(6),"duplicate_candidates":v(7),
 "discards":{"invalid_length":v(8),"invalid_preamble":v(9),"invalid_crc":v(10),"job_lookup":v(11),"core":v(12),"address_interval":v(13),"register_response":v(14),"parser_invariant":v(15)},
 "blocked":{"wrong_session":v(16),"job_lookup":v(17),"work_stale":v(18),"target_mismatch":v(19),"other":v(20)}})
}
#[cfg(test)]
#[path = "mining_progress/tests.rs"]
mod tests;
