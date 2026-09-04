use super::*;

#[test]
fn context_round_trips_with_matching_allocation() {
    // Arrange
    let allocation = RtcAllocationFailureReceipt::new(84, 0x804);
    let source_hash = allocation_source_hash("d369154d");
    let context =
        RtcAllocationContextReceipt::new(allocation, source_hash, StartupStage::UsbInstall);

    // Act
    let marker = AllocationFailureContextMarker::maybe_from_receipts(allocation, context)
        .expect("matching complete receipts");
    let rendered = marker.marker();

    // Assert
    assert_eq!(
        AllocationFailureContextMarker::maybe_parse(&rendered),
        Some(marker)
    );
    assert_eq!(marker.source_hash(), source_hash);
    assert_eq!(marker.stage(), StartupStage::UsbInstall);
    assert_eq!(marker.allocation().requested_bytes(), 84);
    assert_eq!(marker.allocation().capabilities(), 0x804);
}

#[test]
fn torn_context_does_not_discard_valid_v1_failure() {
    // Arrange
    let allocation = RtcAllocationFailureReceipt::new(84, 0x804);
    let mut context = RtcAllocationContextReceipt::new(allocation, 7, StartupStage::Statistics);
    context.checksum ^= 1;

    // Act / Assert
    assert_eq!(
        AllocationFailureContextMarker::maybe_from_receipts(allocation, context),
        None
    );
    assert!(AllocationFailureMarker::from_receipt(allocation).is_some());
}

#[test]
fn context_rejects_a_different_allocation_size() {
    // Arrange
    let allocation = RtcAllocationFailureReceipt::new(84, 0x804);
    let context = RtcAllocationContextReceipt::new(allocation, 7, StartupStage::Statistics);
    let different = RtcAllocationFailureReceipt::new(85, 0x804);

    // Act / Assert
    assert_eq!(
        AllocationFailureContextMarker::maybe_from_receipts(different, context),
        None
    );
}

#[test]
fn context_rejects_a_different_capability_mask() {
    // Arrange
    let allocation = RtcAllocationFailureReceipt::new(84, 0x804);
    let context = RtcAllocationContextReceipt::new(allocation, 7, StartupStage::Statistics);
    let different = RtcAllocationFailureReceipt::new(84, 0x80c);

    // Act / Assert
    assert_eq!(
        AllocationFailureContextMarker::maybe_from_receipts(different, context),
        None
    );
}

#[test]
fn prior_image_context_preserves_its_original_source_identity() {
    // Arrange
    let allocation = RtcAllocationFailureReceipt::new(84, 0x804);
    let prior_source = allocation_source_hash("d369154d");
    let current_source = allocation_source_hash("de3b0d65");
    let context =
        RtcAllocationContextReceipt::new(allocation, prior_source, StartupStage::RuntimeReady);

    // Act
    let marker = AllocationFailureContextMarker::maybe_from_receipts(allocation, context)
        .expect("prior image receipt is valid");

    // Assert
    assert_eq!(marker.source_hash(), prior_source);
    assert_ne!(marker.source_hash(), current_source);
}

#[test]
fn integrity_checked_unknown_stage_is_rejected() {
    // Arrange
    let allocation = RtcAllocationFailureReceipt::new(84, 0x804);
    let mut context = RtcAllocationContextReceipt::new(allocation, 7, StartupStage::Statistics);
    context.stage = 99;
    context.checksum = context.expected_checksum();

    // Act / Assert
    assert_eq!(
        AllocationFailureContextMarker::maybe_from_receipts(allocation, context),
        None
    );
}

#[test]
fn context_parser_rejects_open_fields() {
    // Arrange
    let allocation = RtcAllocationFailureReceipt::new(84, 0x804);
    let context = RtcAllocationContextReceipt::new(allocation, 7, StartupStage::Statistics);
    let rendered = AllocationFailureContextMarker::maybe_from_receipts(allocation, context)
        .expect("complete receipts")
        .marker();
    let malformed = [
        rendered.replace("stage=statistics", "stage=arbitrary_task"),
        rendered.replace("redacted=true", "redacted=false"),
        format!("{rendered} extra=value"),
        rendered.replace(
            "source_hash=0000000000000007",
            "source_hash=0000000000000000",
        ),
    ];

    // Act / Assert
    for line in malformed {
        assert_eq!(AllocationFailureContextMarker::maybe_parse(&line), None);
    }
}
