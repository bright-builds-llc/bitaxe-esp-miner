use crate::*;

pub(crate) fn run_classify_phase35_flash_command(
    args: ClassifyPhase35FlashArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    use phase35_flash::{classify_phase35_flash, FlashBoundary};

    let metrics_input = environment.workspace_path(&args.metrics_input);
    let private_log_input = environment.workspace_path(&args.private_log_input);
    let projection_output = environment.workspace_path(&args.projection_output);
    let metrics_metadata = validate_private_input(&metrics_input)?;
    let log_metadata = validate_private_input(&private_log_input)?;
    if (metrics_metadata.dev(), metrics_metadata.ino()) == (log_metadata.dev(), log_metadata.ino())
    {
        bail!("Phase 35 flash boundary input aliases another input");
    }
    let identities = [
        canonical_private_path(&metrics_input)?,
        canonical_private_path(&private_log_input)?,
        validate_private_output(&projection_output)?,
    ];
    for left in 0..identities.len() {
        for right in (left + 1)..identities.len() {
            if identities[left] == identities[right] {
                bail!("Phase 35 flash boundary paths must be distinct");
            }
        }
    }

    let metrics = fs::read(metrics_input.as_std_path())
        .context("failed to read private Phase 35 flash metrics")?;
    let private_log = fs::read(private_log_input.as_std_path())
        .context("failed to read private Phase 35 flash child log")?;
    let projection = classify_phase35_flash(&metrics, &private_log)
        .map_err(|_| anyhow::anyhow!("category=flash_boundary_invalid"))?;
    let terminal_boundary = projection.terminal_boundary;
    let mut projection_bytes =
        serde_json::to_vec_pretty(&projection).context("failed to encode flash projection")?;
    projection_bytes.push(b'\n');
    write_private_new(&projection_output, &projection_bytes)?;

    if terminal_boundary != FlashBoundary::Ready {
        bail!("category={}", terminal_boundary.as_str());
    }
    Ok("category=ready".to_owned())
}

pub(crate) fn run_probe_phase35_http_command(
    args: ProbePhase35HttpArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    use phase35_http_probe::probe_phase35_http;

    let metrics_output = environment.workspace_path(&args.metrics_output);
    let headers_output = environment.workspace_path(&args.headers_output);
    let body_output = environment.workspace_path(&args.body_output);
    let identities = [
        validate_private_output(&metrics_output)?,
        validate_private_output(&headers_output)?,
        validate_private_output(&body_output)?,
    ];
    for left in 0..identities.len() {
        for right in (left + 1)..identities.len() {
            if identities[left] == identities[right] {
                bail!("Phase 35 HTTP probe output paths must be distinct");
            }
        }
    }

    let result = probe_phase35_http(&args.url).map_err(|_| {
        anyhow::anyhow!("Phase 35 HTTP probe rejected its private request contract")
    })?;
    let mut metrics =
        serde_json::to_vec_pretty(&result.metrics).context("failed to encode HTTP metrics")?;
    metrics.push(b'\n');
    write_private_new(&metrics_output, &metrics)?;
    write_private_new(&headers_output, &result.headers)?;
    write_private_new(&body_output, &result.body)?;
    Ok("status=probe_complete".to_owned())
}

pub(crate) fn run_classify_phase35_http_command(
    args: ClassifyPhase35HttpArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    use phase35_http::{classify_phase35_http, HttpTerminalCategory};

    let metrics_input = environment.workspace_path(&args.metrics_input);
    let body_input = environment.workspace_path(&args.body_input);
    let projection_output = environment.workspace_path(&args.projection_output);
    let hostname_output = environment.workspace_path(&args.hostname_output);

    let metrics_metadata = validate_private_input(&metrics_input)?;
    let body_metadata = validate_private_input(&body_input)?;
    if (metrics_metadata.dev(), metrics_metadata.ino())
        == (body_metadata.dev(), body_metadata.ino())
    {
        bail!("Phase 35 HTTP diagnostic input aliases another input");
    }

    let metrics_identity = canonical_private_path(&metrics_input)?;
    let body_identity = canonical_private_path(&body_input)?;
    let projection_identity = validate_private_output(&projection_output)?;
    let hostname_identity = validate_private_output(&hostname_output)?;
    let identities = [
        metrics_identity,
        body_identity,
        projection_identity,
        hostname_identity,
    ];
    for left in 0..identities.len() {
        for right in (left + 1)..identities.len() {
            if identities[left] == identities[right] {
                bail!("Phase 35 HTTP diagnostic paths must be distinct");
            }
        }
    }

    let metrics = fs::read(metrics_input.as_std_path())
        .context("failed to read private Phase 35 HTTP metrics")?;
    let body =
        fs::read(body_input.as_std_path()).context("failed to read private Phase 35 HTTP body")?;
    let classified = classify_phase35_http(&metrics, &body)
        .map_err(|_| anyhow::anyhow!("category=http_diagnostic_invalid"))?;
    let mut projection =
        serde_json::to_vec_pretty(&classified.projection).context("failed to encode projection")?;
    projection.push(b'\n');
    write_private_new(&projection_output, &projection)?;

    if classified.terminal_category != HttpTerminalCategory::Ready {
        bail!("category={}", classified.terminal_category.as_str());
    }
    let hostname = classified
        .maybe_hostname
        .as_deref()
        .context("ready HTTP classification omitted private hostname")?;
    let mut hostname_bytes = hostname.as_bytes().to_vec();
    hostname_bytes.push(b'\n');
    write_private_new(&hostname_output, &hostname_bytes)?;

    Ok("category=ready".to_owned())
}

