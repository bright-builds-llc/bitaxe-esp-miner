//! Reset-retained, value-free Rust panic evidence.

/// Magic identifying a complete RTC panic receipt.
pub const PANIC_RECEIPT_MAGIC: u32 = 0x4258_5052;
/// Magic identifying a complete RTC allocation-failure receipt.
pub const ALLOCATION_FAILURE_RECEIPT_MAGIC: u32 = 0x4258_4146;

/// Integrity-checked source location written before a Rust panic aborts.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcPanicReceipt {
    magic: u32,
    file_hash: u32,
    line: u32,
    checksum: u32,
}

impl RtcPanicReceipt {
    /// Empty RTC representation.
    pub const ZERO: Self = Self {
        magic: 0,
        file_hash: 0,
        line: 0,
        checksum: 0,
    };

    /// Creates one receipt from a static Rust panic location.
    #[must_use]
    pub fn new(file: &str, line: u32) -> Self {
        let file_hash = fnv1a_32(file.as_bytes());
        let mut receipt = Self {
            magic: PANIC_RECEIPT_MAGIC,
            file_hash,
            line,
            checksum: 0,
        };
        receipt.checksum = receipt.expected_checksum();
        receipt
    }

    /// Returns whether the receipt is complete and internally consistent.
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.magic == PANIC_RECEIPT_MAGIC
            && self.file_hash != 0
            && self.line != 0
            && self.checksum == self.expected_checksum()
    }

    /// Returns the value-free source-path digest.
    #[must_use]
    pub const fn file_hash(self) -> u32 {
        self.file_hash
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }

    fn expected_checksum(self) -> u32 {
        self.magic.rotate_left(5) ^ self.file_hash.rotate_left(13) ^ self.line.rotate_left(21)
    }
}

/// Integrity-checked allocation failure written before the allocator aborts.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcAllocationFailureReceipt {
    magic: u32,
    requested_bytes: u32,
    capabilities: u32,
    checksum: u32,
}

impl RtcAllocationFailureReceipt {
    /// Empty RTC representation.
    pub const ZERO: Self = Self {
        magic: 0,
        requested_bytes: 0,
        capabilities: 0,
        checksum: 0,
    };

    /// Creates one receipt without allocating.
    #[must_use]
    pub fn new(requested_bytes: usize, capabilities: u32) -> Self {
        let mut receipt = Self {
            magic: ALLOCATION_FAILURE_RECEIPT_MAGIC,
            requested_bytes: u32::try_from(requested_bytes).unwrap_or(u32::MAX),
            capabilities,
            checksum: 0,
        };
        receipt.checksum = receipt.expected_checksum();
        receipt
    }

    /// Returns whether the receipt is complete and internally consistent.
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.magic == ALLOCATION_FAILURE_RECEIPT_MAGIC
            && self.requested_bytes != 0
            && self.checksum == self.expected_checksum()
    }

    /// Returns the failed allocation size.
    #[must_use]
    pub const fn requested_bytes(self) -> u32 {
        self.requested_bytes
    }

    /// Returns the ESP-IDF capability mask.
    #[must_use]
    pub const fn capabilities(self) -> u32 {
        self.capabilities
    }

    fn expected_checksum(self) -> u32 {
        self.magic.rotate_left(7)
            ^ self.requested_bytes.rotate_left(17)
            ^ self.capabilities.rotate_left(23)
    }
}

/// Closed Worker CDC projection of one valid allocation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationFailureMarker {
    requested_bytes: u32,
    capabilities: u32,
}

impl AllocationFailureMarker {
    /// Creates a marker only from a valid receipt.
    #[must_use]
    pub fn from_receipt(receipt: RtcAllocationFailureReceipt) -> Option<Self> {
        receipt.is_valid().then_some(Self {
            requested_bytes: receipt.requested_bytes(),
            capabilities: receipt.capabilities(),
        })
    }

    /// Renders the exact value-free Worker CDC marker.
    #[must_use]
    pub fn marker(self) -> String {
        format!(
            "allocation_failure_receipt schema=v1 requested_bytes={} capabilities={:08x} redacted=true",
            self.requested_bytes, self.capabilities
        )
    }

