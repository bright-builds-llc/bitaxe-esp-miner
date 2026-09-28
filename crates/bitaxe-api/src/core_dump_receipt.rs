//! Source/boot-bound numeric checkpoints for the native core-dump store.

/// Fixed RTC representation: commit, version, identity, stage, values and integrity.
pub const WORDS: usize = 20;
/// Commit marker written only after every other word.
pub const COMMIT: u32 = 0x4344_5331;
/// Fixed representation shared with the audited native writer.
pub type ReceiptWords = [u32; WORDS];

/// Closed native-store stage names, indexed by their retained numeric value.
pub const STAGES: [&str; 11] = [
    "ready",
    "store_entered",
    "init_entered",
    "init_returned",
    "prepare_entered",
    "prepare_returned",
    "start_entered",
    "start_returned",
    "end_entered",
    "end_returned",
    "store_returned",
];

/// Integrity mixing over all payload words; allocation-free and inlined into IRAM writers.
#[inline(always)]
pub fn checksum(words: &ReceiptWords) -> u32 {
    let mut checksum = 0x9e37_79b9_u32;
    let mut index = 1;
    while index < 18 {
        checksum = checksum.rotate_left(5) ^ words[index];
        index += 1;
    }
    checksum
}

/// Creates a current-boot binding before the fatal path can write diagnostics.
pub fn ready(source_hash: u64, boot_ordinal: u64, capacity_bytes: u32) -> ReceiptWords {
    let mut words = [0; WORDS];
    words[0] = COMMIT;
    words[1] = 1;
    words[2] = source_hash as u32;
    words[3] = (source_hash >> 32) as u32;
    words[4] = boot_ordinal as u32;
    words[5] = (boot_ordinal >> 32) as u32;
    words[7] = capacity_bytes;
    words[17] = 1;
    words[18] = checksum(&words);
    words[19] = !words[18];
    words
}

/// Updates one checkpoint without sentinel error codes or heap use.
#[inline(always)]
pub fn checkpoint(words: &mut ReceiptWords, stage: u32, field: u32, value: u32, result: i32) {
    words[6] = stage;
    if field == 9 {
        words[9] = value;
        words[8] |= 1;
    }
    if field == 10 {
        words[10] = value;
        words[8] |= 2;
    }
    if field == 16 {
        words[16] = value;
    }
    // The native panic path cannot read a compiler-generated DROM switch table.
    let result_index = if (3..=9).contains(&stage) && stage & 1 == 1 {
        (stage / 2 + 10) as usize
    } else if stage == 10 {
        15
    } else {
        0
    };
    if result_index != 0 {
        words[result_index] = result as u32;
        words[8] |= 1 << (result_index - 9);
    }
    words[17] = words[17].wrapping_add(1);
    words[18] = checksum(words);
    words[19] = !words[18];
}

/// Publishes a checkpoint so an interrupted write cannot look complete.
#[inline(always)]
pub fn commit_words(words: &ReceiptWords, mut write: impl FnMut(usize, u32)) {
    write(0, 0);
    let mut index = 1;
    while index < WORDS {
        write(index, words[index]);
        index += 1;
    }
    write(0, COMMIT);
}

/// Classification is independent of renderer and never exposes invalid numeric payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Valid,
    Unavailable,
    Corrupt,
    WrongFirmware,
    WrongBoot,
}
impl Status {
    /// Closed diagnostic label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Unavailable => "unavailable",
            Self::Corrupt => "corrupt",
            Self::WrongFirmware => "wrong_firmware",
            Self::WrongBoot => "wrong_boot",
        }
    }
}

/// Validates integrity before checking the expected source and exact boot identity.
pub fn classify(words: &ReceiptWords, source_hash: u64, expected_boot: u64) -> Status {
    if words.iter().all(|word| *word == 0) {
        return Status::Unavailable;
    }
    if words[0] != COMMIT
        || words[1] != 1
        || words[6] > 10
        || words[8] & !127 != 0
        || words[16] > 1
        || words[17] == 0
        || words[18] != checksum(words)
        || words[19] != !words[18]
    {
        return Status::Corrupt;
    }
    if u64::from(words[2]) | (u64::from(words[3]) << 32) != source_hash {
        return Status::WrongFirmware;
    }
    if expected_boot == 0 || u64::from(words[4]) | (u64::from(words[5]) << 32) != expected_boot {
        return Status::WrongBoot;
    }
    Status::Valid
}

/// Diagnostic provenance, separate from receipt validity.
#[derive(Clone, Copy)]
pub enum Origin {
    CurrentBoot,
    PreviousBoot,
}
impl Origin {
    const fn label(self) -> &'static str {
        match self {
            Self::CurrentBoot => "current_boot",
            Self::PreviousBoot => "previous_boot",
        }
    }
}

/// Renders in ordinary task context only; this allocates and must never run in a panic handler.
pub fn marker(
    words: &ReceiptWords,
    source_hash: u64,
    expected_boot: u64,
    origin: Origin,
) -> String {
    let status = classify(words, source_hash, expected_boot);
    if status != Status::Valid {
        return format!(
            "core_dump_store_receipt schema=v1 origin={} status={} redacted=true",
            origin.label(),
            status.label()
        );
    }
    let optional = |index: usize, signed: bool| {
        if words[8] & (1 << (index - 9)) == 0 {
            "unavailable".to_owned()
        } else if signed {
            (words[index] as i32).to_string()
        } else {
            words[index].to_string()
        }
    };
    format!("core_dump_store_receipt schema=v1 origin={} status=valid source_hash={source_hash:016x} boot_ordinal={expected_boot} stage={} capacity_bytes={} requested_bytes={} prepared_bytes={} init_result={} prepare_result={} start_result={} end_result={} store_result={} self_test_marked={} redacted=true",
        origin.label(), STAGES[words[6] as usize], words[7], optional(9, false), optional(10, false),
        optional(11, true), optional(12, true), optional(13, true), optional(14, true), optional(15, true), words[16] == 1)
}

#[cfg(test)]
#[path = "core_dump_receipt/tests.rs"]
mod tests;
