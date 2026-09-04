use std::time::Duration;

use anyhow::{bail, Result};
use bitaxe_api::boot_identity::{ResetReasonCategory, WorkerUsbBootMarker};

/// Closed classification for one bounded USB reboot-loop observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbRebootLoopCategory {
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
}

impl UsbRebootLoopObservation {
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
}

/// Observes a flapping macOS USB profile without writing or changing control lines.
pub fn observe_usb_reboot_loop(port: &str, timeout: Duration) -> Result<UsbRebootLoopObservation> {
    if timeout.is_zero() || timeout > Duration::from_secs(30) {
        bail!("reboot-loop timeout is outside the 1..=30 second bound");
    }
    let capture = crate::macos::capture_reconnecting_receive_only(port, timeout)?;
    classify_capture(&capture.bytes, capture.open_count)
}

fn classify_capture(bytes: &[u8], open_count: u16) -> Result<UsbRebootLoopObservation> {
    const MAX_MARKERS: usize = 256;
    let text = String::from_utf8_lossy(bytes);
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
    } else {
        UsbRebootLoopCategory::UsbStackReset
    };
    let latest = *markers.last().expect("nonempty marker collection");
    Ok(UsbRebootLoopObservation {
        category,
        marker_count: u16::try_from(markers.len()).unwrap_or(u16::MAX),
        reconnect_count: open_count.saturating_sub(1),
        latest_boot_ordinal: latest.boot_ordinal(),
        latest_reset_reason: latest.reset_reason(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(ordinal: u64, reason: ResetReasonCategory, uptime_ms: u64) -> String {
        WorkerUsbBootMarker::new(ordinal, reason, uptime_ms).marker()
    }

    #[test]
    fn advancing_ordinals_prove_a_chip_reset_and_preserve_latest_reason() {
        // Arrange
        let input = format!(
            "{}\n{}\n",
            marker(7, ResetReasonCategory::SoftwareCpu, 500),
            marker(8, ResetReasonCategory::Panic, 400)
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
