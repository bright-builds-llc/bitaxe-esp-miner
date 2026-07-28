use serde::Deserialize;

use crate::logs::{
    checked_retained_pair_bytes, log_download_headers, AcceptedStateReplayCadence,
    RawLogStreamPlanner, RetainedLogBuffer, RetainedPair, RetainedPairError, RuntimeHeartbeatModel,
    ACCEPTED_STATE_MONITOR_ATTACHMENT_MS, ACCEPTED_STATE_REPLAY_INTERVAL_MS,
    ACCEPTED_STATE_REPLAY_WINDOW_MS, ACCEPTED_STATE_RESTORE_WATCH_MS, DOWNLOAD_CONTENT_DISPOSITION,
    DOWNLOAD_CONTENT_TYPE, LOG_CHUNK_BYTES, LOG_RETENTION_BYTES,
};

const HEARTBEAT_SESSION: [u32; 4] = [0, 1, u32::MAX, 0x1234_abcd];

fn retained_text(buffer: &RetainedLogBuffer) -> String {
    buffer.download_chunks().concat()
}

#[test]
fn retained_pair_rejects_empty_records() {
    // Arrange
    let marker = "";
    let runtime_health = "runtime_health status=healthy";

    // Act
    let result = RetainedPair::try_new(marker, runtime_health);

    // Assert
    assert_eq!(result, Err(RetainedPairError::EmptyRecord));
}

#[test]
fn retained_pair_rejects_embedded_line_breaks() {
    // Arrange
    let marker = "operator_snapshot revision=1\npartial=true";
    let runtime_health = "runtime_health status=healthy";

    // Act
    let result = RetainedPair::try_new(marker, runtime_health);

    // Assert
    assert_eq!(result, Err(RetainedPairError::EmbeddedLineBreak));
}

#[test]
fn retained_pair_normalizes_exactly_one_newline_per_record() {
    // Arrange
    let marker = "operator_snapshot revision=1\n\n";
    let runtime_health = "runtime_health status=healthy\r\n";

    // Act
    let pair = RetainedPair::try_new(marker, runtime_health)
        .expect("complete records should construct a retained pair");

    // Assert
    assert_eq!(pair.marker(), "operator_snapshot revision=1\n");
    assert_eq!(pair.runtime_health(), "runtime_health status=healthy\n");
    assert_eq!(
        pair.required_bytes(),
        pair.marker().len() + pair.runtime_health().len()
    );
}

#[test]
fn retained_pair_size_rejects_checked_arithmetic_overflow() {
    // Arrange
    let marker_bytes = usize::MAX;
    let runtime_health_bytes = 1;

    // Act
    let result = checked_retained_pair_bytes(marker_bytes, runtime_health_bytes);

    // Assert
    assert_eq!(result, Err(RetainedPairError::SizeOverflow));
}

#[test]
fn retained_pair_rejects_unavailable_storage_without_mutation() {
    // Arrange
    let pair = RetainedPair::try_new(
        "operator_snapshot revision=1",
        "runtime_health status=healthy",
    )
    .expect("complete records should construct a retained pair");
    let mut buffer = RetainedLogBuffer::empty();

    // Act
    let result = buffer.try_append_pair(&pair);

    // Assert
    assert_eq!(result, Err(RetainedPairError::StorageUnavailable));
    assert_eq!(buffer.total_written(), 0);
    assert_eq!(retained_text(&buffer), "");
}

#[test]
fn retained_pair_rejects_capacity_one_byte_short_without_partial_append() {
    // Arrange
    let pair = RetainedPair::try_new(
        "operator_snapshot revision=1",
        "runtime_health status=healthy",
    )
    .expect("complete records should construct a retained pair");
    let mut buffer = RetainedLogBuffer::with_capacity(pair.required_bytes() - 1);

    // Act
    let result = buffer.try_append_pair(&pair);

    // Assert
    assert_eq!(result, Err(RetainedPairError::PairExceedsCapacity));
    assert_eq!(buffer.total_written(), 0);
    assert!(!retained_text(&buffer).contains("operator_snapshot"));
    assert!(!retained_text(&buffer).contains("runtime_health"));
}

