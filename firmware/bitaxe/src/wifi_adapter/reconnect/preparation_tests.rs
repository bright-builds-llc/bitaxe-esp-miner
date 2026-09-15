use super::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Subscription {
    _sender: Sender<WifiReconnectEvent>,
    drops: Arc<AtomicU32>,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}
fn subscriptions(
    sender: &Sender<WifiReconnectEvent>,
    drops: &Arc<AtomicU32>,
) -> (Subscription, Subscription) {
    (
        Subscription {
            _sender: sender.clone(),
            drops: Arc::clone(drops),
        },
        Subscription {
            _sender: sender.clone(),
            drops: Arc::clone(drops),
        },
    )
}
fn await_cancelled(sender: &Sender<WifiReconnectEvent>) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while sender.send(WifiReconnectEvent::Ipv4Assigned).is_ok() {
        assert!(
            Instant::now() < deadline,
            "cancelled worker releases its receiver"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn real_preparation_queues_events_until_subscriptions_are_installed_and_activated() {
    // Arrange
    let _exclusive = crate::TEST_LOCK.lock().expect("fixture lock");
    crate::RUNS.store(0, Ordering::Relaxed);
    let (events, received) = mpsc::channel();
    let (exit, exited) = mpsc::channel();
    *crate::SINK.lock().expect("sink") = Some((events, exit));
    let mut prepared = prepare().expect("real preparation call site");
    prepared
        .sender()
        .send(WifiReconnectEvent::Ipv4Assigned)
        .expect("queue before activation");
    let drops = Arc::new(AtomicU32::new(0));
    let (wifi, ip) = subscriptions(prepared.sender(), &drops);
    let (mut maybe_wifi, mut maybe_ip) = (None, None);
    // Act / Assert
    assert!(matches!(
        received.recv_timeout(Duration::from_millis(30)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    assert_eq!(crate::RUNS.load(Ordering::Relaxed), 0);
    assert!(prepared.activate_subscribed(&mut maybe_wifi, &mut maybe_ip, wifi, ip));
    assert!(maybe_wifi.is_some() && maybe_ip.is_some());
    assert_eq!(
        received
            .recv_timeout(Duration::from_secs(1))
            .expect("queued event"),
        WifiReconnectEvent::Ipv4Assigned
    );
    assert_eq!(crate::RUNS.load(Ordering::Relaxed), 1);
    drop(prepared);
    drop(maybe_wifi);
    drop(maybe_ip);
    exited
        .recv_timeout(Duration::from_secs(1))
        .expect("worker exits after owned subscriptions release");
    assert_eq!(drops.load(Ordering::Relaxed), 2);
}

#[test]
fn dropping_unused_or_failed_subscription_preparation_never_processes_an_event() {
    // Arrange
    let _exclusive = crate::TEST_LOCK.lock().expect("fixture lock");
    crate::RUNS.store(0, Ordering::Relaxed);
    let prepared = prepare().expect("real preparation");
    let sender = prepared.sender().clone();
    let drops = Arc::new(AtomicU32::new(0));
    let (first_subscription, second_subscription) = subscriptions(&sender, &drops);
    sender
        .send(WifiReconnectEvent::Ipv4Assigned)
        .expect("queued event");
    // Act: setup is abandoned before the production activation transaction.
    drop(first_subscription);
    drop(second_subscription);
    drop(prepared);
    // Assert
    await_cancelled(&sender);
    assert_eq!(crate::RUNS.load(Ordering::Relaxed), 0);
    assert_eq!(drops.load(Ordering::Relaxed), 2);
}

#[test]
fn fallible_preparation_preserves_spawn_errno_and_cannot_create_an_active_owner() {
    // Arrange
    let mut attempts = 0;
    // Act
    let result = prepare_with(|_| {
        attempts += 1;
        Err(io::Error::from_raw_os_error(12))
    });
    // Assert
    assert_eq!(attempts, 1);
    assert_eq!(result.err().expect("spawn error").raw_os_error(), Some(12));
}

#[test]
fn exited_worker_rolls_back_exactly_the_new_owner_subscriptions() {
    // Arrange
    let mut prepared = prepare_with(|_| {
        let handle = std::thread::spawn(|| ());
        while !handle.is_finished() {
            std::thread::yield_now();
        }
        prepared_thread::prepare(|_| Ok(handle))
    })
    .expect("completed fixture thread");
    let drops = Arc::new(AtomicU32::new(0));
    let (wifi, ip) = subscriptions(prepared.sender(), &drops);
    let (mut maybe_wifi, mut maybe_ip) = (None, None);
    // Act
    let activated = prepared.activate_subscribed(&mut maybe_wifi, &mut maybe_ip, wifi, ip);
    // Assert
    assert!(!activated);
    assert!(maybe_wifi.is_none() && maybe_ip.is_none());
    assert_eq!(drops.load(Ordering::Relaxed), 2);
}

#[test]
fn missing_or_invalid_station_configuration_never_attempts_reconnect_reservation() {
    // Arrange
    let mut attempts = 0;
    // Act
    let missing = prepare_credentials_with(CredentialState::<()>::Missing, || {
        attempts += 1;
        Err(io::Error::from_raw_os_error(12))
    });
    let invalid = prepare_credentials_with(CredentialState::<()>::Invalid, || {
        attempts += 1;
        Err(io::Error::from_raw_os_error(12))
    });
    // Assert
    assert!(matches!(missing, Ok(PreparedCredentials::Missing)));
    assert!(matches!(invalid, Ok(PreparedCredentials::Invalid)));
    assert_eq!(attempts, 0);
}

#[test]
fn valid_station_configuration_requires_one_successful_reservation() {
    // Arrange
    let mut attempts = 0;
    // Act
    let result = prepare_credentials_with(CredentialState::Valid(()), || {
        attempts += 1;
        Err(io::Error::from_raw_os_error(12))
    });
    // Assert
    assert_eq!(attempts, 1);
    assert_eq!(
        result
            .err()
            .expect("required reservation fails")
            .raw_os_error(),
        Some(12)
    );
}

#[test]
fn valid_preparation_carries_the_exact_captured_configuration_and_one_worker() {
    // Arrange
    let _exclusive = crate::TEST_LOCK.lock().expect("fixture lock");
    crate::RUNS.store(0, Ordering::Relaxed);
    let mut attempts = 0;
    // Act
    let result = prepare_credentials_with(CredentialState::Valid(37u32), || {
        attempts += 1;
        prepare()
    })
    .expect("prepared station");
    // Assert
    let PreparedCredentials::Valid {
        credentials,
        reconnect,
    } = result
    else {
        panic!("station must retain its worker");
    };
    assert_eq!(credentials, 37);
    assert_eq!(attempts, 1);
    let sender = reconnect.sender().clone();
    drop(reconnect);
    await_cancelled(&sender);
    assert_eq!(crate::RUNS.load(Ordering::Relaxed), 0);
}
