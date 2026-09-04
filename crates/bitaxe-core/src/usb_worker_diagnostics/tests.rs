use super::*;

#[test]
fn late_observer_gets_one_finite_burst_after_the_arm_window() {
    // Arrange
    let mut replay = WorkerDiagnosticReplay::default();
    replay.line_coding(115_200, 100_000);
    replay.line_state(true, 100_010);

    // Act / Assert
    assert_eq!(replay.maybe_due_slot(105_109, true), None);
    for slot in 0..DIAGNOSTIC_REPORT_SLOTS {
        let now = 105_110 + slot as u64 * LINE_INTERVAL_MS;
        assert_eq!(replay.maybe_due_slot(now, true), Some(slot));
        replay.advance(now);
    }
    replay.line_state(true, 106_010);
    assert_eq!(replay.maybe_due_slot(110_000, true), None);
}

#[test]
fn maintenance_control_sequence_cancels_diagnostics_through_commit() {
    // Arrange
    let mut replay = WorkerDiagnosticReplay::default();
    replay.line_coding(115_200, 0);
    replay.line_state(true, 10);

    // Act
    replay.line_coding(1_200, 50);
    assert_eq!(replay.maybe_due_slot(6_000, false), None);
    replay.line_coding(115_200, 100);

    // Assert
    assert_eq!(replay.maybe_due_slot(6_000, false), None);
    assert_eq!(replay.maybe_due_slot(7_000, true), None);
}

#[test]
fn unread_fifo_cannot_extend_report_past_twenty_seconds() {
    // Arrange
    let mut replay = WorkerDiagnosticReplay::default();
    replay.line_state(true, 0);
    replay.line_coding(115_200, 1);

    // Act / Assert
    assert_eq!(replay.maybe_due_slot(19_999, true), Some(0));
    assert_eq!(replay.maybe_due_slot(20_001, true), None);
}

#[derive(Default)]
struct Transport {
    bytes: Vec<u8>,
    capacity: usize,
    maybe_short_write: Option<usize>,
    refuse_newline_once: bool,
}

impl CdcEvidenceTransport for Transport {
    fn available(&self) -> usize {
        self.capacity.saturating_sub(self.bytes.len())
    }
    fn write(&mut self, bytes: &[u8]) -> usize {
        if bytes == b"\n" && self.refuse_newline_once {
            self.refuse_newline_once = false;
            return 0;
        }
        let length = self
            .maybe_short_write
            .take()
            .unwrap_or(bytes.len())
            .min(self.available());
        self.bytes.extend_from_slice(&bytes[..length]);
        length
    }
    fn flush(&mut self) {}
}

#[test]
fn full_fifo_keeps_existing_maintenance_receipt_and_retries_complete_line() {
    // Arrange
    let receipt = b"usb_maintenance={\"status\":\"ready\"}\n";
    let diagnostic = b"worker_usb_boot schema=v1\n";
    let mut transport = Transport {
        bytes: receipt.to_vec(),
        capacity: 64,
        ..Default::default()
    };
    let mut writer = CdcEvidenceWriter::default();

    // Act / Assert
    transport.capacity = receipt.len() + diagnostic.len();
    assert!(!writer.try_emit(&mut transport, diagnostic));
    assert_eq!(transport.bytes, receipt);
    transport.bytes.clear();
    assert!(writer.try_emit(&mut transport, diagnostic));
    assert_eq!(transport.bytes, diagnostic);
}

#[test]
fn short_diagnostic_write_cannot_corrupt_the_next_maintenance_receipt() {
    // Arrange
    let mut writer = CdcEvidenceWriter::default();
    let mut transport = Transport {
        capacity: 512,
        maybe_short_write: Some(5),
        ..Default::default()
    };
    let receipt = b"usb_maintenance={\"status\":\"committed\"}\n";

    // Act
    assert!(!writer.try_emit(&mut transport, b"diagnostic=closed\n"));
    assert!(writer.try_emit(&mut transport, receipt));

    // Assert
    assert_eq!(transport.bytes, [b"diagn\n".as_slice(), receipt].concat());
}

#[test]
fn retained_allowlist_rejects_extra_payload_and_network_fields() {
    // Arrange
    let valid = "usb_memory_checkpoint stage=usb_install free_bytes=100 largest_block_bytes=90 reserve_bytes=98304 redacted=true";

    // Act / Assert
    assert!(is_worker_diagnostic_retained_line(valid));
    assert!(!is_worker_diagnostic_retained_line(&format!(
        "{valid} payload=secret"
    )));
    assert!(!is_worker_diagnostic_retained_line("usb_memory_checkpoint stage=private-network free_bytes=100 largest_block_bytes=90 reserve_bytes=98304 redacted=true"));
    assert!(!is_worker_diagnostic_retained_line(
        "bwg_worker_start_failure category=startup_failed detail=private-url redacted=true"
    ));
}

#[test]
fn diagnostics_reserve_capacity_for_both_maintenance_receipts() {
    // Arrange
    let mut writer = CdcEvidenceWriter::default();
    let mut transport = Transport {
        capacity: 512,
        ..Default::default()
    };
    let diagnostic = [b"x".repeat(190), b"\n".to_vec()].concat();

    // Act
    assert!(writer.try_emit_diagnostic(&mut transport, &diagnostic));
    assert!(writer.try_emit_diagnostic(&mut transport, &diagnostic));
    assert!(!writer.try_emit_diagnostic(&mut transport, &diagnostic));

    // Assert
    assert!(writer.try_emit(&mut transport, b"usb_maintenance={\"status\":\"ready\"}\n"));
    assert!(writer.try_emit(
        &mut transport,
        b"usb_maintenance={\"status\":\"committed\"}\n"
    ));
}

#[test]
fn deferred_short_write_repair_precedes_the_next_receipt() {
    // Arrange
    let mut writer = CdcEvidenceWriter::new();
    let mut transport = Transport {
        capacity: 512,
        maybe_short_write: Some(5),
        refuse_newline_once: true,
        ..Default::default()
    };
    let receipt = b"usb_maintenance={\"status\":\"ready\"}\n";

    // Act
    assert!(!writer.try_emit_diagnostic(&mut transport, b"diagnostic=closed\n"));
    assert!(writer.try_emit(&mut transport, receipt));

    // Assert
    assert_eq!(transport.bytes, [b"diagn\n".as_slice(), receipt].concat());
}
