//! Typed running-platform identity and runtime-health facts.

use serde::{Deserialize, Serialize};

/// One platform fact that is either proved by the running firmware or explicitly unavailable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PlatformFact<T> {
    /// The running firmware proved the enclosed value.
    Available { value: T },
    /// The running firmware could not prove a value.
    Unavailable { reason: PlatformUnavailableReason },
}

impl<T> PlatformFact<T> {
    /// Constructs a proved platform fact.
    #[must_use]
    pub const fn available(value: T) -> Self {
        Self::Available { value }
    }

    /// Constructs an explicitly unavailable platform fact.
    #[must_use]
    pub const fn unavailable(reason: PlatformUnavailableReason) -> Self {
        Self::Unavailable { reason }
    }

    /// Borrows the proved value without treating a compatibility fallback as proof.
    #[must_use]
    pub const fn maybe_value(&self) -> Option<&T> {
        match self {
            Self::Available { value } => Some(value),
            Self::Unavailable { .. } => None,
        }
    }
}

/// Closed reasons why a running-platform fact could not be proved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformUnavailableReason {
    /// The value exists only in a host fixture and is not running-device truth.
    FixtureOnly,
    /// ESP-IDF did not return a usable value.
    EspIdfUnavailable,
    /// The embedded static-asset identity was absent or malformed.
    StaticAssetUnavailable,
    /// ESP-IDF did not identify the running application partition.
    RunningPartitionUnavailable,
    /// ESP-IDF returned an unknown or unsupported reset code.
    UnknownResetReason,
    /// The monotonic runtime counter was unavailable or nonpositive.
    UptimeUnavailable,
    /// The requested heap fact was unavailable or zero.
    HeapUnavailable,
}

/// Closed firmware board identity. No other board can receive an available claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformBoard {
    /// Bitaxe Ultra board version 205.
    #[serde(rename = "205")]
    Ultra205,
}

/// Closed firmware ASIC identity. No other ASIC can receive an available claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformAsic {
    /// BM1366 ASIC used by the Ultra 205 target.
    #[serde(rename = "BM1366")]
    Bm1366,
}

/// Closed decoded ESP-IDF reset-reason vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformResetReason {
    PowerOn,
    ExternalPin,
    SoftwareCpu,
    Panic,
    InterruptWatchdog,
    TaskWatchdog,
    OtherWatchdog,
    DeepSleep,
    Brownout,
    Sdio,
    Usb,
    Jtag,
    EfuseError,
    PowerGlitch,
    CpuLockup,
}

impl PlatformResetReason {
    /// Decodes the stable ESP-IDF reset-reason discriminants without guessing unknown codes.
    #[must_use]
    pub const fn decode(raw_reason: i32) -> PlatformFact<Self> {
        let reason = match raw_reason {
            1 => Self::PowerOn,
            2 => Self::ExternalPin,
            3 => Self::SoftwareCpu,
            4 => Self::Panic,
            5 => Self::InterruptWatchdog,
            6 => Self::TaskWatchdog,
            7 => Self::OtherWatchdog,
            8 => Self::DeepSleep,
            9 => Self::Brownout,
            10 => Self::Sdio,
            11 => Self::Usb,
            12 => Self::Jtag,
            13 => Self::EfuseError,
            14 => Self::PowerGlitch,
            15 => Self::CpuLockup,
            _ => {
                return PlatformFact::Unavailable {
                    reason: PlatformUnavailableReason::UnknownResetReason,
                };
            }
        };
        PlatformFact::Available { value: reason }
    }

    /// Returns the legacy AxeOS-compatible display text for this proved reason.
    #[must_use]
    pub const fn compatibility_text(self) -> &'static str {
        match self {
            Self::PowerOn => "Reset due to power-on event",
            Self::ExternalPin => "Reset by external pin",
            Self::SoftwareCpu => "Software CPU reset",
            Self::Panic => "Software reset due to panic",
            Self::InterruptWatchdog => "Interrupt watchdog reset",
            Self::TaskWatchdog => "Task watchdog reset",
            Self::OtherWatchdog => "Other watchdog reset",
            Self::DeepSleep => "Reset after deep sleep",
            Self::Brownout => "Brownout reset",
            Self::Sdio => "Reset over SDIO",
            Self::Usb => "Reset by USB peripheral",
            Self::Jtag => "Reset by JTAG",
            Self::EfuseError => "Reset due to eFuse error",
            Self::PowerGlitch => "Reset due to power glitch",
            Self::CpuLockup => "Reset due to CPU lockup",
        }
    }
}

/// Complete platform candidate captured inside one coherent operator snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformIdentity {
    pub esp_idf_version: PlatformFact<String>,
    pub axe_os_static_asset: PlatformFact<String>,
    pub board: PlatformFact<PlatformBoard>,
    pub asic: PlatformFact<PlatformAsic>,
    pub running_partition: PlatformFact<String>,
    pub reset_reason: PlatformFact<PlatformResetReason>,
    pub uptime_milliseconds: PlatformFact<u64>,
    pub internal_heap_free_bytes: PlatformFact<u64>,
    pub internal_heap_minimum_free_bytes: PlatformFact<u64>,
    pub internal_heap_largest_free_block_bytes: PlatformFact<u64>,
    pub psram_available: PlatformFact<bool>,
}

