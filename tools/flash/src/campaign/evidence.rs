use super::markers::{
    AsicBridgeMarker, CampaignAsicEventTrace, CampaignFailureMarker, JobTransitionMarker,
    ObservationFreshnessMarker, ObservationRequirementsMarker, OperatorSensorDiagnosticMarker,
    PoolConfigMarker, ReadinessTransitionMarker, SafeStopMarker, SafetyMarker, SubmitOutcomeMarker,
};
use super::*;

#[derive(Serialize)]
struct CampaignObservationEvidence<'a> {
    schema: &'static str,
    stage: &'static str,
    marker_count: u64,
    maximum_active_marker_gap_ms: u64,
    terminal_marker: Option<&'a super::markers::CampaignStatusMarker>,
}

#[derive(Serialize)]
struct CampaignResultEvidence<'a> {
    schema: &'static str,
    evidence_class: &'static str,
    stage: &'static str,
    profile: &'static str,
    duration_seconds: u64,
    status: &'static str,
    terminal_category: &'static str,
    package_admitted: bool,
    runtime_identity: &'static str,
    runtime_attestation_status: &'static str,
    runtime_attestation_parse_failure: &'static str,
    runtime_attestation_parse_failure_counts:
        &'a serial::RuntimeAttestationParseFailureCountsEvidence,
    serial_outcome_detail: &'static str,
    pool_config: &'static str,
    marker_count: u64,
    submit_outcome: &'static str,
    qualified_candidate_count: u64,
    below_pool_target_count: u64,
    duplicate_candidate_count: u64,
    accepted_share_count: u64,
    rejected_share_count: u64,
    job_transition: Option<&'a JobTransitionMarker>,
    asic_bridge: Option<&'a AsicBridgeMarker>,
    maximum_active_marker_gap_ms: u64,
    terminal_reason: &'static str,
    protocol_gate: &'static str,
    readiness_transition: Option<&'a ReadinessTransitionMarker>,
    operator_sensor: Option<&'a OperatorSensorDiagnosticMarker>,
    active_ms: u64,
    safety: &'static str,
    fresh_observation_count: u8,
    observation_freshness: Option<&'a ObservationFreshnessMarker>,
    observation_requirements: Option<&'a ObservationRequirementsMarker>,
    failure_observation_freshness: Option<&'a ObservationFreshnessMarker>,
    campaign_failure: Option<&'a CampaignFailureMarker>,
    mineonboot: Option<bool>,
    safe_stop: &'static str,
    usb_cleanup: &'static str,
    observations_sha256: &'a str,
    diagnostics_sha256: &'a str,
    flash_diagnostics_sha256: &'a str,
    mining_diagnostics_sha256: &'a str,
    network_continuity_sha256: &'a str,
    network_status: &'a str,
    network_required_window_count: usize,
    network_covered_window_count: usize,
    watchdog_valid: bool,
    watchdog_failure: &'a str,
    work_renewal_valid: bool,
    terminal_http_valid: bool,
    terminal_websocket_valid: bool,
    terminal_pool_persisted: bool,
    redacted: bool,
    parity_promotion: bool,
}

#[derive(Clone)]
pub(super) struct CampaignEvidencePaths {
    diagnostics: Utf8PathBuf,
    flash_diagnostics: Utf8PathBuf,
    mining_diagnostics: Utf8PathBuf,
    network_continuity: Utf8PathBuf,
    observations: Utf8PathBuf,
    result: Utf8PathBuf,
    seal: Utf8PathBuf,
}

