//! Seeded, insertion-ordered event scheduling without threads or wall clocks.
use std::collections::BTreeMap;

use crate::ModelError;

#[derive(Debug, Clone)]
pub struct DeterministicScheduler<T> {
    now_ms: u64,
    next_sequence: u64,
    random_state: u64,
    events: BTreeMap<(u64, u64), T>,
}

impl<T> DeterministicScheduler<T> {
    pub fn new(seed: u64) -> Self {
        Self {
            now_ms: 0,
            next_sequence: 0,
            random_state: seed,
            events: BTreeMap::new(),
        }
    }

    pub fn now_ms(&self) -> u64 {
        self.now_ms
    }

    /// SplitMix64 is fixed here so replay does not depend on a RNG crate version.
    pub fn next_random(&mut self) -> u64 {
        self.random_state = self.random_state.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.random_state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    pub fn schedule(&mut self, at_ms: u64, event: T) -> Result<u64, ModelError> {
        if at_ms < self.now_ms {
            return Err(ModelError::Invalid("event in past"));
        }
        let sequence = self.next_sequence;
        self.next_sequence = sequence
            .checked_add(1)
            .ok_or(ModelError::Invalid("event sequence overflow"))?;
        self.events.insert((at_ms, sequence), event);
        Ok(sequence)
    }

    pub fn schedule_jittered(
        &mut self,
        delay_ms: u64,
        jitter_ms: u32,
        event: T,
    ) -> Result<u64, ModelError> {
        let jitter = self.next_random() % (u64::from(jitter_ms) + 1);
        let at_ms = self
            .now_ms
            .checked_add(delay_ms)
            .and_then(|time| time.checked_add(jitter))
            .ok_or(ModelError::Invalid("event time overflow"))?;
        self.schedule(at_ms, event)
    }

    /// Drains through a deadline in timestamp then insertion order.
    pub fn advance_to(&mut self, deadline_ms: u64) -> Result<Vec<(u64, T)>, ModelError> {
        if deadline_ms < self.now_ms {
            return Err(ModelError::Invalid("clock regression"));
        }
        let mut result = Vec::new();
        while let Some((&(at_ms, sequence), _)) = self.events.first_key_value() {
            if at_ms > deadline_ms {
                break;
            }
            if let Some(event) = self.events.remove(&(at_ms, sequence)) {
                result.push((at_ms, event));
            }
        }
        self.now_ms = deadline_ms;
        Ok(result)
    }
}
