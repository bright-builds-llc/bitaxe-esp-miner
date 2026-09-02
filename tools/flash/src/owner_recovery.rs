use crate::*;

pub(crate) const OWNER_RECOVERY_PLAN: &str =
    "docs/parity/work-plans/20260901T161405Z-NATIVE-USB-SERIAL-OWNER-RECOVERY/PLAN.md";
pub(crate) const OWNER_RECOVERY_ROOT: &str = "scratch/native-usb-owner-recovery/attempt-001";
pub(crate) const OWNER_RECOVERY_MANIFEST: &str =
    "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
pub(crate) const OWNER_RECOVERY_BUNDLE: &str =
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const OWNER_RECOVERY_PLAN_SHA256: &str =
    "8d59b142bacc2d7aab7614ee3a3f51ed015abb0b0badf9707dbe819f21db4cc2";
const PREDECESSOR_STATE: &str =
    "scratch/native-usb-config-ap-recovery/attempt-001/state.private.json";
const OBSERVATION_STATE: &str = "observation-state.private.json";
const RECOVERY_RESULT: &str = "recovery-result.private.json";
const MANUAL_CHECKPOINT: &str = "manual-bootstrap.private.json";

pub(crate) struct OwnerRecoveryExitCapture {
    pub(crate) force_download_bit_set: bool,
    pub(crate) transport: UsbProfile,
    pub(crate) reenumerated: bool,
    pub(crate) monitor: MonitorOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerRecoveryState {
    schema_version: String,
    source_commit: String,
    reference_commit: String,
    plan_sha256: String,
    manifest_sha256: String,
    restore_bundle_sha256: String,
    stage: String,
    initial_transport: String,
    passive_marker_status: String,
    execution_owner: String,
    rom_entry_path: String,
    force_download_bit_category: String,
    reset_adapter: String,
    passive_observation_count: u8,
    rom_probe_count: u8,
    manual_prompt_count: u8,
    rom_admission_count: u8,
    force_bit_read_count: u8,
    application_exit_count: u8,
    enumeration_changed: bool,
    physical_identity_match: bool,
    physical_identity_digest: String,
    device_write_observed: bool,
    host_network_effect: bool,
    cleanup_complete: bool,
    terminal_category: String,
    redaction_status: String,
}

struct OwnerRecoveryAdmission {
    source_commit: String,
    reference_commit: String,
    manifest_sha256: String,
    restore_bundle_sha256: String,
    expected: ExpectedRuntimeAttestationIdentity,
}

pub(crate) fn run_owner_recovery(
    command: OwnerRecoveryCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    validate_invocation(&command)?;
    let admission = admit_owner_recovery(environment)?;
    match command.action {
        OwnerRecoveryAction::Observe => run_observation(&command, &admission, environment),
        OwnerRecoveryAction::Recover => run_recovery(&command, &admission, environment),
    }
}

fn validate_invocation(command: &OwnerRecoveryCommand) -> Result<()> {
    ensure_ultra_205(command.board)?;
    let expected_checkpoint = Utf8Path::new(OWNER_RECOVERY_ROOT).join(MANUAL_CHECKPOINT);
    let checkpoint_valid = match command.action {
        OwnerRecoveryAction::Observe => command.manual_checkpoint.is_none(),
        OwnerRecoveryAction::Recover => command
            .manual_checkpoint
            .as_deref()
            .is_none_or(|path| path == expected_checkpoint),
    };
    if command.package_manifest != Utf8Path::new(OWNER_RECOVERY_MANIFEST)
        || command.restore_bundle != Utf8Path::new(OWNER_RECOVERY_BUNDLE)
        || command.private_root != Utf8Path::new(OWNER_RECOVERY_ROOT)
        || command.plan != Utf8Path::new(OWNER_RECOVERY_PLAN)
        || !command.redact_evidence
        || !checkpoint_valid
    {
        bail!("owner_recovery=blocked reason=invocation");
    }
    Ok(())
}

fn admit_owner_recovery(environment: &impl FlashEnvironment) -> Result<OwnerRecoveryAdmission> {
    let plan =
        environment.read_bytes(&environment.workspace_path(Utf8Path::new(OWNER_RECOVERY_PLAN)))?;
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    if sha256_bytes(&plan) != OWNER_RECOVERY_PLAN_SHA256
        || !tasks.contains("### task-native-usb-rom-exit-discriminator-205")
    {
        bail!("owner_recovery=blocked reason=plan_identity");
    }
    let provenance = environment.current_provenance()?;
    let source_commit = provenance.build_identity().source_commit().to_owned();
    let manifest_bytes = environment
        .read_bytes(&environment.workspace_path(Utf8Path::new(OWNER_RECOVERY_MANIFEST)))?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    if provenance.build_identity().source_dirty()
        || environment.pushed_firmware_commit() != source_commit
        || manifest
            .get("source_commit")
            .and_then(serde_json::Value::as_str)
            != Some(source_commit.as_str())
    {
        bail!("owner_recovery=blocked reason=source_identity");
    }
    let predecessor_path = environment.workspace_path(Utf8Path::new(PREDECESSOR_STATE));
    require_private_path(&predecessor_path, 0o600, false)?;
    let predecessor: serde_json::Value =
        serde_json::from_slice(&environment.read_bytes(&predecessor_path)?)?;
    if predecessor.get("stage").and_then(serde_json::Value::as_str) != Some("nvs_match")
        || predecessor
            .get("nvs_match")
            .and_then(serde_json::Value::as_bool)
            != Some(true)
        || predecessor
            .get("device_write_observed")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
    {
        bail!("owner_recovery=blocked reason=predecessor_state");
    }
    let bundle_path = environment.workspace_path(Utf8Path::new(OWNER_RECOVERY_BUNDLE));
    require_private_path(&bundle_path, 0o600, false)?;
    let bundle_bytes = environment.read_bytes(&bundle_path)?;
    let bundle: serde_json::Value = serde_json::from_slice(&bundle_bytes)?;
    let identity = bundle
        .get("installed_identity")
        .and_then(serde_json::Value::as_object)
        .context("owner_recovery=blocked reason=restore_identity")?;
    let expected = ExpectedRuntimeAttestationIdentity {
        firmware_commit: required_identity(identity, "source_commit", 40)?,
        reference_commit: required_identity(identity, "reference_commit", 40)?,
        app_elf_sha256: required_identity(identity, "app_elf_sha256", 64)?,
    };
    if expected.reference_commit != provenance.reference_commit() {
        bail!("owner_recovery=blocked reason=restore_identity_reference");
    }
    Ok(OwnerRecoveryAdmission {
        source_commit,
        reference_commit: provenance.reference_commit().to_owned(),
        manifest_sha256: sha256_bytes(&manifest_bytes),
        restore_bundle_sha256: sha256_bytes(&bundle_bytes),
        expected,
    })
}

fn run_observation(
    command: &OwnerRecoveryCommand,
    admission: &OwnerRecoveryAdmission,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let root = environment.workspace_path(&command.private_root);
    if fs::symlink_metadata(root.as_std_path()).is_ok() {
        bail!("owner_recovery=blocked reason=root_exists");
    }
    create_private_root(&root, &command.private_root, environment)?;
    environment.begin_usb_session(UsbOperation::Recover, &command.port)?;
    let initial_transport = environment.usb_profile(&command.port)?;
    let physical_identity_digest = environment.usb_physical_identity_digest()?;
    let mut state = base_state(admission, initial_transport, physical_identity_digest);

    match initial_transport {
        UsbProfile::WorkerRuntime => {
            state.stage = "complete".to_owned();
            state.passive_marker_status = "worker_descriptor".to_owned();
            state.execution_owner = "application".to_owned();
            state.terminal_category = "complete".to_owned();
        }
        UsbProfile::SerialJtagRuntime => {
            state.stage = "passive_observation".to_owned();
            state.passive_observation_count = 1;
            let monitor = CommandSpec::new(
                "bitaxe-receive-only",
                ["observe", "--port", command.port.as_str()],
            );
            let bytes = environment.receive_only(&monitor, 25)?;
            write_private_new_bytes(&root.join("passive-monitor.private.log"), &bytes)?;
            let status = classify_runtime_boot_attestations(
                &String::from_utf8_lossy(&bytes),
                &admission.expected,
            );
            state.passive_marker_status = status.label().to_owned();
            if status == RuntimeAttestationStatus::Trusted {
                state.stage = "complete".to_owned();
                state.execution_owner = "application".to_owned();
                state.terminal_category = "complete".to_owned();
            } else if weak_application_evidence(status) {
                probe_rom_or_request_manual(command, &mut state, environment)?;
            } else {
                state.terminal_category = marker_terminal(status).to_owned();
            }
        }
        UsbProfile::RomDownloader | UsbProfile::Unknown => {
            state.terminal_category = "transport_unknown".to_owned();
        }
    }
    environment.finish_usb_session()?;
    state.cleanup_complete = true;
    write_state(&root.join(OBSERVATION_STATE), &state)?;
    emit_line("owner_recovery", &state.stage)
}

fn probe_rom_or_request_manual(
    command: &OwnerRecoveryCommand,
    state: &mut OwnerRecoveryState,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    state.rom_probe_count = 1;
    let probe = owner_rom_probe_command(&command.port);
    match environment.execute_owner_rom_probe(&probe) {
        Ok(output) if board_info_reports_esp32s3(&output) => {
            state.stage = "rom_admitted".to_owned();
            state.execution_owner = "rom".to_owned();
            state.rom_entry_path = "already_rom".to_owned();
            state.rom_admission_count = 1;
            state.terminal_category = "rom_admitted".to_owned();
        }
        Ok(_) => state.terminal_category = "rom_admission_invalid".to_owned(),
        Err(_) => {
            let current = environment.current_usb_physical_identity_digest(&command.port)?;
            if current != state.physical_identity_digest {
                state.physical_identity_match = false;
                state.terminal_category = "physical_identity_drift".to_owned();
                return Ok(());
            }
            state.stage = "manual_required".to_owned();
            state.terminal_category = "manual_required".to_owned();
        }
    }
    Ok(())
}

fn run_recovery(
    command: &OwnerRecoveryCommand,
    admission: &OwnerRecoveryAdmission,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let root = environment.workspace_path(&command.private_root);
    require_private_path(&root, 0o700, true)?;
    let observation_path = root.join(OBSERVATION_STATE);
    require_private_path(&observation_path, 0o600, false)?;
    if fs::symlink_metadata(root.join(RECOVERY_RESULT).as_std_path()).is_ok() {
        bail!("owner_recovery=blocked reason=recovery_consumed");
    }
    let observation: OwnerRecoveryState =
        serde_json::from_slice(&environment.read_bytes(&observation_path)?)?;
    if observation.schema_version != "bitaxe-native-usb-owner-recovery-private-v1"
        || !matches!(
            observation.stage.as_str(),
            "rom_admitted" | "manual_required"
        )
        || observation.source_commit != admission.source_commit
        || observation.plan_sha256 != OWNER_RECOVERY_PLAN_SHA256
        || !observation.cleanup_complete
    {
        bail!("owner_recovery=blocked reason=state");
    }
    let manual = observation.stage == "manual_required";
    if manual {
        let expected = Utf8Path::new(OWNER_RECOVERY_ROOT).join(MANUAL_CHECKPOINT);
        if command.manual_checkpoint.as_deref() != Some(expected.as_path()) {
            bail!("owner_recovery=blocked reason=manual_checkpoint");
        }
        validate_manual_checkpoint(&environment.workspace_path(&expected))?;
    } else if command.manual_checkpoint.is_some() {
        bail!("owner_recovery=blocked reason=unexpected_manual_checkpoint");
    }

    let recovery_port = resolve_physical_port(
        environment,
        &command.port,
        &observation.physical_identity_digest,
    )?;
    environment.begin_usb_session(UsbOperation::Recover, &recovery_port)?;
    let current_identity = environment.usb_physical_identity_digest()?;
    if current_identity != observation.physical_identity_digest {
        bail!("physical_identity_drift");
    }
    let mut result = observation.clone();
    result.stage = "application_exit_sent".to_owned();
    result.cleanup_complete = false;
    if manual {
        result.manual_prompt_count = 1;
        let probe = owner_rom_probe_command(&recovery_port);
        let board_info = environment.execute_owner_rom_probe(&probe)?;
        if !board_info_reports_esp32s3(&board_info) {
            bail!("owner_recovery=blocked reason=rom_admission_invalid");
        }
        result.rom_entry_path = "manual_boot_reset".to_owned();
        result.rom_admission_count = 1;
    }
    let esptool = find_managed_esptool(environment)?;
    let capture = environment.execute_owner_recovery_exit(&esptool, 30)?;
    result.force_download_bit_category = if capture.force_download_bit_set {
        "set".to_owned()
    } else {
        "clear".to_owned()
    };
    result.force_bit_read_count = 1;
    result.application_exit_count = 1;
    result.reset_adapter = "managed_esptool_hard_reset".to_owned();
    result.enumeration_changed |= capture.reenumerated;
    write_private_new_bytes(
        &root.join("recovery-monitor.private.log"),
        &capture.monitor.bytes,
    )?;
    let status = classify_recovered_application(
        capture.transport,
        &capture.monitor.bytes,
        &admission.expected,
    );
    result.passive_marker_status = status.to_owned();
    if status == "worker_descriptor" || status == "trusted" {
        result.stage = "complete".to_owned();
        result.execution_owner = "application".to_owned();
        result.terminal_category = "complete".to_owned();
    } else {
        result.execution_owner = "unknown".to_owned();
        result.terminal_category = format!("application_{status}");
    }
    environment.finish_usb_session()?;
    result.cleanup_complete = true;
    write_state(&root.join(RECOVERY_RESULT), &result)?;
    emit_line("owner_recovery", &result.stage)
}

fn classify_recovered_application(
    transport: UsbProfile,
    bytes: &[u8],
    expected: &ExpectedRuntimeAttestationIdentity,
) -> &'static str {
    match transport {
        UsbProfile::WorkerRuntime => "worker_descriptor",
        UsbProfile::SerialJtagRuntime => {
            classify_runtime_boot_attestations(&String::from_utf8_lossy(bytes), expected).label()
        }
        UsbProfile::RomDownloader | UsbProfile::Unknown => "transport_unknown",
    }
}

