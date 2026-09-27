use super::super::line_admission::ReceiveLineAdmission;

#[test]
fn ephemeral_ingress_discards_the_untrusted_first_line() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();

    // Act
    let admitted = admission.admit(b"mining_campaign_status={invalid}\ntrusted\n");

    // Assert
    assert_eq!(admitted, Some(b"trusted\n".as_slice()));
}

#[test]
fn ephemeral_ingress_waits_for_a_split_initial_boundary() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();

    // Act
    let first = admission.admit(b"partial-marker");
    let second = admission.admit(b"-tail\ntrusted\n");

    // Assert
    assert_eq!(first, None);
    assert_eq!(second, Some(b"trusted\n".as_slice()));
}

#[test]
fn ephemeral_ingress_forwards_post_boundary_chunks_unchanged() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();
    let _discarded = admission.admit(b"first-line\n");

    // Act
    let admitted = admission.admit(b"second\nthird\n");

    // Assert
    assert_eq!(admitted, Some(b"second\nthird\n".as_slice()));
}

#[test]
fn ephemeral_ingress_requires_a_fresh_boundary_after_reopen() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();
    let _discarded = admission.admit(b"first-open\n");
    let _admitted = admission.admit(b"trusted-before-reopen\n");

    // Act
    admission.reset();
    let discarded_after_reopen = admission.admit(b"reopen-fragment\n");
    let admitted_after_reopen = admission.admit(b"trusted-after-reopen\n");

    // Assert
    assert_eq!(discarded_after_reopen, None);
    assert_eq!(
        admitted_after_reopen,
        Some(b"trusted-after-reopen\n".as_slice())
    );
}

#[test]
fn ephemeral_receive_session_owns_boundary_reset_and_admission() {
    // Arrange
    let implementation = include_str!("observation.rs");

    // Act
    let reader_open = implementation
        .find("ReceiveOnlyReader::open")
        .expect("reader admission");
    let boundary_reset = implementation
        .find("line_admission.reset()")
        .expect("boundary reset");
    let chunk_read = implementation
        .find("reader.read_available()")
        .expect("chunk read");
    let boundary_admission = implementation
        .find("line_admission.admit(&chunk)")
        .expect("boundary admission");

    // Assert
    assert!(reader_open < boundary_reset);
    assert!(boundary_reset < chunk_read);
    assert!(chunk_read < boundary_admission);
    assert_eq!(
        implementation
            .matches("line_admission.admit(&chunk)")
            .count(),
        1
    );
    assert!(implementation.contains("if feed_chunks"));
    assert!(implementation.contains("bytes.extend_from_slice"));
}
