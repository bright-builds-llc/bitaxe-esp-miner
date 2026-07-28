use crate::*;

pub(crate) fn run_flash(
    command: &FlashCommand,
    environment: &impl FlashEnvironment,
) -> Result<FlashOutcome> {
    let PreparedFlash {
        outcome,
        execution_command,
        _execution_snapshot,
    } = prepare_flash(command, environment)?;
    emit_flash_outcome(
        &outcome,
        command.common.evidence_mode != Some(EvidenceMode::Dual),
    )?;

    if !command.common.dry_run {
        let port = maybe_command_port(&execution_command)
            .context("usb_session=blocked reason=port_unavailable")?;
        environment.begin_usb_session(UsbOperation::Flash, &port)?;
        environment.execute(&execution_command)?;
        environment.phase35_stage_readiness_gate("after-factory", &port)?;
        if let Some(nvs_seed) = &outcome.nvs_seed {
            environment.execute(&nvs_seed.command)?;
            environment.phase35_stage_readiness_gate("after-nvs", &port)?;
        }
    }

    write_evidence_if_requested(&command.common, &outcome, "flash", environment)?;
    Ok(outcome)
}

pub(crate) fn run_monitor(
    command: &MonitorCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let command_spec = prepare_monitor_command(&command.common, environment)?;
    emit_command("monitor_command", &command_spec)?;

    if !command.common.dry_run {
        let port = maybe_command_port(&command_spec)
            .context("usb_session=blocked reason=port_unavailable")?;
        environment.begin_usb_session(UsbOperation::Monitor, &port)?;
        let bytes = environment.receive_only(&command_spec, command.capture_timeout_seconds)?;
        write_receive_only_console(&bytes)?;
    }

    Ok(())
}

pub(crate) fn run_flash_monitor(
    command: &FlashMonitorCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let resolved_dir = maybe_resolved_evidence_dir(&command.common, environment);
    if command.common.evidence_mode.is_some() && resolved_dir.is_none() {
        bail!("--evidence-mode dual requires --evidence-dir");
    }
    let dual_paths = if command.common.evidence_mode == Some(EvidenceMode::Dual) {
        let evidence_dir = resolved_dir
            .as_deref()
            .context("dual evidence mode requires an evidence directory")?;
        environment
            .approve_private_evidence_root(evidence_dir)
            .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=root_admission_failed"))?;
        Some(
            evidence::preflight_dual_paths(evidence_dir).map_err(|_| {
                anyhow::anyhow!("dual_evidence=failed reason=path_preflight_failed")
            })?,
        )
    } else {
        None
    };

    let mut flash_common = command.common.clone();
    flash_common.evidence_dir = None;
    let flash_command = FlashCommand {
        common: flash_common,
        image: command.image.clone(),
        manifest: command.manifest.clone(),
        wifi_credentials: command.wifi_credentials.clone(),
    };
    let flash_outcome = run_flash(&flash_command, environment).map_err(|error| {
        if command.common.evidence_mode == Some(EvidenceMode::Dual) {
            return anyhow::anyhow!("dual_evidence=failed reason=flash_workflow_failed");
        }
        error
    })?;

    let Some(evidence_dir) = resolved_dir else {
        return run_receive_only_flash_monitor(command, environment);
    };
    run_evidence_flash_monitor(
        command,
        environment,
        &evidence_dir,
        dual_paths,
        &flash_outcome,
    )
}

fn run_receive_only_flash_monitor(
    command: &FlashMonitorCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let monitor_command = prepare_monitor_command(&command.common, environment)?;
    emit_command("monitor_command", &monitor_command)?;

    if !command.common.dry_run {
        let port = maybe_command_port(&monitor_command)
            .context("usb_session=blocked reason=port_unavailable")?;
        environment.begin_usb_session(UsbOperation::FlashMonitor, &port)?;
        let bytes = environment.receive_only(&monitor_command, command.capture_timeout_seconds)?;
        write_receive_only_console(&bytes)?;
    }

    Ok(())
}

