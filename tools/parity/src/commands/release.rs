use crate::*;

pub(crate) fn run_release_evidence_command(
    args: ReleaseEvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let manifest_path = environment.workspace_path(&args.manifest);
    let manifest_json = std::fs::read_to_string(manifest_path.as_std_path())
        .with_context(|| format!("failed to read package manifest {manifest_path}"))?;
    let manifest = parse_release_evidence_manifest_json(&manifest_json, &args.manifest)?;
    let current_git_head = environment.current_git_head()?;
    let source_commit_is_ancestor_of_head = args.allow_post_source_evidence_commits
        && current_git_head != manifest.source_commit
        && environment.source_commit_is_ancestor_of_head(&manifest.source_commit)?;
    let post_source_changed_paths =
        if args.allow_post_source_evidence_commits && current_git_head != manifest.source_commit {
            environment.changed_paths_since(&manifest.source_commit)?
        } else {
            Vec::new()
        };

    let maybe_flash_evidence = if let Some(flash_evidence_path) = &args.maybe_flash_evidence_json {
        let workspace_flash_evidence_path = environment.workspace_path(flash_evidence_path);
        let flash_evidence_json =
            std::fs::read_to_string(workspace_flash_evidence_path.as_std_path()).with_context(
                || format!("failed to read flash evidence {workspace_flash_evidence_path}"),
            )?;
        Some(parse_flash_evidence_json(
            &flash_evidence_json,
            &workspace_flash_evidence_path,
        )?)
    } else {
        None
    };

    let maybe_redaction_review = if let Some(redaction_review_path) = &args.maybe_redaction_review {
        let workspace_redaction_review_path = environment.workspace_path(redaction_review_path);
        Some(
            std::fs::read_to_string(workspace_redaction_review_path.as_std_path()).with_context(
                || format!("failed to read redaction review {workspace_redaction_review_path}"),
            )?,
        )
    } else {
        None
    };
    let (evidence_root, maybe_flash_evidence_json_path) =
        release_evidence_validation_paths(&args, environment);

    let documents = ReleaseEvidenceDocuments {
        manifest,
        current_git_head,
        allow_post_source_evidence_commits: args.allow_post_source_evidence_commits,
        source_commit_is_ancestor_of_head,
        post_source_changed_paths,
        evidence_root,
        maybe_flash_evidence_json_path,
        maybe_flash_evidence,
        maybe_redaction_review,
    };
    let report = validate_release_evidence(&documents, args.require_redaction_passed);
    let output = render_release_evidence_report(&documents, &report);

    if !report.passed() {
        bail!("release evidence failed:\n{output}");
    }

    Ok(output)
}

pub(crate) fn release_evidence_validation_paths(
    args: &ReleaseEvidenceArgs,
    environment: &LocalEnvironment,
) -> (Utf8PathBuf, Option<Utf8PathBuf>) {
    (
        environment.workspace_path(&args.evidence_root),
        args.maybe_flash_evidence_json
            .as_ref()
            .map(|path| environment.workspace_path(path)),
    )
}

pub(crate) fn run_mining_allow_command(
    args: MiningAllowArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let manifest_path = environment.workspace_path(&args.manifest);
    let documents =
        mining_allow::load_mining_allow_documents(&environment.workspace_dir, &manifest_path)?;
    let filters = mining_allow::MiningAllowFilters {
        maybe_surface: args.maybe_surface,
        maybe_allowed_command: args.maybe_allowed_command,
    };
    let report = mining_allow::validate_mining_allow_documents(&documents, &filters);
    let output = mining_allow::render_mining_allow_report(&documents.manifest, &report);

    if !report.passed() {
        bail!("mining allow failed:\n{output}");
    }

    Ok(output)
}