pub(crate) fn run_admit_phase35_evidence_command(
    args: AdmitPhase35EvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    use phase35_evidence::{
        detector_run_capability_digest, inventory_artifact_digest, inventory_artifact_equals,
        load_phase35_evidence_root, validate_phase35_evidence, Phase35EvidenceError,
        PHASE35_LIFECYCLE_ID,
    };
    use phase35_promotion::{
        evaluate_phase35_promotion, ChecklistSnapshot, Phase35EvidenceSource, Phase35LiveRechecks,
    };

    let evidence_root = environment.workspace_path(&args.root);
    let (input, artifacts) =
        load_phase35_evidence_root(&evidence_root).map_err(anyhow::Error::msg)?;
    let validated = validate_phase35_evidence(&input, &artifacts).map_err(anyhow::Error::msg)?;

    let current_head = environment
        .current_git_head()
        .map_err(|_| anyhow::anyhow!(Phase35EvidenceError::StaleCurrentHead))?;
    environment
        .run_reference_guard()
        .map_err(|_| anyhow::anyhow!(Phase35EvidenceError::DirtyReference))?;
    let reference_commit = environment
        .reference_commit()
        .map_err(|_| anyhow::anyhow!(Phase35EvidenceError::DirtyReference))?;
    let actual_digest = |role: &str| {
        inventory_artifact_digest(&input, &artifacts, role).map_err(anyhow::Error::msg)
    };
    let no_actuation_verified = inventory_artifact_equals(
        &input,
        &artifacts,
        "no_actuation",
        b"no_actuation_verified=true\n",
    )
    .map_err(anyhow::Error::msg)?;
    let live = Phase35LiveRechecks {
        lifecycle_id: PHASE35_LIFECYCLE_ID.to_owned(),
        current_head,
        reference_commit,
        reference_clean: true,
        manifest_schema: input.exact_package.manifest_schema.clone(),
        manifest_digest: actual_digest("package_manifest")?,
        executable_image_digest: actual_digest("executable_image")?,
        factory_image_digest: actual_digest("factory_image")?,
        package_digest: actual_digest("package")?,
        runtime_identity_digest: actual_digest("runtime_identity")?,
        detector_capability_digest: detector_run_capability_digest(&input.detector_run),
        detector_single_candidate: input.detector_run.single_candidate_verified,
        detector_board_info: input.detector_run.board_info_verified,
        board_category: input.detector_run.board_category.clone(),
        root_contract_digest: input.admission_facts.root_contract_digest.clone(),
        root_event_chain_verified: true,
        no_actuation_verified,
        evidence_sources: vec![Phase35EvidenceSource::ProtectedEvidenceRoot],
    };

    let checklist_contents = read_phase36_public_checklist(
        &environment.workspace_dir,
        Utf8Path::new(PHASE36_STAGING_ROOT),
        Utf8Path::new(PHASE36_DESTINATION_ROOT),
        Utf8Path::new(PHASE35_CHECKLIST_PATH),
        Utf8Path::new(PHASE35_MANIFEST_PATH),
    )
    .map_err(anyhow::Error::msg)?;
    let checklist =
        ChecklistSnapshot::capture(checklist_contents, live).map_err(anyhow::Error::msg)?;
    let matrix = evaluate_phase35_promotion(&validated, &checklist).map_err(anyhow::Error::msg)?;
    let projection = validated
        .shareable_projection()
        .map_err(anyhow::Error::msg)?;
    let documents = Phase35GenerationDocuments {
        projection_json: serde_json::to_string_pretty(&projection)?,
        matrix_json: serde_json::to_string_pretty(&matrix)?,
        projected_checklist: matrix.projected_checklist.clone(),
        expected_checklist_fingerprint: matrix.checklist_fingerprint_before.clone(),
    };
    publish_phase35_generation(
        &environment.workspace_dir,
        &args.staging,
        Utf8Path::new(PHASE35_DESTINATION_ROOT),
        Utf8Path::new(PHASE35_CHECKLIST_PATH),
        &documents,
        Phase35PublicationOptions::default(),
    )
    .map_err(anyhow::Error::msg)?;

    serde_json::to_string_pretty(&serde_json::json!({
        "status": "admitted",
        "evidence_root_digest": matrix.evidence_root_digest,
        "promoted_rows": matrix.promoted_row_ids(),
    }))
    .context("failed to serialize Phase 35 admission result")
}

pub(crate) fn run_validate_phase35_evidence_command(
    args: ValidatePhase35EvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    use phase35_evidence::{
        load_phase35_evidence_root, validate_phase35_evidence, Phase35EvidenceError,
    };

    let evidence_root = environment.workspace_path(&args.root);
    let (input, artifacts) =
        load_phase35_evidence_root(&evidence_root).map_err(|error| anyhow::anyhow!(error))?;
    let validated =
        validate_phase35_evidence(&input, &artifacts).map_err(|error| anyhow::anyhow!(error))?;

    let current_head = environment
        .current_git_head()
        .map_err(|_| anyhow::anyhow!(Phase35EvidenceError::StaleCurrentHead))?;
    if current_head != input.exact_package.source_commit {
        return Err(anyhow::anyhow!(Phase35EvidenceError::StaleCurrentHead));
    }
    environment
        .run_reference_guard()
        .map_err(|_| anyhow::anyhow!(Phase35EvidenceError::DirtyReference))?;
    let reference_commit = environment
        .reference_commit()
        .map_err(|_| anyhow::anyhow!(Phase35EvidenceError::DirtyReference))?;
    if reference_commit != input.exact_package.reference_commit {
        return Err(anyhow::anyhow!(Phase35EvidenceError::DirtyReference));
    }
    let projection = validated
        .shareable_projection()
        .map_err(|error| anyhow::anyhow!(error))?;
    serde_json::to_string_pretty(&projection)
        .map_err(|_| anyhow::anyhow!(Phase35EvidenceError::ForbiddenProjectionField))
}
