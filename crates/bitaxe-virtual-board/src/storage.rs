//! Isolated persistence and NOR partition operations; no host backing files.
use crate::ModelError;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResetReason {
    PowerOn,
    Software,
    Panic,
}
#[derive(Debug, Clone)]
pub struct PersistentStore {
    values: BTreeMap<String, Vec<u8>>,
    pub reject_writes: bool,
    pub boot_ordinal: u64,
    pub reset_reason: ResetReason,
    pub replay_high_water: u64,
    pub charged_ms: u64,
    pub maybe_pending_ordinal: Option<u64>,
    pub next_ordinal: u64,
    partitions: BTreeMap<String, Vec<u8>>,
}
impl Default for PersistentStore {
    fn default() -> Self {
        let mut values = BTreeMap::new();
        values.insert(
            "device_identity".to_owned(),
            b"SYNTHETIC-ULTRA205-NOT-PHYSICAL".to_vec(),
        );
        Self {
            values,
            reject_writes: false,
            boot_ordinal: 1,
            reset_reason: ResetReason::PowerOn,
            replay_high_water: 0,
            charged_ms: 0,
            maybe_pending_ordinal: None,
            next_ordinal: 1,
            partitions: BTreeMap::from([
                ("coredump".to_owned(), vec![0xff; 0xee000]),
                ("nvs".to_owned(), vec![0xff; 0x6000]),
            ]),
        }
    }
}
impl PersistentStore {
    pub fn read(&self, key: &str) -> Option<&[u8]> {
        self.values.get(key).map(Vec::as_slice)
    }
    /// Atomically commits the provided key set; injected failure changes nothing.
    pub fn transaction(&mut self, writes: &[(String, Vec<u8>)]) -> Result<(), ModelError> {
        if self.reject_writes {
            return Err(ModelError::Persistence);
        }
        for (key, value) in writes {
            self.values.insert(key.clone(), value.clone());
        }
        Ok(())
    }
    pub fn admit_replay(&mut self, sequence: u64) -> Result<(), ModelError> {
        if sequence <= self.replay_high_water {
            return Err(ModelError::Invalid("replayed sequence"));
        }
        if self.reject_writes {
            return Err(ModelError::Persistence);
        }
        self.replay_high_water = sequence;
        Ok(())
    }
    /// Durable charge precedes modeled effects. Failure does not refund a charge.
    pub fn reserve_charge(&mut self, duration_ms: u64) -> Result<u64, ModelError> {
        if self.maybe_pending_ordinal.is_some() {
            return Err(ModelError::Invalid("pending accounting"));
        }
        if self.reject_writes {
            return Err(ModelError::Persistence);
        }
        let ordinal = self.next_ordinal;
        let next = ordinal
            .checked_add(1)
            .ok_or(ModelError::Invalid("ordinal overflow"))?;
        let charged = self
            .charged_ms
            .checked_add(duration_ms)
            .ok_or(ModelError::Invalid("charge overflow"))?;
        self.next_ordinal = next;
        self.charged_ms = charged;
        self.maybe_pending_ordinal = Some(ordinal);
        Ok(ordinal)
    }
    pub fn complete_charge(&mut self, ordinal: u64) -> Result<(), ModelError> {
        if self.maybe_pending_ordinal != Some(ordinal) {
            return Err(ModelError::Invalid("wrong pending ordinal"));
        }
        if self.reject_writes {
            return Err(ModelError::Persistence);
        }
        self.maybe_pending_ordinal = None;
        Ok(())
    }
    pub fn reset(&mut self, reason: ResetReason) -> Result<(), ModelError> {
        self.boot_ordinal = self
            .boot_ordinal
            .checked_add(1)
            .ok_or(ModelError::Invalid("boot overflow"))?;
        self.reset_reason = reason;
        Ok(())
    }
    pub fn partition(&self, name: &str) -> Result<&[u8], ModelError> {
        self.partitions
            .get(name)
            .map(Vec::as_slice)
            .ok_or(ModelError::Unsupported("partition"))
    }
    pub fn write_partition(
        &mut self,
        name: &str,
        offset: usize,
        bytes: &[u8],
    ) -> Result<(), ModelError> {
        if self.reject_writes {
            return Err(ModelError::Persistence);
        }
        let region = self
            .partitions
            .get_mut(name)
            .ok_or(ModelError::Unsupported("partition"))?;
        let end = offset
            .checked_add(bytes.len())
            .ok_or(ModelError::Invalid("partition span"))?;
        let target = region
            .get_mut(offset..end)
            .ok_or(ModelError::Invalid("partition span"))?;
        if target
            .iter()
            .zip(bytes)
            .any(|(before, after)| before & after != *after)
        {
            return Err(ModelError::Invalid("NOR zero to one"));
        }
        target.copy_from_slice(bytes);
        Ok(())
    }
    pub fn erase_partition(&mut self, name: &str) -> Result<(), ModelError> {
        if self.reject_writes {
            return Err(ModelError::Persistence);
        }
        self.partitions
            .get_mut(name)
            .ok_or(ModelError::Unsupported("partition"))?
            .fill(0xff);
        Ok(())
    }
}
