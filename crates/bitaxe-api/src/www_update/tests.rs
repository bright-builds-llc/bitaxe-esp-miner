use super::{
    admit_www_body, www_protocol_error_response, www_success_response, www_write_error_response,
    WwwReceive, WwwStep, WwwUpdateStatus, WwwWritePlan, WWW_ERASE_STEP_BYTES,
    WWW_RECEIVE_CHUNK_BYTES,
};

const ULTRA_205_WWW_BYTES: usize = 3 * 1024 * 1024;

fn exact_plan(size: usize) -> WwwWritePlan {
    admit_www_body(size, Some(size)).expect("an exact-size body is admitted")
}

#[test]
fn missing_partition_is_rejected_with_upstream_copy() {
    // Arrange
    let content_len = ULTRA_205_WWW_BYTES;

    // Act
    let response = admit_www_body(content_len, None).expect_err("no partition");

    // Assert
    assert_eq!(response.status, 500);
    assert_eq!(response.body, "WWW partition not found");
}

#[test]
fn oversized_body_is_rejected_with_upstream_copy() {
    // Arrange
    let content_len = ULTRA_205_WWW_BYTES + 1;

    // Act
    let response = admit_www_body(content_len, Some(ULTRA_205_WWW_BYTES)).expect_err("too large");

    // Assert
    assert_eq!(response.status, 400);
    assert_eq!(response.body, "File provided is too large for device");
}

#[test]
fn undersized_body_is_rejected_before_any_erase() {
    // Arrange
    let content_len = ULTRA_205_WWW_BYTES - 1;

    // Act
    let response = admit_www_body(content_len, Some(ULTRA_205_WWW_BYTES)).expect_err("too small");

    // Assert
    assert_eq!(response.status, 400);
    assert_eq!(response.body, "File provided is too small for device");
}

#[test]
fn empty_body_is_rejected_before_any_erase() {
    // Act
    let response = admit_www_body(0, Some(ULTRA_205_WWW_BYTES)).expect_err("empty");

    // Assert
    assert_eq!(response.body, "File provided is too small for device");
}

#[test]
fn exact_size_body_is_admitted() {
    // Act
    let plan = admit_www_body(ULTRA_205_WWW_BYTES, Some(ULTRA_205_WWW_BYTES));

    // Assert
    assert_eq!(plan.map(WwwWritePlan::size), Ok(ULTRA_205_WWW_BYTES));
}

#[test]
fn erase_covers_the_ultra_205_partition_in_64_kib_steps() {
    // Arrange
    let plan = exact_plan(ULTRA_205_WWW_BYTES);

    // Act
    let ranges: Vec<_> = plan.erase_ranges().collect();

    // Assert
    assert_eq!(ranges.len(), 48);
    assert_eq!(ranges.first(), Some(&(0, WWW_ERASE_STEP_BYTES)));
    assert_eq!(
        ranges.last(),
        Some(&(
            ULTRA_205_WWW_BYTES - WWW_ERASE_STEP_BYTES,
            WWW_ERASE_STEP_BYTES
        ))
    );
}

#[test]
fn erase_clamps_the_final_step_to_the_partition_end() {
    // Arrange
    let plan = exact_plan(WWW_ERASE_STEP_BYTES + 4096);

    // Act
    let ranges: Vec<_> = plan.erase_ranges().collect();

    // Assert
    assert_eq!(
        ranges,
        vec![(0, WWW_ERASE_STEP_BYTES), (WWW_ERASE_STEP_BYTES, 4096)]
    );
}

#[test]
fn complete_stream_writes_contiguously_to_the_partition_end() {
    // Arrange
    let plan = exact_plan(ULTRA_205_WWW_BYTES);
    let mut cursor = plan.cursor();
    let mut expected_offset = 0;

    // Act
    while !cursor.is_complete() {
        let step = cursor.on_receive(WwwReceive::Data(cursor.next_read_len()));
        let WwwStep::Write { offset, len } = step else {
            panic!("data must produce a write, got {step:?}");
        };
        assert_eq!(offset, expected_offset);
        expected_offset += len;
        cursor.wrote(len);
    }

    // Assert
    assert_eq!(expected_offset, ULTRA_205_WWW_BYTES);
}

