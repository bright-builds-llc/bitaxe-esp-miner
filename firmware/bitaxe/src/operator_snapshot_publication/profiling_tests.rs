use super::*;
thread_local! { static NOW: Cell<u64> = const { Cell::new(100) }; }
fn now() -> u64 {
    NOW.with(Cell::get)
}
fn advance(delta: u64) {
    NOW.with(|clock| clock.set(clock.get() + delta));
}

#[test]
fn production_publisher_profiles_completion_retention_and_issuance_exclusively() {
    // Arrange
    NOW.with(|clock| clock.set(100));
    let publisher = OperatorSnapshotPublisher::new();
    let timing = LiveStageProfiler::new(now);
    let collection = [
        LiveStage::VisibleState,
        LiveStage::Platform,
        LiveStage::HealthSafety,
        LiveStage::ConfirmedSettings,
        LiveStage::SettingsTransactionWait,
        LiveStage::SettingsNvsRead,
        LiveStage::Wifi,
    ];
    // Act
    let result = publisher.publish_profiled(
        BootSessionId::from_words([1, 2, 3, 4]),
        || {
            for stage in collection {
                timing.measure(stage, || advance(10));
            }
        },
        |(), identity| {
            advance(300);
            identity
        },
        |_| {
            advance(400);
            Ok::<_, &'static str>(())
        },
        |identity| {
            advance(500);
            Ok::<_, &'static str>(identity.revision().get())
        },
        &timing,
    );
    // Assert
    assert_eq!(result.expect("published").output, 1);
    assert!(timing.measurements().complete);
    assert_eq!(
        timing.measurements().durations_us,
        [10, 10, 10, 10, 10, 10, 10, 0, 300, 400, 500]
    );
}

#[test]
fn failed_retention_remains_a_failure_with_explicit_incomplete_profiling() {
    // Arrange
    NOW.with(|clock| clock.set(100));
    let publisher = OperatorSnapshotPublisher::new();
    let timing = LiveStageProfiler::new(now);
    let issued = Cell::new(false);
    // Act
    let result = publisher.publish_profiled(
        BootSessionId::from_words([1, 2, 3, 4]),
        || (),
        |(), _| (),
        |_| {
            advance(400);
            Err("retention failure")
        },
        |()| {
            issued.set(true);
            Ok::<_, &'static str>(())
        },
        &timing,
    );
    // Assert
    assert!(matches!(
        result,
        Err(OperatorSnapshotPublishError::Retention { .. })
    ));
    assert!(!issued.get());
    assert_eq!(
        timing.measurements().durations_us[LiveStage::Retention as usize],
        400
    );
    assert!(!timing.measurements().complete);
}

#[test]
fn profiled_and_ordinary_publications_share_one_retained_sequence() {
    // Arrange
    let publisher = OperatorSnapshotPublisher::new();
    let timing = LiveStageProfiler::new(now);
    let retained = std::cell::RefCell::new(Vec::new());
    let issued = std::cell::RefCell::new(Vec::new());
    let boot = BootSessionId::from_words([1, 2, 3, 4]);
    // Act
    publisher
        .publish_profiled(
            boot,
            || (),
            |(), identity| identity,
            |identity| {
                retained.borrow_mut().push(identity.revision().get());
                Ok::<_, &'static str>(())
            },
            |identity| {
                issued.borrow_mut().push(identity.revision().get());
                Ok::<_, &'static str>(())
            },
            &timing,
        )
        .expect("profiled publication");
    publisher
        .publish(
            boot,
            || (),
            |(), identity| identity,
            |identity| {
                retained.borrow_mut().push(identity.revision().get());
                Ok::<_, &'static str>(())
            },
            |identity| {
                issued.borrow_mut().push(identity.revision().get());
                Ok::<_, &'static str>(())
            },
        )
        .expect("ordinary publication");
    // Assert
    assert_eq!(*retained.borrow(), [1, 2]);
    assert_eq!(*issued.borrow(), [1, 2]);
}
