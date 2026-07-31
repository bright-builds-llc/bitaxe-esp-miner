use super::markers::{
    campaign_marker_failure, CampaignFailureMarker, ObservationFreshnessMarker,
    ObservationRequirementsMarker, PoolConfigMarker, SafeStopMarker, SafetyMarker,
    SubmitOutcomeMarker,
};
use super::*;

#[derive(Serialize)]
struct CampaignObservationEvidence<'a> {
    schema: &'static str,
    stage: &'static str,
    markers: &'a [CampaignStatusMarker],
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
    serial_outcome_detail: &'static str,
    pool_config: &'static str,
    marker_count: usize,
    submit_outcome: &'static str,
    qualified_candidate_count: u64,
    below_pool_target_count: u64,
    duplicate_candidate_count: u64,
    terminal_reason: &'static str,
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
    redacted: bool,
    parity_promotion: bool,
}

#[derive(Clone)]
pub(super) struct CampaignEvidencePaths {
    diagnostics: Utf8PathBuf,
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
        observations: root.join("campaign-observations.private.json"),
        result: root.join("campaign-result.json"),
        seal: root.join("campaign-result.sha256"),
    };
    for path in [
        &paths.diagnostics,
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

        let observations = CampaignObservationEvidence {
            schema: CAMPAIGN_OBSERVATIONS_SCHEMA,
            stage: command.stage.as_str(),
            markers: &attempt.markers,
        };
        let mut observation_bytes = serde_json::to_vec_pretty(&observations)?;
        observation_bytes.push(b'\n');
        write_private_new_bytes(&paths.observations, &observation_bytes)
            .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::EvidenceSealFailed))?;
        let observations_sha256 = sha256_bytes(&observation_bytes);

        let maybe_terminal = attempt.markers.last();
        let maybe_failure_marker = maybe_admission.and_then(|admission| {
            attempt.markers.iter().find(|marker| {
                campaign_marker_failure(marker, admission) == Some(terminal_category)
            })
        });
        let evidence = CampaignResultEvidence {
            schema: CAMPAIGN_RESULT_SCHEMA,
            evidence_class: PROTECTED_OPERATIONAL,
            stage: command.stage.as_str(),
            profile: maybe_admission
                .and_then(|admission| admission.maybe_profile)
                .map_or("none", MiningCampaignProfile::as_str),
            duration_seconds: command.duration_seconds,
            status: if result.is_ok() { "accepted" } else { "failed" },
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
            serial_outcome_detail: attempt.serial_outcome_detail.as_str(),
            pool_config: maybe_terminal.map_or("not_observed", |marker| match marker.pool_config {
                PoolConfigMarker::NotRead => "not_read",
                PoolConfigMarker::LocalOwnerSupplied => "local_owner_supplied",
            }),
            marker_count: attempt.markers.len(),
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
            terminal_reason: maybe_terminal
                .map_or("not_observed", |marker| marker.terminal_reason.label()),
            active_ms: maybe_terminal.map_or(0, |marker| marker.active_ms),
            safety: maybe_terminal.map_or("not_observed", |marker| match marker.safety {
                SafetyMarker::Fresh => "fresh",
                SafetyMarker::Stale => "stale",
            }),
            fresh_observation_count: maybe_terminal
                .map_or(0, |marker| marker.fresh_observation_count),
            observation_freshness: maybe_terminal.map(|marker| &marker.observation_freshness),
            observation_requirements: maybe_terminal.map(|marker| &marker.observation_requirements),
            failure_observation_freshness: maybe_failure_marker
                .map(|marker| &marker.observation_freshness),
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
