//! Bounded Stratum v1 work queue and clean-jobs valid-job tracking.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/work_queue.c`
//! - `reference/esp-miner/main/work_queue.h`
//! - `reference/esp-miner/main/system.h`
//! - Parity checklist rows `STR-003` and `STR-006`

use std::collections::{HashMap, VecDeque};

use bitaxe_asic::bm1366::{result::Bm1366ValidJobIds, work::Bm1366JobId};

use crate::error::StratumV1Error;
use crate::v1::mining::MiningWork;

pub const STRATUM_WORK_QUEUE_CAPACITY: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWorkQueue<T, const N: usize> {
    items: VecDeque<T>,
}

impl<T, const N: usize> BoundedWorkQueue<T, N> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::with_capacity(N),
        }
    }

    pub fn enqueue(&mut self, item: T) -> Result<(), StratumV1Error> {
        if self.items.len() == N {
            return Err(StratumV1Error::QueueFull);
        }

        self.items.push_back(item);
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<T, StratumV1Error> {
        self.items.pop_front().ok_or(StratumV1Error::QueueEmpty)
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub const fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Default for BoundedWorkQueue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningWorkQueue {
    queue: BoundedWorkQueue<MiningWork, STRATUM_WORK_QUEUE_CAPACITY>,
    valid_jobs: Bm1366ValidJobIds,
    active_work: HashMap<Bm1366JobId, MiningWork>,
}

impl MiningWorkQueue {
    pub fn new() -> Self {
        Self {
            queue: BoundedWorkQueue::new(),
            valid_jobs: Bm1366ValidJobIds::empty(),
            active_work: HashMap::new(),
        }
    }

    pub fn enqueue_work(&mut self, work: MiningWork) -> Result<(), StratumV1Error> {
        let asic_job_id = work.asic_job_id;
        if work.clean_jobs {
            self.clear_jobs();
        }

        self.queue.enqueue(work)?;
        self.valid_jobs.insert(asic_job_id);
        Ok(())
    }

    pub fn dequeue_work(&mut self) -> Result<MiningWork, StratumV1Error> {
        let work = self.queue.dequeue()?;
        self.active_work
            .insert(work.asic_job_id.lookup_key(), work.clone());
        Ok(work)
    }

    pub fn clear_jobs(&mut self) {
        self.queue.clear();
        self.valid_jobs = Bm1366ValidJobIds::empty();
        self.active_work.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub const fn capacity(&self) -> usize {
        STRATUM_WORK_QUEUE_CAPACITY
    }

    pub const fn valid_jobs(&self) -> &Bm1366ValidJobIds {
        &self.valid_jobs
    }

    pub fn maybe_active_work(&self, job_id: Bm1366JobId) -> Option<&MiningWork> {
        self.active_work.get(&job_id.lookup_key())
    }
}

impl Default for MiningWorkQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod work_queue_tests {
    use bitaxe_asic::bm1366::work::Bm1366JobId;

    use super::*;
    use crate::error::StratumV1Error;
    use crate::v1::messages::{ExtranonceAssignment, MiningNotify};
    use crate::v1::mining::{MiningWork, MiningWorkBuilder};

    #[test]
    fn work_queue_default_is_empty_and_reports_capacity() {
        // Arrange
        let queue = BoundedWorkQueue::<MiningWork, 12>::new();

        // Act
        let is_empty = queue.is_empty();

        // Assert
        assert!(is_empty);
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.capacity(), 12);
    }

    #[test]
    fn work_queue_overflow_returns_queue_full() {
        // Arrange
        let mut queue = BoundedWorkQueue::<MiningWork, STRATUM_WORK_QUEUE_CAPACITY>::new();

        // Act
        for index in 0..STRATUM_WORK_QUEUE_CAPACITY {
            queue
                .enqueue(sample_work(index as u8))
                .expect("queue should accept work up to capacity");
        }
        let overflow = queue.enqueue(sample_work(120));

        // Assert
        assert_eq!(overflow, Err(StratumV1Error::QueueFull));
    }

    #[test]
    fn work_queue_dequeues_in_fifo_order() {
        // Arrange
        let mut queue = BoundedWorkQueue::<MiningWork, STRATUM_WORK_QUEUE_CAPACITY>::new();
        queue.enqueue(sample_work(0)).expect("first work enqueues");
        queue.enqueue(sample_work(8)).expect("second work enqueues");

        // Act
        let first = queue.dequeue().expect("first work dequeues");
        let second = queue.dequeue().expect("second work dequeues");

        // Assert
        assert_eq!(first.asic_job_id.raw(), 0);
        assert_eq!(second.asic_job_id.raw(), 8);
    }

    #[test]
    fn mining_loop_clean_jobs_clears_queue_and_valid_job_tracking() {
        // Arrange
        let mut queue = MiningWorkQueue::new();
        let stale_job_id = Bm1366JobId::new(0x28);
        queue
            .enqueue_work(sample_work(stale_job_id.raw()))
            .expect("work enqueues");
        assert!(queue.valid_jobs().contains(stale_job_id));

        // Act
        queue.clear_jobs();

        // Assert
        assert!(queue.is_empty());
        assert!(!queue.valid_jobs().contains(stale_job_id));
    }

    #[test]
    fn mining_work_queue_enqueue_clean_jobs_replaces_stale_work_and_valid_jobs() {
        // Arrange
        let mut queue = MiningWorkQueue::new();
        let stale_job_id = Bm1366JobId::new(0x28);
        let clean_job_id = Bm1366JobId::new(0x30);
        queue
            .enqueue_work(sample_work(stale_job_id.raw()))
            .expect("stale work should enqueue");

        // Act
        queue
            .enqueue_work(sample_work_with_clean_jobs(clean_job_id.raw(), true))
            .expect("clean work should replace stale work");

        // Assert
        assert_eq!(queue.len(), 1);
        assert!(!queue.valid_jobs().contains(stale_job_id));
        assert!(queue.valid_jobs().contains(clean_job_id));
        let remaining = queue
            .dequeue_work()
            .expect("clean work should be the only queued item");
        assert_eq!(remaining.asic_job_id, clean_job_id);
        assert!(queue.is_empty());
    }

    fn sample_work(job_id: u8) -> MiningWork {
        sample_work_with_clean_jobs(job_id, false)
    }

    fn sample_work_with_clean_jobs(job_id: u8, clean_jobs: bool) -> MiningWork {
        MiningWorkBuilder::new(
            MiningNotify {
                job_id: format!("job-{job_id}"),
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
        .build(Bm1366JobId::new(job_id))
        .expect("sample work should build")
    }
}