#[test]
fn short_reads_still_write_at_the_received_position() {
    // Arrange
    let mut cursor = exact_plan(4000).cursor();
    cursor.wrote(700);

    // Act
    let step = cursor.on_receive(WwwReceive::Data(300));

    // Assert
    assert_eq!(
        step,
        WwwStep::Write {
            offset: 700,
            len: 300
        }
    );
    assert_eq!(cursor.next_read_len(), WWW_RECEIVE_CHUNK_BYTES);
}

#[test]
fn timeout_retries_without_moving_the_cursor() {
    // Arrange
    let cursor = exact_plan(4000).cursor();

    // Act
    let step = cursor.on_receive(WwwReceive::Timeout);

    // Assert
    assert_eq!(step, WwwStep::Retry);
    assert_eq!(cursor.remaining(), 4000);
}

#[test]
fn interrupted_stream_stops_with_protocol_error() {
    // Arrange
    let mut cursor = exact_plan(4000).cursor();
    cursor.wrote(1000);

    // Act
    let step = cursor.on_receive(WwwReceive::Failed(-1));

    // Assert
    assert_eq!(step, WwwStep::ProtocolError { code: -1 });
    assert!(!cursor.is_complete());
}

#[test]
fn closed_stream_is_a_protocol_error() {
    // Arrange
    let cursor = exact_plan(4000).cursor();

    // Act
    let step = cursor.on_receive(WwwReceive::Data(0));

    // Assert
    assert_eq!(step, WwwStep::ProtocolError { code: 0 });
}

#[test]
fn progress_is_computed_before_each_chunk_is_counted() {
    // Arrange
    let mut cursor = exact_plan(4000).cursor();

    // Act
    let percents: Vec<_> = (0..4)
        .map(|_| match cursor.wrote(1000).status {
            WwwUpdateStatus::Working { percent } => percent,
            other => panic!("unexpected status {other:?}"),
        })
        .collect();

    // Assert
    assert_eq!(percents, vec![0, 25, 50, 75]);
    assert!(cursor.is_complete());
}

#[test]
fn adapter_yields_after_every_sixteen_chunks() {
    // Arrange
    let mut cursor = exact_plan(32 * WWW_RECEIVE_CHUNK_BYTES).cursor();

    // Act
    let yields: Vec<_> = (1..=32)
        .filter(|_| cursor.wrote(WWW_RECEIVE_CHUNK_BYTES).yield_now)
        .collect();

    // Assert
    assert_eq!(yields, vec![16, 32]);
}

#[test]
fn status_text_matches_upstream_labels() {
    // Act
    let labels = [
        WwwUpdateStatus::Starting,
        WwwUpdateStatus::Working { percent: 42 },
        WwwUpdateStatus::ProtocolError,
        WwwUpdateStatus::WriteError,
        WwwUpdateStatus::Finished,
    ]
    .map(WwwUpdateStatus::status_text);

    // Assert
    assert_eq!(
        labels,
        [
            "Starting...",
            "Working (42%)",
            "Protocol Error",
            "Write Error",
            "Finished...",
        ]
    );
}

#[test]
fn responses_match_upstream_copy() {
    // Act
    let responses = [
        www_success_response(),
        www_protocol_error_response(),
        www_write_error_response(),
    ];

    // Assert
    assert_eq!(
        responses.map(|response| (response.status, response.body)),
        [
            (200, "WWW update complete\n"),
            (500, "Protocol Error"),
            (500, "Write Error"),
        ]
    );
    assert!(responses
        .iter()
        .all(|response| response.content_type == Some("text/plain")));
}
