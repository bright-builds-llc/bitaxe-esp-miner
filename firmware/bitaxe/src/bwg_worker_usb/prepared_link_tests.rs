use super::*;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

#[test]
fn real_link_preparation_performs_no_read_and_activation_runs_exactly_once() {
    // Arrange
    let _exclusive = crate::TEST_LOCK.lock().expect("fixture lock");
    crate::READS.store(0, Ordering::Relaxed);
    let mut prepared = prepare().expect("real preparation");
    // Act / Assert
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(crate::READS.load(Ordering::Relaxed), 0);
    assert!(prepared.activate());
    assert!(!prepared.activate());
    let deadline = Instant::now() + Duration::from_secs(1);
    while crate::READS.load(Ordering::Relaxed) == 0 {
        assert!(Instant::now() < deadline, "activated fixture thread runs");
        std::thread::yield_now();
    }
    assert_eq!(crate::READS.load(Ordering::Relaxed), 1);
}

#[test]
fn dropped_link_preparation_never_reads_or_acquires_application_authority() {
    // Arrange
    let _exclusive = crate::TEST_LOCK.lock().expect("fixture lock");
    crate::READS.store(0, Ordering::Relaxed);
    let prepared = prepare().expect("real preparation");
    // Act
    drop(prepared);
    std::thread::sleep(Duration::from_millis(30));
    // Assert
    assert_eq!(crate::READS.load(Ordering::Relaxed), 0);
}

#[test]
fn failed_link_reservation_preserves_errno_without_starting_a_reader() {
    // Arrange
    let mut attempts = 0;
    // Act
    let result = prepare_with(|| {
        attempts += 1;
        Err(io::Error::from_raw_os_error(12))
    });
    // Assert
    assert_eq!(attempts, 1);
    assert_eq!(
        result.err().expect("failed allocation").raw_os_error(),
        Some(12)
    );
}
