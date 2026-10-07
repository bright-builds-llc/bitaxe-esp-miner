use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use bitaxe_api::{ApiSnapshot, OperatorSnapshotRevision, SystemInfoWire};
use tungstenite::Message;

use super::judge::{
    clock_origin, judge, ClockObservation, ExpectedPackage, JournalLine, WINDOW_MS, WORK_GATE_MS,
};
use super::sample::{SoakSample, SoakTransport};
use super::*;

const ORIGIN: u64 = 1_000_000;

fn sample(active_ms: i64, sequence: u64) -> SoakSample {
    let active = (0..WORK_GATE_MS as i64).contains(&active_ms);
    SoakSample {
        boot_session: "session".into(),
        boot_ordinal: 7,
        source_commit: "a".repeat(40),
        app_elf_sha256: "b".repeat(64),
        source_dirty: false,
        mining_active: active,
        mining_paused: !active,
        start_mining_on_boot: false,
        safety_valid: true,
        maybe_watchdog_feed_sequence: Some(sequence),
        maybe_checkpoint_sequence: Some(sequence),
        shares_accepted: sequence / 100,
        shares_rejected: 0,
        revision: sequence,
        maybe_pool_matches_initial: None,
    }
}

fn line(transport: SoakTransport, active_ms: i64, sample: SoakSample) -> JournalLine {
    JournalLine {
        schema: JOURNAL_SCHEMA.into(),
        host_unix_ms: (ORIGIN as i64 + active_ms) as u64,
        maybe_transport: Some(transport),
        event: "sample".into(),
        sample: Some(sample),
        detail: None,
    }
}

/// A complete soak: WebSocket every second, HTTP every two seconds, from 10 s before work to 50 s after.
fn complete() -> Vec<JournalLine> {
    let mut lines = Vec::new();
    for step in -10..=650_i64 {
        let active_ms = step * 1_000;
        let sequence = (step + 20) as u64;
        lines.push(line(
            SoakTransport::Websocket,
            active_ms,
            sample(active_ms, sequence),
        ));
        if step % 2 == 0 {
            let mut http = sample(active_ms + 1, sequence);
            http.maybe_pool_matches_initial = Some(true);
            lines.push(line(SoakTransport::Http, active_ms + 1, http));
        }
    }
    lines
}

fn clock() -> Vec<ClockObservation> {
    [(10_000, 10_000), (300_400, 300_000), (590_200, 590_000)]
        .map(|(after, active_ms)| ClockObservation {
            observed_unix_ms: ORIGIN + after,
            active_ms,
        })
        .to_vec()
}

fn expected() -> ExpectedPackage {
    ExpectedPackage {
        source_commit: "a".repeat(40),
        app_elf_sha256: "b".repeat(64),
    }
}

#[test]
fn a_complete_soak_credits_all_twenty_windows_and_passes() {
    // Arrange
    let lines = complete();
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert_eq!(judgement.failures, Vec::<&str>::new());
    assert!(judgement.passed);
    assert_eq!(judgement.windows.len(), 20);
    assert!(judgement.max_websocket_gap_ms <= 1_000);
}

#[test]
fn a_websocket_gap_over_five_seconds_fails() {
    // Arrange
    let lines: Vec<_> = complete()
        .into_iter()
        .filter(|line| {
            let active = line.host_unix_ms as i64 - ORIGIN as i64;
            !(line.maybe_transport == Some(SoakTransport::Websocket)
                && (150_001..156_000).contains(&active))
        })
        .collect();
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"websocket_gap_exceeded"));
    assert_eq!(judgement.max_websocket_gap_ms, 6_000);
}

#[test]
fn a_window_without_two_http_samples_is_uncredited() {
    // Arrange
    let window = 7 * WINDOW_MS as i64;
    let lines: Vec<_> = complete()
        .into_iter()
        .filter(|line| {
            let active = line.host_unix_ms as i64 - ORIGIN as i64;
            !(line.maybe_transport == Some(SoakTransport::Http)
                && (window + 2_000..window + WINDOW_MS as i64).contains(&active))
        })
        .collect();
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"http_window_uncredited"));
    assert!(!judgement.windows[7].http.credited);
    assert!(judgement.windows[6].http.credited);
}

