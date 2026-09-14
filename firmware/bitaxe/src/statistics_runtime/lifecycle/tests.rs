use super::*;
use std::sync::mpsc;
use std::time::Duration;

struct Exit(mpsc::Sender<()>);
impl Drop for Exit {
    fn drop(&mut self) {
        let _receiver_may_have_closed = self.0.send(());
    }
}

fn fixture() -> (Prepared, mpsc::Receiver<()>, mpsc::Receiver<()>) {
    let (sample, sampled) = mpsc::channel();
    let (exit, exited) = mpsc::channel();
    let prepared = prepare(|gate| {
        thread::Builder::new().spawn(move || {
            let _exit = Exit(exit);
            if gate.wait() {
                sample.send(()).expect("sampling receiver");
            }
        })
    })
    .expect("prepare");
    (prepared, sampled, exited)
}

#[test]
fn allocated_owner_does_not_sample_until_the_original_activation_boundary() {
    // Arrange
    let (mut prepared, sampled, exited) = fixture();
    // Act / Assert
    assert!(matches!(
        sampled.recv_timeout(Duration::from_millis(30)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    assert!(prepared.activate());
    sampled
        .recv_timeout(Duration::from_secs(1))
        .expect("sample after activation");
    exited
        .recv_timeout(Duration::from_secs(1))
        .expect("fixture owner exits");
}

#[test]
fn spurious_unparks_cannot_activate_a_prepared_owner() {
    // Arrange
    let (prepared, sampled, exited) = fixture();
    // Act
    for _ in 0..3 {
        prepared
            .maybe_thread
            .as_ref()
            .expect("owner")
            .thread()
            .unpark();
    }
    // Assert
    assert!(matches!(
        sampled.recv_timeout(Duration::from_millis(30)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    drop(prepared);
    exited
        .recv_timeout(Duration::from_secs(1))
        .expect("cancelled owner exits");
    assert!(sampled.recv().is_err());
}

#[test]
fn dropping_unused_preparation_exits_without_sampling() {
    // Arrange
    let (prepared, sampled, exited) = fixture();
    // Act
    drop(prepared);
    // Assert
    exited
        .recv_timeout(Duration::from_secs(1))
        .expect("cancelled owner exits");
    assert!(sampled.recv().is_err());
}

#[test]
fn failed_spawn_returns_the_original_errno_without_an_owner() {
    // Arrange / Act
    let result = prepare(|_| Err(io::Error::from_raw_os_error(12)));
    // Assert
    assert_eq!(
        result.err().expect("spawn failure").raw_os_error(),
        Some(12)
    );
}

#[test]
fn an_owner_that_exited_before_activation_is_not_reported_active() {
    // Arrange
    let (exit, exited) = mpsc::channel();
    let mut prepared = prepare(|_| {
        thread::Builder::new().spawn(move || {
            exit.send(()).expect("exit receiver");
        })
    })
    .expect("prepare");
    exited
        .recv_timeout(Duration::from_secs(1))
        .expect("owner ran");
    while !prepared.maybe_thread.as_ref().expect("owner").is_finished() {
        thread::yield_now();
    }
    // Act / Assert
    assert!(!prepared.activate());
}
