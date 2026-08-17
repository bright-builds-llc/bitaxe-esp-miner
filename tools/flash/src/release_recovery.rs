use crate::*;

pub(crate) const RELEASE_RECOVERY_PRIVATE_ROOT: &str = "scratch/rel003-large-erase/attempt-001";
pub(crate) const RELEASE_RECOVERY_MANIFEST: &str =
    "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
pub(crate) const RELEASE_RECOVERY_WIFI_CREDENTIALS: &str = "wifi-credentials.json";
pub(crate) const RELEASE_RECOVERY_DETECTOR_OUTPUT: &str =
    "scratch/rel003-large-erase/wrapper-001/detector.stdout";
pub(crate) const RELEASE_RECOVERY_PLAN: &str =
    "docs/parity/work-plans/20260817T005227Z-REL-003/PLAN.md";
const RELEASE_RECOVERY_PLAN_SHA256: &str =
    "042e6e11fa69c44c4cde59c680755ce757193de74cb5a7910d763af819b7a6df";
pub(crate) const RELEASE_RECOVERY_PROJECTION: &str =
    "docs/parity/evidence/rel003-large-erase/release-recovery-projection.json";
pub(crate) const RELEASE_RECOVERY_TASK: &str = "task-parity-rel003-large-erase-recovery";
const RELEASE_RECOVERY_RESULT_SCHEMA: &str = "bitaxe-release-recovery-result-v1";