#[test]
fn a_stalled_watchdog_in_one_window_is_uncredited() {
    // Arrange
    let mut lines = complete();
    for line in &mut lines {
        let active = line.host_unix_ms as i64 - ORIGIN as i64;
        if (3 * WINDOW_MS as i64..4 * WINDOW_MS as i64).contains(&active) {
            if let Some(sample) = line.sample.as_mut() {
                sample.maybe_watchdog_feed_sequence = Some(5);
            }
        }
    }
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(!judgement.windows[3].websocket.credited);
    assert!(!judgement.passed);
}

#[test]
fn an_unsafe_active_sample_uncredits_its_window() {
    // Arrange
    let mut lines = complete();
    lines[200].sample.as_mut().expect("sample").safety_valid = false;
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(!judgement.passed);
}

#[test]
fn gate_observations_that_disagree_by_more_than_a_second_fail_clock_correlation() {
    // Arrange
    let mut observations = clock();
    observations[1].observed_unix_ms += 1_500;
    // Act
    let judgement = judge(&complete(), &observations, &expected());
    // Assert
    assert!(judgement.failures.contains(&"clock_correlation_failed"));
}

#[test]
fn the_clock_origin_is_the_earliest_offset_with_its_spread() {
    // Arrange / Act
    let origin = clock_origin(&clock());
    // Assert
    assert_eq!(origin, Some((ORIGIN, 400)));
}

#[test]
fn missing_terminal_paused_state_fails_both_transports() {
    // Arrange
    let lines: Vec<_> = complete()
        .into_iter()
        .filter(|line| line.host_unix_ms < ORIGIN + WORK_GATE_MS)
        .collect();
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"terminal_http_unconfirmed"));
    assert!(judgement
        .failures
        .contains(&"terminal_websocket_unconfirmed"));
}

#[test]
fn changed_pool_settings_fail_retention() {
    // Arrange
    let mut lines = complete();
    let last_http = lines
        .iter_mut()
        .rev()
        .find(|line| line.maybe_transport == Some(SoakTransport::Http))
        .expect("http");
    last_http
        .sample
        .as_mut()
        .expect("sample")
        .maybe_pool_matches_initial = Some(false);
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"pool_settings_changed"));
}

#[test]
fn a_different_package_or_boot_fails_identity() {
    // Arrange
    let mut lines = complete();
    lines[400].sample.as_mut().expect("sample").boot_ordinal = 8;
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"identity_changed"));
}

#[test]
fn a_soak_without_an_accepted_share_fails() {
    // Arrange
    let mut lines = complete();
    for line in &mut lines {
        if let Some(sample) = line.sample.as_mut() {
            sample.shares_accepted = 3;
        }
    }
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"no_accepted_share"));
}

fn handoff(port: u16) -> String {
    format!(
        r#"{{"schema":"soak-observer-input-v1","ipv4":"192.168.1.2","port":{port},"observedUnixMs":10000,"expiresUnixMs":15000,"bindingVerified":true}}"#
    )
}

#[test]
fn the_handoff_requires_a_fresh_verified_private_endpoint_and_its_own_schema() {
    // Arrange
    let valid = handoff(80);
    // Act / Assert
    assert!(origin(valid.as_bytes(), 11_000).is_ok());
    for invalid in [
        valid.replace("192.168.1.2", "127.0.0.1"),
        valid.replace("soak-observer-input-v1", "cpu0-cadence-observer-input-v1"),
        valid.replace("true", "false"),
        valid.replace("15000", "16000"),
    ] {
        assert_eq!(
            origin(invalid.as_bytes(), 11_000).err(),
            Some("invalid_input")
        );
    }
    assert!(origin(valid.as_bytes(), 15_000).is_err());
}

fn wire(sequence: u64) -> SystemInfoWire {
    let mut wire = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    wire.operator_snapshot_revision = OperatorSnapshotRevision::new(sequence).expect("revision");
    wire.runtime_health.maybe_task_watchdog_feed_sequence = Some(sequence);
    wire.stratum_url = "private-pool-fixture.invalid".into();
    wire
}

