use bitaxe_worker_control::{WorkerControlFrameAccumulator, WorkerControlFrameAccumulatorError};

#[test]
fn reconstructs_one_fragmented_frame_without_exposing_partial_bytes() {
    // Arrange
    let mut accumulator = WorkerControlFrameAccumulator::new();

    // Act
    let first = accumulator
        .push(b"{\"profile\":\"bwg-worker-")
        .expect("first fragment should buffer");
    let second = accumulator
        .push(b"possession/0.1\"}\n")
        .expect("second fragment should complete");

    // Assert
    assert!(first.is_none());
    assert_eq!(
        second.expect("one frame should complete"),
        b"{\"profile\":\"bwg-worker-possession/0.1\"}\n"
    );
}

#[test]
fn rejects_multiple_frames_in_one_buffered_transfer() {
    // Arrange
    let mut accumulator = WorkerControlFrameAccumulator::new();

    // Act
    let result = accumulator.push(b"{}\n{}\n");

    // Assert
    assert_eq!(
        result.expect_err("multiple frames must fail"),
        WorkerControlFrameAccumulatorError::MultipleFrames
    );
}

#[test]
fn rejects_an_oversized_unterminated_frame() {
    // Arrange
    let mut accumulator = WorkerControlFrameAccumulator::new();
    let oversized = vec![b'A'; 65_537];

    // Act
    let result = accumulator.push(&oversized);

    // Assert
    assert_eq!(
        result.expect_err("oversized frame must fail"),
        WorkerControlFrameAccumulatorError::Oversized
    );
}