pub(crate) fn owner_rom_probe_command(port: &str) -> CommandSpec {
    CommandSpec::new(
        "espflash",
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
        ],
    )
}

fn resolve_physical_port(
    environment: &impl FlashEnvironment,
    preferred_port: &str,
    expected_identity: &str,
) -> Result<String> {
    let mut candidates = BTreeSet::from([preferred_port.to_owned()]);
    candidates.extend(likely_port_candidates(&environment.list_ports()?));
    let matches = candidates
        .into_iter()
        .filter(|candidate| {
            environment
                .current_usb_physical_identity_digest(candidate)
                .is_ok_and(|identity| identity == expected_identity)
        })
        .collect::<Vec<_>>();
    let [port] = matches.as_slice() else {
        bail!("physical_identity_drift");
    };
    Ok(port.clone())
}

fn weak_application_evidence(status: RuntimeAttestationStatus) -> bool {
    matches!(
        status,
        RuntimeAttestationStatus::Missing | RuntimeAttestationStatus::InsufficientSamples
    )
}

fn marker_terminal(status: RuntimeAttestationStatus) -> &'static str {
    match status {
        RuntimeAttestationStatus::Malformed => "application_marker_malformed",
        RuntimeAttestationStatus::MixedSessionOrOrdinal => "application_marker_mixed_session",
        RuntimeAttestationStatus::StaticFieldsMismatch => "application_marker_inconsistent",
        RuntimeAttestationStatus::NonMonotonicUptime => "application_marker_non_monotonic",
        RuntimeAttestationStatus::PackageIdentityMismatch => "application_identity_mismatch",
        RuntimeAttestationStatus::IncompleteReadiness => "application_readiness_incomplete",
        RuntimeAttestationStatus::Trusted
        | RuntimeAttestationStatus::Missing
        | RuntimeAttestationStatus::InsufficientSamples => "application_marker_unavailable",
    }
}

