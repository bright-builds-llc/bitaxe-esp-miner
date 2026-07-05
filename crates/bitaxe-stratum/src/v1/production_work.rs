//! Production BM1366 work registry for pool-derived Stratum v1 work.

use std::collections::HashMap;
use std::fmt;

use bitaxe_asic::bm1366::{
    production::ProductionWorkPayload,
    result::Bm1366ValidJobIds,
    work::Bm1366JobId,
};

use crate::error::StratumV1Error;
use crate::v1::messages::PoolDifficulty;
use crate::v1::mining::MiningWork;
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
pub struct ProductionTargetContext {
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
pub struct ProductionWorkRecord {
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
    fn from_work(
        generation: PoolSessionGeneration,
        work: MiningWork,
        dispatched: bool,
    ) -> Self {
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
pub struct ProductionDispatch {
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

#[derive(Clone, PartialEq)]
pub struct ProductionWorkRegistry {
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
        Self {
            generation: PoolSessionGeneration::initial(),
            queue: BoundedWorkQueue::new(),
            valid_jobs: Bm1366ValidJobIds::empty(),
            active_work: HashMap::new(),
        }
    }

    #[must_use]
    pub const fn generation(&self) -> PoolSessionGeneration {
        self.generation
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
        self.active_work.insert(work.asic_job_id.lookup_key(), record);

        Ok(ProductionDispatch {
            generation,
            work_payload,
            work,
        })
    }

    #[must_use]
    pub fn maybe_active_work(&self, job_id: Bm1366JobId) -> Option<&ProductionWorkRecord> {
        let maybe_record = self.active_work.get(&job_id.lookup_key());
        let record = maybe_record?;
        if record.generation != self.generation {
            return None;
        }

        Some(record)
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

impl Default for ProductionWorkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_asic::bm1366::work::Bm1366JobId;

    use super::*;
    use crate::error::StratumV1Error;
    use crate::v1::messages::{ExtranonceAssignment, MiningNotify, PoolDifficulty};
    use crate::v1::mining::{MiningWork, MiningWorkBuilder};

    #[test]
    fn production_work_enqueue_registers_valid_job_for_current_generation() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x28);
        let work = sample_work(job_id, "pool-job-hidden", false);

        // Act
        registry
            .enqueue_pool_work(work)
            .expect("pool work should enqueue");

        // Assert
        assert_eq!(registry.generation(), PoolSessionGeneration::initial());
        assert!(registry.valid_jobs().contains(job_id));
    }

    #[test]
    fn production_work_dispatch_preserves_pool_context_and_payload() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x30);
        registry
            .enqueue_pool_work(sample_work(job_id, "pool-job-hidden", false))
            .expect("pool work should enqueue");

        // Act
        let dispatch = registry.dispatch_next().expect("work should dispatch");
        let active = registry
            .maybe_active_work(job_id)
            .expect("dispatched work should be active");

        // Assert
        assert_eq!(dispatch.generation, PoolSessionGeneration::initial());
        assert_eq!(dispatch.work_payload.job_id(), job_id);
        assert_eq!(dispatch.work.asic_job_id, job_id);
        assert_eq!(active.generation, PoolSessionGeneration::initial());
        assert_eq!(active.asic_job_id, job_id);
        assert_eq!(active.ntime, 0x6470_25b5);
        assert_eq!(active.target_context.compact_nbits, 0x1705_ae3a);
        assert_eq!(
            active.target_context.maybe_pool_difficulty,
            Some(PoolDifficulty { difficulty: 1.25 })
        );
        assert!(active.dispatched);
        assert!(!active.result_seen);
    }

    #[test]
    fn production_work_generation_advances_once_per_session_invalidation() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();

        // Act
        registry.invalidate_for_reconnect();
        let after_reconnect = registry.generation();
        registry.invalidate_for_authorization_reset();
        let after_authorization = registry.generation();
        registry.invalidate_for_session_replacement();
        let after_replacement = registry.generation();

