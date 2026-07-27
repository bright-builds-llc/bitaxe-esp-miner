use crate::*;

mod phase33;
mod phase35;
mod release;

pub(crate) use phase33::*;
pub(crate) use phase35::*;
pub(crate) use release::*;

pub(crate) fn run_revise_checklist_documentation_command(
    args: &ReviseChecklistDocumentationArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let outcome =
        checklist_revision::publish_current_revision(&environment.workspace_dir, &args.change_spec)
            .map_err(anyhow::Error::msg)?;
    Ok(format!(
        "checklist_revision={} affected_rows={} checklist_sha256={}",
        outcome.revision_id, outcome.affected_rows, outcome.checklist_sha256
    ))
}

pub(crate) fn run_classify_phase36_evidence_command(
    args: &ClassifyPhase36EvidenceArgs,
) -> Result<String> {
    let classification = phase36_evidence::load_and_classify_phase36_root(&args.root)
        .map_err(|error| anyhow::anyhow!("category={error}"))?;
    serde_json::to_string_pretty(&classification)
        .map_err(|_| anyhow::anyhow!("category=partial_public_output"))
}

pub(crate) fn run_classify_phase36_effects_command(
    args: &ClassifyPhase36EffectsArgs,
) -> Result<String> {
    use phase36_evidence::effects::IndependentEffectAdmission;

    let admission = phase36_evidence::effects::classify_independent_effect_root(&args.root)
        .map_err(|error| anyhow::anyhow!("category={error}"))?;
    match admission {
        IndependentEffectAdmission::Insufficient { .. } => {
            Ok("category=independent_effect_observation_insufficient".to_owned())
        }
        validated @ IndependentEffectAdmission::Validated { .. } => {
            serde_json::to_string_pretty(&validated)
                .map_err(|_| anyhow::anyhow!("category=partial_public_output"))
        }
    }
}

pub(crate) fn run_phase36_synthetic_capture_command(
    args: &Phase36SyntheticCaptureArgs,
) -> Result<String> {
    let candidate = phase36_evidence::capture::write_synthetic_capture(
        &args.private_output,
        &args.candidate_output,
        &args.capability_digest,
    )
    .map_err(|error| anyhow::anyhow!("category={error}"))?;
    Ok(format!(
        "category=synthetic_complete\ncandidate_digest={}\nprivate_capture_digest={}",
        candidate.candidate_digest, candidate.private_capture_digest
    ))
}

pub(crate) fn run_phase36_hardware_capture_command(
    args: &Phase36HardwareCaptureArgs,
) -> Result<String> {
    if args.capture_timeout_seconds < 360 {
        anyhow::bail!("category=phase36_broker_capture_timeout_invalid");
    }
    if !args.private_parent.is_absolute()
        || !args.attempt_handle_file.is_absolute()
        || !args.candidate_output.is_absolute()
        || !args.wifi_credentials.is_absolute()
    {
        anyhow::bail!("category=phase36_broker_path_invalid");
    }

    let disposition = phase36_broker::run_phase36_hardware_transaction(
        args.board,
        &args.private_parent,
        &args.attempt_handle_file,
        &args.candidate_output,
        &args.wifi_credentials,
        args.capture_timeout_seconds,
    )
    .map_err(|error| anyhow::anyhow!("category={error}"))?;
    match disposition {
        phase36_broker::Phase36HardwareDisposition::SealedEligible => {
            Ok("category=sealed_eligible".to_owned())
        }
        phase36_broker::Phase36HardwareDisposition::SealedNonPromotion {
            first_failure,
            secondary_failure,
            recovery_disposition,
        } => Ok(format!(
            "category=sealed_non_promotion\nfirst_failure={first_failure:?}\nsecondary_failure={secondary_failure:?}\nrecovery_disposition={recovery_disposition:?}"
        )),
    }
}

pub(crate) fn run_inspect_phase36_candidate_command(
    args: &InspectPhase36CandidateArgs,
) -> Result<String> {
    let projection = phase36_evidence::capture::inspect_candidate_file(&args.candidate_input)
        .map_err(|error| anyhow::anyhow!("category={error}"))?;
    serde_json::to_string_pretty(&projection)
        .map_err(|_| anyhow::anyhow!("category=phase36_capture_encoding_failed"))
}

pub(crate) fn run_classify_phase36_candidate_command(
    args: &ClassifyPhase36CandidateArgs,
) -> Result<String> {
    let projection = phase36_evidence::capture::classify_candidate_files(
        &args.private_input,
        &args.candidate_input,
        &args.classification_output,
    )
    .map_err(|error| anyhow::anyhow!("category={error}"))?;
    Ok(format!(
        "category={}\ncandidate_digest={}\nprivate_capture_digest={}",
        projection.category, projection.candidate_digest, projection.private_capture_digest
    ))
}

pub(crate) fn run_reevaluate_phase36_attempt31_command(
    args: &ReevaluatePhase36Attempt31Args,
) -> Result<String> {
    let request = phase36_offline::Phase36OfflineRequest::from_args(args);
    let outcome = phase36_offline::reevaluate_attempt31(&request)
        .map_err(|error| anyhow::anyhow!("category={error}"))?;
    serde_json::to_string_pretty(&outcome)
        .map_err(|_| anyhow::anyhow!("category=partial_public_output"))
}
