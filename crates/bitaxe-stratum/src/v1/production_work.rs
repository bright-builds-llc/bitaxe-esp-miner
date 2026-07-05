//! Production BM1366 work registry for pool-derived Stratum v1 work.

#[cfg(test)]
mod tests {
    use bitaxe_asic::bm1366::work::Bm1366JobId;

    use super::*;
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