#[derive(Serialize)]
struct ReleaseRecoveryResult<'a> {
    schema_version: &'static str,
    status: &'static str,
    terminal_category: &'a str,
    large_erase_completed: bool,
    factory_restore_completed: bool,
    recovery_flash_used: bool,
    recovery_complete: bool,
    cleanup_complete: bool,
    projection_published: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RestoreFailure {
    Factory,
    WifiSeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RecoveryAttempt {
    used: bool,
    complete: bool,
    cleanup_complete: bool,
}

pub(crate) fn run_release_recovery(
    command: &ReleaseRecoveryCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    validate_release_recovery_paths(command)?;
    let private_root = environment.workspace_path(&command.private_root);
    let projection = environment.workspace_path(&command.projection);
    let plan_path = environment.workspace_path(&command.plan);
    let detector_path = environment.workspace_path(&command.detector_output);
    environment.approve_private_evidence_root(&private_root)?;
    preflight_release_projection(&projection, environment)?;
    let plan_bytes = environment.read_bytes(&plan_path)?;
    validate_release_plan_and_task(&plan_bytes, environment)?;
    validate_protected_detector_paths(&detector_path)?;
    let detector_document = environment.read_to_string(&detector_path)?;
    let port = admitted_detector_port(&detector_document)?;

    let manifest_path = environment.workspace_path(&command.package_manifest);
    let manifest_bytes = environment.read_bytes(&manifest_path)?;
    let manifest: PackageManifest = serde_json::from_slice(&manifest_bytes)
        .context("release_recovery=blocked reason=manifest_invalid")?;
    if manifest.source_commit != environment.firmware_commit()
        || manifest.source_commit != environment.pushed_firmware_commit()
        || manifest.reference_commit != environment.reference_commit()
    {
        bail!("release_recovery=blocked reason=package_not_exact_pushed_source");
    }
    let package_manifest_sha256 = sha256_bytes(&manifest_bytes);
    let restore_command = release_restore_flash_command(command, &port);
    let prepared_restore = prepare_flash(&restore_command, environment)
        .context("release_recovery=blocked reason=restore_preflight_failed")?;
    let expected_runtime = prepared_restore
        .outcome
        .runtime_identity
        .clone()
        .context("release_recovery=blocked reason=runtime_identity_unavailable")?;
    if environment.read_bytes(&manifest_path)? != manifest_bytes {
        bail!("release_recovery=blocked reason=manifest_changed_after_admission");
    }

    create_release_private_root(&private_root)?;
    let erase_command = release_erase_command(&port);
    emit_line("large_erase_command", PROTECTED_OPERATIONAL)?;
    environment.begin_usb_session(UsbOperation::VerifyDurability, &port)?;
    let erase_result = environment.execute(&erase_command);
    let erase_effect = last_command_effect(environment);
    if erase_result.is_err() || erase_effect != UsbDeviceEffectState::Completed {
        let erase_cleanup_complete = environment.finish_usb_session().is_ok();
        let recovery = if erase_effect != UsbDeviceEffectState::None && erase_cleanup_complete {
            attempt_release_recovery_flash(&restore_command, environment)
        } else {
            RecoveryAttempt {
                used: false,
                complete: false,
                cleanup_complete: true,
            }
        };
        write_release_result(
            &private_root,
            ReleaseRecoveryResult {
                schema_version: RELEASE_RECOVERY_RESULT_SCHEMA,
                status: "failed",
                terminal_category: "large_erase_failed",
                large_erase_completed: false,
                factory_restore_completed: false,
                recovery_flash_used: recovery.used,
                recovery_complete: recovery.complete,
                cleanup_complete: erase_cleanup_complete && recovery.cleanup_complete,
                projection_published: false,
            },
        )?;
        bail!("release_recovery=failed reason=large_erase_failed");
    }

    let restore_result = execute_prepared_restore(&prepared_restore, environment);
    let factory_restore_completed = restore_result.is_ok();
    let runtime_result = if factory_restore_completed {
        observe_restored_runtime(
            command,
            &port,
            &private_root,
            &expected_runtime,
            environment,
        )
    } else {
        Err(anyhow::anyhow!(
            "release_recovery=failed reason=restore_incomplete"
        ))
    };
    let restore_cleanup = environment.finish_usb_session();
    let restore_cleanup_complete = restore_cleanup.is_ok();
    if restore_result.is_err() || runtime_result.is_err() || !restore_cleanup_complete {
        let recovery = if restore_result == Err(RestoreFailure::Factory) && restore_cleanup_complete
        {
            attempt_release_recovery_flash(&restore_command, environment)
        } else {
            RecoveryAttempt {
                used: false,
                complete: false,
                cleanup_complete: true,
            }
        };
        write_release_result(
            &private_root,
            ReleaseRecoveryResult {
                schema_version: RELEASE_RECOVERY_RESULT_SCHEMA,
                status: "failed",
                terminal_category: if runtime_result.is_err() && factory_restore_completed {
                    "runtime_proof_failed"
                } else if restore_result == Err(RestoreFailure::WifiSeed) {
                    "wifi_seed_restore_failed"
                } else {
                    "factory_restore_failed"
                },
                large_erase_completed: true,
                factory_restore_completed,
                recovery_flash_used: recovery.used,
                recovery_complete: recovery.complete,
                cleanup_complete: restore_cleanup_complete && recovery.cleanup_complete,
                projection_published: false,
            },
        )?;
        bail!("release_recovery=failed reason=restore_or_runtime_proof_failed");
    }

    if environment.read_bytes(&plan_path)? != plan_bytes
        || environment.read_bytes(&manifest_path)? != manifest_bytes
    {
        bail!("release_recovery=failed reason=admitted_input_changed");
    }
    validate_release_plan_and_task(&plan_bytes, environment)?;

    let evidence = ReleaseRecoveryEvidence {
        schema_version: RELEASE_RECOVERY_EVIDENCE_SCHEMA.to_owned(),
        board: 205,
        attempt_ordinal: 1,
        source_commit: manifest.source_commit,
        reference_commit: manifest.reference_commit,
        package_manifest_sha256,
        plan_sha256: RELEASE_RECOVERY_PLAN_SHA256.to_owned(),
        detector_admitted: true,
        large_erase_completed: true,
        factory_restore_completed: true,
        wifi_seed_restored: true,
        mineonboot_disabled: true,
        runtime_identity_trusted: true,
        spiffs_ready: true,
        passive_safe_state_confirmed: true,
        cleanup_complete: true,
        recovery_flash_used: false,
        redaction_status: "passed".to_owned(),
    };
    evidence
        .validate()
        .map_err(|error| anyhow::anyhow!("release_recovery=failed reason={error}"))?;
    write_release_projection(&projection, &evidence)?;
    emit_line("release_recovery", "verified")
}

fn release_restore_flash_command(command: &ReleaseRecoveryCommand, port: &str) -> FlashCommand {
    FlashCommand {
        common: CommonArgs {
            board: command.board,
            port: Some(port.to_owned()),
            dry_run: false,
            redact_evidence: false,
            evidence_mode: None,
            evidence_dir: None,
        },
        image: None,
        manifest: Some(command.package_manifest.clone()),
        wifi_credentials: Some(command.wifi_credentials.clone()),
    }
}

fn release_restore_monitor_common(command: &ReleaseRecoveryCommand, port: &str) -> CommonArgs {
    CommonArgs {
        board: command.board,
        port: Some(port.to_owned()),
        dry_run: false,
        redact_evidence: false,
        evidence_mode: None,
        evidence_dir: None,
    }
}

pub(crate) fn release_erase_command(port: &str) -> CommandSpec {
    CommandSpec::new(
        "espflash",
        [
            "erase-flash",
            "--chip",
            "esp32s3",
            "--port",
            port,
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
            "--skip-update-check",
        ],
    )
}

fn attempt_release_recovery_flash(
    command: &FlashCommand,
    environment: &impl FlashEnvironment,
) -> RecoveryAttempt {
    let result = run_flash(command, environment);
    let completed = environment.device_effect_state() == UsbDeviceEffectState::Completed;
    let cleanup_complete = environment.finish_usb_session().is_ok();
    RecoveryAttempt {
        used: true,
        complete: result.is_ok() && completed && cleanup_complete,
        cleanup_complete,
    }
}

fn execute_prepared_restore(
    prepared: &PreparedFlash,
    environment: &impl FlashEnvironment,
) -> std::result::Result<(), RestoreFailure> {
    if environment.execute(&prepared.execution_command).is_err()
        || last_command_effect(environment) != UsbDeviceEffectState::Completed
    {
        return Err(RestoreFailure::Factory);
    }
    if let Some(nvs_seed) = &prepared.outcome.nvs_seed {
        if environment.execute(&nvs_seed.command).is_err()
            || last_command_effect(environment) != UsbDeviceEffectState::Completed
        {
            return Err(RestoreFailure::WifiSeed);
        }
    }
    Ok(())
}

fn observe_restored_runtime(
    command: &ReleaseRecoveryCommand,
    port: &str,
    private_root: &Utf8Path,
    expected_runtime: &ExpectedRuntimeAttestationIdentity,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let common = release_restore_monitor_common(command, port);
    let monitor_command = prepare_evidence_monitor_command(&common, environment)?;
    emit_line("monitor_command", PROTECTED_OPERATIONAL)?;
    environment.begin_usb_session(UsbOperation::VerifyDurability, port)?;
    let log_path = private_root.join("restore-monitor.private.log");
    let capture = environment.execute_capturing(
        &monitor_command,
        &log_path,
        command.capture_timeout_seconds,
        EvidenceRedactionMode::DeveloperRaw,
        true,
    )?;
    let monitor_log = environment.read_to_string(&log_path)?;
    let outcome = monitor_capture_outcome(
        &capture.status,
        &monitor_log,
        command.capture_timeout_seconds,
        &expected_runtime.firmware_commit,
        &expected_runtime.reference_commit,
        Some(expected_runtime),
        true,
    );
    if !outcome.accepted() {
        bail!("release_recovery=failed reason=runtime_proof_untrusted");
    }
    Ok(())
}

fn last_command_effect(environment: &impl FlashEnvironment) -> UsbDeviceEffectState {
    environment
        .last_usb_command_diagnostic()
        .map_or(UsbDeviceEffectState::None, |diagnostic| {
            diagnostic.device_effect_state
        })
}

fn validate_release_recovery_paths(command: &ReleaseRecoveryCommand) -> Result<()> {
    if command.private_root != Utf8Path::new(RELEASE_RECOVERY_PRIVATE_ROOT)
        || command.package_manifest != Utf8Path::new(RELEASE_RECOVERY_MANIFEST)
        || command.wifi_credentials != Utf8Path::new(RELEASE_RECOVERY_WIFI_CREDENTIALS)
        || command.detector_output != Utf8Path::new(RELEASE_RECOVERY_DETECTOR_OUTPUT)
        || command.plan != Utf8Path::new(RELEASE_RECOVERY_PLAN)
        || command.projection != Utf8Path::new(RELEASE_RECOVERY_PROJECTION)
        || command.capture_timeout_seconds != 360
    {
        bail!("release_recovery=blocked reason=path_or_timeout_contract_mismatch");
    }
    Ok(())
}

fn validate_release_plan_and_task(
    plan_bytes: &[u8],
    environment: &impl FlashEnvironment,
) -> Result<()> {
    if sha256_bytes(plan_bytes) != RELEASE_RECOVERY_PLAN_SHA256 {
        bail!("release_recovery=blocked reason=plan_digest_mismatch");
    }
    let plan =
        std::str::from_utf8(plan_bytes).context("release_recovery=blocked reason=plan_invalid")?;
    for marker in [
        "- Parity row: `REL-003`",
        "- Active task: `task-parity-rel003-large-erase-recovery`",
        "Starting command 2 consumes\nattempt-001. Never reuse it",
    ] {
        if plan.matches(marker).count() != 1 {
            bail!("release_recovery=blocked reason=plan_contract_mismatch");
        }
    }
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    let heading = format!("### {RELEASE_RECOVERY_TASK} | ");
    let start = tasks
        .find(&heading)
        .context("release_recovery=blocked reason=task_missing")?;
    if tasks[start + heading.len()..].contains(&heading) {
        bail!("release_recovery=blocked reason=task_duplicate");
    }
    let maybe_end = tasks[start + heading.len()..]
        .find("\n### ")
        .map(|offset| start + heading.len() + offset);
    let block = &tasks[start..maybe_end.unwrap_or(tasks.len())];
    for marker in [
        RELEASE_RECOVERY_PLAN,
        "capture-release-recovery-evidence",
        "erase the complete flash",
        "never reuse it or erase again",
    ] {
        if !block.contains(marker) {
            bail!("release_recovery=blocked reason=task_contract_mismatch");
        }
    }
    Ok(())
}

fn admitted_detector_port(document: &str) -> Result<String> {
    if document.matches("configuration_candidate:").count() != 1
        || document.matches("usb_session: ready").count() != 1
    {
        bail!("release_recovery=blocked reason=detector_not_admitted");
    }
    let ports = document
        .lines()
        .filter_map(|line| line.trim().strip_prefix("port: "))
        .collect::<Vec<_>>();
    let [port] = ports.as_slice() else {
        bail!("release_recovery=blocked reason=detector_port_invalid");
    };
    if !is_likely_port(port) {
        bail!("release_recovery=blocked reason=detector_port_invalid");
    }
    Ok((*port).to_owned())
}

fn validate_protected_detector_paths(detector_path: &Utf8Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let detector_metadata = fs::symlink_metadata(detector_path.as_std_path())?;
        let wrapper = detector_path
            .parent()
            .context("release_recovery=blocked reason=wrapper_missing")?;
        let wrapper_metadata = fs::symlink_metadata(wrapper.as_std_path())?;
        let detector_mode = detector_metadata.permissions().mode() & 0o777;
        let wrapper_mode = wrapper_metadata.permissions().mode() & 0o777;
        if !detector_metadata.is_file()
            || detector_metadata.file_type().is_symlink()
            || !wrapper_metadata.is_dir()
            || wrapper_metadata.file_type().is_symlink()
            || detector_mode != 0o600
            || wrapper_mode != 0o700
        {
            bail!("release_recovery=blocked reason=protected_mode_invalid");
        }
    }
    Ok(())
}