fn base_state(
    admission: &OwnerRecoveryAdmission,
    transport: UsbProfile,
    physical_identity_digest: String,
) -> OwnerRecoveryState {
    OwnerRecoveryState {
        schema_version: "bitaxe-native-usb-owner-recovery-private-v1".to_owned(),
        source_commit: admission.source_commit.clone(),
        reference_commit: admission.reference_commit.clone(),
        plan_sha256: OWNER_RECOVERY_PLAN_SHA256.to_owned(),
        manifest_sha256: admission.manifest_sha256.clone(),
        restore_bundle_sha256: admission.restore_bundle_sha256.clone(),
        stage: "prepared".to_owned(),
        initial_transport: transport_label(transport).to_owned(),
        passive_marker_status: "not_observed".to_owned(),
        execution_owner: "unknown".to_owned(),
        rom_entry_path: "none".to_owned(),
        force_download_bit_category: "not_read".to_owned(),
        reset_adapter: "none".to_owned(),
        passive_observation_count: 0,
        rom_probe_count: 0,
        manual_prompt_count: 0,
        rom_admission_count: 0,
        force_bit_read_count: 0,
        application_exit_count: 0,
        enumeration_changed: false,
        physical_identity_match: true,
        physical_identity_digest,
        device_write_observed: false,
        host_network_effect: false,
        cleanup_complete: false,
        terminal_category: "pending".to_owned(),
        redaction_status: "passed".to_owned(),
    }
}