fn run_evidence_flash_monitor(
    command: &FlashMonitorCommand,
    environment: &impl FlashEnvironment,
    evidence_dir: &Utf8Path,
    dual_paths: Option<evidence::DualEvidencePaths>,
    flash_outcome: &FlashOutcome,
) -> Result<()> {
    let monitor_command =
        prepare_evidence_monitor_command(&command.common, environment).map_err(|error| {
            if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                return anyhow::anyhow!("dual_evidence=failed reason=monitor_preparation_failed");
            }
            error
        })?;
    emit_operational_command(
        "monitor_command",
        &monitor_command,
        command.common.evidence_mode != Some(EvidenceMode::Dual),
    )?;
    let log_path = evidence_dir.join("flash-monitor.log");
    let capture_log_path = dual_paths
        .as_ref()
        .map(|paths| paths.private_log.as_path())
        .unwrap_or(log_path.as_path());
    let capture_outcome = if command.common.dry_run {
        let dry_run_text =
            "dry-run: receive-only monitor was not executed; no hardware log captured\n";
        if let Some(paths) = &dual_paths {
            evidence::write_dual_private_text(&paths.private_log, dry_run_text).map_err(|_| {
                anyhow::anyhow!("dual_evidence=failed reason=private_capture_failed")
            })?;
        } else {
            environment.write_evidence(&log_path, dry_run_text)?;
        }
        dry_run_monitor_capture_outcome(command.capture_timeout_seconds)
    } else {
        let port = maybe_command_port(&monitor_command)
            .context("usb_session=blocked reason=port_unavailable")?;
        environment.begin_usb_session(UsbOperation::FlashMonitor, &port)?;
        let capture_result = environment
            .execute_capturing(
                &monitor_command,
                capture_log_path,
                command.capture_timeout_seconds,
                if dual_paths.is_some() {
                    EvidenceRedactionMode::DeveloperRaw
                } else {
                    EvidenceRedactionMode::from_common(&command.common)
                },
                dual_paths.is_some(),
            )
            .map_err(|error| {
                if command.common.evidence_mode != Some(EvidenceMode::Dual) {
                    return error;
                }
                if format!("{error:#}").contains("evidence_sanitization_invalid") {
                    return anyhow::anyhow!("evidence_sanitization_invalid");
                }
                anyhow::anyhow!("dual_evidence=failed reason=capture_failed")
            })?;
        let monitor_log = environment
            .read_to_string(capture_log_path)
            .map_err(|error| {
                if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                    return anyhow::anyhow!(
                        "dual_evidence=failed reason=private_capture_unreadable"
                    );
                }
                error.context(format!("failed to read monitor log {capture_log_path}"))
            })?;
        monitor_capture_outcome(
            &capture_result.status,
            &monitor_log,
            command.capture_timeout_seconds,
            &environment.firmware_commit(),
            &environment.reference_commit(),
            flash_outcome.runtime_identity.as_ref(),
            dual_paths.is_some(),
        )
    };
    let maybe_private_sha256 = dual_paths
        .as_ref()
        .map(|paths| evidence::private_log_sha256(&paths.private_log))
        .transpose()
        .map_err(|error| {
            if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                return anyhow::anyhow!("dual_evidence=failed reason=private_digest_failed");
            }
            error
        })?;
    write_flash_monitor_evidence_if_requested(
        &command.common,
        flash_outcome,
        &monitor_command,
        evidence_dir,
        MonitorEvidenceArtifacts {
            admitted_log: &log_path,
            dual_paths: dual_paths.as_ref(),
            private_log_sha256: maybe_private_sha256.as_deref(),
        },
        &capture_outcome,
        environment,
    )
    .map_err(|error| {
        if command.common.evidence_mode == Some(EvidenceMode::Dual) {
            return anyhow::anyhow!("dual_evidence=failed reason=evidence_record_failed");
        }
        error
    })?;
    validate_evidence_capture(command, &monitor_command, evidence_dir, &capture_outcome)
}

fn validate_evidence_capture(
    command: &FlashMonitorCommand,
    monitor_command: &CommandSpec,
    evidence_dir: &Utf8Path,
    capture_outcome: &MonitorCaptureOutcome,
) -> Result<()> {
    if command.common.dry_run || capture_outcome.accepted() {
        return Ok(());
    }
    if command.common.evidence_mode == Some(EvidenceMode::Dual)
        && capture_outcome.ready_for_private_classification()
    {
        return Ok(());
    }
    if command.common.evidence_mode == Some(EvidenceMode::Dual) {
        bail!("dual_evidence=failed reason=capture_not_accepted");
    }
    let port = maybe_command_port(monitor_command).unwrap_or_else(|| UNAVAILABLE.to_owned());
    let user_evidence_dir = command
        .common
        .evidence_dir
        .as_deref()
        .unwrap_or(evidence_dir);
    let projection = capture_outcome.projection();
    bail!(
        "{}\n{}",
        projection.conclusion,
        evidence_capture_failure_guidance(
            &port,
            user_evidence_dir,
            capture_outcome.boot_transcript_status.label(),
            capture_outcome.runtime_attestation_status.label(),
        )
    );
}
