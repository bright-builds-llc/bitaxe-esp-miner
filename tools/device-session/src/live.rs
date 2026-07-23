use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::macos::{MacOsDeviceAdapter, ReceiveOnlyReader};
use crate::{
    current_platform, DevicePhase, PlatformCategory, PrivateBootB, SerialPhase, SessionArtifacts,
    SessionEvent, SessionRequest, SessionState, TerminalCategory,
};
use bitaxe_http_transport::{ExchangeObservation, StrictHttpClient};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(250);
const INITIAL_DEVICE_TIMEOUT: Duration = Duration::from_secs(10);
const PRE_RESTART_DELIVERY_TIMEOUT: Duration = Duration::from_secs(30);
const HTTP_POLL_INTERVAL: Duration = Duration::from_secs(1);

pub fn run_live_session(
    request: SessionRequest,
    mut artifacts: SessionArtifacts,
    timeout: Duration,
) -> Result<TerminalCategory> {
    if !request.schema_is_valid() {
        anyhow::bail!("device-session request schema is invalid");
    }
    let mut state = SessionState::new(
        request.baseline.clone(),
        request.expected_postcondition.clone(),
        request.trusted_origin.clone(),
    );
    apply_event(
        &mut state,
        &mut artifacts,
        SessionEvent::PlatformObserved {
            category: current_platform(),
        },
    )?;
    if current_platform() != PlatformCategory::Macos {
        apply_event(&mut state, &mut artifacts, SessionEvent::CleanupComplete)?;
        let terminal = state.terminal_category();
        artifacts.finish(&state)?;
        return Ok(terminal);
    }

    let session_started = Instant::now();
    let deadline = session_started + timeout;
    let initial_deadline = session_started + INITIAL_DEVICE_TIMEOUT.min(timeout);
    let mut selected_port = request.admitted_port.clone();
    while Instant::now() < initial_deadline && !state.device_ready(DevicePhase::Initial) {
        let observation = MacOsDeviceAdapter::initial_sample(
            &request.admitted_port,
            &request.physical_identity_digest,
        )?;
        if let Some(port) = observation.maybe_port {
            selected_port = port;
        }
        apply_event(&mut state, &mut artifacts, observation.event)?;
        if state.terminal_category() != TerminalCategory::Incomplete {
            break;
        }
        thread::sleep(SAMPLE_INTERVAL);
    }
    if !state.device_ready(DevicePhase::Initial) {
        return finish_failed_session(state, artifacts, session_started);
    }

    let mut reader = match ReceiveOnlyReader::open(&selected_port) {
        Ok(reader) => {
            apply_event(&mut state, &mut artifacts, SessionEvent::ReaderArmed)?;
            Some(reader)
        }
        Err(_) => {
            apply_event(&mut state, &mut artifacts, SessionEvent::ReaderStartFailed)?;
            return finish_failed_session(state, artifacts, session_started);
        }
    };
    let pre_delivery_deadline =
        Instant::now() + PRE_RESTART_DELIVERY_TIMEOUT.min(remaining(session_started, timeout));
    while Instant::now() < pre_delivery_deadline {
        if let Some(active_reader) = reader.as_mut() {
            let bytes = match active_reader.read_available() {
                Ok(bytes) => bytes,
                Err(_) => {
                    apply_event(&mut state, &mut artifacts, SessionEvent::ReaderStartFailed)?;
                    return finish_failed_session(state, artifacts, session_started);
                }
            };
            if !bytes.is_empty() {
                if !artifacts.record_serial(&bytes)? {
                    state.apply(SessionEvent::AdmissionRejected);
                    return finish_failed_session(state, artifacts, session_started);
                }
                apply_event(
                    &mut state,
                    &mut artifacts,
                    SessionEvent::SerialBytes {
                        phase: SerialPhase::PreRestart,
                        count: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                    },
                )?;
                break;
            }
        }
        thread::sleep(SAMPLE_INTERVAL);
    }
    if !state.projection().pre_restart_serial_delivery {
        return finish_failed_session(state, artifacts, session_started);
    }

    let http = StrictHttpClient::new(&request.trusted_origin)?;
    let baseline_http = http.get_system_info(deadline)?;
    record_http(&mut state, &mut artifacts, "baseline", &baseline_http)?;
    match baseline_http {
        observation
            if observation.response_received
                && matches!(observation.response_status, 200..=299)
                && baseline_matches(&request, &observation.body) =>
        {
            apply_event(&mut state, &mut artifacts, SessionEvent::BaselineConfirmed)?;
        }
        _ => {
            apply_event(&mut state, &mut artifacts, SessionEvent::BaselineMismatch)?;
            return finish_failed_session(state, artifacts, session_started);
        }
    }

    apply_event(
        &mut state,
        &mut artifacts,
        SessionEvent::RestartRequestStarted,
    )?;
    let restart = http.post_restart_once(deadline)?;
    record_http(&mut state, &mut artifacts, "restart", &restart)?;
    if restart.request_bytes_written > 0 {
        apply_event(
            &mut state,
            &mut artifacts,
            SessionEvent::RestartRequestBytesWritten {
                count: restart.request_bytes_written,
            },
        )?;
    }
    if restart.request_write_complete {
        apply_event(
            &mut state,
            &mut artifacts,
            SessionEvent::RestartRequestWriteComplete,
        )?;
    }
    if restart.response_received {
        let response_event = if matches!(restart.response_status, 200..=299) {
            SessionEvent::RestartResponseReceived
        } else {
            SessionEvent::RestartResponseRejected
        };
        apply_event(&mut state, &mut artifacts, response_event)?;
    }
    if state.terminal_category() != TerminalCategory::Incomplete {
        return finish_failed_session(state, artifacts, session_started);
    }

    let mut next_http_poll = Instant::now();
    let mut service_loss_recorded = false;
    let mut disappearance_recorded = false;
    while Instant::now() < deadline && !state.authoritative_quorum_satisfied() {
        if let Some(active_reader) = reader.as_mut() {
            if fs::metadata(active_reader.port()).is_err() {
                reader = None;
                if !disappearance_recorded {
                    apply_event(&mut state, &mut artifacts, SessionEvent::DeviceAbsent)?;
                    disappearance_recorded = true;
                }
            } else {
                let bytes = match active_reader.read_available() {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        reader = None;
                        apply_event(&mut state, &mut artifacts, SessionEvent::ReaderLost)?;
                        Vec::new()
                    }
                };
                if !bytes.is_empty() {
                    if !artifacts.record_serial(&bytes)? {
                        state.apply(SessionEvent::AdmissionRejected);
                        break;
                    }
                    apply_event(
                        &mut state,
                        &mut artifacts,
                        SessionEvent::SerialBytes {
                            phase: SerialPhase::PostRestart,
                            count: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                        },
                    )?;
                }
            }
        }
        if !state.device_ready(DevicePhase::Recovery) {
            let observation = MacOsDeviceAdapter::recovery_sample(
                &request.physical_identity_digest,
                &selected_port,
            )?;
            if let Some(port) = observation.maybe_port {
                selected_port = port;
            }
            apply_event(&mut state, &mut artifacts, observation.event)?;
            if state.terminal_category() != TerminalCategory::Incomplete {
                break;
            }
            if reader.is_none() && state.device_ready(DevicePhase::Recovery) {
                match ReceiveOnlyReader::open(&selected_port) {
                    Ok(new_reader) => {
                        reader = Some(new_reader);
                        apply_event(&mut state, &mut artifacts, SessionEvent::ReaderReacquired)?;
                    }
                    Err(_) => {
                        apply_event(&mut state, &mut artifacts, SessionEvent::ReaderStartFailed)?;
                        break;
                    }
                }
            }
        }
        if Instant::now() >= next_http_poll {
            let polled = http.get_system_info(deadline)?;
            record_http(&mut state, &mut artifacts, "recovery", &polled)?;
            match polled {
                observation
                    if observation.response_received
                        && matches!(observation.response_status, 200..=299) =>
                {
                    if let Some(boot_b) = parse_boot_b(&request.trusted_origin, &observation.body) {
                        apply_event(
                            &mut state,
                            &mut artifacts,
                            SessionEvent::BootBObserved { boot_b },
                        )?;
                    }
                }
                _ if !service_loss_recorded => {
                    apply_event(
                        &mut state,
                        &mut artifacts,
                        SessionEvent::ServiceLossObserved,
                    )?;
                    service_loss_recorded = true;
                }
                _ => {}
            }
            next_http_poll = next_poll_deadline(next_http_poll, Instant::now());
        }
        thread::sleep(Duration::from_millis(100));
    }

    drop(reader);
    thread::sleep(Duration::from_millis(50));
    if MacOsDeviceAdapter::holder_count(&selected_port)? > 0 {
        apply_event(&mut state, &mut artifacts, SessionEvent::CleanupFailed)?;
    } else if state.authoritative_quorum_satisfied() {
        apply_event(&mut state, &mut artifacts, SessionEvent::CleanupComplete)?;
    } else {
        apply_event(
            &mut state,
            &mut artifacts,
            SessionEvent::ObservationWindowExpired {
                duration_millis: elapsed_millis(session_started),
            },
        )?;
        apply_event(&mut state, &mut artifacts, SessionEvent::CleanupComplete)?;
    }
    let terminal = state.terminal_category();
    artifacts.finish(&state)?;
    Ok(terminal)
}

