use crate::*;

pub(crate) fn write_evidence_if_requested(
    common: &CommonArgs,
    outcome: &FlashOutcome,
    command_kind: &str,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let Some(evidence_dir) = resolved_evidence_dir(common, environment) else {
        return Ok(());
    };

    let log_path = evidence_dir.join("flash-monitor.log");
    let capture_outcome = no_monitor_capture_outcome();
    let command_display = flash_workflow_command(outcome);
    let flash_command_display = outcome.command.display();
    write_evidence_record(
        common,
        outcome,
        &evidence_dir,
        EvidenceRecordInput {
            command_kind,
            command: &command_display,
            flash_command: &flash_command_display,
            monitor_command: UNAVAILABLE,
            log_path: &log_path,
            private_log_path: None,
            private_log_sha256: None,
            admitted_log_sha256: None,
            capture_outcome: &capture_outcome,
        },
        environment,
    )
}

pub(crate) fn write_flash_monitor_evidence_if_requested(
    common: &CommonArgs,
    outcome: &FlashOutcome,
    monitor_command: &CommandSpec,
    evidence_dir: &Utf8Path,
    artifacts: MonitorEvidenceArtifacts<'_>,
    capture_outcome: &MonitorCaptureOutcome,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let flash_workflow_command = flash_workflow_command(outcome);
    let monitor_command_display = monitor_command.display();
    let command = format!("{flash_workflow_command}\nmonitor: {monitor_command_display}");
    let flash_command_display = outcome.command.display();
    write_evidence_record(
        common,
        outcome,
        evidence_dir,
        EvidenceRecordInput {
            command_kind: "flash-monitor",
            command: &command,
            flash_command: &flash_command_display,
            monitor_command: &monitor_command_display,
            log_path: artifacts.admitted_log,
            private_log_path: artifacts
                .dual_paths
                .map(|paths| paths.private_log.as_path()),
            private_log_sha256: artifacts.private_log_sha256,
            admitted_log_sha256: None,
            capture_outcome,
        },
        environment,
    )
}

pub(crate) fn write_evidence_record(
    common: &CommonArgs,
    outcome: &FlashOutcome,
    evidence_dir: &Utf8Path,
    input: EvidenceRecordInput<'_>,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let redaction_mode = EvidenceRedactionMode::from_common(common);
    let dual_mode = common.evidence_mode == Some(EvidenceMode::Dual);
    let capture_projection = input.capture_outcome.projection();
    let record = EvidenceRecord {
        command: input.command.to_owned(),
        command_kind: input.command_kind.to_owned(),
        board: common.board.to_string(),
        port: command_port(&outcome.command).unwrap_or_else(|| UNAVAILABLE.to_owned()),
        firmware_commit: environment.firmware_commit(),
        reference_commit: environment.reference_commit(),
        manifest_path: outcome
            .manifest
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
        flash_image_path: outcome.flash_image.as_str().to_owned(),
        timestamp: unix_timestamp(),
        log_path: input.log_path.as_str().to_owned(),
        flash_command: input.flash_command.to_owned(),
        monitor_command: input.monitor_command.to_owned(),
        nvs_seed_status: if outcome.nvs_seed.is_some() {
            "provided".to_owned()
        } else {
            "not_provided".to_owned()
        },
        nvs_seed_command: outcome
            .nvs_seed
            .as_ref()
            .map(|seed| seed.command.display())
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
        nvs_seed_partition_offset: if outcome.nvs_seed.is_some() {
            NVS_PARTITION_OFFSET.to_owned()
        } else {
            UNAVAILABLE.to_owned()
        },
        nvs_seed_partition_size: if outcome.nvs_seed.is_some() {
            NVS_PARTITION_SIZE.to_owned()
        } else {
            UNAVAILABLE.to_owned()
        },
        redaction_mode: if dual_mode {
            "dual".to_owned()
        } else {
            redaction_mode.as_str().to_owned()
        },
        commit_ready: !dual_mode && redaction_mode.commit_ready(),
        wifi_credentials_source: if outcome.nvs_seed.is_some() {
            "provided-redacted".to_owned()
        } else {
            "not-provided".to_owned()
        },
        monitor_log_path: input.log_path.as_str().to_owned(),
        private_log_role: input
            .private_log_path
            .map(|_| "classifier-input-private".to_owned()),
        private_monitor_log_path: input.private_log_path.map(|path| path.as_str().to_owned()),
        private_monitor_log_sha256: input.private_log_sha256.map(str::to_owned),
        monitor_log_sha256: input.admitted_log_sha256.map(str::to_owned),
        capture_mode: capture_projection.capture_mode.to_owned(),
        capture_status: capture_projection.capture_status,
        capture_timeout_seconds: input.capture_outcome.capture_timeout_seconds,
        flash_status: if common.dry_run {
            "dry_run".to_owned()
        } else {
            "completed".to_owned()
        },
        monitor_evidence_status: capture_projection.monitor_evidence_status.to_owned(),
        boot_transcript_status: input
            .capture_outcome
            .boot_transcript_status
            .label()
            .to_owned(),
        runtime_attestation_status: input
            .capture_outcome
            .runtime_attestation_status
            .label()
            .to_owned(),
        trust_basis: capture_projection.trust_basis.to_owned(),
        trusted_output: capture_projection.trusted_output,
        observed_firmware_commit: input.capture_outcome.observed_firmware_commit.clone(),
        observed_reference_commit: input.capture_outcome.observed_reference_commit.clone(),
        conclusion: capture_projection.conclusion.to_owned(),
    };
    if dual_mode {
        let paths = evidence::DualEvidencePaths {
            private_log: input
                .private_log_path
                .context("dual evidence record requires private log path")?
                .to_owned(),
            admitted_log: input.log_path.to_owned(),
            private_record: evidence_dir.join("flash-command-evidence.private.json"),
            admitted_record: evidence_dir.join("flash-command-evidence.json"),
        };
        let private_json = serde_json::to_string_pretty(&record)
            .context("failed to serialize private evidence")?;
        evidence::write_dual_private_text(&paths.private_record, &private_json)?;
        return Ok(());
    }

    let json = serde_json::to_string_pretty(&record).context("failed to serialize evidence")?;
    environment.write_evidence(
        &evidence_dir.join("flash-command-evidence.json"),
        &sanitize_evidence_text(&json, redaction_mode),
    )
}

