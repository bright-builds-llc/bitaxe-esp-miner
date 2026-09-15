#![allow(dead_code)]
#[path = "boot_diagnostic_cache.rs"]
mod boot_diagnostic_cache;

#[test]
fn typed_boot_producers_record_before_general_logging() {
    // Arrange
    let source = include_str!("startup.rs");
    let checkpoint = source
        .split("fn retain_usb_memory_checkpoint(")
        .nth(1)
        .expect("checkpoint producer")
        .split("fn retain_bwg_worker_start_failure(")
        .next()
        .expect("producer boundary");
    let failure = source
        .split("fn retain_bwg_worker_start_failure(")
        .nth(1)
        .expect("failure producer")
        .split("fn bwg_worker_start_failure_detail(")
        .next()
        .expect("failure boundary");
    // Act / Assert
    assert!(
        checkpoint
            .find("CACHE.record_checkpoint(")
            .expect("typed checkpoint")
            < checkpoint
                .find("crate::info_retained(")
                .expect("general log")
    );
    assert!(
        failure
            .find("CACHE.record_failure(")
            .expect("typed failure")
            < failure.find("log::warn!").expect("general log")
    );
    assert!(!include_str!("boot_evidence/worker_diagnostics.rs").contains("log_buffer::"));
}
