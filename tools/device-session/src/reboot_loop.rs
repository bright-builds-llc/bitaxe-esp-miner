use std::time::Duration;

use anyhow::{bail, Result};
use bitaxe_api::boot_identity::{ResetReasonCategory, WorkerUsbBootMarker};
use bitaxe_api::panic_receipt::{
    AllocationFailureContextMarker, AllocationFailureMarker, RustPanicMarker,
};

mod diagnostics;
pub use diagnostics::{UsbMemoryCheckpoint, UsbRuntimeIdentity, UsbStartupProgress};

/// Closed classification for one bounded USB reboot-loop observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbRebootLoopCategory {
    /// Multiple same-boot markers arrived without reopening the observer.
    StableBoot,
    /// One boot ordinal mounted repeatedly, so only the USB stack cycled.
    UsbStackReset,
    /// The reset-retained ordinal advanced, proving a chip reset.
    ChipReset,
    /// The bounded observation did not contain enough consistent samples.
    Inconclusive,
}

impl UsbRebootLoopCategory {
    /// Returns the stable console label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::StableBoot => "stable_boot",
            Self::UsbStackReset => "usb_stack_reset",
            Self::ChipReset => "chip_reset",
            Self::Inconclusive => "inconclusive",
        }
    }
}

/// Safe aggregate from a same-connector reconnecting capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbRebootLoopObservation {
    category: UsbRebootLoopCategory,
    marker_count: u16,
    reconnect_count: u16,
    latest_boot_ordinal: u64,
    latest_reset_reason: ResetReasonCategory,
    latest_rust_panic: Option<RustPanicMarker>,
    latest_allocation_failure: Option<AllocationFailureMarker>,
    maybe_allocation_context: Option<AllocationFailureContextMarker>,
    diagnostics: diagnostics::WorkerDiagnostics,
}

impl UsbRebootLoopObservation {
    /// Returns validated source/stage context for the prior allocation failure.
    pub const fn maybe_allocation_context(&self) -> Option<AllocationFailureContextMarker> {
        self.maybe_allocation_context
    }
    /// Returns exact current application identity, when the firmware emitted it.
    pub fn maybe_runtime_identity(&self) -> Option<&UsbRuntimeIdentity> {
        self.diagnostics.maybe_identity.as_ref()
    }

    /// Returns validated, deduplicated startup heap checkpoints.
    pub fn memory_checkpoints(&self) -> &[UsbMemoryCheckpoint] {
        &self.diagnostics.memory
    }

    /// Returns the latest closed progress from the current boot, before or after Worker admission.
    pub fn maybe_startup_progress(&self) -> Option<&UsbStartupProgress> {
        self.diagnostics.maybe_startup.as_ref()
    }

    /// Reports an explicit Worker startup failure independently from missing evidence.
    pub const fn worker_start_failed(&self) -> bool {
        self.diagnostics.worker_start_failed
    }

    /// Requires runtime evidence to match the caller's prevalidated exact package.
    pub fn require_identity(&self, expected: &UsbRuntimeIdentity) -> Result<()> {
        match self.maybe_runtime_identity() {
            Some(observed) if observed == expected => Ok(()),
            Some(_) => bail!("usb_diagnostics=runtime_identity_mismatch"),
            None => bail!("usb_diagnostics=runtime_identity_missing"),
        }
    }
    /// Returns the reboot-loop classification.
    #[must_use]
    pub const fn category(&self) -> UsbRebootLoopCategory {
        self.category
    }

    /// Returns the bounded valid-marker count.
    #[must_use]
    pub const fn marker_count(&self) -> u16 {
        self.marker_count
    }

    /// Returns the number of reader reopens after initial admission.
    #[must_use]
    pub const fn reconnect_count(&self) -> u16 {
        self.reconnect_count
    }

    /// Returns the latest reset-retained boot ordinal.
    #[must_use]
    pub const fn latest_boot_ordinal(&self) -> u64 {
        self.latest_boot_ordinal
    }

