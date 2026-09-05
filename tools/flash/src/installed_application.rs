use crate::*;

pub(crate) struct InstalledApplicationExit {
    pub(crate) force_download_bit_set: bool,
    pub(crate) transport: UsbProfile,
    pub(crate) reenumerated: bool,
}

pub(crate) fn installed_rom_probe_args(port: &str) -> Vec<String> {
    [
        "board-info",
        "--chip",
        "esp32s3",
        "--port",
        port,
        "--non-interactive",
        "--before",
        "no-reset",
        "--after",
        "no-reset",
    ]
    .map(str::to_owned)
    .to_vec()
}

pub(crate) fn installed_transport_label(profile: UsbProfile) -> &'static str {
    match profile {
        UsbProfile::WorkerRuntime => "worker_runtime",
        UsbProfile::SerialJtagRuntime => "serial_jtag_runtime",
        UsbProfile::RomDownloader => "rom_downloader",
        UsbProfile::Unknown => "unknown",
    }
}

pub(crate) fn run_start_installed(
    command: &StartInstalledCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let expected = UsbRuntimeIdentity::new(
        &command.expected_source_commit,
        &command.expected_app_elf_sha256,
    )?;
    ensure_ultra_205(command.board)?;
    if !command.redact_evidence || command.port.trim().is_empty() {
        bail!("start_installed=blocked reason=invocation");
    }
    let tasks = environment
        .read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))
        .map_err(|_| anyhow::anyhow!("start_installed=blocked reason=task_unavailable"))?;
    admit_start_installed_task(&tasks)?;
    let provenance = environment
        .current_provenance()
        .map_err(|_| anyhow::anyhow!("start_installed=blocked reason=tooling_identity"))?;
    if provenance.build_identity().source_dirty()
        || environment.pushed_firmware_commit() != provenance.build_identity().source_commit()
    {
        bail!("start_installed=blocked reason=tooling_not_clean_and_pushed");
    }
    let manifest_path = match &command.manifest {
        Some(path) => environment.workspace_path(path),
        None => environment
            .bazel_bin()?
            .join(PACKAGE_MANIFEST_RELATIVE_PATH),
    };
    let manifest: PackageManifest =
        serde_json::from_str(&environment.read_to_string(&manifest_path)?)
            .context("start_installed=blocked reason=package_manifest")?;
    if manifest.schema_version != 4 {
        bail!("start_installed=blocked reason=state_preserving_manifest_required");
    }
    let admitted =
        validate_identity_admission(&manifest_path, &manifest, &provenance, environment)?;
    if admitted.runtime_identity.firmware_commit != expected.firmware_commit
        || admitted.runtime_identity.app_elf_sha256 != expected.app_elf_sha256
    {
        bail!("start_installed=blocked reason=package_identity_mismatch");
    }
    let esptool = environment
        .prepare_application_exit()
        .map_err(|_| anyhow::anyhow!("start_installed=blocked reason=managed_tool_contract"))?;
    let root = environment.workspace_path(&command.private_root);
    create_start_installed_root(&root, environment)
        .map_err(|_| anyhow::anyhow!("start_installed=blocked reason=private_root"))?;

    let mut phase = "session_admission";
    let mut maybe_exit = None;
    let mut maybe_observation = None;
    let operation = (|| {
        environment.begin_installed_session(&command.port, &root.join("usb-session"))?;
        phase = "rom_exit";
        maybe_exit = Some(environment.execute_application_exit(&esptool)?);
        phase = "runtime_observation";
        let observation = environment.observe_installed_runtime()?;
        let identity_result = observation.require_identity(&expected);
        maybe_observation = Some(observation);
        phase = "runtime_identity";
        identity_result?;
        Ok(())
    })();
    let cleanup = environment.finish_usb_session();
    let cleanup_complete = cleanup.is_ok();
    if operation.is_ok() && !cleanup_complete {
        phase = "cleanup";
    }
    let failure = match (&operation, &cleanup) {
        (Err(error), _) => start_installed_failure(error, phase),
        (_, Err(_)) => "cleanup_failed",
        _ => "complete",
    };
    let record = serde_json::json!({
        "schema_version": "bitaxe-start-installed-v1",
        "terminal_category": failure,
        "failure_stage": if failure == "complete" { "none" } else { phase },
        "expected_source_commit": expected.firmware_commit,
        "expected_app_elf_sha256": expected.app_elf_sha256,
        "force_download_bit_set": maybe_exit.as_ref().map(|exit| exit.force_download_bit_set),
        "transport_after_reset": maybe_exit.as_ref().map(|exit| installed_transport_label(exit.transport)),
        "reenumerated": maybe_exit.as_ref().map(|exit| exit.reenumerated),
        "runtime": maybe_observation.as_ref().map(|observation| installed_observation_record(observation, &expected)),
        "device_write_observed": false,
        "host_network_effect": false,
        "cleanup_complete": cleanup_complete,
        "redacted": true,
    });
    let bytes = serde_json::to_vec_pretty(&record)?;
    let write_result = write_private_new_bytes(&root.join("result.json"), &bytes);
    if failure != "complete" {
        emit_line("start_installed", failure)?;
        if write_result.is_err() {
            emit_line("evidence_record", "write_failed")?;
        }
        bail!("start_installed=failed category={failure}");
    }
    write_result
        .map_err(|_| anyhow::anyhow!("start_installed=failed category=evidence_write_failed"))?;
    emit_line("start_installed", "complete")?;
    emit_line("runtime_identity", "exact_match")?;
    if let Some(observation) = &maybe_observation {
        emit_line("usb_reboot_loop", observation.category().label())?;
        emit_line("marker_count", &observation.marker_count().to_string())?;
    }
    Ok(())
}