impl PlatformIdentity {
    /// Returns a host-fixture candidate that authenticates no running-platform fact.
    #[must_use]
    pub fn fixture_only() -> Self {
        let unavailable = PlatformUnavailableReason::FixtureOnly;
        Self {
            esp_idf_version: PlatformFact::unavailable(unavailable),
            axe_os_static_asset: PlatformFact::unavailable(unavailable),
            board: PlatformFact::unavailable(unavailable),
            asic: PlatformFact::unavailable(unavailable),
            running_partition: PlatformFact::unavailable(unavailable),
            reset_reason: PlatformFact::unavailable(unavailable),
            uptime_milliseconds: PlatformFact::unavailable(unavailable),
            internal_heap_free_bytes: PlatformFact::unavailable(unavailable),
            internal_heap_minimum_free_bytes: PlatformFact::unavailable(unavailable),
            internal_heap_largest_free_block_bytes: PlatformFact::unavailable(unavailable),
            psram_available: PlatformFact::unavailable(unavailable),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn available_platform_facts_serialize_proved_values() {
        // Arrange
        let identity = all_available_identity();

        // Act
        let value = serde_json::to_value(identity).expect("platform identity should serialize");

        // Assert
        assert_eq!(
            value["espIdfVersion"],
            json!({"state": "available", "value": "v5.5.4"})
        );
        assert_eq!(
            value["axeOsStaticAsset"],
            json!({"state": "available", "value": "bitaxe-rust-phase-07-static-compatibility"})
        );
        assert_eq!(
            value["board"],
            json!({"state": "available", "value": "205"})
        );
        assert_eq!(
            value["asic"],
            json!({"state": "available", "value": "BM1366"})
        );
        assert_eq!(
            value["runningPartition"],
            json!({"state": "available", "value": "factory"})
        );
        assert_eq!(
            value["resetReason"],
            json!({"state": "available", "value": "power_on"})
        );
        assert_eq!(value["uptimeMilliseconds"]["value"], json!(42));
        assert_eq!(value["internalHeapFreeBytes"]["value"], json!(1024));
        assert_eq!(value["internalHeapMinimumFreeBytes"]["value"], json!(512));
        assert_eq!(
            value["internalHeapLargestFreeBlockBytes"]["value"],
            json!(256)
        );
        assert_eq!(value["psramAvailable"]["value"], json!(true));
    }

    #[test]
    fn fixture_platform_facts_are_explicitly_unavailable() {
        // Arrange
        let identity = PlatformIdentity::fixture_only();

        // Act
        let value = serde_json::to_value(identity).expect("platform identity should serialize");

        // Assert
        for field in [
            "espIdfVersion",
            "axeOsStaticAsset",
            "board",
            "asic",
            "runningPartition",
            "resetReason",
            "uptimeMilliseconds",
            "internalHeapFreeBytes",
            "internalHeapMinimumFreeBytes",
            "internalHeapLargestFreeBlockBytes",
            "psramAvailable",
        ] {
            assert_eq!(value[field]["state"], "unavailable");
            assert_eq!(value[field]["reason"], "fixture_only");
            assert!(value[field].get("value").is_none());
        }
    }

    #[test]
    fn reset_decoder_distinguishes_software_cpu_and_rejects_unknown_codes() {
        // Arrange
        let software_cpu_code = 3;
        let unknown_codes = [0, 16, i32::MAX];

        // Act
        let software_cpu = PlatformResetReason::decode(software_cpu_code);
        let unknown = unknown_codes.map(PlatformResetReason::decode);

        // Assert
        assert_eq!(
            software_cpu,
            PlatformFact::available(PlatformResetReason::SoftwareCpu)
        );
        assert!(unknown.into_iter().all(|fact| {
            fact == PlatformFact::unavailable(PlatformUnavailableReason::UnknownResetReason)
        }));
    }

    #[test]
    fn closed_board_and_asic_vocabularies_reject_alternate_claims() {
        // Arrange
        let alternate_board = r#"{"state":"available","value":"601"}"#;
        let alternate_asic = r#"{"state":"available","value":"BM1370"}"#;

        // Act
        let board_result = serde_json::from_str::<PlatformFact<PlatformBoard>>(alternate_board);
        let asic_result = serde_json::from_str::<PlatformFact<PlatformAsic>>(alternate_asic);

        // Assert
        assert!(board_result.is_err());
        assert!(asic_result.is_err());
    }

    #[test]
    fn numeric_zero_does_not_change_an_unavailable_fact_to_available() {
        // Arrange
        let unavailable_uptime =
            PlatformFact::<u64>::unavailable(PlatformUnavailableReason::UptimeUnavailable);
        let compatibility_uptime = 0_u64;

        // Act
        let maybe_proved_uptime = unavailable_uptime.maybe_value().copied();

        // Assert
        assert_eq!(compatibility_uptime, 0);
        assert_eq!(maybe_proved_uptime, None);
    }

    fn all_available_identity() -> PlatformIdentity {
        PlatformIdentity {
            esp_idf_version: PlatformFact::available("v5.5.4".to_owned()),
            axe_os_static_asset: PlatformFact::available(
                "bitaxe-rust-phase-07-static-compatibility".to_owned(),
            ),
            board: PlatformFact::available(PlatformBoard::Ultra205),
            asic: PlatformFact::available(PlatformAsic::Bm1366),
            running_partition: PlatformFact::available("factory".to_owned()),
            reset_reason: PlatformFact::available(PlatformResetReason::PowerOn),
            uptime_milliseconds: PlatformFact::available(42),
            internal_heap_free_bytes: PlatformFact::available(1024),
            internal_heap_minimum_free_bytes: PlatformFact::available(512),
            internal_heap_largest_free_block_bytes: PlatformFact::available(256),
            psram_available: PlatformFact::available(true),
        }
    }
}