        // Assert
        assert_eq!(PoolSessionGeneration::initial().raw(), 0);
        assert_eq!(after_reconnect.raw(), 1);
        assert_eq!(after_authorization.raw(), 2);
        assert_eq!(after_replacement.raw(), 3);
    }

    #[test]
    fn production_work_record_debug_redacts_raw_context() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x38);
        registry
            .enqueue_pool_work(sample_work(job_id, "pool-job-hidden", false))
            .expect("pool work should enqueue");
        let _dispatch = registry.dispatch_next().expect("work should dispatch");
        let record = registry
            .maybe_active_work(job_id)
            .expect("dispatched work should be active");

        // Act
        let rendered = format!("{record:?}");

        // Assert
        assert!(rendered.contains("ProductionWorkRecord"));
        assert!(rendered.contains("generation"));
        assert!(!rendered.contains("pool-job-hidden"));
        assert!(!rendered.contains("4de05269"));
        assert!(!rendered.contains("00000000"));
        assert!(!rendered.contains("1705ae3a"));
        assert!(!rendered.contains("647025b5"));
        assert!(!rendered.contains("0x38"));
    }

    #[test]
    fn production_dispatch_debug_redacts_raw_context() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        registry
            .enqueue_pool_work(sample_work(Bm1366JobId::new(0x40), "pool-job-hidden", false))
            .expect("pool work should enqueue");
        let dispatch = registry.dispatch_next().expect("work should dispatch");

        // Act
        let rendered = format!("{dispatch:?}");

        // Assert
        assert!(rendered.contains("ProductionDispatch"));
        assert!(rendered.contains("generation"));
        assert!(!rendered.contains("pool-job-hidden"));
        assert!(!rendered.contains("4de05269"));
        assert!(!rendered.contains("00000000"));
        assert!(!rendered.contains("1705ae3a"));
        assert!(!rendered.contains("647025b5"));
        assert!(!rendered.contains("0x40"));
    }

    #[test]
    fn production_work_clean_jobs_invalidates_queued_active_and_valid_jobs() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let queued_stale_job_id = Bm1366JobId::new(0x48);
        let active_stale_job_id = Bm1366JobId::new(0x50);
        let current_job_id = Bm1366JobId::new(0x58);
        registry
            .enqueue_pool_work(sample_work(queued_stale_job_id, "queued-stale-job", false))
            .expect("queued stale work should enqueue");
        registry
            .enqueue_pool_work(sample_work(active_stale_job_id, "active-stale-job", false))
            .expect("active stale work should enqueue");
        let stale_dispatch = registry.dispatch_next().expect("stale work should dispatch");
        assert_eq!(stale_dispatch.work.asic_job_id, queued_stale_job_id);
        assert!(registry.valid_jobs().contains(queued_stale_job_id));
        assert!(registry.valid_jobs().contains(active_stale_job_id));

        // Act
        registry
            .enqueue_pool_work(sample_work(current_job_id, "current-job", true))
            .expect("clean-jobs work should enqueue");

        // Assert
        assert_eq!(registry.generation().raw(), 1);
        assert!(!registry.valid_jobs().contains(queued_stale_job_id));
        assert!(!registry.valid_jobs().contains(active_stale_job_id));
        assert!(registry.valid_jobs().contains(current_job_id));
        assert!(registry.maybe_active_work(queued_stale_job_id).is_none());
        let current_dispatch = registry
            .dispatch_next()
            .expect("current work should be the only queued dispatch");
        assert_eq!(current_dispatch.work.asic_job_id, current_job_id);
    }

    #[test]
    fn production_work_reconnect_advances_generation_and_clears_work() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let clean_job_id = Bm1366JobId::new(0x60);
        let active_job_id = Bm1366JobId::new(0x68);
        registry
            .enqueue_pool_work(sample_work(clean_job_id, "clean-job", true))
            .expect("clean work should enqueue");
        assert_eq!(registry.generation().raw(), 1);
        registry
            .enqueue_pool_work(sample_work(active_job_id, "active-job", false))
            .expect("active work should enqueue");
        let _active_dispatch = registry.dispatch_next().expect("work should dispatch");
        assert!(registry.valid_jobs().contains(clean_job_id));
        assert!(registry.valid_jobs().contains(active_job_id));
        assert!(registry.maybe_active_work(clean_job_id).is_some());

        // Act
        registry.invalidate_for_reconnect();

        // Assert
        assert_eq!(registry.generation().raw(), 2);
        assert!(!registry.valid_jobs().contains(clean_job_id));
        assert!(!registry.valid_jobs().contains(active_job_id));
        assert!(registry.maybe_active_work(clean_job_id).is_none());
        assert!(matches!(
            registry.dispatch_next(),
            Err(StratumV1Error::QueueEmpty)
        ));
    }

    #[test]
    fn production_work_records_pool_context() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x70);
        registry
            .enqueue_pool_work(sample_work(job_id, "context-job", false))
            .expect("pool work should enqueue");

        // Act
        let dispatch = registry.dispatch_next().expect("work should dispatch");
        let active = registry
            .maybe_active_work(job_id)
            .expect("dispatched work should be active");

        // Assert
        assert_eq!(dispatch.generation.raw(), 0);
        assert_eq!(active.stratum_job_id, "context-job");
        assert_eq!(active.extranonce2, "00000000");
        assert_eq!(active.ntime, 0x6470_25b5);
        assert_eq!(active.target_context.compact_nbits, 0x1705_ae3a);
        assert_eq!(
            active.target_context.maybe_pool_difficulty,
            Some(PoolDifficulty { difficulty: 1.25 })
        );
    }

    #[test]
    fn production_work_registry_debug_redacts_queued_active_context() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x78);
        registry
            .enqueue_pool_work(sample_work(job_id, "registry-hidden-job", false))
            .expect("pool work should enqueue");
        let _dispatch = registry.dispatch_next().expect("work should dispatch");

        // Act
        let rendered = format!("{registry:?}");

        // Assert
        assert!(rendered.contains("ProductionWorkRegistry"));
        assert!(!rendered.contains("registry-hidden-job"));
        assert!(!rendered.contains("4de05269"));
        assert!(!rendered.contains("00000000"));
        assert!(!rendered.contains("1705ae3a"));
        assert!(!rendered.contains("647025b5"));
        assert!(!rendered.contains("0x78"));
    }

    fn sample_work(job_id: Bm1366JobId, stratum_job_id: &str, clean_jobs: bool) -> MiningWork {
        MiningWorkBuilder::new(
            MiningNotify {
                job_id: stratum_job_id.to_owned(),
                prev_block_hash: "00".repeat(32),
                coinbase_1: "0200000001".to_owned(),
                coinbase_2: "ffffffff".to_owned(),
                merkle_branches: Vec::new(),
                version: 0x2000_0004,
                nbits: 0x1705_ae3a,
                ntime: 0x6470_25b5,
                clean_jobs,
            },
            ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            },
        )
        .with_pool_difficulty(PoolDifficulty { difficulty: 1.25 })
        .build(job_id)
        .expect("sample work should build")
    }
}
