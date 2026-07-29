//! Production BM1366 work registry for pool-derived Stratum v1 work.

use std::collections::HashMap;
use std::fmt;

use bitaxe_asic::bm1366::{
    production::{ProductionAsicBlocker, ProductionWorkPayload},
    result::{Bm1366NonceResult, Bm1366ValidJobIds},
    work::Bm1366JobId,
};

use crate::error::StratumV1Error;
use crate::v1::messages::PoolDifficulty;
use crate::v1::mining::{MiningWork, ShareSubmission};
use crate::v1::queue::{BoundedWorkQueue, STRATUM_WORK_QUEUE_CAPACITY};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolSessionGeneration(u64);

impl PoolSessionGeneration {
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ProductionTargetContext {
    pub compact_nbits: u32,
    pub maybe_pool_difficulty: Option<PoolDifficulty>,
}

impl fmt::Debug for ProductionTargetContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionTargetContext")
            .field("redaction", &"target_context_redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct ProductionWorkRecord {
    pub generation: PoolSessionGeneration,
    pub stratum_job_id: String,
    pub asic_job_id: Bm1366JobId,
    pub extranonce2: String,
    pub ntime: u32,
    pub target_context: ProductionTargetContext,
    pub work: MiningWork,
    pub dispatched: bool,
    pub result_seen: bool,
}

impl ProductionWorkRecord {
    fn from_work(generation: PoolSessionGeneration, work: MiningWork, dispatched: bool) -> Self {
        Self {
            generation,
            stratum_job_id: work.stratum_job_id.clone(),
            asic_job_id: work.asic_job_id,
            extranonce2: work.extranonce2.clone(),
            ntime: work.ntime,
            target_context: ProductionTargetContext {
                compact_nbits: u32::from_le_bytes(work.fields.nbits),
                maybe_pool_difficulty: work.maybe_pool_difficulty,
            },
            work,
            dispatched,
            result_seen: false,
        }
    }
}

impl fmt::Debug for ProductionWorkRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionWorkRecord")
            .field("generation", &self.generation)
            .field("job", &"redacted")
            .field("target_context", &"redacted")
            .field("work_payload", &"redacted")
            .field("dispatched", &self.dispatched)
            .field("result_seen", &self.result_seen)
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct ProductionDispatch {
    pub generation: PoolSessionGeneration,
    pub work_payload: ProductionWorkPayload,
    pub work: MiningWork,
}

impl fmt::Debug for ProductionDispatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionDispatch")
            .field("generation", &self.generation)
            .field("job", &"redacted")
            .field("target_context", &"redacted")
            .field("work_payload", &"redacted")
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProductionNonceObservation {
    pub observed_generation: PoolSessionGeneration,
    pub result: Bm1366NonceResult,
}

impl fmt::Debug for ProductionNonceObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionNonceObservation")
            .field("observed_generation", &self.observed_generation)
            .field("nonce_result", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SubmitIntent {
    pub generation: PoolSessionGeneration,
    pub asic_job_id: Bm1366JobId,
    submission: ShareSubmission,
}

impl SubmitIntent {
    #[must_use]
    pub const fn submission(&self) -> &ShareSubmission {
        &self.submission
    }
}

impl fmt::Debug for SubmitIntent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SubmitIntent")
            .field("generation", &self.generation)
            .field("asic_job", &"redacted")
            .field("submit_context", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum CorrelationOutcome {
    SubmitIntent(SubmitIntent),
    Blocked { reason: ProductionAsicBlocker },
}

impl fmt::Debug for CorrelationOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubmitIntent(intent) => formatter
                .debug_tuple("CorrelationOutcome::SubmitIntent")
                .field(intent)
                .finish(),
            Self::Blocked { reason } => formatter
                .debug_struct("CorrelationOutcome::Blocked")
                .field("reason", &reason.as_str())
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct ProductionWorkRegistry {
    generation: PoolSessionGeneration,
    queue: BoundedWorkQueue<MiningWork, STRATUM_WORK_QUEUE_CAPACITY>,
    valid_jobs: Bm1366ValidJobIds,
    active_work: HashMap<Bm1366JobId, ProductionWorkRecord>,
}

impl fmt::Debug for ProductionWorkRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionWorkRegistry")
            .field("generation", &self.generation)
            .field("queued_work", &"redacted")
            .field("active_work", &"redacted")
            .field("valid_jobs", &"redacted")
            .finish()
    }
}

impl ProductionWorkRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_generation(PoolSessionGeneration::initial())
    }

    pub(crate) fn new_with_generation(generation: PoolSessionGeneration) -> Self {
        Self {
            generation,
            queue: BoundedWorkQueue::new(),
            valid_jobs: Bm1366ValidJobIds::empty(),
            active_work: HashMap::new(),
        }
    }

    #[must_use]
    pub const fn generation(&self) -> PoolSessionGeneration {
        self.generation
    }

    pub(crate) fn rebase_generation(&mut self, generation: PoolSessionGeneration) {
        self.generation = generation;
    }

    pub fn enqueue_pool_work(&mut self, work: MiningWork) -> Result<(), StratumV1Error> {
        let asic_job_id = work.asic_job_id;
        if work.clean_jobs {
            self.invalidate_for_clean_jobs();
        }

        self.queue.enqueue(work)?;
        self.valid_jobs.insert(asic_job_id);
        Ok(())
    }

    pub fn dispatch_next(&mut self) -> Result<ProductionDispatch, StratumV1Error> {
        let work = self.queue.dequeue()?;
        let generation = self.generation;
        let work_payload = ProductionWorkPayload::new(work.asic_job_id, work.fields);
        let record = ProductionWorkRecord::from_work(generation, work.clone(), true);
        self.active_work
            .insert(work.asic_job_id.lookup_key(), record);

        Ok(ProductionDispatch {
            generation,
            work_payload,
            work,
        })
    }

    #[must_use]
    #[cfg(test)]
    pub fn maybe_active_work(&self, job_id: Bm1366JobId) -> Option<&ProductionWorkRecord> {
        let maybe_record = self.active_work.get(&job_id.lookup_key());
        let record = maybe_record?;
        if record.generation != self.generation {
            return None;
        }

        Some(record)
    }

    #[must_use]
    pub fn correlate_nonce_result(
        &mut self,
        observation: ProductionNonceObservation,
    ) -> CorrelationOutcome {
        if observation.observed_generation != self.generation {
            return CorrelationOutcome::Blocked {
                reason: ProductionAsicBlocker::WrongSession,
            };
        }

        let maybe_record = self
            .active_work
            .get_mut(&observation.result.job_id.lookup_key());
        let Some(record) = maybe_record else {
            return CorrelationOutcome::Blocked {
                reason: ProductionAsicBlocker::JobUncorrelated,
            };
        };

        if record.generation != self.generation {
            return CorrelationOutcome::Blocked {
                reason: ProductionAsicBlocker::WorkStale,
            };
        }

        if record.result_seen {
            return CorrelationOutcome::Blocked {
                reason: ProductionAsicBlocker::DuplicateResult,
            };
        }

        if !stored_work_context_matches_nonce_result(record, observation.result) {
            return CorrelationOutcome::Blocked {
                reason: ProductionAsicBlocker::TargetMismatch,
            };
        }

        let Ok(submission) = ShareSubmission::from_nonce_result(&record.work, observation.result)
        else {
            return CorrelationOutcome::Blocked {
                reason: ProductionAsicBlocker::TargetMismatch,
            };
        };

        record.result_seen = true;
        CorrelationOutcome::SubmitIntent(SubmitIntent {
            generation: self.generation,
            asic_job_id: record.asic_job_id,
            submission,
        })
    }

    #[must_use]
    pub const fn valid_jobs(&self) -> &Bm1366ValidJobIds {
        &self.valid_jobs
    }

    pub fn invalidate_for_clean_jobs(&mut self) {
        self.advance_generation_and_clear_work();
    }

    pub fn invalidate_for_reconnect(&mut self) {
        self.advance_generation_and_clear_work();
    }

    pub fn invalidate_for_authorization_reset(&mut self) {
        self.advance_generation_and_clear_work();
    }

    pub fn invalidate_for_session_replacement(&mut self) {
        self.advance_generation_and_clear_work();
    }

    fn advance_generation_and_clear_work(&mut self) {
        self.generation = self.generation.next();
        self.queue.clear();
        self.valid_jobs = Bm1366ValidJobIds::empty();
        self.active_work.clear();
    }
}

fn stored_work_context_matches_nonce_result(
    record: &ProductionWorkRecord,
    result: Bm1366NonceResult,
) -> bool {
    // This guards stored work-context drift before submit-intent creation. It is
    // deliberately not a nonce-vs-target proof or share-hash validation.
    let work_compact_nbits = u32::from_le_bytes(record.work.fields.nbits);
    record.target_context.compact_nbits == work_compact_nbits
        && result.job_id.lookup_key() == record.asic_job_id.lookup_key()
}

#[cfg(test)]
impl ProductionWorkRegistry {
    fn force_active_record_generation_for_test(
        &mut self,
        job_id: Bm1366JobId,
        generation: PoolSessionGeneration,
    ) {
        if let Some(record) = self.active_work.get_mut(&job_id.lookup_key()) {
            record.generation = generation;
        }
    }

    fn force_active_compact_nbits_for_test(&mut self, job_id: Bm1366JobId, compact_nbits: u32) {
        if let Some(record) = self.active_work.get_mut(&job_id.lookup_key()) {
            record.target_context.compact_nbits = compact_nbits;
        }
    }
}

impl Default for ProductionWorkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