    /// Returns the latest closed reset reason.
    #[must_use]
    pub const fn latest_reset_reason(&self) -> ResetReasonCategory {
        self.latest_reset_reason
    }

    /// Returns the previous boot's Rust panic receipt when present.
    #[must_use]
    pub const fn latest_rust_panic(&self) -> Option<RustPanicMarker> {
        self.latest_rust_panic
    }

    /// Returns the previous boot's allocation failure when present.
    #[must_use]
    pub const fn latest_allocation_failure(&self) -> Option<AllocationFailureMarker> {
        self.latest_allocation_failure
    }
}

/// Observes a flapping macOS USB profile using the fixed-115200 receive-only adapter.
/// It sends no payload and never selects the maintenance baud.
pub fn observe_usb_reboot_loop(port: &str, timeout: Duration) -> Result<UsbRebootLoopObservation> {
    if timeout.is_zero() || timeout > Duration::from_secs(30) {
        bail!("reboot-loop timeout is outside the 1..=30 second bound");
    }
    let capture = crate::macos::capture_reconnecting_receive_only(port, timeout)?;
    parse_usb_reboot_diagnostics(&capture.bytes, capture.open_count)
}

/// Parses a bounded transcript without admitting a device or proving its physical origin.
/// Callers must separately retain the physical lease and compare expected identity.
pub fn parse_usb_reboot_diagnostics(
    bytes: &[u8],
    open_count: u16,
) -> Result<UsbRebootLoopObservation> {
    if bytes.len() > 64 * 1024 || open_count == 0 {
        bail!("usb_diagnostics=invalid_capture_bounds");
    }
    classify_capture(bytes, open_count)
}

fn classify_capture(bytes: &[u8], open_count: u16) -> Result<UsbRebootLoopObservation> {
    const MAX_MARKERS: usize = 256;
    let text = String::from_utf8_lossy(bytes);
    let latest_boot_text = final_boot_segment(&text);
    let latest_rust_panic = latest_boot_text
        .lines()
        .filter_map(RustPanicMarker::parse)
        .next_back();
    let latest_allocation_failure = latest_boot_text
        .lines()
        .filter_map(AllocationFailureMarker::parse)
        .next_back();
    let maybe_allocation_context = latest_boot_text
        .lines()
        .filter(|line| line.starts_with("allocation_failure_context "))
        .map(|line| {
            AllocationFailureContextMarker::maybe_parse(line)
                .ok_or_else(|| anyhow::anyhow!("usb_diagnostics=malformed_allocation_context"))
        })
        .collect::<Result<Vec<_>>>()?
        .last()
        .copied();
    if maybe_allocation_context
        .is_some_and(|context| Some(context.allocation()) != latest_allocation_failure)
    {
        bail!("usb_diagnostics=allocation_context_mismatch");
    }
    let markers = text
        .lines()
        .filter_map(WorkerUsbBootMarker::parse)
        .take(MAX_MARKERS.saturating_add(1))
        .collect::<Vec<_>>();
    if markers.is_empty() {
        bail!("reboot-loop capture contains no valid marker");
    }
    if markers.len() > MAX_MARKERS {
        bail!("reboot-loop capture contains too many markers");
    }
    let consistent = markers.windows(2).all(|pair| {
        pair[1].boot_ordinal() > pair[0].boot_ordinal()
            || (pair[1].boot_ordinal() == pair[0].boot_ordinal()
                && pair[1].uptime_ms() >= pair[0].uptime_ms())
    });
    let distinct_ordinals = markers
        .windows(2)
        .filter(|pair| pair[1].boot_ordinal() != pair[0].boot_ordinal())
        .count();
    let category = if !consistent || markers.len() < 2 {
        UsbRebootLoopCategory::Inconclusive
    } else if distinct_ordinals > 0 {
        UsbRebootLoopCategory::ChipReset
    } else if open_count > 1 {
        UsbRebootLoopCategory::UsbStackReset
    } else {
        UsbRebootLoopCategory::StableBoot
    };
    let latest = *markers.last().expect("nonempty marker collection");
    Ok(UsbRebootLoopObservation {
        category,
        marker_count: u16::try_from(markers.len()).unwrap_or(u16::MAX),
        reconnect_count: open_count.saturating_sub(1),
        latest_boot_ordinal: latest.boot_ordinal(),
        latest_reset_reason: latest.reset_reason(),
        latest_rust_panic,
        latest_allocation_failure,
        maybe_allocation_context,
        diagnostics: diagnostics::WorkerDiagnostics::parse(latest_boot_text)?,
    })
}