fn finish_failed_session(
    mut state: SessionState,
    mut artifacts: SessionArtifacts,
    started: Instant,
) -> Result<TerminalCategory> {
    if state.terminal_category() == TerminalCategory::Incomplete {
        apply_event(
            &mut state,
            &mut artifacts,
            SessionEvent::ObservationWindowExpired {
                duration_millis: elapsed_millis(started),
            },
        )?;
    }
    apply_event(&mut state, &mut artifacts, SessionEvent::CleanupComplete)?;
    let terminal = state.terminal_category();
    artifacts.finish(&state)?;
    Ok(terminal)
}

fn apply_event(
    state: &mut SessionState,
    artifacts: &mut SessionArtifacts,
    event: SessionEvent,
) -> Result<()> {
    if !artifacts.record_event(&event)? {
        state.apply(SessionEvent::AdmissionRejected);
        return Ok(());
    }
    state.apply(event);
    Ok(())
}

fn record_http(
    state: &mut SessionState,
    artifacts: &mut SessionArtifacts,
    stage: &str,
    observation: &ExchangeObservation,
) -> Result<()> {
    let private = serde_json::json!({
        "stage": stage,
        "request_bytes_written": observation.request_bytes_written,
        "request_write_complete": observation.request_write_complete,
        "response_received": observation.response_received,
        "response_status": observation.response_status,
        "response_body_bytes": observation.body,
    });
    if !artifacts.record_http_value(&private)? {
        state.apply(SessionEvent::AdmissionRejected);
    }
    Ok(())
}

