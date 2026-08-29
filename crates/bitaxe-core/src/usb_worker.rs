//! Pure Worker USB wire and bounded-write contracts.
//!
//! Descriptor bytes preserve `firmware/bitaxe/bwg/native/bwg_usb.c` at source
//! commit `902108b7fc5d1941b8734732e6ea8dd6a8350a23`, including the pinned
//! TinyUSB `TUD_CONFIG_DESCRIPTOR` and `TUD_CDC_DESCRIPTOR` expansions.

/// Exact packed USB device descriptor for the TinyUSB Worker profile.
pub const WORKER_DEVICE_DESCRIPTOR: [u8; 18] = [
    18, 1, 0, 2, 0xef, 2, 1, 64, 0x09, 0x12, 0x7a, 0xb1, 1, 0, 1, 2, 0, 1,
];

/// Exact configuration, vendor-control, and CDC-evidence descriptor sequence.
pub const WORKER_CONFIGURATION_DESCRIPTOR: [u8; 98] = [
    9, 2, 98, 0, 3, 1, 0, 0x80, 50, 9, 4, 0, 0, 2, 0xff, 0x42, 1, 3, 7, 5, 1, 2, 64, 0, 0, 7, 5,
    0x81, 2, 64, 0, 0, 8, 11, 1, 2, 2, 2, 0, 0, 9, 4, 1, 0, 1, 2, 2, 0, 4, 5, 0x24, 0, 0x20, 1, 5,
    0x24, 1, 0, 2, 4, 0x24, 2, 6, 5, 0x24, 6, 1, 2, 7, 5, 0x82, 3, 8, 0, 1, 9, 4, 2, 0, 2, 0x0a, 0,
    0, 0, 7, 5, 3, 2, 64, 0, 0, 7, 5, 0x83, 2, 64, 0, 0,
];

/// Language bytes and NUL-terminated strings retained by `UsbRuntime`.
pub const WORKER_STRING_DESCRIPTORS: [&[u8]; 5] = [
    b"\x09\x04",
    b"Bright Builds\0",
    b"Bitaxe Ultra 205 BWG Worker\0",
    b"BWG Worker Control\0",
    b"BWG Worker Evidence\0",
];

/// Maximum cumulative waits admitted by one vendor-frame write.
pub const MAX_VENDOR_WRITE_WAITS: u16 = 2_000;

/// Closed failure classes produced by the pure bounded-write policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbWriteFailure {
    UnavailableTransport,
    PartialWrite,
    Timeout,
}

/// One effect request or terminal result from the bounded-write policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorWriteStep {
    Complete,
    Write { offset: usize, length: usize },
    Wait,
    Continue,
    Failed(UsbWriteFailure),
}

/// Pure progress state for one exact Worker vendor frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VendorWriteProgress {
    target_length: usize,
    written: usize,
    waits: u16,
    maybe_failure: Option<UsbWriteFailure>,
}

impl VendorWriteProgress {
    /// Creates progress for one nonempty frame.
    pub fn new(target_length: usize) -> Result<Self, UsbWriteFailure> {
        if target_length == 0 {
            return Err(UsbWriteFailure::UnavailableTransport);
        }
        Ok(Self {
            target_length,
            written: 0,
            waits: 0,
            maybe_failure: None,
        })
    }

    /// Chooses the next action from current mount and FIFO availability.
    pub fn next(&mut self, mounted: bool, available: usize) -> VendorWriteStep {
        if let Some(failure) = self.maybe_failure {
            return VendorWriteStep::Failed(failure);
        }
        if self.written == self.target_length {
            return VendorWriteStep::Complete;
        }
        if !mounted {
            return self.fail(if self.written == 0 {
                UsbWriteFailure::UnavailableTransport
            } else {
                UsbWriteFailure::PartialWrite
            });
        }
        if available == 0 {
            return self.wait_or_timeout();
        }
        VendorWriteStep::Write {
            offset: self.written,
            length: available.min(self.target_length - self.written),
        }
    }

    /// Records one TinyUSB write result without allowing impossible progress.
    pub fn record_write(&mut self, requested: usize, written: usize) -> VendorWriteStep {
        if let Some(failure) = self.maybe_failure {
            return VendorWriteStep::Failed(failure);
        }
        let remaining = self.target_length.saturating_sub(self.written);
        if requested == 0 || requested > remaining || written > requested {
            return self.fail(UsbWriteFailure::PartialWrite);
        }
        if written == 0 {
            return self.wait_or_timeout();
        }
        let Some(total) = self.written.checked_add(written) else {
            return self.fail(UsbWriteFailure::PartialWrite);
        };
        self.written = total;
        if self.written == self.target_length {
            VendorWriteStep::Complete
        } else {
            VendorWriteStep::Continue
        }
    }

    fn wait_or_timeout(&mut self) -> VendorWriteStep {
        let Some(waits) = self.waits.checked_add(1) else {
            return self.fail(UsbWriteFailure::Timeout);
        };
        self.waits = waits;
        if waits > MAX_VENDOR_WRITE_WAITS {
            self.fail(if self.written == 0 {
                UsbWriteFailure::Timeout
            } else {
                UsbWriteFailure::PartialWrite
            })
        } else {
            VendorWriteStep::Wait
        }
    }