fn final_boot_segment(text: &str) -> &str {
    let mut maybe_ordinal = None;
    let mut segment_start = text.len();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if let Some(marker) = WorkerUsbBootMarker::parse(line) {
            if maybe_ordinal != Some(marker.boot_ordinal()) {
                maybe_ordinal = Some(marker.boot_ordinal());
                segment_start = offset;
            }
        }
        offset += line.len();
    }
    &text[segment_start..]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(ordinal: u64, reason: ResetReasonCategory, uptime_ms: u64) -> String {
        WorkerUsbBootMarker::new(ordinal, reason, uptime_ms).marker()
    }

    #[test]
    fn startup_progress_is_scoped_to_the_current_boot_without_identity_admission() {
        // Arrange
        let input = format!("{}\nusb_startup schema=v1 stage=nvs state=failed first_failure=nvs uptime_ms=600 redacted=true\n{}\nusb_startup schema=v1 stage=network state=entered first_failure=none uptime_ms=200 redacted=true\n",
            marker(2, ResetReasonCategory::Panic, 500), marker(3, ResetReasonCategory::Panic, 100));
        let expected =
            UsbRuntimeIdentity::new(&"a".repeat(40), &"b".repeat(64)).expect("expected identity");
        // Act
        let observation = classify_capture(input.as_bytes(), 2).expect("valid reboot progress");
        let progress = observation
            .maybe_startup_progress()
            .expect("latest startup");
        // Assert
        assert_eq!(progress.stage, "network");
        assert_eq!(progress.maybe_first_failure, None);
        assert_eq!(progress.uptime_ms, 200);
        assert!(observation.require_identity(&expected).is_err());
    }

    #[test]
    fn periodic_same_boot_evidence_without_reconnect_is_stable() {
        // Arrange
        let input = format!(
            "{}\n{}\n",
            marker(2, ResetReasonCategory::Panic, 500),
            marker(2, ResetReasonCategory::Panic, 2500)
        );
        // Act
        let observation = classify_capture(input.as_bytes(), 1).expect("valid observed boot");
        // Assert
        assert_eq!(observation.category().label(), "stable_boot");
    }

    #[test]
    fn exact_package_requirement_rejects_a_different_runtime() {
        // Arrange
        let input = format!("{}\nusb_runtime_identity schema=v1 firmware_commit={} app_elf_sha256={} redacted=true\n",
            marker(2, ResetReasonCategory::Panic, 500), "a".repeat(40), "b".repeat(64));
        let expected =
            UsbRuntimeIdentity::new(&"c".repeat(40), &"b".repeat(64)).expect("expected identity");
        let observation = classify_capture(input.as_bytes(), 1).expect("valid observed identity");
        // Act / Assert
        assert!(observation.require_identity(&expected).is_err());
    }

    #[test]
    fn flashed_package_is_not_substituted_for_missing_runtime_identity() {
        // Arrange
        let input = format!("{}\n", marker(2, ResetReasonCategory::Panic, 500));
        let expected =
            UsbRuntimeIdentity::new(&"a".repeat(40), &"b".repeat(64)).expect("expected identity");
        let observation = classify_capture(input.as_bytes(), 1).expect("valid boot marker");
        // Act / Assert
        assert!(observation.require_identity(&expected).is_err());
    }

    #[test]
    fn earlier_boot_identity_cannot_verify_a_later_silent_boot() {
        // Arrange
        let input = format!("{}\nusb_runtime_identity schema=v1 firmware_commit={} app_elf_sha256={} redacted=true\n{}\n",
            marker(2, ResetReasonCategory::Panic, 500), "a".repeat(40), "b".repeat(64),
            marker(3, ResetReasonCategory::Panic, 100));
        let expected =
            UsbRuntimeIdentity::new(&"a".repeat(40), &"b".repeat(64)).expect("expected identity");
        let observation = classify_capture(input.as_bytes(), 2).expect("valid reboot transcript");
        // Act / Assert
        assert!(observation.require_identity(&expected).is_err());
    }

    #[test]
    fn different_heap_facts_on_two_boots_preserve_chip_reset_evidence() {
        // Arrange
        let input = format!("{}\nusb_memory_checkpoint stage=usb_install free_bytes=50000 largest_block_bytes=12000 reserve_bytes=98304 redacted=true\n{}\nusb_memory_checkpoint stage=usb_install free_bytes=40000 largest_block_bytes=10000 reserve_bytes=98304 redacted=true\n",
            marker(2, ResetReasonCategory::Panic, 500), marker(3, ResetReasonCategory::Panic, 100));
        // Act
        let observation =
            classify_capture(input.as_bytes(), 2).expect("valid differing boot heaps");
        // Assert
        assert_eq!(observation.category(), UsbRebootLoopCategory::ChipReset);
        assert_eq!(observation.memory_checkpoints()[0].free_bytes, 40000);
    }

    #[test]
    fn advancing_ordinals_prove_a_chip_reset_and_preserve_latest_reason() {
        // Arrange
        let panic = RustPanicMarker::from_receipt(bitaxe_api::panic_receipt::RtcPanicReceipt::new(
            "source.rs",
            91,
        ))
        .expect("panic receipt must be valid")
        .marker();
        let allocation = AllocationFailureMarker::from_receipt(
            bitaxe_api::panic_receipt::RtcAllocationFailureReceipt::new(8_192, 8),
        )
        .expect("allocation receipt must be valid")
        .marker();
        let input = format!(
            "{}\n{}\n{}\n{}\n",
            marker(7, ResetReasonCategory::SoftwareCpu, 500),
            marker(8, ResetReasonCategory::Panic, 400),
            panic,
            allocation
        );

        // Act
        let observation = classify_capture(input.as_bytes(), 2).expect("capture must classify");

        // Assert
        assert_eq!(observation.category(), UsbRebootLoopCategory::ChipReset);
        assert_eq!(observation.latest_boot_ordinal(), 8);
        assert_eq!(
            observation.latest_reset_reason(),
            ResetReasonCategory::Panic
        );
        assert_eq!(observation.reconnect_count(), 1);
        assert_eq!(
            observation.latest_rust_panic().map(RustPanicMarker::line),
            Some(91)
        );
        assert_eq!(
            observation
                .latest_allocation_failure()
                .map(AllocationFailureMarker::requested_bytes),
            Some(8_192)
        );
    }

    #[test]
    fn one_ordinal_with_increasing_uptime_proves_a_usb_stack_reset() {
        // Arrange
        let input = format!(
            "{}\n{}\n",
            marker(7, ResetReasonCategory::SoftwareCpu, 500),
            marker(7, ResetReasonCategory::SoftwareCpu, 2_500)
        );

        // Act
        let observation = classify_capture(input.as_bytes(), 2).expect("capture must classify");

        // Assert
        assert_eq!(observation.category(), UsbRebootLoopCategory::UsbStackReset);
    }

    #[test]
    fn regressed_or_single_marker_capture_is_inconclusive() {
        // Arrange
        let single = format!("{}\n", marker(7, ResetReasonCategory::Other, 500));
        let regressed = format!(
            "{}\n{}\n",
            marker(8, ResetReasonCategory::Other, 500),
            marker(7, ResetReasonCategory::Other, 400)
        );

        // Act / Assert
        assert_eq!(
            classify_capture(single.as_bytes(), 1)
                .expect("single capture must parse")
                .category(),
            UsbRebootLoopCategory::Inconclusive
        );
        assert_eq!(
            classify_capture(regressed.as_bytes(), 2)
                .expect("regressed capture must parse")
                .category(),
            UsbRebootLoopCategory::Inconclusive
        );
    }
}
