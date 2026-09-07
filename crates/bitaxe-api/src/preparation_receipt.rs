//! Integrity checked reset-retained preparation progress, never execution authority.

pub const WORDS: usize = 18;
pub const VALID: u32 = 0x4258_5053;
const WRITING: u32 = 0x4258_5057;
const VERSION: u32 = 1;
pub type Slot = [u32; WORDS];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Started = 1,
    Completed = 2,
    Failed = 3,
}
impl Outcome {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
    fn parse(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Started),
            2 => Some(Self::Completed),
            3 => Some(Self::Failed),
            _ => None,
        }
    }
}
/// Closed adapter failure; native error text and values are never retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Failure {
    None = 0,
    Cancelled = 1,
    SafetyUnavailable = 2,
    OwnerUnavailable = 3,
    QueueFull = 4,
    ReplyTimeout = 5,
    HardwareWriteFailed = 6,
    FanTimeout = 7,
    UnsupportedProfile = 8,
    AsicFailed = 9,
    AsicPlanInvalid = 10,
    CoolingTimeout = 11,
    CoolingProofRequired = 12,
}
impl Failure {
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Cancelled => "cancelled",
            Self::SafetyUnavailable => "safety_unavailable",
            Self::OwnerUnavailable => "owner_unavailable",
            Self::QueueFull => "queue_full",
            Self::ReplyTimeout => "reply_timeout",
            Self::HardwareWriteFailed => "hardware_write_failed",
            Self::FanTimeout => "fan_timeout",
            Self::UnsupportedProfile => "unsupported_profile",
            Self::AsicFailed => "asic_failed",
            Self::AsicPlanInvalid => "asic_plan_invalid",
            Self::CoolingTimeout => "cooling_timeout",
            Self::CoolingProofRequired => "cooling_proof_required",
        }
    }
    fn parse(raw: u32) -> Option<Self> {
        match raw {
            0 => Some(Self::None),
            1 => Some(Self::Cancelled),
            2 => Some(Self::SafetyUnavailable),
            3 => Some(Self::OwnerUnavailable),
            4 => Some(Self::QueueFull),
            5 => Some(Self::ReplyTimeout),
            6 => Some(Self::HardwareWriteFailed),
            7 => Some(Self::FanTimeout),
            8 => Some(Self::UnsupportedProfile),
            9 => Some(Self::AsicFailed),
            10 => Some(Self::AsicPlanInvalid),
            11 => Some(Self::CoolingTimeout),
            12 => Some(Self::CoolingProofRequired),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub source_hash: u64,
    pub boot_ordinal: u64,
    pub generation: u32,
    pub sequence: u32,
    pub uptime_ms: u64,
    pub last_completed_step: u32,
    pub current_step: u32,
    pub outcome: Outcome,
    pub failure: Failure,
    pub maybe_heap_free: Option<u32>,
    pub maybe_heap_largest: Option<u32>,
    pub maybe_stack_free: Option<u32>,
}
impl Receipt {
    pub fn encode(self) -> Option<Slot> {
        if (self.outcome == Outcome::Failed) != (self.failure != Failure::None)
            || self.source_hash == 0
            || self.boot_ordinal == 0
            || self.generation == 0
            || self.sequence == 0
            || !(1..=9).contains(&self.current_step)
            || self.last_completed_step > 9
            || (self.outcome == Outcome::Completed && self.last_completed_step != self.current_step)
            || (self.outcome == Outcome::Started && self.last_completed_step >= self.current_step)
            || (self.outcome == Outcome::Failed && self.last_completed_step > self.current_step)
        {
            return None;
        }
        let mut words = [0; WORDS];
        words[0] = VALID;
        words[1] = VERSION;
        put_u64(&mut words, 2, self.source_hash);
        put_u64(&mut words, 4, self.boot_ordinal);
        words[6] = self.generation;
        words[7] = self.sequence;
        put_u64(&mut words, 8, self.uptime_ms);
        words[10] = self.last_completed_step;
        words[11] = self.current_step;
        words[12] = self.outcome as u32 | ((self.failure as u32) << 8);
        for (bit, value) in [
            self.maybe_heap_free,
            self.maybe_heap_largest,
            self.maybe_stack_free,
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(value) = value {
                words[13] |= 1 << bit;
                words[14 + bit] = value;
            }
        }
        words[17] = checksum(&words[1..17]);
        Some(words)
    }
}
fn put_u64(words: &mut Slot, index: usize, value: u64) {
    words[index] = value as u32;
    words[index + 1] = (value >> 32) as u32;
}
fn get_u64(words: &Slot, index: usize) -> u64 {
    u64::from(words[index]) | (u64::from(words[index + 1]) << 32)
}
fn checksum(words: &[u32]) -> u32 {
    let mut crc = !0u32;
    for word in words {
        for byte in word.to_le_bytes() {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
            }
        }
    }
    !crc
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Valid,
    Incomplete,
    Corrupt,
    Unavailable,
    WrongFirmware,
}
impl Status {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Incomplete => "incomplete",
            Self::Corrupt => "corrupt",
            Self::Unavailable => "unavailable",
            Self::WrongFirmware => "wrong_firmware",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub status: Status,
    pub interrupted: bool,
    pub maybe_receipt: Option<Receipt>,
    pub maybe_slot: Option<usize>,
}
impl Snapshot {
    pub const fn unavailable() -> Self {
        Self {
            status: Status::Unavailable,
            interrupted: false,
            maybe_receipt: None,
            maybe_slot: None,
        }
    }
    pub fn for_boot(self, boot: u64) -> Self {
        if self
            .maybe_receipt
            .is_some_and(|receipt| receipt.boot_ordinal != boot)
        {
            Self::unavailable()
        } else {
            self
        }
    }
    pub fn marker(self, previous: bool) -> String {
        let origin = if previous {
            "previous_boot"
        } else {
            "current_boot"
        };
        let prefix = format!(
            "worker_preparation_receipt schema=v1 origin={origin} status={}",
            self.status.label()
        );
        let Some(r) = self.maybe_receipt else {
            return format!("{prefix} redacted=true");
        };
        format!("{prefix} interrupted={} source_hash={:016x} boot_ordinal={} generation={} sequence={} uptime_ms={} last_completed_step={} current_step={} outcome={} failure={} heap_free={} heap_largest={} stack_free={} redacted=true",self.interrupted,r.source_hash,r.boot_ordinal,r.generation,r.sequence,r.uptime_ms,r.last_completed_step,r.current_step,r.outcome.label(),r.failure.label(),metric(r.maybe_heap_free),metric(r.maybe_heap_largest),metric(r.maybe_stack_free))
    }
}
fn metric(value: Option<u32>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
}
fn decode(words: &Slot) -> Result<Receipt, Status> {
    if words.iter().all(|word| *word == 0) {
        return Err(Status::Unavailable);
    }
    if words[0] == 0 || words[0] == WRITING {
        return Err(Status::Incomplete);
    }
    if words[0] != VALID
        || words[1] != VERSION
        || words[17] != checksum(&words[1..17])
        || words[13] & !7 != 0
    {
        return Err(Status::Corrupt);
    }
    for bit in 0..3 {
        if words[13] & (1 << bit) == 0 && words[14 + bit] != 0 {
            return Err(Status::Corrupt);
        }
    }
    let receipt = Receipt {
        source_hash: get_u64(words, 2),
        boot_ordinal: get_u64(words, 4),
        generation: words[6],
        sequence: words[7],
        uptime_ms: get_u64(words, 8),
        last_completed_step: words[10],
        current_step: words[11],
        outcome: Outcome::parse(words[12] & 0xff).ok_or(Status::Corrupt)?,
        failure: Failure::parse(words[12] >> 8).ok_or(Status::Corrupt)?,
        maybe_heap_free: (words[13] & 1 != 0).then_some(words[14]),
        maybe_heap_largest: (words[13] & 2 != 0).then_some(words[15]),
        maybe_stack_free: (words[13] & 4 != 0).then_some(words[16]),
    };
    receipt.encode().ok_or(Status::Corrupt)?;
    Ok(receipt)
}
/// Selects only committed, verified progress. Partial writes never invent a newer step.
pub fn recover(slots: &[Slot; 2], expected_source: u64) -> Snapshot {
    let decoded = [decode(&slots[0]), decode(&slots[1])];
    if let [Ok(left), Ok(right)] = decoded {
        if left.boot_ordinal == right.boot_ordinal
            && ((left.sequence == right.sequence && left != right)
                || (left.sequence < right.sequence && left.generation > right.generation)
                || (right.sequence < left.sequence && right.generation > left.generation))
        {
            return Snapshot {
                status: Status::Corrupt,
                ..Snapshot::unavailable()
            };
        }
    }
    let mut maybe_latest: Option<(usize, Receipt)> = None;
    for (index, result) in decoded.iter().enumerate() {
        if let Ok(receipt) = result {
            if maybe_latest.is_none_or(|(_, old)| {
                (receipt.boot_ordinal, receipt.sequence) > (old.boot_ordinal, old.sequence)
            }) {
                maybe_latest = Some((index, *receipt));
            }
        }
    }
    if let Some((index, receipt)) = maybe_latest {
        if receipt.source_hash != expected_source {
            return Snapshot {
                status: Status::WrongFirmware,
                maybe_slot: Some(index),
                ..Snapshot::unavailable()
            };
        }
        return Snapshot {
            status: Status::Valid,
            interrupted: matches!(
                decoded[1 - index],
                Err(Status::Incomplete | Status::Corrupt)
            ),
            maybe_receipt: Some(receipt),
            maybe_slot: Some(index),
        };
    }
    let status = if decoded.contains(&Err(Status::Corrupt)) {
        Status::Corrupt
    } else if decoded.contains(&Err(Status::Incomplete)) {
        Status::Incomplete
    } else {
        Status::Unavailable
    };
    Snapshot {
        status,
        ..Snapshot::unavailable()
    }
}
/// Scopes live diagnostics to this boot without presenting an older committed receipt as current.
pub fn recover_for_boot(slots: &[Slot; 2], expected_source: u64, boot: u64) -> Snapshot {
    let scoped = slots.map(|slot| match decode(&slot) {
        Ok(receipt) if receipt.boot_ordinal != boot => [0; WORDS],
        _ => slot,
    });
    recover(&scoped, expected_source)
}

/// Reads the validity word twice so an old commit cannot validate uncommitted new fields.
pub fn read_slot(mut load: impl FnMut(usize) -> u32) -> Slot {
    let before = load(0);
    let mut words = [0; WORDS];
    for (index, word) in words.iter_mut().enumerate().skip(1) {
        *word = load(index);
    }
    let after = load(0);
    words[0] = if before == after { before } else { WRITING };
    words
}

/// Exact commit sequence shared with the RTC adapter. Caller owns serialization and barriers.
pub fn commit_words(words: Slot, mut store: impl FnMut(usize, u32)) {
    store(0, WRITING);
    for (index, word) in words.iter().enumerate().skip(1) {
        store(index, *word);
    }
    store(0, VALID);
}
#[cfg(test)]
mod tests;