    fn fail(&mut self, failure: UsbWriteFailure) -> VendorWriteStep {
        self.maybe_failure = Some(failure);
        VendorWriteStep::Failed(failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_DEVICE_DESCRIPTOR: [u8; 18] = [
        18, 1, 0, 2, 0xef, 2, 1, 64, 0x09, 0x12, 0x7a, 0xb1, 1, 0, 1, 2, 0, 1,
    ];

    const EXPECTED_CONFIGURATION_DESCRIPTOR: [u8; 98] = [
        9, 2, 98, 0, 3, 1, 0, 0x80, 50, 9, 4, 0, 0, 2, 0xff, 0x42, 1, 3, 7, 5, 1, 2, 64, 0, 0, 7,
        5, 0x81, 2, 64, 0, 0, 8, 11, 1, 2, 2, 2, 0, 0, 9, 4, 1, 0, 1, 2, 2, 0, 4, 5, 0x24, 0, 0x20,
        1, 5, 0x24, 1, 0, 2, 4, 0x24, 2, 6, 5, 0x24, 6, 1, 2, 7, 5, 0x82, 3, 8, 0, 1, 9, 4, 2, 0,
        2, 0x0a, 0, 0, 0, 7, 5, 3, 2, 64, 0, 0, 7, 5, 0x83, 2, 64, 0, 0,
    ];

    #[test]
    fn worker_descriptors_match_the_current_tinyusb_macro_expansion() {
        // Arrange / Act / Assert
        assert_eq!(WORKER_DEVICE_DESCRIPTOR, EXPECTED_DEVICE_DESCRIPTOR);
        assert_eq!(
            WORKER_CONFIGURATION_DESCRIPTOR,
            EXPECTED_CONFIGURATION_DESCRIPTOR
        );
        assert_eq!(
            WORKER_STRING_DESCRIPTORS,
            [
                b"\x09\x04".as_slice(),
                b"Bright Builds\0".as_slice(),
                b"Bitaxe Ultra 205 BWG Worker\0".as_slice(),
                b"BWG Worker Control\0".as_slice(),
                b"BWG Worker Evidence\0".as_slice(),
            ]
        );
    }

    #[test]
    fn worker_descriptor_fields_preserve_profile_and_endpoint_identity() {
        // Arrange / Act / Assert
        assert_eq!(&WORKER_DEVICE_DESCRIPTOR[8..12], &[0x09, 0x12, 0x7a, 0xb1]);
        assert_eq!(
            &WORKER_CONFIGURATION_DESCRIPTOR[..9],
            &[9, 2, 98, 0, 3, 1, 0, 0x80, 50]
        );
        assert_eq!(
            &WORKER_CONFIGURATION_DESCRIPTOR[14..18],
            &[0xff, 0x42, 1, 3]
        );
        assert_eq!(WORKER_CONFIGURATION_DESCRIPTOR[20], 0x01);
        assert_eq!(WORKER_CONFIGURATION_DESCRIPTOR[27], 0x81);
        assert_eq!(WORKER_CONFIGURATION_DESCRIPTOR[70], 0x82);
        assert_eq!(WORKER_CONFIGURATION_DESCRIPTOR[86], 0x03);
        assert_eq!(WORKER_CONFIGURATION_DESCRIPTOR[93], 0x83);
    }

    #[test]
    fn vendor_write_completes_across_partial_writes() {
        // Arrange
        let mut progress = VendorWriteProgress::new(64).expect("nonempty frame");

        // Act / Assert
        assert_eq!(
            progress.next(true, 32),
            VendorWriteStep::Write {
                offset: 0,
                length: 32
            }
        );
        assert_eq!(progress.record_write(32, 16), VendorWriteStep::Continue);
        assert_eq!(
            progress.next(true, 64),
            VendorWriteStep::Write {
                offset: 16,
                length: 48
            }
        );
        assert_eq!(progress.record_write(48, 48), VendorWriteStep::Complete);
    }

    #[test]
    fn vendor_write_zero_progress_is_bounded() {
        // Arrange
        let mut progress = VendorWriteProgress::new(1).expect("nonempty frame");

        // Act
        for _ in 0..MAX_VENDOR_WRITE_WAITS {
            assert_eq!(progress.record_write(1, 0), VendorWriteStep::Wait);
        }
        let terminal = progress.record_write(1, 0);

        // Assert
        assert_eq!(terminal, VendorWriteStep::Failed(UsbWriteFailure::Timeout));
    }

    #[test]
    fn vendor_write_timeout_after_progress_is_partial() {
        // Arrange
        let mut progress = VendorWriteProgress::new(2).expect("nonempty frame");
        let _step = progress.record_write(2, 1);

        // Act
        for _ in 0..MAX_VENDOR_WRITE_WAITS {
            assert_eq!(progress.next(true, 0), VendorWriteStep::Wait);
        }
        let terminal = progress.next(true, 0);

        // Assert
        assert_eq!(
            terminal,
            VendorWriteStep::Failed(UsbWriteFailure::PartialWrite)
        );
    }

    #[test]
    fn vendor_write_distinguishes_unavailable_and_partial_disconnect() {
        // Arrange
        let mut untouched = VendorWriteProgress::new(4).expect("nonempty frame");
        let mut partial = VendorWriteProgress::new(4).expect("nonempty frame");
        let _step = partial.record_write(4, 2);

        // Act / Assert
        assert_eq!(
            untouched.next(false, 0),
            VendorWriteStep::Failed(UsbWriteFailure::UnavailableTransport)
        );
        assert_eq!(
            partial.next(false, 0),
            VendorWriteStep::Failed(UsbWriteFailure::PartialWrite)
        );
    }

    #[test]
    fn vendor_write_rejects_empty_or_impossible_progress() {
        // Arrange
        let empty = VendorWriteProgress::new(0);
        let mut progress = VendorWriteProgress::new(2).expect("nonempty frame");

        // Act / Assert
        assert_eq!(empty, Err(UsbWriteFailure::UnavailableTransport));
        assert_eq!(
            progress.record_write(2, 3),
            VendorWriteStep::Failed(UsbWriteFailure::PartialWrite)
        );
    }
}
