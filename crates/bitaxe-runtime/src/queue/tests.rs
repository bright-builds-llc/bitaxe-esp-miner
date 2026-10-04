use super::*;
use std::thread;

#[test]
fn values_arrive_in_send_order() {
    // Arrange
    let (sender, receiver) = sync_channel(4);

    // Act
    for value in 1_u8..=3 {
        sender.send(value).expect("receiver is connected");
    }

    // Assert
    assert_eq!(receiver.try_recv(), Ok(1));
    assert_eq!(receiver.recv(), Ok(2));
    assert_eq!(receiver.recv_timeout(Duration::from_millis(1)), Ok(3));
    assert_eq!(receiver.try_recv(), Err(TryRecvError::Empty));
}

#[test]
fn a_full_bounded_queue_refuses_try_send() {
    // Arrange
    let (sender, _receiver) = sync_channel(1);
    sender.try_send(1_u8).expect("space for one value");

    // Act
    let outcome = sender.try_send(2);

    // Assert
    assert_eq!(outcome, Err(TrySendError::Full(2)));
}

#[test]
fn a_blocked_send_completes_once_the_receiver_makes_space() {
    // Arrange
    let (sender, receiver) = sync_channel(1);
    sender.send(1_u8).expect("space for one value");
    let blocked = thread::spawn(move || sender.send(2));

    // Act
    let first = receiver.recv_timeout(Duration::from_secs(5));
    let second = receiver.recv_timeout(Duration::from_secs(5));

    // Assert
    assert_eq!(first, Ok(1));
    assert_eq!(second, Ok(2));
    assert_eq!(blocked.join().expect("sender thread"), Ok(()));
}

#[test]
fn an_unbounded_queue_never_reports_full() {
    // Arrange
    let (sender, receiver) = channel();

    // Act
    for value in 0..64_u8 {
        sender.try_send(value).expect("unbounded");
    }

    // Assert
    let received: Vec<u8> = (0..64).map_while(|_| receiver.try_recv().ok()).collect();
    assert_eq!(received, (0..64).collect::<Vec<u8>>());
}

#[test]
fn a_zero_capacity_behaves_as_one() {
    // Arrange
    let (sender, _receiver) = sync_channel(0);

    // Act
    let first = sender.try_send(1_u8);
    let second = sender.try_send(2);

    // Assert
    assert_eq!(first, Ok(()));
    assert_eq!(second, Err(TrySendError::Full(2)));
}

#[test]
fn queued_values_drain_before_the_last_sender_disconnects() {
    // Arrange
    let (sender, receiver) = channel();
    let clone = sender.clone();
    sender.send(1_u8).expect("receiver is connected");
    drop(sender);

    // Act
    let while_clone_lives = receiver.try_recv();
    let empty_while_clone_lives = receiver.try_recv();
    drop(clone);

    // Assert
    assert_eq!(while_clone_lives, Ok(1));
    assert_eq!(empty_while_clone_lives, Err(TryRecvError::Empty));
    assert_eq!(receiver.try_recv(), Err(TryRecvError::Disconnected));
    assert_eq!(receiver.recv(), Err(RecvError));
}

#[test]
fn dropping_the_last_sender_wakes_a_waiting_receiver() {
    // Arrange
    let (sender, receiver) = sync_channel::<u8>(1);
    let dropper = thread::spawn(move || drop(sender));

    // Act
    let outcome = receiver.recv_timeout(Duration::from_secs(5));

    // Assert
    dropper.join().expect("dropper thread");
    assert_eq!(outcome, Err(RecvTimeoutError::Disconnected));
}

#[test]
fn an_idle_receiver_times_out() {
    // Arrange
    let (_sender, receiver) = sync_channel::<u8>(1);

    // Act
    let outcome = receiver.recv_timeout(Duration::from_millis(5));

    // Assert
    assert_eq!(outcome, Err(RecvTimeoutError::Timeout));
}

#[test]
fn dropping_the_receiver_fails_sends_and_wakes_a_blocked_sender() {
    // Arrange
    let (sender, receiver) = sync_channel(1);
    sender.send(1_u8).expect("space for one value");
    let probe = sender.clone();
    let blocked = thread::spawn(move || sender.send(2));

    // Act
    drop(receiver);

    // Assert
    assert_eq!(blocked.join().expect("sender thread"), Err(SendError(2)));
    assert_eq!(probe.try_send(3), Err(TrySendError::Disconnected(3)));
}