pub(crate) fn flash_workflow_command(outcome: &FlashOutcome) -> String {
    let flash = format!("flash: {}", outcome.command.display());
    let Some(nvs_seed) = &outcome.nvs_seed else {
        return flash;
    };

    format!("{flash}\nnvs_seed: {}", nvs_seed.command.display())
}

pub(crate) fn resolved_evidence_dir(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Option<Utf8PathBuf> {
    common
        .evidence_dir
        .as_deref()
        .map(|path| environment.workspace_path(path))
}

pub(crate) fn command_port(command: &CommandSpec) -> Option<String> {
    command
        .args
        .windows(2)
        .find(|window| window[0] == "--port")
        .map(|window| window[1].clone())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EvidenceRecord {
    pub(crate) command: String,
    pub(crate) command_kind: String,
    pub(crate) board: String,
    pub(crate) port: String,
    pub(crate) firmware_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) manifest_path: String,
    pub(crate) flash_image_path: String,
    pub(crate) timestamp: String,
    pub(crate) log_path: String,
    pub(crate) flash_command: String,
    pub(crate) monitor_command: String,
    pub(crate) nvs_seed_status: String,
    pub(crate) nvs_seed_command: String,
    pub(crate) nvs_seed_partition_offset: String,
    pub(crate) nvs_seed_partition_size: String,
    pub(crate) redaction_mode: String,
    pub(crate) commit_ready: bool,
    pub(crate) wifi_credentials_source: String,
    pub(crate) monitor_log_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) private_log_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) private_monitor_log_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) private_monitor_log_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) monitor_log_sha256: Option<String>,
    pub(crate) capture_mode: String,
    pub(crate) capture_status: CaptureStatus,
    pub(crate) capture_timeout_seconds: u64,
    #[serde(default = "unavailable_evidence_status")]
    pub(crate) flash_status: String,
    #[serde(default = "unavailable_evidence_status")]
    pub(crate) monitor_evidence_status: String,
    #[serde(default = "unavailable_evidence_status")]
    pub(crate) boot_transcript_status: String,
    #[serde(default = "unavailable_evidence_status")]
    pub(crate) runtime_attestation_status: String,
    #[serde(default = "no_trust_basis")]
    pub(crate) trust_basis: String,
    pub(crate) trusted_output: bool,
    pub(crate) observed_firmware_commit: String,
    pub(crate) observed_reference_commit: String,
    pub(crate) conclusion: String,
}

pub(crate) fn unavailable_evidence_status() -> String {
    UNAVAILABLE.to_owned()
}

pub(crate) fn no_trust_basis() -> String {
    "none".to_owned()
}

