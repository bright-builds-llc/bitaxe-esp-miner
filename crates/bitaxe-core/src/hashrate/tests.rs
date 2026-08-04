use super::*;

fn assert_near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
}

#[test]
fn instantaneous_register_scales_and_rejects_sentinel_values() {
    // Arrange
    let mut monitor = HashrateMonitor::new(1, 4);

    // Act
    let updated = monitor.observe(0, HashrateRegister::Instantaneous, 100, 0);
    let sentinel = monitor.observe(0, HashrateRegister::Instantaneous, 0x007f_ffff, 0);
    let snapshot = monitor.sample(true);

    // Assert
    assert_eq!(updated, Ok(HashrateObservationOutcome::Updated));
    assert_eq!(
        sentinel,
        Ok(HashrateObservationOutcome::IgnoredRegisterSentinel)
    );
    assert_near(snapshot.current_ghs, 1.677_721_6);
    assert_near(snapshot.asics[0].domain_ghs[0], 1.677_721_6);
}

#[test]
fn wrapping_counter_uses_two_to_the_thirty_two_hash_unit() {
    // Arrange
    let mut monitor = HashrateMonitor::new(1, 4);
    monitor
        .observe(0, HashrateRegister::TotalCount, u32::MAX - 1, 1_000_000)
        .expect("baseline should be admitted");

    // Act
    let outcome = monitor.observe(0, HashrateRegister::TotalCount, 1, 2_000_000);
    let snapshot = monitor.sample(true);

    // Assert
    assert_eq!(outcome, Ok(HashrateObservationOutcome::Updated));
    assert_near(snapshot.current_ghs, 3.0 * HASH_COUNTER_UNIT_HASHES / 1e9);
}

#[test]
fn subsecond_update_does_not_move_the_counter_baseline() {
    // Arrange
    let mut monitor = HashrateMonitor::new(1, 4);
    monitor
        .observe(0, HashrateRegister::TotalCount, 10, 1_000_000)
        .expect("baseline should be admitted");

    // Act
    let ignored = monitor.observe(0, HashrateRegister::TotalCount, 20, 1_500_000);
    let updated = monitor.observe(0, HashrateRegister::TotalCount, 30, 2_000_000);
    let snapshot = monitor.sample(true);

    // Assert
    assert_eq!(ignored, Ok(HashrateObservationOutcome::IgnoredTooSoon));
    assert_eq!(updated, Ok(HashrateObservationOutcome::Updated));
    assert_near(snapshot.current_ghs, 20.0 * HASH_COUNTER_UNIT_HASHES / 1e9);
}

#[test]
fn admission_rejects_invalid_asic_domain_and_regressed_time() {
    // Arrange
    let mut monitor = HashrateMonitor::new(1, 4);
    monitor
        .observe(0, HashrateRegister::TotalCount, 1, 2_000_000)
        .expect("baseline should be admitted");

    // Act
    let asic = monitor.observe(1, HashrateRegister::TotalCount, 2, 3_000_000);
    let domain = monitor.observe(0, HashrateRegister::DomainCount(4), 2, 3_000_000);
    let time = monitor.observe(0, HashrateRegister::TotalCount, 2, 1_000_000);

    // Assert
    assert_eq!(asic, Err(HashrateObservationError::AsicOutOfRange));
    assert_eq!(domain, Err(HashrateObservationError::DomainOutOfRange));
    assert_eq!(time, Err(HashrateObservationError::TimestampRegression));
}

#[test]
fn stop_resets_counter_baseline_and_prevents_resume_spike() {
    // Arrange
    let mut monitor = HashrateMonitor::new(1, 4);
    monitor
        .observe(0, HashrateRegister::TotalCount, 100, 1_000_000)
        .expect("baseline should be admitted");
    monitor
        .observe(0, HashrateRegister::TotalCount, 110, 2_000_000)
        .expect("sample should be admitted");
    let before_stop = monitor.sample(true);

    // Act
    let stopped = monitor.sample(false);
    let resumed = monitor.observe(0, HashrateRegister::TotalCount, 1, 20_000_000);
    let after_resume = monitor.sample(true);

    // Assert
    assert!(before_stop.current_ghs > 0.0);
    assert_eq!(stopped.current_ghs, 0.0);
    assert_eq!(resumed, Ok(HashrateObservationOutcome::BaselineEstablished));
    assert_eq!(after_resume.current_ghs, 0.0);
    assert!(after_resume.one_minute_ghs > 0.0);
}

#[test]
fn sample_computes_error_percentage_and_per_asic_domains() {
    // Arrange
    let mut monitor = HashrateMonitor::new(1, 4);
    for (register, value) in [
        (HashrateRegister::TotalCount, 100),
        (HashrateRegister::ErrorCount, 10),
        (HashrateRegister::DomainCount(2), 25),
    ] {
        monitor
            .observe(0, register, 0, 1_000_000)
            .expect("baseline should be admitted");
        monitor
            .observe(0, register, value, 2_000_000)
            .expect("sample should be admitted");
    }

    // Act
    let snapshot = monitor.sample(true);

    // Assert
    assert_near(snapshot.error_percentage, 10.0);
    assert_eq!(snapshot.asics[0].error_count, 10);
    assert_near(snapshot.asics[0].domain_ghs[2], snapshot.current_ghs / 4.0);
}

#[test]
fn hierarchical_windows_blend_across_all_reference_boundaries() {
    // Arrange
    let mut monitor = HashrateMonitor::new(1, 4);

    // Act
    for index in 0..660_u64 {
        let value = if index < 60 { 60 } else { 120 };
        monitor
            .observe(0, HashrateRegister::Instantaneous, value, index * 1_000_000)
            .expect("instantaneous value should be admitted");
        let _ = monitor.sample(true);
    }
    let snapshot = monitor.sample(true);

    // Assert
    assert!(snapshot.one_minute_ghs > 1.9);
    assert!(snapshot.ten_minute_ghs > snapshot.one_minute_ghs / 2.0);
    assert!(snapshot.one_hour_ghs > 0.0);
    assert!(snapshot.one_hour_ghs <= snapshot.ten_minute_ghs);
}