fn create_private_root(
    root: &Utf8Path,
    requested: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    environment.approve_private_evidence_root(requested)?;
    let parent = root
        .parent()
        .context("owner_recovery=blocked reason=root_parent")?;
    fs::create_dir_all(parent.as_std_path())?;
    set_private_directory_mode(parent)?;
    fs::create_dir(root.as_std_path())?;
    set_private_directory_mode(root)
}

fn validate_manual_checkpoint(path: &Utf8Path) -> Result<()> {
    require_private_path(path, 0o600, false)?;
    let checkpoint: serde_json::Value = serde_json::from_slice(&fs::read(path.as_std_path())?)?;
    if checkpoint
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("bitaxe-native-usb-owner-recovery-checkpoint-v1")
        || checkpoint.get("action").and_then(serde_json::Value::as_str) != Some("manual_boot_reset")
        || checkpoint.get("status").and_then(serde_json::Value::as_str) != Some("accepted")
    {
        bail!("owner_recovery=blocked reason=manual_checkpoint");
    }
    Ok(())
}

fn write_state(path: &Utf8Path, state: &OwnerRecoveryState) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(state)?;
    bytes.push(b'\n');
    write_private_new_bytes(path, &bytes)
}

fn require_private_path(path: &Utf8Path, mode: u32, directory: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())?;
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        bail!("owner_recovery=blocked reason=private_path_type");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != mode {
        bail!("owner_recovery=blocked reason=private_path_mode");
    }
    Ok(())
}

