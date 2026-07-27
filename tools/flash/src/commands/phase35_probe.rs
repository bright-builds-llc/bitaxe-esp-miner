use crate::*;

pub(crate) fn run_phase35_probe(
    command: &Phase35ProbeCommand,
    environment: &LocalFlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if command.timeout_seconds == 0 || command.timeout_seconds > 420 {
        bail!("phase35_probe=blocked reason=invalid_timeout");
    }

    let stage_root = environment.workspace_path(&command.stage_root);
    environment
        .approve_private_evidence_root(&stage_root)
        .map_err(|_| anyhow::anyhow!("phase35_probe=blocked reason=root_admission_failed"))?;
    fs::create_dir_all(stage_root.as_std_path())
        .context("failed to create private Phase 35 probe root")?;
    set_private_directory_mode(&stage_root)?;

    let log_path = stage_root.join("probe.private.log");
    let metrics_path = stage_root.join("probe.metrics.json");
    if log_path.exists() || metrics_path.exists() {
        bail!("phase35_probe=blocked reason=destination_exists");
    }

    let command_spec = phase35_probe_command(&environment.espflash_bin, &command.port);
    let started = Instant::now();
    let capture = evidence::capture_command(
        &command_spec,
        &environment.espflash_bin,
        &log_path,
        command.timeout_seconds,
        EvidenceRedactionMode::DeveloperRaw,
        true,
    )?;
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let launched = !matches!(capture.status, CaptureProcessStatus::SpawnFailed);
    let success = matches!(capture.status, CaptureProcessStatus::ExitedSuccess);
    let log = fs::read_to_string(log_path.as_std_path())
        .context("failed to read sanitized Phase 35 probe log")?;
    let checksum_observed = phase35_probe_checksum_observed(&log);
    let connected = launched && (success || flash_log_connected(&log));
    let device_info_complete = connected && (success || flash_log_device_info_complete(&log));
    let transfer_started = device_info_complete && checksum_observed;
    let completed = transfer_started && success;
    let metrics = serde_json::json!({
        "schema_version": PHASE35_FLASH_SCHEMA,
        "stage": "probe",
        "tool_version_valid": environment.espflash_version == format!("espflash {ESPFLASH_EXPECTED_VERSION}"),
        "launched": launched,
        "connected": connected,
        "device_info_complete": device_info_complete,
        "transfer_started": transfer_started,
        "completed": completed,
        "duration_millis": if launched { duration_millis } else { 0 },
    });
    let mut encoded = serde_json::to_vec_pretty(&metrics)?;
    encoded.push(b'\n');
    write_private_new_bytes(&metrics_path, &encoded)?;

    if !completed {
        bail!("phase35_probe=failed reason=child_boundary");
    }
    emit_line("phase35_probe", "ready")
}