fn create_release_private_root(root: &Utf8Path) -> Result<()> {
    let parent = root
        .parent()
        .context("release_recovery=blocked reason=private_root_parent_invalid")?;
    fs::create_dir_all(parent.as_std_path())?;
    fs::create_dir(root.as_std_path())
        .context("release_recovery=blocked reason=private_root_not_fresh")?;
    set_private_directory_mode(root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::symlink_metadata(root.as_std_path())?
            .permissions()
            .mode()
            & 0o777
            != 0o700
        {
            bail!("release_recovery=blocked reason=private_root_mode_invalid");
        }
    }
    Ok(())
}

fn preflight_release_projection(
    projection: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let expected = environment.workspace_path(Utf8Path::new(RELEASE_RECOVERY_PROJECTION));
    if projection != expected {
        bail!("release_recovery=blocked reason=projection_path_invalid");
    }
    let parent = projection
        .parent()
        .context("release_recovery=blocked reason=projection_parent_invalid")?;
    fs::create_dir_all(parent.as_std_path())?;
    for candidate in [
        projection.to_owned(),
        Utf8PathBuf::from(format!("{projection}.candidate")),
    ] {
        match fs::symlink_metadata(candidate.as_std_path()) {
            Ok(_) => bail!("release_recovery=blocked reason=projection_already_exists"),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn write_release_result(root: &Utf8Path, result: ReleaseRecoveryResult<'_>) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(&result)?;
    bytes.push(b'\n');
    write_private_new_bytes(&root.join("release-recovery-result.json"), &bytes)
}

fn write_release_projection(path: &Utf8Path, evidence: &ReleaseRecoveryEvidence) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o644);
    }
    let mut file = options.open(path.as_std_path())?;
    serde_json::to_writer_pretty(&mut file, evidence)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o644))?;
        if fs::metadata(path.as_std_path())?.permissions().mode() & 0o777 != 0o644 {
            bail!("release_recovery=failed reason=projection_mode_invalid");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_erase_command_is_one_fixed_esp32s3_vector() {
        // Arrange / Act
        let command = release_erase_command("/dev/private-port");

        // Assert
        assert_eq!(
            command,
            CommandSpec::new(
                "espflash",
                [
                    "erase-flash",
                    "--chip",
                    "esp32s3",
                    "--port",
                    "/dev/private-port",
                    "--non-interactive",
                    "--before",
                    "usb-reset",
                    "--after",
                    "hard-reset",
                    "--skip-update-check",
                ],
            )
        );
    }
}
