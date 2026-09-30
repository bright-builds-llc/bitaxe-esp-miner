//! Explicit heap accounting. These tickets model memory; they are not pointers.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Internal,
    Dma,
    Psram,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeapSnapshot {
    pub free_bytes: usize,
    pub largest_block: usize,
    pub reserve_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureRule {
    pub maybe_attempt: Option<u64>,
    pub maybe_bytes: Option<usize>,
    pub maybe_capability: Option<Capability>,
    pub maybe_phase: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationTicket {
    id: u64,
    boot_epoch: u64,
    pub capability: Capability,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AllocationError {
    #[error("injected allocation failure")]
    Injected,
    #[error("no contiguous capability-compatible memory")]
    Exhausted,
    #[error("invalid allocation size or alignment")]
    Invalid,
    #[error("unknown allocation ticket")]
    UnknownTicket,
}

#[derive(Debug, Clone)]
struct Region {
    free: BTreeMap<usize, usize>,
    reserve: usize,
}
impl Region {
    fn from_blocks(blocks: &[usize], reserve: usize) -> Self {
        let mut offset = 0;
        let mut free = BTreeMap::new();
        for size in blocks {
            free.insert(offset, *size);
            // One occupied byte separates explicitly fragmented blocks.
            offset += size + 1;
        }
        Self { free, reserve }
    }
    fn snapshot(&self) -> HeapSnapshot {
        HeapSnapshot {
            free_bytes: self.free.values().sum(),
            largest_block: self.free.values().copied().max().unwrap_or_default(),
            reserve_bytes: self.reserve,
        }
    }
    fn allocate(&mut self, bytes: usize, alignment: usize, use_reserve: bool) -> Option<usize> {
        let reserve = if use_reserve { 0 } else { self.reserve };
        if self.snapshot().free_bytes < bytes.saturating_add(reserve) {
            return None;
        }
        let maybe_fit = self.free.iter().find_map(|(&start, &size)| {
            let aligned = start.checked_add(alignment - 1)? & !(alignment - 1);
            let prefix = aligned - start;
            (size >= prefix.checked_add(bytes)?).then_some((start, size, aligned, prefix))
        });
        let (start, size, aligned, prefix) = maybe_fit?;
        self.free.remove(&start);
        if prefix > 0 {
            self.free.insert(start, prefix);
        }
        let suffix = size - prefix - bytes;
        if suffix > 0 {
            self.free.insert(aligned + bytes, suffix);
        }
        Some(aligned)
    }
    fn release(&mut self, mut offset: usize, mut size: usize) {
        if let Some((&start, &len)) = self.free.range(..offset).next_back() {
            if start + len == offset {
                self.free.remove(&start);
                offset = start;
                size += len;
            }
        }
        if let Some((&start, &len)) = self.free.range(offset..).next() {
            if offset + size == start {
                self.free.remove(&start);
                size += len;
            }
        }
        self.free.insert(offset, size);
    }
}

/// DMA allocations share internal memory; PSRAM is never a DMA fallback.
#[derive(Debug, Clone)]
pub struct CapabilityHeap {
    internal: Region,
    psram: Region,
    initial_internal: Region,
    initial_psram: Region,
    boot_epoch: u64,
    allocations: BTreeMap<u64, (Capability, usize, usize)>,
    attempt: u64,
    next_ticket: u64,
    pub default_internal_threshold: usize,
    pub failure_rules: Vec<FailureRule>,
}
impl CapabilityHeap {
    pub fn new(internal_blocks: &[usize], psram_bytes: usize, internal_reserve: usize) -> Self {
        Self {
            internal: Region::from_blocks(internal_blocks, internal_reserve),
            psram: Region::from_blocks(&[psram_bytes], 0),
            initial_internal: Region::from_blocks(internal_blocks, internal_reserve),
            initial_psram: Region::from_blocks(&[psram_bytes], 0),
            boot_epoch: 1,
            allocations: BTreeMap::new(),
            attempt: 0,
            next_ticket: 1,
            default_internal_threshold: 2048,
            failure_rules: Vec::new(),
        }
    }
    pub fn healthy() -> Self {
        Self::new(&[256 * 1024], 8 * 1024 * 1024, 98304)
    }
    /// Retained status001 totals, not a reconstructed physical free-list.
    pub fn observed_pressure() -> Self {
        Self::new(&[2176, 1275], 8 * 1024 * 1024, 0)
    }
    pub fn snapshot(&self, capability: Capability) -> HeapSnapshot {
        match capability {
            Capability::Psram => self.psram.snapshot(),
            _ => self.internal.snapshot(),
        }
    }
    pub fn allocate(
        &mut self,
        bytes: usize,
        alignment: usize,
        capability: Capability,
        phase: &str,
    ) -> Result<AllocationTicket, AllocationError> {
        if bytes == 0 || !alignment.is_power_of_two() {
            return Err(AllocationError::Invalid);
        }
        self.attempt = self
            .attempt
            .checked_add(1)
            .ok_or(AllocationError::Invalid)?;
        if self.failure_rules.iter().any(|rule| {
            rule.maybe_attempt
                .is_none_or(|attempt| attempt == self.attempt)
                && rule.maybe_bytes.is_none_or(|size| size == bytes)
                && rule.maybe_capability.is_none_or(|caps| caps == capability)
                && rule
                    .maybe_phase
                    .as_deref()
                    .is_none_or(|label| label == phase)
        }) {
            return Err(AllocationError::Injected);
        }
        let candidates: &[Capability] = match capability {
            Capability::Default if bytes <= self.default_internal_threshold => {
                &[Capability::Internal, Capability::Psram]
            }
            Capability::Default => &[Capability::Psram, Capability::Internal],
            Capability::Internal => &[Capability::Internal],
            Capability::Dma => &[Capability::Dma],
            Capability::Psram => &[Capability::Psram],
        };
        for selected in candidates {
            let region = if *selected == Capability::Psram {
                &mut self.psram
            } else {
                &mut self.internal
            };
            if let Some(offset) =
                region.allocate(bytes, alignment, capability != Capability::Default)
            {
                let id = self.next_ticket;
                self.next_ticket = id.checked_add(1).ok_or(AllocationError::Invalid)?;
                self.allocations.insert(id, (*selected, offset, bytes));
                return Ok(AllocationTicket {
                    id,
                    boot_epoch: self.boot_epoch,
                    capability: *selected,
                    bytes,
                });
            }
        }
        Err(AllocationError::Exhausted)
    }
    pub fn free(&mut self, ticket: AllocationTicket) -> Result<(), AllocationError> {
        if ticket.boot_epoch != self.boot_epoch {
            return Err(AllocationError::UnknownTicket);
        }
        let Some((capability, offset, bytes)) = self.allocations.remove(&ticket.id) else {
            return Err(AllocationError::UnknownTicket);
        };
        let region = if capability == Capability::Psram {
            &mut self.psram
        } else {
            &mut self.internal
        };
        region.release(offset, bytes);
        Ok(())
    }
    /// Reclaims boot-owned reservations and prevents a previous boot's token
    /// from releasing a new allocation even when its numeric ID is reused.
    pub fn reset_boot(&mut self) -> Result<(), AllocationError> {
        let next_epoch = self
            .boot_epoch
            .checked_add(1)
            .ok_or(AllocationError::Invalid)?;
        self.internal = self.initial_internal.clone();
        self.psram = self.initial_psram.clone();
        self.allocations.clear();
        self.attempt = 0;
        self.next_ticket = 1;
        self.boot_epoch = next_epoch;
        Ok(())
    }
    pub fn live_allocations(&self) -> usize {
        self.allocations.len()
    }
}