pub(crate) fn run_operator_evidence_command(
    args: OperatorEvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let evidence_root = environment.workspace_path(&args.evidence_root);
    let documents = load_operator_evidence_documents(&evidence_root)?;
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: args.require_redaction_passed,
    };
    let report = validate_operator_evidence_documents_with_snapshot_coherence(
        args.profile,
        &documents,
        &filters,
        args.require_operator_snapshot_coherence,
    );
    let output = render_operator_evidence_report(&documents, &report);

    if !report.passed() {
        bail!("operator evidence failed:\n{output}");
    }

    Ok(output)
}

pub(crate) fn run_safety_allow_command(
    args: SafetyAllowArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let manifest_path = environment.workspace_path(&args.manifest);
    let documents =
        safety_allow::load_safety_allow_documents(&environment.workspace_dir, &manifest_path)?;
    let filters = safety_allow::SafetyAllowFilters {
        maybe_surface: args.maybe_surface,
        maybe_allowed_command: args.maybe_allowed_command,
    };
    let report = safety_allow::validate_safety_allow_documents(&documents, &filters);
    let output = safety_allow::render_safety_allow_report(&documents.manifest, &report);

    if !report.passed() {
        bail!("safety allow failed:\n{output}");
    }

    Ok(output)
}

pub(crate) fn run_release_gate_command(
    args: ReleaseGateArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let license_inventory_path = environment.workspace_path(&args.license_inventory);
    let provenance_path = environment.workspace_path(&args.provenance);
    let cargo_about_path = environment.workspace_path(&args.cargo_about);
    let maybe_manifest_path = args
        .manifest
        .as_ref()
        .map(|manifest| environment.workspace_path(manifest));

    let license_inventory_markdown = std::fs::read_to_string(license_inventory_path.as_std_path())
        .with_context(|| format!("failed to read license inventory {license_inventory_path}"))?;
    let provenance_markdown = std::fs::read_to_string(provenance_path.as_std_path())
        .with_context(|| format!("failed to read provenance manifest {provenance_path}"))?;
    let maybe_cargo_about_html = maybe_read_text(&cargo_about_path)?;
    let maybe_manifest_json = if let Some(manifest_path) = &maybe_manifest_path {
        maybe_read_text(manifest_path)?
    } else {
        None
    };

    let documents = ReleaseGateDocuments {
        license_inventory_path: args.license_inventory,
        license_inventory_markdown,
        provenance_path: args.provenance,
        provenance_markdown,
        cargo_about_path: args.cargo_about,
        maybe_cargo_about_html,
        maybe_manifest_path: args.manifest,
        maybe_manifest_json,
    };
    let report = validate_release_gate(&documents);
    let output = render_release_gate_report(&report);

    if !report.passed() {
        bail!("release gate failed:\n{output}");
    }

    Ok(output)
}

pub(crate) fn run_api_compare_command(
    args: ApiCompareArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let openapi_path = environment.workspace_path(&args.openapi);
    let route_manifest_path = environment.workspace_path(&args.route_manifest);
    let static_usage_path = environment.workspace_path(&args.static_usage);

    let openapi_yaml = std::fs::read_to_string(openapi_path.as_std_path())
        .with_context(|| format!("failed to read OpenAPI contract {openapi_path}"))?;
    let route_manifest_json = std::fs::read_to_string(route_manifest_path.as_std_path())
        .with_context(|| format!("failed to read API compare manifest {route_manifest_path}"))?;
    let static_usage_json = std::fs::read_to_string(static_usage_path.as_std_path())
        .with_context(|| format!("failed to read AxeOS route usage fixture {static_usage_path}"))?;

    let request = api_compare::ApiCompareRequest {
        openapi_yaml: &openapi_yaml,
        route_manifest_json: &route_manifest_json,
        static_usage_json: &static_usage_json,
    };
    let loader = api_compare::WorkspaceFixtureLoader::new(environment.workspace_dir.clone());
    let report = api_compare::run_api_compare(&request, &loader)?;
    let output = api_compare::render_api_compare_report(&report);

    if report.has_validation_errors() {
        bail!("api compare failed:\n{output}");
    }

    Ok(output)
}