fn required_identity(
    identity: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    length: usize,
) -> Result<String> {
    identity
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(str::to_owned)
        .with_context(|| {
            format!("owner_recovery=blocked reason=restore_identity_field field={key}")
        })
}

fn transport_label(profile: UsbProfile) -> &'static str {
    match profile {
        UsbProfile::WorkerRuntime => "worker_runtime",
        UsbProfile::SerialJtagRuntime => "serial_jtag_runtime",
        UsbProfile::RomDownloader => "rom_downloader",
        UsbProfile::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_missing_or_insufficient_evidence_allows_manual_recovery() {
        // Arrange / Act / Assert
        assert!(weak_application_evidence(RuntimeAttestationStatus::Missing));
        assert!(weak_application_evidence(
            RuntimeAttestationStatus::InsufficientSamples
        ));
        for status in [
            RuntimeAttestationStatus::Malformed,
            RuntimeAttestationStatus::MixedSessionOrOrdinal,
            RuntimeAttestationStatus::StaticFieldsMismatch,
            RuntimeAttestationStatus::NonMonotonicUptime,
            RuntimeAttestationStatus::PackageIdentityMismatch,
            RuntimeAttestationStatus::IncompleteReadiness,
        ] {
            assert!(!weak_application_evidence(status));
        }
    }

    #[test]
    fn recovered_worker_descriptor_authenticates_without_serial_bytes() {
        // Arrange
        let expected = ExpectedRuntimeAttestationIdentity {
            firmware_commit: "1".repeat(40),
            reference_commit: "2".repeat(40),
            app_elf_sha256: "3".repeat(64),
        };

        // Act
        let status = classify_recovered_application(UsbProfile::WorkerRuntime, b"", &expected);

        // Assert
        assert_eq!(status, "worker_descriptor");
    }

    #[test]
    fn marker_failures_map_to_closed_terminal_categories() {
        // Arrange / Act / Assert
        assert_eq!(
            marker_terminal(RuntimeAttestationStatus::PackageIdentityMismatch),
            "application_identity_mismatch"
        );
        assert_eq!(
            marker_terminal(RuntimeAttestationStatus::NonMonotonicUptime),
            "application_marker_non_monotonic"
        );
    }

    #[test]
    fn rom_probe_is_read_only_and_preserves_the_current_profile() {
        // Arrange / Act
        let command = owner_rom_probe_command("admitted");

        // Assert
        assert_eq!(command.program, "espflash");
        assert_eq!(
            command.args,
            [
                "board-info",
                "--chip",
                "esp32s3",
                "--port",
                "admitted",
                "--non-interactive",
                "--before",
                "no-reset",
                "--after",
                "no-reset",
            ]
        );
    }

    #[test]
    fn owner_recovery_source_contains_no_device_write_surface() {
        // Arrange
        let source = include_str!("owner_recovery.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");

        // Act / Assert
        for forbidden in ["write-bin", "write_flash", "erase_flash", "erase-flash"] {
            assert!(!source.contains(forbidden));
        }
    }
}