#[test]
fn retained_pair_failure_preserves_preexisting_bytes_and_counter() {
    // Arrange
    let pair = RetainedPair::try_new(
        "operator_snapshot revision=1",
        "runtime_health status=healthy",
    )
    .expect("complete records should construct a retained pair");
    let mut buffer = RetainedLogBuffer::with_capacity(pair.required_bytes() - 1);
    buffer.append("preexisting\n");
    let before = buffer.clone();

    // Act
    let result = buffer.try_append_pair(&pair);

    // Assert
    assert_eq!(result, Err(RetainedPairError::PairExceedsCapacity));
    assert_eq!(buffer, before);
    assert_eq!(retained_text(&buffer), "preexisting\n");
}

#[test]
fn retained_pair_appends_marker_then_health_as_complete_lines() {
    // Arrange
    let pair = RetainedPair::try_new(
        "operator_snapshot revision=1\n",
        "runtime_health status=healthy",
    )
    .expect("complete records should construct a retained pair");
    let mut buffer = RetainedLogBuffer::with_capacity(pair.required_bytes());

    // Act
    let result = buffer.try_append_pair(&pair);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(buffer.total_written(), pair.required_bytes() as u64);
    assert_eq!(
        retained_text(&buffer),
        "operator_snapshot revision=1\nruntime_health status=healthy\n"
    );
}

#[test]
fn runtime_heartbeat_renders_exact_redacted_marker() {
    // Arrange
    let mut model = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);

    // Act
    let sample = model
        .maybe_take_due(1_000)
        .expect("first heartbeat should be due");

    // Assert
    assert_eq!(
        sample.marker(),
        "runtime_heartbeat session=0000000000000001ffffffff1234abcd sequence=0 uptime_ms=1000 cadence_ms=1000 listener_armed=false redacted=true"
    );
}

#[test]
fn runtime_heartbeat_is_first_due_at_one_second() {
    // Arrange
    let mut model = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);

    // Act
    let before = model.maybe_take_due(999);
    let due = model.maybe_take_due(1_000);

    // Assert
    assert!(before.is_none());
    assert!(due.is_some());
}

#[test]
fn runtime_heartbeat_labels_cadence_at_two_minute_boundary() {
    // Arrange
    let mut before = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);
    let mut boundary = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);
    let mut after = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);

    // Act
    let before_marker = before
        .maybe_take_due(119_999)
        .expect("sample should be due")
        .marker();
    let boundary_marker = boundary
        .maybe_take_due(120_000)
        .expect("sample should be due")
        .marker();
    let after_marker = after
        .maybe_take_due(120_001)
        .expect("sample should be due")
        .marker();

    // Assert
    assert!(before_marker.contains("cadence_ms=1000"));
    assert!(boundary_marker.contains("cadence_ms=1000"));
    assert!(after_marker.contains("cadence_ms=10000"));
}

#[test]
fn runtime_heartbeat_boundary_schedules_steady_deadline() {
    // Arrange
    let mut model = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);

    // Act
    let sample = model.maybe_take_due(120_000);

    // Assert
    assert!(sample.is_some());
    assert_eq!(model.next_deadline_ms(), 130_000);
}

#[test]
fn runtime_heartbeat_delayed_wakeup_coalesces_missed_ticks() {
    // Arrange
    let mut model = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);

    // Act
    let delayed = model
        .maybe_take_due(75_432)
        .expect("delayed sample should be due");
    let duplicate = model.maybe_take_due(75_432);

    // Assert
    assert!(delayed.marker().contains("sequence=0 uptime_ms=75432"));
    assert!(duplicate.is_none());
    assert_eq!(model.next_deadline_ms(), 76_432);
}

#[test]
fn runtime_heartbeat_sequence_increments_once_per_due_sample() {
    // Arrange
    let mut model = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);

    // Act
    let first = model
        .maybe_take_due(1_000)
        .expect("first sample should be due");
    let second = model
        .maybe_take_due(2_000)
        .expect("second sample should be due");

    // Assert
    assert!(first.marker().contains("sequence=0"));
    assert!(second.marker().contains("sequence=1"));
}