pub(crate) fn validate_evidence_record_capture_state(
    record: &EvidenceRecord,
) -> Result<MonitorCaptureState> {
    let boot_status = parse_boot_transcript_status(&record.boot_transcript_status)?;
    let runtime_status =
        parse_runtime_attestation_evidence_status(&record.runtime_attestation_status)?;
    let state = match record.capture_status {
        CaptureStatus::Completed | CaptureStatus::TimedOutAfterTrustedOutput => {
            let basis = match record.trust_basis.as_str() {
                "boot_transcript" => MonitorTrustBasis::BootTranscript,
                "runtime_attestation" => MonitorTrustBasis::RuntimeAttestation,
                _ => bail!("trusted capture requires a recognized trust basis"),
            };
            let completion = if record.capture_status == CaptureStatus::Completed {
                TrustedCaptureCompletion::Completed
            } else {
                TrustedCaptureCompletion::TimedOut
            };
            MonitorCaptureState::Trusted { completion, basis }
        }
        CaptureStatus::TimedOutPendingPrivateClassification => {
            MonitorCaptureState::PendingPrivateClassification
        }
        CaptureStatus::TimedOutAfterPrivateClassification => {
            MonitorCaptureState::AdmittedPrivateClassification
        }
        CaptureStatus::TimedOutWithoutTrustedOutput => MonitorCaptureState::Untrusted {
            timed_out: true,
            conclusion: record.conclusion.clone(),
        },
        CaptureStatus::Failed => MonitorCaptureState::Untrusted {
            timed_out: false,
            conclusion: record.conclusion.clone(),
        },
        CaptureStatus::DryRun => match record.capture_mode.as_str() {
            "dry_run" => MonitorCaptureState::DryRun,
            "not_applicable" => MonitorCaptureState::NotRequested,
            _ => bail!("dry-run capture has an invalid capture mode"),
        },
    };

    let projection = state.projection();
    if record.capture_mode != projection.capture_mode
        || record.capture_status != projection.capture_status
        || record.monitor_evidence_status != projection.monitor_evidence_status
        || record.trust_basis != projection.trust_basis
        || record.trusted_output != projection.trusted_output
        || record.conclusion != projection.conclusion
    {
        bail!("capture wire fields contradict the typed capture state");
    }

    match state {
        MonitorCaptureState::NotRequested
            if boot_status != BootTranscriptStatus::NotRequested
                || runtime_status != RuntimeAttestationEvidenceStatus::NotRequested =>
        {
            bail!("not-requested capture contains captured evidence state");
        }
        MonitorCaptureState::DryRun
            if boot_status != BootTranscriptStatus::NotCaptured
                || runtime_status != RuntimeAttestationEvidenceStatus::NotCaptured =>
        {
            bail!("dry-run capture contains captured evidence state");
        }
        MonitorCaptureState::Trusted {
            basis: MonitorTrustBasis::BootTranscript,
            ..
        } if boot_status != BootTranscriptStatus::Trusted => {
            bail!("boot-transcript trust basis lacks a trusted transcript");
        }
        MonitorCaptureState::Trusted {
            basis: MonitorTrustBasis::RuntimeAttestation,
            ..
        } if runtime_status
            != RuntimeAttestationEvidenceStatus::Observed(RuntimeAttestationStatus::Trusted) =>
        {
            bail!("runtime-attestation trust basis lacks trusted attestation");
        }
        _ => {}
    }

    Ok(state)
}

pub(crate) fn apply_monitor_capture_state(
    record: &mut EvidenceRecord,
    state: &MonitorCaptureState,
) {
    let projection = state.projection();
    record.capture_mode = projection.capture_mode.to_owned();
    record.capture_status = projection.capture_status;
    record.monitor_evidence_status = projection.monitor_evidence_status.to_owned();
    record.trust_basis = projection.trust_basis.to_owned();
    record.trusted_output = projection.trusted_output;
    record.conclusion = projection.conclusion.to_owned();
}

pub(crate) fn parse_boot_transcript_status(value: &str) -> Result<BootTranscriptStatus> {
    match value {
        "trusted" => Ok(BootTranscriptStatus::Trusted),
        "missing" => Ok(BootTranscriptStatus::Missing),
        "untrusted" => Ok(BootTranscriptStatus::Untrusted),
        "not_captured" => Ok(BootTranscriptStatus::NotCaptured),
        "not_requested" => Ok(BootTranscriptStatus::NotRequested),
        _ => bail!("unknown boot transcript status"),
    }
}

pub(crate) fn parse_runtime_attestation_evidence_status(
    value: &str,
) -> Result<RuntimeAttestationEvidenceStatus> {
    let status = match value {
        "trusted" => RuntimeAttestationStatus::Trusted,
        "missing" => RuntimeAttestationStatus::Missing,
        "malformed" => RuntimeAttestationStatus::Malformed,
        "insufficient_samples" => RuntimeAttestationStatus::InsufficientSamples,
        "mixed_session_or_ordinal" => RuntimeAttestationStatus::MixedSessionOrOrdinal,
        "static_fields_mismatch" => RuntimeAttestationStatus::StaticFieldsMismatch,
        "non_monotonic_uptime" => RuntimeAttestationStatus::NonMonotonicUptime,
        "package_identity_mismatch" => RuntimeAttestationStatus::PackageIdentityMismatch,
        "incomplete_readiness" => RuntimeAttestationStatus::IncompleteReadiness,
        "not_captured" => return Ok(RuntimeAttestationEvidenceStatus::NotCaptured),
        "not_requested" => return Ok(RuntimeAttestationEvidenceStatus::NotRequested),
        _ => bail!("unknown runtime attestation status"),
    };
    Ok(RuntimeAttestationEvidenceStatus::Observed(status))
}