pub(super) fn preflight_campaign_evidence(root: &Utf8Path) -> Result<CampaignEvidencePaths> {
    match fs::symlink_metadata(root.as_std_path()) {
        Ok(_) => bail!("campaign evidence attempt already exists"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let parent = root.parent().context("campaign evidence parent missing")?;
    let parent_metadata = fs::symlink_metadata(parent.as_std_path())?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        bail!("campaign evidence parent invalid");
    }
    #[cfg(unix)]
    if parent_metadata.permissions().mode() & 0o777 != 0o700 {
        bail!("campaign evidence parent is not private");
    }
    fs::create_dir(root.as_std_path())?;
    set_private_directory_mode(root)?;
    #[cfg(unix)]
    if fs::metadata(root.as_std_path())?.permissions().mode() & 0o777 != 0o700 {
        bail!("campaign evidence root is not private");
    }
    let paths = CampaignEvidencePaths {
        diagnostics: root.join("campaign-diagnostics.private.json"),
        flash_diagnostics: root.join("campaign-flash.private.json"),
        mining_diagnostics: root.join("campaign-mining-diagnostics.private.json"),
        network_continuity: root.join("campaign-network.private.json"),
        observations: root.join("campaign-observations.private.json"),
        result: root.join("campaign-result.json"),
        seal: root.join("campaign-result.sha256"),
    };
    for path in [
        &paths.diagnostics,
        &paths.flash_diagnostics,
        &paths.mining_diagnostics,
        &paths.network_continuity,
        &paths.observations,
        &paths.result,
        &paths.seal,
    ] {
        if path.parent() != Some(root) || fs::symlink_metadata(path.as_std_path()).is_ok() {
            bail!("campaign evidence destination invalid");
        }
    }
    Ok(paths)
}

pub(super) fn finish_campaign_attempt(
    command: &MiningCampaignCommand,
    maybe_admission: Option<CampaignAdmission>,
    paths: &CampaignEvidencePaths,
    attempt: &CampaignAttempt,
    result: std::result::Result<CampaignTerminalCategory, CampaignFailure>,
) -> Result<()> {
    let terminal_category = match &result {
        Ok(category) => *category,
        Err(failure) => failure.category,
    };
    let seal_result = (|| -> Result<()> {
        let mut diagnostic_bytes = serde_json::to_vec_pretty(&attempt.serial_diagnostics)?;
        diagnostic_bytes.push(b'\n');
        write_private_new_bytes(&paths.diagnostics, &diagnostic_bytes)
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        let diagnostics_sha256 = sha256_bytes(&diagnostic_bytes);

        let flash_diagnostics = CampaignFlashDiagnosticsEvidence {
            schema: CAMPAIGN_FLASH_DIAGNOSTICS_SCHEMA,
            factory: attempt.factory_flash_diagnostic.as_ref(),
            nvs: attempt.nvs_flash_diagnostic.as_ref(),
            raw_output_included: false,
        };
        let mut flash_diagnostic_bytes = serde_json::to_vec_pretty(&flash_diagnostics)?;
        flash_diagnostic_bytes.push(b'\n');
        write_private_new_bytes(&paths.flash_diagnostics, &flash_diagnostic_bytes)
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        let flash_diagnostics_sha256 = sha256_bytes(&flash_diagnostic_bytes);

        let mining_diagnostics = CampaignMiningDiagnosticsEvidence {
            schema: CAMPAIGN_MINING_DIAGNOSTICS_SCHEMA,
            asic_bridge: attempt
                .marker_aggregate
                .terminal
                .as_ref()
                .map(|marker| &marker.asic_bridge),
            event_trace: &attempt.marker_aggregate.asic_event_trace,
        };
        let mut mining_diagnostic_bytes = serde_json::to_vec_pretty(&mining_diagnostics)?;
        mining_diagnostic_bytes.push(b'\n');
        write_private_new_bytes(&paths.mining_diagnostics, &mining_diagnostic_bytes)
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        let mining_diagnostics_sha256 = sha256_bytes(&mining_diagnostic_bytes);

        let mut network_bytes = serde_json::to_vec_pretty(&attempt.network_evidence)?;
        network_bytes.push(b'\n');
        write_private_new_bytes(&paths.network_continuity, &network_bytes)
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        let network_continuity_sha256 = sha256_bytes(&network_bytes);

        let observations = CampaignObservationEvidence {
            schema: CAMPAIGN_OBSERVATIONS_SCHEMA,
            stage: command.stage.as_str(),
            marker_count: attempt.marker_aggregate.marker_count,
            maximum_active_marker_gap_ms: attempt.marker_aggregate.maximum_active_marker_gap_ms,
            terminal_marker: attempt.marker_aggregate.terminal.as_ref(),
        };
        let mut observation_bytes = serde_json::to_vec_pretty(&observations)?;
        observation_bytes.push(b'\n');
        write_private_new_bytes(&paths.observations, &observation_bytes)
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        let observations_sha256 = sha256_bytes(&observation_bytes);

        let maybe_terminal = attempt.marker_aggregate.terminal.as_ref();
        let evidence = CampaignResultEvidence {
            schema: CAMPAIGN_RESULT_SCHEMA,
            evidence_class: PROTECTED_OPERATIONAL,
            stage: command.stage.as_str(),
            profile: maybe_admission
                .and_then(|admission| admission.maybe_profile)
                .map_or("none", MiningCampaignProfile::as_str),
            duration_seconds: command.duration_seconds,
            status: match &result {
                Ok(CampaignTerminalCategory::JobTransitionNotObserved) => "inconclusive",
                Ok(_) => "accepted",
                Err(_) => "failed",
            },
            terminal_category: terminal_category.as_str(),
            package_admitted: attempt.package_admitted,
            runtime_identity: if attempt.runtime_identity_trusted {
                "trusted"
            } else {
                "not_trusted"
            },
            runtime_attestation_status: attempt
                .maybe_runtime_attestation_status
                .map_or("not_observed", RuntimeAttestationStatus::label),
            runtime_attestation_parse_failure: attempt
                .serial_diagnostics
                .runtime_attestation_parse_failure(),
            runtime_attestation_parse_failure_counts: attempt
                .serial_diagnostics
                .runtime_attestation_parse_failure_counts(),
            serial_outcome_detail: attempt.serial_outcome_detail.as_str(),
            pool_config: maybe_terminal.map_or("not_observed", |marker| match marker.pool_config {
                PoolConfigMarker::NotRead => "not_read",
                PoolConfigMarker::LocalOwnerSupplied => "local_owner_supplied",
            }),
            marker_count: attempt.marker_aggregate.marker_count,
            submit_outcome: maybe_terminal.map_or("none", |marker| match marker.submit_outcome {
                SubmitOutcomeMarker::None => "none",
                SubmitOutcomeMarker::Accepted => "accepted",
                SubmitOutcomeMarker::Rejected => "rejected",
            }),
            qualified_candidate_count: maybe_terminal
                .map_or(0, |marker| marker.qualified_candidate_count),
            below_pool_target_count: maybe_terminal
                .map_or(0, |marker| marker.below_pool_target_count),
            duplicate_candidate_count: maybe_terminal
                .map_or(0, |marker| marker.duplicate_candidate_count),
            accepted_share_count: maybe_terminal.map_or(0, |marker| marker.accepted_share_count),
            rejected_share_count: maybe_terminal.map_or(0, |marker| marker.rejected_share_count),
            job_transition: maybe_terminal.map(|marker| &marker.job_transition),
            asic_bridge: maybe_terminal.map(|marker| &marker.asic_bridge),
            maximum_active_marker_gap_ms: attempt.marker_aggregate.maximum_active_marker_gap_ms,
            terminal_reason: maybe_terminal
                .map_or("not_observed", |marker| marker.terminal_reason.label()),
            protocol_gate: maybe_terminal
                .map_or("not_observed", |marker| marker.protocol_gate.label()),
            readiness_transition: maybe_terminal.map(|marker| &marker.readiness_transition),
            operator_sensor: maybe_terminal.map(|marker| &marker.operator_sensor),
            active_ms: maybe_terminal.map_or(0, |marker| marker.active_ms),
            safety: maybe_terminal.map_or("not_observed", |marker| match marker.safety {
                SafetyMarker::Fresh => "fresh",
                SafetyMarker::Stale => "stale",
            }),
            fresh_observation_count: maybe_terminal
                .map_or(0, |marker| marker.fresh_observation_count),
            observation_freshness: maybe_terminal.map(|marker| &marker.observation_freshness),
            observation_requirements: maybe_terminal.map(|marker| &marker.observation_requirements),
            failure_observation_freshness: attempt
                .marker_aggregate
                .failure_observation_freshness
                .as_ref(),
            campaign_failure: maybe_terminal.map(|marker| &marker.failure),
            mineonboot: maybe_terminal.map(|marker| marker.mineonboot),
            safe_stop: maybe_terminal.map_or("not_observed", |marker| match marker.safe_stop {
                SafeStopMarker::NotRequired => "not_required",
                SafeStopMarker::Pending => "pending",
                SafeStopMarker::Confirmed => "confirmed",
            }),
            usb_cleanup: if attempt.usb_cleanup_complete {
                "ready"
            } else {
                "not_proven"
            },
            observations_sha256: &observations_sha256,
            diagnostics_sha256: &diagnostics_sha256,
            flash_diagnostics_sha256: &flash_diagnostics_sha256,
            mining_diagnostics_sha256: &mining_diagnostics_sha256,
            network_continuity_sha256: &network_continuity_sha256,
            network_status: attempt.network_evidence.status,
            network_required_window_count: attempt.network_evidence.required_window_count,
            network_covered_window_count: attempt.network_evidence.covered_window_count,
            watchdog_valid: attempt.network_evidence.watchdog_valid,
            watchdog_failure: attempt.network_evidence.watchdog_failure,
            work_renewal_valid: attempt.network_evidence.work_renewal_valid,
            terminal_http_valid: attempt.network_evidence.terminal_http_valid,
            terminal_websocket_valid: attempt.network_evidence.terminal_websocket_valid,
            terminal_pool_persisted: attempt.network_evidence.terminal_pool_persisted,
            redacted: true,
            parity_promotion: false,
        };
        let mut result_bytes = serde_json::to_vec_pretty(&evidence)?;
        result_bytes.push(b'\n');
        write_private_new_bytes(&paths.result, &result_bytes)
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        let result_sha256 = sha256_bytes(&result_bytes);
        write_private_new_bytes(&paths.seal, format!("{result_sha256}\n").as_bytes())
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        Ok(())
    })();
    if let Err(seal_error) = seal_result {
        return match result {
            Err(failure) => {
                Err(anyhow::Error::new(failure).context("campaign_evidence_seal_failure=secondary"))
            }
            Ok(_) => Err(seal_error),
        };
    }

    emit_line("mining_campaign_stage", command.stage.as_str())?;
    emit_line("mining_campaign_result", terminal_category.as_str())?;
    emit_line("campaign_evidence", PROTECTED_OPERATIONAL)?;
    result.map(|_| ()).map_err(anyhow::Error::new)
}

#[derive(Serialize)]
struct CampaignMiningDiagnosticsEvidence<'a> {
    schema: &'static str,
    asic_bridge: Option<&'a AsicBridgeMarker>,
    event_trace: &'a CampaignAsicEventTrace,
}

#[derive(Serialize)]
struct CampaignFlashDiagnosticsEvidence<'a> {
    schema: &'static str,
    factory: Option<&'a UsbCommandDiagnostic>,
    nvs: Option<&'a UsbCommandDiagnostic>,
    raw_output_included: bool,
}