#[test]
fn runtime_heartbeat_listener_state_only_latches_true() {
    // Arrange
    let mut model = RuntimeHeartbeatModel::new(HEARTBEAT_SESSION);
    let before = model
        .maybe_take_due(1_000)
        .expect("first sample should be due");

    // Act
    model.arm_listener();
    model.arm_listener();
    let after = model
        .maybe_take_due(2_000)
        .expect("second sample should be due");

    // Assert
    assert!(before.marker().contains("listener_armed=false"));
    assert!(after.marker().contains("listener_armed=true"));
}

#[derive(Debug, Deserialize)]
struct LogFixtureCases {
    download_headers: HeaderFixture,
    raw_stream: RawStreamFixture,
}

#[derive(Debug, Deserialize)]
struct HeaderFixture {
    content_type: String,
    content_disposition: String,
}

#[derive(Debug, Deserialize)]
struct RawStreamFixture {
    payload: String,
    json_enveloped: bool,
}

fn fixture_cases() -> LogFixtureCases {
    serde_json::from_str(include_str!("../../fixtures/api/log-buffer-cases.json"))
        .expect("log fixture cases should parse")
}

#[test]
fn retained_download_uses_text_headers_chunks_from_beginning_and_empty_terminal_chunk() {
    // Arrange
    let fixture = fixture_cases();
    let mut buffer = RetainedLogBuffer::new();
    buffer.append(&"a".repeat(LOG_CHUNK_BYTES));
    buffer.append("tail");

    // Act
    let headers = log_download_headers();
    let chunks = buffer.download_chunks();

    // Assert
    assert_eq!(headers.content_type, DOWNLOAD_CONTENT_TYPE);
    assert_eq!(headers.content_disposition, DOWNLOAD_CONTENT_DISPOSITION);
    assert_eq!(headers.content_type, fixture.download_headers.content_type);
    assert_eq!(
        headers.content_disposition,
        fixture.download_headers.content_disposition
    );
    assert_eq!(chunks[0].len(), LOG_CHUNK_BYTES);
    assert_eq!(chunks[1], "tail");
    assert_eq!(chunks[2], "");
}

#[test]
fn clamped_readers_resync_to_next_newline_within_bounded_scan() {
    // Arrange
    let mut buffer = RetainedLogBuffer::new();
    let discarded_prefix = "discarded-";
    let retained_prefix = "stale partial line\nkept line\n";
    let tail = "x".repeat(LOG_RETENTION_BYTES - retained_prefix.len());
    buffer.append(discarded_prefix);
    buffer.append(retained_prefix);
    buffer.append(&tail);
    let mut cursor = 0;

    // Act
    let chunk = buffer.read_absolute_chunk(&mut cursor, LOG_CHUNK_BYTES);

    // Assert
    assert!(!chunk.starts_with("stale partial line"));
    assert!(chunk.starts_with("kept line\n"));
}

#[test]
fn retained_buffer_uses_configured_capacity_for_clamping() {
    // Arrange
    let mut buffer = RetainedLogBuffer::with_capacity(20);
    buffer.append("discarded-line\n");
    buffer.append("kept-line\n");
    buffer.append("tail");
    let mut cursor = 0;

    // Act
    let chunk = buffer.read_absolute_chunk(&mut cursor, LOG_CHUNK_BYTES);

    // Assert
    assert_eq!(buffer.capacity(), 20);
    assert!(!chunk.contains("discarded-line"));
    assert!(chunk.starts_with("kept-line\n"));
}

#[test]
fn empty_retained_buffer_drops_bytes_without_panicking() {
    // Arrange
    let mut buffer = RetainedLogBuffer::empty();
    let mut cursor = 0;

    // Act
    buffer.append("not retained\n");
    let chunk = buffer.read_absolute_chunk(&mut cursor, LOG_CHUNK_BYTES);

    // Assert
    assert_eq!(buffer.capacity(), 0);
    assert_eq!(buffer.total_written(), 13);
    assert_eq!(chunk, "");
}