fn baseline_matches(request: &SessionRequest, body: &[u8]) -> bool {
    let Some(observed) = parse_boot_b(&request.trusted_origin, body) else {
        return false;
    };
    observed.boot_session == request.baseline.boot_session
        && observed.boot_ordinal == request.baseline.boot_ordinal
        && observed.source_commit == request.baseline.source_commit
        && observed.reference_commit == request.baseline.reference_commit
        && observed.app_elf_sha256 == request.baseline.app_elf_sha256
        && observed.hostname_sha256 == request.expected_postcondition.hostname_sha256
}

fn parse_boot_b(trusted_origin: &str, body: &[u8]) -> Option<PrivateBootB> {
    let value: Value = serde_json::from_slice(body).ok()?;
    Some(PrivateBootB {
        boot_session: required_string(&value, "bootSession")?,
        boot_ordinal: value.get("bootOrdinal")?.as_u64()?,
        reset_reason_category: required_string(&value, "resetReasonCategory")?,
        trusted_origin: trusted_origin.to_owned(),
        source_commit: required_string(&value, "sourceCommit")?,
        reference_commit: required_string(&value, "referenceCommit")?,
        app_elf_sha256: required_string(&value, "appElfSha256")?,
        hostname_sha256: sha256(required_string(&value, "hostname")?.as_bytes()),
    })
}

fn required_string(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)?
        .as_str()
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex_lower(&digest.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn remaining(started: Instant, timeout: Duration) -> Duration {
    timeout.saturating_sub(started.elapsed())
}

fn elapsed_millis(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn next_poll_deadline(mut previous: Instant, now: Instant) -> Instant {
    loop {
        previous += HTTP_POLL_INTERVAL;
        if previous > now {
            return previous;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_b_parser_extracts_only_required_private_fields() {
        // Arrange
        let body = serde_json::json!({
            "bootSession": "boot-b",
            "bootOrdinal": 8,
            "resetReasonCategory": "software_cpu",
            "sourceCommit": "source",
            "referenceCommit": "reference",
            "appElfSha256": "a".repeat(64),
            "hostname": "private-host",
            "ignoredSensitiveField": "not-copied"
        });

        // Act
        let boot_b = parse_boot_b(
            "http://private-device",
            serde_json::to_string(&body)
                .expect("body must serialize")
                .as_bytes(),
        )
        .expect("body must parse");

        // Assert
        assert_eq!(boot_b.boot_session, "boot-b");
        assert_eq!(boot_b.boot_ordinal, 8);
        assert_eq!(boot_b.trusted_origin, "http://private-device");
        assert_eq!(boot_b.hostname_sha256.len(), 64);
    }

    #[test]
    fn missed_poll_slots_are_skipped_without_catch_up_bursts() {
        // Arrange
        let started = Instant::now();
        let delayed = started + (HTTP_POLL_INTERVAL * 4) + Duration::from_millis(1);

        // Act
        let next = next_poll_deadline(started, delayed);

        // Assert
        assert!(next > delayed);
        assert!(next <= delayed + HTTP_POLL_INTERVAL);
    }
}
