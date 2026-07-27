use crate::*;

pub(crate) fn resolve_port(
    maybe_port: Option<&str>,
    environment: &impl FlashEnvironment,
) -> Result<String> {
    if let Some(port) = maybe_port {
        return Ok(port.to_owned());
    }

    let ports_output = environment.list_ports()?;
    let candidates = likely_port_candidates(&ports_output);
    match candidates.len() {
        0 => bail!(
            "No serial ports found. Connect an Ultra 205 over USB or pass an explicit port, for example: --port /dev/cu.usbmodem101"
        ),
        1 => Ok(candidates[0].clone()),
        _ => bail!(
            "Ambiguous serial ports:\n{}",
            candidates
                .iter()
                .map(|port| format!("- use --port {port}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

pub(crate) fn prepare_monitor_command(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Result<CommandSpec> {
    ensure_ultra_205(common.board)?;
    let port = resolve_port(common.port.as_deref(), environment)?;
    Ok(CommandSpec::new(
        "bitaxe-receive-only",
        ["observe", "--port", port.as_str()],
    ))
}

pub(crate) fn prepare_evidence_monitor_command(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Result<CommandSpec> {
    ensure_ultra_205(common.board)?;
    let port = resolve_port(common.port.as_deref(), environment)?;
    Ok(CommandSpec::new(
        "bitaxe-receive-only",
        ["observe", "--port", port.as_str()],
    ))
}

pub(crate) fn command_with_port(command_spec: &CommandSpec, port: &str) -> Result<Vec<String>> {
    let mut args = command_spec.args.clone();
    let Some(port_index) = args.iter().position(|argument| argument == "--port") else {
        bail!("supervised command is missing --port");
    };
    let Some(value) = args.get_mut(port_index.saturating_add(1)) else {
        bail!("supervised command has an incomplete --port argument");
    };
    *value = port.to_owned();
    Ok(args)
}

pub(crate) fn monitor_log_has_trusted_boot_markers(log: &str) -> bool {
    monitor_log_has_message(log, "bitaxe-rust boot: board=Ultra 205 asic=BM1366")
        && monitor_log_has_message(
            log,
            "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        )
        && monitor_log_has_token(log, "spiffs_mount=available")
        && monitor_log_has_token(log, "axeos_api_route_shell=started")
        && [
            "ota_boot_validation=",
            "reset_reason=",
            "firmware_commit=",
            "reference_commit=",
            "esp_idf_version=",
        ]
        .iter()
        .all(|marker| monitor_log_marker_value(log, marker) != UNAVAILABLE)
}

pub(crate) fn monitor_log_has_message(log: &str, marker: &str) -> bool {
    let prefixed_marker = format!(": {marker}");
    log.lines()
        .map(str::trim)
        .any(|line| line == marker || line.ends_with(&prefixed_marker))
}

pub(crate) fn monitor_log_has_token(log: &str, marker: &str) -> bool {
    log.lines()
        .flat_map(str::split_whitespace)
        .any(|token| token == marker)
}

pub(crate) fn monitor_capture_outcome(
    process_status: &CaptureProcessStatus,
    monitor_log: &str,
    capture_timeout_seconds: u64,
    expected_firmware_commit: &str,
    expected_reference_commit: &str,
    maybe_runtime_identity: Option<&ExpectedRuntimeAttestationIdentity>,
    allow_private_classification: bool,
) -> MonitorCaptureOutcome {
    let observed_firmware_commit = monitor_log_marker_value(monitor_log, "firmware_commit=");
    let observed_reference_commit = monitor_log_marker_value(monitor_log, "reference_commit=");
    let maybe_trust_failure = monitor_trust_failure(
        monitor_log,
        &observed_firmware_commit,
        expected_firmware_commit,
        &observed_reference_commit,
        expected_reference_commit,
    );
    let boot_transcript_status = if maybe_trust_failure.is_none() {
        BootTranscriptStatus::Trusted
    } else if !monitor_log_has_trusted_boot_markers(monitor_log) {
        BootTranscriptStatus::Missing
    } else {
        BootTranscriptStatus::Untrusted
    };
    let runtime_attestation_status = maybe_runtime_identity
        .map_or(RuntimeAttestationStatus::Missing, |identity| {
            classify_runtime_boot_attestations(monitor_log, identity)
        });
    let maybe_trust_basis = if maybe_trust_failure.is_none() {
        Some(MonitorTrustBasis::BootTranscript)
    } else if runtime_attestation_status == RuntimeAttestationStatus::Trusted {
        Some(MonitorTrustBasis::RuntimeAttestation)
    } else {
        None
    };
    let state = match (process_status, maybe_trust_basis) {
        (CaptureProcessStatus::ExitedSuccess, Some(basis)) => MonitorCaptureState::Trusted {
            completion: TrustedCaptureCompletion::Completed,
            basis,
        },
        (CaptureProcessStatus::TimedOut, Some(basis)) => MonitorCaptureState::Trusted {
            completion: TrustedCaptureCompletion::TimedOut,
            basis,
        },
        (CaptureProcessStatus::TimedOut, None) if allow_private_classification => {
            MonitorCaptureState::PendingPrivateClassification
        }
        (CaptureProcessStatus::TimedOut, None) => MonitorCaptureState::Untrusted {
            timed_out: true,
            conclusion: maybe_trust_failure.map_or_else(
                || "failed - evidence capture is not trusted".to_owned(),
                |failure| format!("failed - evidence capture is not trusted: {failure}"),
            ),
        },
        (
            CaptureProcessStatus::SpawnFailed
            | CaptureProcessStatus::ExitedSuccess
            | CaptureProcessStatus::ExitedFailure(_),
            None,
        ) => MonitorCaptureState::Untrusted {
            timed_out: false,
            conclusion: maybe_trust_failure.map_or_else(
                || "failed - evidence capture is not trusted".to_owned(),
                |failure| format!("failed - evidence capture is not trusted: {failure}"),
            ),
        },
        (CaptureProcessStatus::SpawnFailed | CaptureProcessStatus::ExitedFailure(_), Some(_)) => {
            MonitorCaptureState::Untrusted {
                timed_out: false,
                conclusion: "failed - monitor process did not complete successfully".to_owned(),
            }
        }
    };

    MonitorCaptureOutcome {
        state,
        capture_timeout_seconds,
        observed_firmware_commit,
        observed_reference_commit,
        boot_transcript_status,
        runtime_attestation_status: RuntimeAttestationEvidenceStatus::Observed(
            runtime_attestation_status,
        ),
    }
}

pub(crate) fn monitor_log_marker_value(log: &str, marker: &str) -> String {
    log.lines()
        .flat_map(str::split_whitespace)
        .find_map(|token| token.strip_prefix(marker))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| UNAVAILABLE.to_owned())
}

pub(crate) fn monitor_trust_failure(
    monitor_log: &str,
    observed_firmware_commit: &str,
    expected_firmware_commit: &str,
    observed_reference_commit: &str,
    expected_reference_commit: &str,
) -> Option<String> {
    if !monitor_log_has_trusted_boot_markers(monitor_log) {
        return Some("missing trusted Ultra 205 boot markers".to_owned());
    }

    if !commit_marker_matches_expected(observed_firmware_commit, expected_firmware_commit) {
        return Some(format!(
            "observed firmware_commit={observed_firmware_commit} did not match source commit={expected_firmware_commit}"
        ));
    }

    if !commit_marker_matches_expected(observed_reference_commit, expected_reference_commit) {
        return Some(format!(
            "observed reference_commit={observed_reference_commit} did not match reference commit={expected_reference_commit}"
        ));
    }

    None
}

pub(crate) fn commit_marker_matches_expected(observed: &str, expected: &str) -> bool {
    observed != UNAVAILABLE
        && expected != UNAVAILABLE
        && observed.len() >= MIN_COMMIT_PREFIX_LEN
        && observed.len() <= expected.len()
        && observed
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        && expected
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        && expected.starts_with(observed)
}

pub(crate) fn dry_run_monitor_capture_outcome(
    capture_timeout_seconds: u64,
) -> MonitorCaptureOutcome {
    MonitorCaptureOutcome {
        state: MonitorCaptureState::DryRun,
        capture_timeout_seconds,
        observed_firmware_commit: UNAVAILABLE.to_owned(),
        observed_reference_commit: UNAVAILABLE.to_owned(),
        boot_transcript_status: BootTranscriptStatus::NotCaptured,
        runtime_attestation_status: RuntimeAttestationEvidenceStatus::NotCaptured,
    }
}

pub(crate) fn no_monitor_capture_outcome() -> MonitorCaptureOutcome {
    MonitorCaptureOutcome {
        state: MonitorCaptureState::NotRequested,
        capture_timeout_seconds: 0,
        observed_firmware_commit: UNAVAILABLE.to_owned(),
        observed_reference_commit: UNAVAILABLE.to_owned(),
        boot_transcript_status: BootTranscriptStatus::NotRequested,
        runtime_attestation_status: RuntimeAttestationEvidenceStatus::NotRequested,
    }
}

pub(crate) fn evidence_capture_failure_guidance(
    port: &str,
    evidence_dir: &Utf8Path,
    boot_transcript_status: &str,
    runtime_attestation_status: &str,
) -> String {
    [
        "evidence capture failed and is not trusted".to_owned(),
        "flash_status=completed".to_owned(),
        "monitor_evidence_status=untrusted".to_owned(),
        format!("boot_transcript_status={boot_transcript_status}"),
        format!("runtime_attestation_status={runtime_attestation_status}"),
        "next: just detect-ultra205".to_owned(),
        format!("diagnostic only: just monitor port={port}"),
        format!("evidence_dir={evidence_dir}"),
        "do not reflash automatically; use a verified state-changing fix before another flash attempt"
            .to_owned(),
    ]
    .join("\n")
}

pub(crate) fn likely_port_candidates(ports_output: &str) -> Vec<String> {
    let mut candidates = BTreeSet::new();
    for token in ports_output.split_whitespace() {
        let port = token.trim_matches(|character: char| {
            matches!(character, ',' | ';' | ':' | '(' | ')' | '[' | ']')
        });

        if is_likely_port(port) {
            candidates.insert(port.to_owned());
        }
    }

    candidates.into_iter().collect()
}

pub(crate) fn is_likely_port(port: &str) -> bool {
    if port.starts_with("/dev/cu.usbmodem")
        || port.starts_with("/dev/cu.usbserial")
        || port.starts_with("/dev/ttyUSB")
        || port.starts_with("/dev/ttyACM")
    {
        return true;
    }

    let Some(suffix) = port.strip_prefix("COM") else {
        return false;
    };

    !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
}

pub(crate) fn ensure_ultra_205(board: BoardId) -> Result<()> {
    if board != BoardId::Ultra205 {
        bail!("Phase 1 supports board=205 only");
    }

    Ok(())
}
