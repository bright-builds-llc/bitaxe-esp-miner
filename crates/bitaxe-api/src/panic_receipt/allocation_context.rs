//! Bounded source and startup context for the existing allocation receipt.

use super::{AllocationFailureMarker, RtcAllocationFailureReceipt};

const MAGIC: u32 = 0x4258_4143;

/// Boot-lifetime boundary active when an allocation failed, across all tasks.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupStage {
    EarlyIdentity = 1,
    Hardware = 2,
    RuntimeServices = 3,
    StorageHttp = 4,
    Network = 5,
    UsbInstall = 6,
    Statistics = 7,
    RuntimeReady = 8,
}

impl StartupStage {
    /// Decodes the bounded atomic/RTC representation.
    #[must_use]
    pub const fn maybe_from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::EarlyIdentity),
            2 => Some(Self::Hardware),
            3 => Some(Self::RuntimeServices),
            4 => Some(Self::StorageHttp),
            5 => Some(Self::Network),
            6 => Some(Self::UsbInstall),
            7 => Some(Self::Statistics),
            8 => Some(Self::RuntimeReady),
            _ => None,
        }
    }

    /// Stable closed evidence label; it does not identify the allocating task.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EarlyIdentity => "early_identity",
            Self::Hardware => "hardware",
            Self::RuntimeServices => "runtime_services",
            Self::StorageHttp => "storage_http",
            Self::Network => "network",
            Self::UsbInstall => "usb_install",
            Self::Statistics => "statistics",
            Self::RuntimeReady => "runtime_ready",
        }
    }

    fn maybe_parse(label: &str) -> Option<Self> {
        (1..=8)
            .filter_map(Self::maybe_from_raw)
            .find(|stage| stage.label() == label)
    }
}

/// Stable FNV-1a digest of the compiled source revision, computed without allocation.
#[must_use]
pub const fn allocation_source_hash(source: &str) -> u64 {
    let bytes = source.as_bytes();
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    hash
}

/// Separate integrity-checked RTC context, joined only to matching allocation fields.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcAllocationContextReceipt {
    source_hash: u64,
    magic: u32,
    requested_bytes: u32,
    capabilities: u32,
    stage: u32,
    checksum: u64,
}

impl RtcAllocationContextReceipt {
    /// Empty RTC representation.
    pub const ZERO: Self = Self {
        source_hash: 0,
        magic: 0,
        requested_bytes: 0,
        capabilities: 0,
        stage: 0,
        checksum: 0,
    };

    /// Creates fixed-size context without allocation or platform calls.
    #[must_use]
    pub fn new(
        allocation: RtcAllocationFailureReceipt,
        source_hash: u64,
        stage: StartupStage,
    ) -> Self {
        let mut receipt = Self {
            source_hash,
            magic: MAGIC,
            requested_bytes: allocation.requested_bytes(),
            capabilities: allocation.capabilities(),
            stage: stage as u32,
            checksum: 0,
        };
        receipt.checksum = receipt.expected_checksum();
        receipt
    }

    fn expected_checksum(self) -> u64 {
        self.source_hash.rotate_left(11)
            ^ u64::from(self.magic).rotate_left(7)
            ^ u64::from(self.requested_bytes).rotate_left(23)
            ^ u64::from(self.capabilities).rotate_left(39)
            ^ u64::from(self.stage).rotate_left(53)
    }
}

/// Closed projection binding a failure to its originating image and startup boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationFailureContextMarker {
    allocation: AllocationFailureMarker,
    source_hash: u64,
    stage: StartupStage,
}

impl AllocationFailureContextMarker {
    /// Rejects torn context or context that disagrees with the paired v1 receipt.
    #[must_use]
    pub fn maybe_from_receipts(
        allocation: RtcAllocationFailureReceipt,
        context: RtcAllocationContextReceipt,
    ) -> Option<Self> {
        let allocation = AllocationFailureMarker::from_receipt(allocation)?;
        if context.magic != MAGIC
            || context.source_hash == 0
            || context.checksum != context.expected_checksum()
            || allocation.requested_bytes() != context.requested_bytes
            || allocation.capabilities() != context.capabilities
        {
            return None;
        }
        Some(Self {
            allocation,
            source_hash: context.source_hash,
            stage: StartupStage::maybe_from_raw(context.stage)?,
        })
    }

    /// Renders bounded fields only, independently of the unchanged v1 marker.
    #[must_use]
    pub fn marker(self) -> String {
        format!(
            "allocation_failure_context schema=v1 requested_bytes={} capabilities={:08x} source_hash={:016x} stage={} redacted=true",
            self.allocation.requested_bytes(), self.allocation.capabilities(), self.source_hash,
            self.stage.label(),
        )
    }

    /// Parses exact closed fields with an optional logger prefix and no open text.
    #[must_use]
    pub fn maybe_parse(line: &str) -> Option<Self> {
        let mut fields = line.split_whitespace();
        fields.find(|token| *token == "allocation_failure_context")?;
        if fields.next()? != "schema=v1" {
            return None;
        }
        let requested_bytes: u32 = fields
            .next()?
            .strip_prefix("requested_bytes=")?
            .parse()
            .ok()?;
        let capabilities_text = fields.next()?.strip_prefix("capabilities=")?;
        let source_text = fields.next()?.strip_prefix("source_hash=")?;
        if capabilities_text.len() != 8 || source_text.len() != 16 {
            return None;
        }
        let capabilities = u32::from_str_radix(capabilities_text, 16).ok()?;
        let source_hash = u64::from_str_radix(source_text, 16).ok()?;
        let stage = StartupStage::maybe_parse(fields.next()?.strip_prefix("stage=")?)?;
        if fields.next()? != "redacted=true" || fields.next().is_some() {
            return None;
        }
        let allocation = RtcAllocationFailureReceipt::new(requested_bytes as usize, capabilities);
        Self::maybe_from_receipts(
            allocation,
            RtcAllocationContextReceipt::new(allocation, source_hash, stage),
        )
    }

    /// Existing v1 allocation identity paired with this context.
    #[must_use]
    pub const fn allocation(self) -> AllocationFailureMarker {
        self.allocation
    }

    /// Originating image's source revision digest, which can differ from this boot.
    #[must_use]
    pub const fn source_hash(self) -> u64 {
        self.source_hash
    }

    /// Global startup boundary active at failure, not the allocating task identity.
    #[must_use]
    pub const fn stage(self) -> StartupStage {
        self.stage
    }
}

#[cfg(test)]
mod tests;
