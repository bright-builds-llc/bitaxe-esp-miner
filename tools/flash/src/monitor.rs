use crate::*;
mod fixed_serial;
pub(crate) use fixed_serial::FixedSerialAssessment;

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

pub(crate) fn monitor_capture_outcome(
    process_status: &CaptureProcessStatus,
    monitor_log: &str,
    capture_timeout_seconds: u64,
    maybe_runtime_identity: Option<&ExpectedRuntimeAttestationIdentity>,
) -> MonitorCaptureOutcome {
    let assessment = fixed_serial::assess(monitor_log, maybe_runtime_identity);
    let state = match process_status {
        CaptureProcessStatus::ExitedSuccess | CaptureProcessStatus::TimedOut
            if assessment.qualified() =>
        {
            MonitorCaptureState::Trusted {
                completion: if *process_status == CaptureProcessStatus::TimedOut {
                    TrustedCaptureCompletion::TimedOut
                } else {
                    TrustedCaptureCompletion::Completed
                },
                basis: MonitorTrustBasis::FixedSerial,
            }
        }
        _ => MonitorCaptureState::Untrusted {
            timed_out: *process_status == CaptureProcessStatus::TimedOut,
            conclusion: if assessment.qualified() {
                "unqualified - exact fixed Serial/JTAG execution observed, but monitor process failed".to_owned()
            } else {
                assessment.conclusion()
            },
        },
    };
    MonitorCaptureOutcome {
        state,
        capture_timeout_seconds,
        observed_firmware_commit: maybe_runtime_identity
            .filter(|_| assessment.execution_present)
            .map_or_else(
                || UNAVAILABLE.to_owned(),
                |identity| identity.firmware_commit.clone(),
            ),
        observed_reference_commit: UNAVAILABLE.to_owned(),
        boot_transcript_status: BootTranscriptStatus::NotApplicable,
        runtime_attestation_status: RuntimeAttestationEvidenceStatus::NotApplicable,
        fixed_serial_assessment: Some(assessment),
    }
}

pub(crate) fn dry_run_monitor_capture_outcome(
    capture_timeout_seconds: u64,
) -> MonitorCaptureOutcome {
    MonitorCaptureOutcome {
        state: MonitorCaptureState::DryRun,
        fixed_serial_assessment: None,
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
        fixed_serial_assessment: None,
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