fn start_installed_failure(error: &anyhow::Error, phase: &'static str) -> &'static str {
    if let Some(error) = error.downcast_ref::<bitaxe_device_session::UsbSessionError>() {
        return error.category.as_str();
    }
    match phase {
        "session_admission" => "session_admission_failed",
        "rom_exit" => "rom_exit_failed",
        "runtime_identity" => "runtime_identity_missing_or_mismatched",
        _ => "runtime_observation_failed",
    }
}

fn create_start_installed_root(root: &Utf8Path, environment: &impl FlashEnvironment) -> Result<()> {
    environment.approve_private_evidence_root(root)?;
    if fs::symlink_metadata(root).is_ok() {
        bail!("private_root_exists");
    }
    let parent = root.parent().context("private_parent_missing")?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        bail!("private_parent_invalid");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o700 {
        bail!("private_parent_mode");
    }
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(root)?;
    Ok(())
}

pub(super) fn admit_start_installed_task(tasks: &str) -> Result<()> {
    const TASK: &str = "task-fixed-usb-serial-qualification";
    let mut active = false;
    let mut collecting = false;
    let mut matches = 0;
    let mut block = String::new();
    for line in tasks.lines() {
        if line.starts_with("## ") {
            active = line.trim() == "## Active";
            collecting = false;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            collecting = active && heading.split_whitespace().next() == Some(TASK);
            if collecting {
                matches += 1;
            }
        }
        if collecting {
            block.push_str(line);
            block.push('\n');
        }
    }
    if matches != 1 {
        bail!("start_installed=blocked reason=active_task");
    }
    let commands = block
        .split('`')
        .filter(|part| part.starts_with("just native-usb-start-installed "))
        .collect::<Vec<_>>();
    let [command] = commands.as_slice() else {
        bail!("start_installed=blocked reason=task_command");
    };
    let fields = command.split_whitespace().collect::<Vec<_>>();
    let boards = fields
        .windows(2)
        .filter(|pair| pair[0] == "--board")
        .map(|pair| pair[1])
        .collect::<Vec<_>>();
    if boards != ["205"] {
        bail!("start_installed=blocked reason=task_board");
    }
    for required in ["--port", "--private-root"] {
        if fields.iter().filter(|field| **field == required).count() != 1 {
            bail!("start_installed=blocked reason=task_command");
        }
    }
    if !fields.contains(&"--redact-evidence") {
        bail!("start_installed=blocked reason=task_redaction");
    }
    Ok(())
}

fn installed_observation_record(
    observation: &UsbRebootLoopObservation,
    expected: &UsbRuntimeIdentity,
) -> serde_json::Value {
    let memory = observation.memory_checkpoints().iter().map(|checkpoint| serde_json::json!({
        "stage": checkpoint.stage, "free_bytes": checkpoint.free_bytes,
        "largest_block_bytes": checkpoint.largest_block_bytes, "reserve_bytes": checkpoint.reserve_bytes,
    })).collect::<Vec<_>>();
    serde_json::json!({
        "identity_match": observation.require_identity(expected).is_ok(),
        "observed_source_commit": observation.maybe_runtime_identity().map(|identity| &identity.firmware_commit),
        "observed_app_elf_sha256": observation.maybe_runtime_identity().map(|identity| &identity.app_elf_sha256),
        "category": observation.category().label(),
        "marker_count": observation.marker_count(),
        "reconnect_count": observation.reconnect_count(),
        "latest_boot_ordinal": observation.latest_boot_ordinal(),
        "latest_reset_reason": observation.latest_reset_reason().label(),
        "rust_panic_receipt": observation.latest_rust_panic().map(|marker| marker.marker()),
        "allocation_failure_receipt": observation.latest_allocation_failure().map(|marker| marker.marker()),
        "allocation_context": observation.maybe_allocation_context().map(|marker| marker.marker()),
        "memory_checkpoints": memory,
        "worker_start_failed": observation.worker_start_failed(),
    })
}
