//! Incremental matcher proving a fresh boot of the exact application in receive-only chunks.

use bitaxe_api::boot_identity::WorkerUsbBootMarker;

use crate::UsbRuntimeIdentity;

/// Largest complete serial record the firmware may emit, including its newline.
const MAX_LINE_BYTES: usize = 66_560;
const IDENTITY_PREFIX: &str = "usb_runtime_identity";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdentityVerdict {
    /// Still waiting for both the exact identity and a boot discriminator.
    Pending,
    /// The exact identity and one consistent boot ordinal were both observed.
    Complete,
    /// A well-formed identity for a different application was observed.
    Mismatch,
    /// Discriminators reported more than one boot ordinal in one window.
    OrdinalAmbiguous,
}

/// Assembles complete LF-terminated lines across arbitrary chunk boundaries.
pub(crate) struct IdentityScanner {
    expected: UsbRuntimeIdentity,
    expected_line: String,
    partial: Vec<u8>,
    discarding_oversized: bool,
    bytes_observed: u64,
    verdict: IdentityVerdict,
    identity_observed: bool,
    maybe_boot: Option<WorkerUsbBootMarker>,
}

impl IdentityScanner {
    pub(crate) fn new(expected: &UsbRuntimeIdentity) -> Self {
        Self {
            expected: expected.clone(),
            expected_line: format!(
                "{IDENTITY_PREFIX} schema=v1 firmware_commit={} app_elf_sha256={} redacted=true",
                expected.firmware_commit, expected.app_elf_sha256
            ),
            partial: Vec::new(),
            discarding_oversized: false,
            bytes_observed: 0,
            verdict: IdentityVerdict::Pending,
            identity_observed: false,
            maybe_boot: None,
        }
    }

    /// Counts every received byte and stops parsing once a terminal verdict exists.
    pub(crate) fn feed(&mut self, chunk: &[u8]) -> IdentityVerdict {
        let count = u64::try_from(chunk.len()).unwrap_or(u64::MAX);
        self.bytes_observed = self.bytes_observed.saturating_add(count);
        for &byte in chunk {
            if self.verdict != IdentityVerdict::Pending {
                break;
            }
            self.push(byte);
        }
        self.verdict
    }

    pub(crate) const fn verdict(&self) -> IdentityVerdict {
        self.verdict
    }

    /// Reports whether the exact identity line arrived, with or without a discriminator.
    pub(crate) const fn identity_observed(&self) -> bool {
        self.identity_observed
    }

    pub(crate) const fn bytes_observed(&self) -> u64 {
        self.bytes_observed
    }

    /// Returns the first boot discriminator seen; its ordinal is this window's ordinal.
    pub(crate) const fn maybe_boot(&self) -> Option<WorkerUsbBootMarker> {
        self.maybe_boot
    }

    fn push(&mut self, byte: u8) {
        if byte == b'\n' {
            let line = std::mem::take(&mut self.partial);
            if !std::mem::replace(&mut self.discarding_oversized, false) {
                self.observe_line(&line);
            }
            return;
        }
        if self.discarding_oversized {
            return;
        }
        if self.partial.len() >= MAX_LINE_BYTES {
            self.partial.clear();
            self.discarding_oversized = true;
            return;
        }
        self.partial.push(byte);
    }

    fn observe_line(&mut self, raw: &[u8]) {
        let Ok(text) = std::str::from_utf8(raw) else {
            return;
        };
        let line = text.strip_suffix('\r').unwrap_or(text);
        if let Some(boot) = WorkerUsbBootMarker::parse(line) {
            self.observe_boot(boot);
        } else if line == self.expected_line {
            self.identity_observed = true;
        } else if line.starts_with(IDENTITY_PREFIX)
            // Only a well-formed but different identity is a mismatch; corrupted
            // lines stay pending so a later intact replay can still decide.
            && UsbRuntimeIdentity::parse(line).is_ok_and(|identity| identity != self.expected)
        {
            self.verdict = IdentityVerdict::Mismatch;
            return;
        }
        if self.verdict == IdentityVerdict::Pending
            && self.identity_observed
            && self.maybe_boot.is_some()
        {
            self.verdict = IdentityVerdict::Complete;
        }
    }

    fn observe_boot(&mut self, boot: WorkerUsbBootMarker) {
        match self.maybe_boot {
            None => self.maybe_boot = Some(boot),
            Some(first) if first.boot_ordinal() != boot.boot_ordinal() => {
                self.verdict = IdentityVerdict::OrdinalAmbiguous;
            }
            Some(_) => {}
        }
    }
}