#[test]
fn raw_ws_client_baseline_starts_at_current_end_not_retained_history() {
    // Arrange
    let mut buffer = RetainedLogBuffer::new();
    buffer.append("retained old line\n");
    let mut stream = RawLogStreamPlanner::new(&buffer);
    stream.set_active_client_count(1, &buffer);
    buffer.append("new live line\n");

    // Act
    let chunks = stream.drain_raw_chunks(&buffer);

    // Assert
    assert_eq!(chunks, vec!["new live line\n"]);
}

#[test]
fn raw_ws_hibernates_without_clients_and_sends_no_backlog_to_later_clients() {
    // Arrange
    let mut buffer = RetainedLogBuffer::new();
    let mut stream = RawLogStreamPlanner::new(&buffer);
    stream.set_active_client_count(0, &buffer);
    buffer.append("not delivered while idle\n");
    let idle_chunks = stream.drain_raw_chunks(&buffer);
    stream.set_active_client_count(1, &buffer);

    // Act
    let reconnect_chunks = stream.drain_raw_chunks(&buffer);

    // Assert
    assert!(idle_chunks.is_empty());
    assert!(reconnect_chunks.is_empty());
}

#[test]
fn raw_ws_additional_client_connect_preserves_pending_live_chunks() {
    // Arrange
    let mut buffer = RetainedLogBuffer::new();
    buffer.append("retained old line\n");
    let mut stream = RawLogStreamPlanner::new(&buffer);
    stream.set_active_client_count(1, &buffer);
    buffer.append("pending live line\n");
    stream.set_active_client_count(2, &buffer);

    // Act
    let chunks = stream.drain_raw_chunks(&buffer);

    // Assert
    assert_eq!(chunks, vec!["pending live line\n"]);
}

#[test]
fn raw_ws_active_client_drop_with_empty_buffer_does_not_replay_history() {
    // Arrange
    let mut buffer = RetainedLogBuffer::new();
    buffer.append("retained old line\n");
    let mut stream = RawLogStreamPlanner::new(&buffer);
    stream.set_active_client_count(2, &buffer);
    buffer.append("delivered live line\n");
    let delivered_chunks = stream.drain_raw_chunks(&buffer);
    stream.set_active_client_count(1, &RetainedLogBuffer::new());
    buffer.append("next live line\n");

    // Act
    let chunks = stream.drain_raw_chunks(&buffer);

    // Assert
    assert_eq!(delivered_chunks, vec!["delivered live line\n"]);
    assert_eq!(chunks, vec!["next live line\n"]);
}

#[test]
fn raw_ws_chunks_are_text_payloads_without_json_envelope() {
    // Arrange
    let fixture = fixture_cases();
    let mut buffer = RetainedLogBuffer::new();
    let mut stream = RawLogStreamPlanner::new(&buffer);
    stream.set_active_client_count(1, &buffer);
    buffer.append(&fixture.raw_stream.payload);

    // Act
    let chunks = stream.drain_raw_chunks(&buffer);

    // Assert
    assert_eq!(chunks, vec![fixture.raw_stream.payload]);
    assert!(!fixture.raw_stream.json_enveloped);
    assert!(!chunks[0].trim_start().starts_with('{'));
}

#[test]
fn accepted_state_replay_selects_only_exact_complete_first_token_lines() {
    // Arrange
    let mut buffer = RetainedLogBuffer::with_capacity(16_384);
    buffer.append("noise accepted_state_snapshot stage=post_enumerate redacted=true\n");
    buffer.append("accepted_state_snapshot_extra stage=post_enumerate redacted=true\n");
    buffer.append("accepted_state_snapshot stage=post_enumerate redacted=true\n");
    buffer.append("accepted_state_snapshot stage=post_mining_ready redacted=true");
    buffer.append(&"x".repeat(LOG_CHUNK_BYTES));
    buffer.append("accepted_state_snapshot stage=post_first_work redacted=true\n");

    // Act
    let lines = buffer.complete_lines_with_first_token("accepted_state_snapshot");

    // Assert
    assert_eq!(
        lines,
        ["accepted_state_snapshot stage=post_enumerate redacted=true"]
    );
}