    /// Parses one exact marker, tolerating a logger prefix.
    #[must_use]
    pub fn parse(line: &str) -> Option<Self> {
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let start = tokens
            .iter()
            .position(|token| *token == "allocation_failure_receipt")?;
        let fields = tokens.get(start..)?;
        if fields.len() != 5 || fields[1] != "schema=v1" || fields[4] != "redacted=true" {
            return None;
        }
        let requested_bytes = fields[2].strip_prefix("requested_bytes=")?.parse().ok()?;
        let capabilities =
            u32::from_str_radix(fields[3].strip_prefix("capabilities=")?, 16).ok()?;
        (requested_bytes != 0).then_some(Self {
            requested_bytes,
            capabilities,
        })
    }

    /// Returns the failed allocation size.
    #[must_use]
    pub const fn requested_bytes(self) -> u32 {
        self.requested_bytes
    }

    /// Returns the ESP-IDF capability mask.
    #[must_use]
    pub const fn capabilities(self) -> u32 {
        self.capabilities
    }
}

/// Closed Worker CDC projection of one valid RTC panic receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustPanicMarker {
    file_hash: u32,
    line: u32,
}

impl RustPanicMarker {
    /// Creates a marker only from a valid receipt.
    #[must_use]
    pub fn from_receipt(receipt: RtcPanicReceipt) -> Option<Self> {
        receipt.is_valid().then_some(Self {
            file_hash: receipt.file_hash(),
            line: receipt.line(),
        })
    }

    /// Renders the exact value-free Worker CDC marker.
    #[must_use]
    pub fn marker(self) -> String {
        format!(
            "rust_panic_receipt schema=v1 file_hash={:08x} line={} redacted=true",
            self.file_hash, self.line
        )
    }

    /// Parses one exact marker, tolerating a logger prefix.
    #[must_use]
    pub fn parse(line: &str) -> Option<Self> {
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let start = tokens
            .iter()
            .position(|token| *token == "rust_panic_receipt")?;
        let fields = tokens.get(start..)?;
        if fields.len() != 5 || fields[1] != "schema=v1" || fields[4] != "redacted=true" {
            return None;
        }
        let file_hash = u32::from_str_radix(fields[2].strip_prefix("file_hash=")?, 16).ok()?;
        let line = fields[3].strip_prefix("line=")?.parse().ok()?;
        let marker = Self { file_hash, line };
        (file_hash != 0 && line != 0).then_some(marker)
    }

    /// Returns the value-free source-path digest.
    #[must_use]
    pub const fn file_hash(self) -> u32 {
        self.file_hash
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }
}

/// Computes the stable source-path digest used by panic receipts.
#[must_use]
pub fn fnv1a_32(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5_u32;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_receipt_round_trips_through_the_closed_marker() {
        // Arrange
        let receipt = RtcPanicReceipt::new("firmware/bitaxe/src/startup.rs", 42);

        // Act
        let marker = RustPanicMarker::from_receipt(receipt).expect("receipt must be valid");
        let rendered = marker.marker();

        // Assert
        assert_eq!(RustPanicMarker::parse(&rendered), Some(marker));
        assert_eq!(marker.line(), 42);
        assert!(!rendered.contains("startup.rs"));
    }

    #[test]
    fn panic_receipt_rejects_torn_or_open_fields() {
        // Arrange
        let mut torn = RtcPanicReceipt::new("source.rs", 7);
        torn.checksum ^= 1;
        let candidates = [
            "rust_panic_receipt schema=v1 file_hash=00000000 line=7 redacted=true",
            "rust_panic_receipt schema=v1 file_hash=12345678 line=0 redacted=true",
            "rust_panic_receipt schema=v1 file_hash=12345678 line=7 redacted=false",
        ];

        // Act / Assert
        assert!(!torn.is_valid());
        assert_eq!(RustPanicMarker::from_receipt(torn), None);
        for candidate in candidates {
            assert_eq!(RustPanicMarker::parse(candidate), None);
        }
    }

    #[test]
    fn allocation_failure_receipt_round_trips_without_open_text() {
        // Arrange
        let receipt = RtcAllocationFailureReceipt::new(16_384, 0x0000_0008);

        // Act
        let marker = AllocationFailureMarker::from_receipt(receipt).expect("receipt must be valid");
        let rendered = marker.marker();

        // Assert
        assert_eq!(AllocationFailureMarker::parse(&rendered), Some(marker));
        assert_eq!(marker.requested_bytes(), 16_384);
        assert_eq!(marker.capabilities(), 8);
    }
}