fn serve(stream: TcpStream) {
    let mut peek = [0_u8; 512];
    let length = stream.peek(&mut peek).unwrap_or(0);
    if String::from_utf8_lossy(&peek[..length]).contains("/api/ws/live") {
        let mut socket = tungstenite::accept(stream).expect("handshake");
        let full = serde_json::json!({"event": "update", "data": wire(1)}).to_string();
        let _ = socket.send(Message::Text(full.into()));
        for sequence in 2..30 {
            let diff = serde_json::json!({"event": "update", "data": {"runtimeHealth": {"taskWatchdogFeedSequence": sequence}}});
            if socket.send(Message::Text(diff.to_string().into())).is_err() {
                return;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
        return;
    }
    let mut stream = stream;
    let mut request = [0_u8; 1024];
    let _ = stream.read(&mut request);
    let body = serde_json::to_string(&wire(1)).expect("body");
    let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
}

#[test]
fn real_sockets_journal_http_and_websocket_samples_without_pool_values() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let origin = Arc::new(Zeroizing::new(format!(
        "http://{}",
        listener.local_addr().expect("address")
    )));
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            thread::spawn(move || serve(stream));
        }
    });
    let mut output = Vec::new();
    let started = Instant::now();
    // Act
    let outcome = observer::run_threads(origin, &mut output, Duration::from_secs(10), || {
        Ok(started.elapsed() >= Duration::from_millis(2_500))
    });
    // Assert
    assert_eq!(outcome, Ok("requested"));
    let text = String::from_utf8(output).expect("journal");
    let lines: Vec<JournalLine> = text
        .lines()
        .map(|line| serde_json::from_str(line).expect("line"))
        .collect();
    let http: Vec<_> = lines
        .iter()
        .filter(|line| line.maybe_transport == Some(SoakTransport::Http) && line.event == "sample")
        .collect();
    let websocket = lines
        .iter()
        .filter(|line| {
            line.maybe_transport == Some(SoakTransport::Websocket) && line.event == "sample"
        })
        .count();
    assert!(http.len() >= 2 && websocket >= 5);
    assert!(http.iter().all(|line| line
        .sample
        .as_ref()
        .and_then(|sample| sample.maybe_pool_matches_initial)
        == Some(true)));
    assert!(!text.contains("private-pool-fixture"));
}

#[test]
fn frozen_active_time_after_the_halt_does_not_break_clock_correlation() {
    // Arrange: through safe-stop the Gate keeps reporting the frozen ~600 s active time.
    let mut observations = clock();
    observations.extend((1..30).map(|seconds| ClockObservation {
        observed_unix_ms: ORIGIN + 600_000 + seconds * 1_000,
        active_ms: 600_010,
    }));
    // Act
    let judgement = judge(&complete(), &observations, &expected());
    // Assert
    assert!(judgement.passed, "{:?}", judgement.failures);
}

#[test]
fn an_inactive_sample_inside_the_edge_guard_does_not_uncredit_the_last_window() {
    // Arrange: a frame produced as the gate closes maps just before the edge.
    let mut lines = complete();
    let mut late = sample(599_600, 640);
    late.mining_active = false;
    lines.push(line(SoakTransport::Websocket, 599_600, late));
    lines.sort_by_key(|line| line.host_unix_ms);
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.windows[19].websocket.credited);
}

#[test]
fn a_final_state_that_is_not_paused_fails() {
    // Arrange
    let mut lines = complete();
    for line in lines.iter_mut().rev().take(4) {
        line.sample.as_mut().expect("sample").mining_paused = false;
    }
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"final_state_not_paused"));
}

#[test]
fn a_pool_change_reverted_before_the_end_still_fails() {
    // Arrange
    let mut lines = complete();
    let middle = lines
        .iter_mut()
        .filter(|line| line.maybe_transport == Some(SoakTransport::Http))
        .nth(100)
        .expect("http");
    middle
        .sample
        .as_mut()
        .expect("sample")
        .maybe_pool_matches_initial = Some(false);
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"pool_settings_changed"));
}

#[test]
fn mining_never_observed_stopped_within_safe_stop_fails_the_terminal_check() {
    // Arrange
    let mut lines = complete();
    for line in &mut lines {
        let active = line.host_unix_ms as i64 - ORIGIN as i64;
        if (600_000..760_000).contains(&active) {
            line.sample.as_mut().expect("sample").mining_active = true;
        }
    }
    // Act
    let judgement = judge(&lines, &clock(), &expected());
    // Assert
    assert!(judgement.failures.contains(&"terminal_http_unconfirmed"));
}