#[test]
fn accepted_state_replay_preserves_equivalent_duplicates_for_validation() {
    // Arrange
    let marker = "accepted_state_snapshot stage=post_max_baud redacted=true\n";
    let mut buffer = RetainedLogBuffer::with_capacity(2_048);
    buffer.append(marker);
    buffer.append(marker);

    // Act
    let lines = buffer.complete_lines_with_first_token("accepted_state_snapshot");

    // Assert
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], lines[1]);
}

#[test]
fn accepted_state_replay_excludes_secret_bearing_noise() {
    // Arrange
    let mut buffer = RetainedLogBuffer::with_capacity(2_048);
    buffer.append("poolPassword=do-not-replay\n");
    buffer.append("wifi-credentials=do-not-replay\n");
    buffer.append("accepted_state_snapshot stage=post_first_work redacted=true\n");

    // Act
    let lines = buffer.complete_lines_with_first_token("accepted_state_snapshot");

    // Assert
    assert_eq!(lines.len(), 1);
    assert!(!lines.join("\n").contains("do-not-replay"));
}

#[test]
fn accepted_state_replay_cadence_is_not_due_before_arming_time() {
    // Arrange
    let mut cadence = AcceptedStateReplayCadence::armed(1_000);

    // Act
    let due = cadence.take_due(999);

    // Assert
    assert!(!due);
}

#[test]
fn accepted_state_replay_cadence_is_due_at_arming_time() {
    // Arrange
    let mut cadence = AcceptedStateReplayCadence::armed(1_000);

    // Act
    let due = cadence.take_due(1_000);

    // Assert
    assert!(due);
}

#[test]
fn accepted_state_replay_cadence_repeats_only_after_fixed_interval() {
    // Arrange
    let mut cadence = AcceptedStateReplayCadence::armed(1_000);
    assert!(cadence.take_due(1_000));

    // Act
    let repeated_immediately = cadence.take_due(1_000);
    let repeated_before_interval = cadence.take_due(1_000 + ACCEPTED_STATE_REPLAY_INTERVAL_MS - 1);
    let repeated_at_interval = cadence.take_due(1_000 + ACCEPTED_STATE_REPLAY_INTERVAL_MS);

    // Assert
    assert!(!repeated_immediately);
    assert!(!repeated_before_interval);
    assert!(repeated_at_interval);
}

#[test]
fn accepted_state_replay_schedule_preserves_post_reattach_opportunity_before_expiry() {
    // Arrange
    let mut cadence = AcceptedStateReplayCadence::armed(0);
    let monitor_ready_ms = ACCEPTED_STATE_RESTORE_WATCH_MS + ACCEPTED_STATE_MONITOR_ATTACHMENT_MS;
    let next_replay_ms = monitor_ready_ms + ACCEPTED_STATE_REPLAY_INTERVAL_MS;

    // Act
    for now_ms in (0..=monitor_ready_ms).step_by(ACCEPTED_STATE_REPLAY_INTERVAL_MS as usize) {
        assert!(cadence.take_due(now_ms));
    }
    let due_after_monitor_reserve = cadence.take_due(next_replay_ms);
    for now_ms in ((next_replay_ms + ACCEPTED_STATE_REPLAY_INTERVAL_MS)
        ..ACCEPTED_STATE_REPLAY_WINDOW_MS)
        .step_by(ACCEPTED_STATE_REPLAY_INTERVAL_MS as usize)
    {
        assert!(cadence.take_due(now_ms));
    }
    let due_at_window_end = cadence.take_due(ACCEPTED_STATE_REPLAY_WINDOW_MS);

    // Assert
    assert_eq!(ACCEPTED_STATE_REPLAY_WINDOW_MS, 1_880_000);
    assert_eq!(ACCEPTED_STATE_REPLAY_INTERVAL_MS, 10_000);
    assert_eq!(ACCEPTED_STATE_RESTORE_WATCH_MS, 1_800_000);
    assert_eq!(ACCEPTED_STATE_MONITOR_ATTACHMENT_MS, 60_000);
    assert_eq!(next_replay_ms, 1_870_000);
    assert!(next_replay_ms < ACCEPTED_STATE_REPLAY_WINDOW_MS);
    assert!(due_after_monitor_reserve);
    assert!(!due_at_window_end);
}
