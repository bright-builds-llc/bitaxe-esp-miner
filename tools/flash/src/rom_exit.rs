use crate::*;

const RTC_OPTION1_ADDRESS: &str = "0x6000812c";
pub(crate) const ROM_EXIT_PLAN: &str =
    "docs/parity/work-plans/20260831T190744Z-NATIVE-USB-ROM-EXIT-DISCRIMINATOR/PLAN.md";
pub(crate) const ROM_EXIT_ROOT: &str = "scratch/native-usb-rom-exit/attempt-001";
pub(crate) const ROM_EXIT_MANIFEST: &str = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
pub(crate) const ROM_EXIT_BUNDLE: &str =
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const ROM_EXIT_PLAN_SHA256: &str =
    "a93c88a5a0aab939c6462792bd31a5f61b60dcee45935cccc6c14466ef2b3262";
const ROM_EXIT_PREDECESSOR_STATE: &str =
    "scratch/native-usb-config-ap-recovery/attempt-001/state.private.json";

pub(crate) struct RomExitHardwareCapture {
    pub(crate) force_download_bit_set: bool,
    pub(crate) transport: UsbProfile,
    pub(crate) reenumerated: bool,
    pub(crate) monitor: MonitorOutput,
}

pub(crate) trait RomExitEnvironment: FlashEnvironment {
    fn execute_rom_exit(
        &self,
        esptool: &Utf8Path,
        observation_seconds: u64,
    ) -> Result<RomExitHardwareCapture>;
}

pub(crate) fn force_download_read_args(port: &str) -> Vec<String> {
    [
        "--chip",
        "esp32s3",
        "--port",
        port,
        "--before",
        "no_reset",
        "--after",
        "no_reset",
        "--no-stub",
        "read_mem",
        RTC_OPTION1_ADDRESS,
    ]
    .map(str::to_owned)
    .to_vec()
}

pub(crate) fn parse_force_download_bit(output: &[u8]) -> Result<bool> {
    let text =
        std::str::from_utf8(output).context("rom_exit=blocked reason=force_bit_output_encoding")?;
    let matches = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix(RTC_OPTION1_ADDRESS))
        .collect::<Vec<_>>();
    let [suffix] = matches.as_slice() else {
        bail!("rom_exit=blocked reason=force_bit_read_shape");
    };
    let value = suffix
        .trim()
        .strip_prefix('=')
        .map(str::trim)
        .and_then(|value| value.strip_prefix("0x"))
        .and_then(|value| u32::from_str_radix(value, 16).ok())
        .context("rom_exit=blocked reason=force_bit_value")?;
    Ok(value & 1 == 1)
}

pub(crate) fn run_rom_exit_diagnostic(
    command: &RomExitDiagnosticCommand,
    environment: &impl RomExitEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if command.package_manifest != Utf8Path::new(ROM_EXIT_MANIFEST)
        || command.restore_bundle != Utf8Path::new(ROM_EXIT_BUNDLE)
        || command.private_root != Utf8Path::new(ROM_EXIT_ROOT)
        || command.plan != Utf8Path::new(ROM_EXIT_PLAN)
        || command.observation_seconds != 30
        || !command.redact_evidence
    {
        bail!("rom_exit=blocked reason=invocation");
    }
    let plan = environment.read_bytes(&environment.workspace_path(&command.plan))?;
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    if sha256_bytes(&plan) != ROM_EXIT_PLAN_SHA256
        || !tasks.contains("### task-native-usb-rom-exit-discriminator-205")
    {
        bail!("rom_exit=blocked reason=plan_identity");
    }
    let provenance = environment.current_provenance()?;
    let manifest_bytes =
        environment.read_bytes(&environment.workspace_path(&command.package_manifest))?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    let source_commit = provenance.build_identity().source_commit();
    if provenance.build_identity().source_dirty()
        || environment.pushed_firmware_commit() != source_commit
        || manifest
            .get("source_commit")
            .and_then(serde_json::Value::as_str)
            != Some(source_commit)
    {
        bail!("rom_exit=blocked reason=source_identity");
    }
    let predecessor_path = environment.workspace_path(Utf8Path::new(ROM_EXIT_PREDECESSOR_STATE));
    require_private_rom_exit_path(&predecessor_path, 0o600, false)?;
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
        bail!("rom_exit=blocked reason=predecessor_state");
    }
    let bundle_path = environment.workspace_path(&command.restore_bundle);
    require_private_rom_exit_path(&bundle_path, 0o600, false)?;
    let bundle_bytes = environment.read_bytes(&bundle_path)?;
    let bundle: serde_json::Value = serde_json::from_slice(&bundle_bytes)?;
    let identity = bundle
        .get("installed_identity")
        .and_then(serde_json::Value::as_object)
        .context("rom_exit=blocked reason=restore_identity")?;
    let expected = ExpectedRuntimeAttestationIdentity {
        firmware_commit: required_identity(identity, "source_commit", 40)?,
        reference_commit: required_identity(identity, "reference_commit", 40)?,
        app_elf_sha256: required_identity(identity, "app_elf_sha256", 64)?,
    };
    let root = environment.workspace_path(&command.private_root);
    if fs::symlink_metadata(root.as_std_path()).is_ok() {
        bail!("rom_exit=blocked reason=root_exists");
    }
    environment.approve_private_evidence_root(&command.private_root)?;
    fs::create_dir_all(
        root.parent()
            .context("rom_exit=blocked reason=root_parent")?,
    )?;
    set_private_directory_mode(
        root.parent()
            .context("rom_exit=blocked reason=root_parent")?,
    )?;
    fs::create_dir(root.as_std_path())?;
    set_private_directory_mode(&root)?;

    let esptool = find_managed_esptool(environment)?;
    environment.begin_usb_session(UsbOperation::Recover, &command.port)?;
    let capture = environment.execute_rom_exit(&esptool, command.observation_seconds)?;
    environment.finish_usb_session()?;
    write_private_new_bytes(
        &root.join("application-monitor.private.log"),
        &capture.monitor.bytes,
    )?;

    let (execution_owner, marker_status) =
        classify_execution_owner(capture.transport, &capture.monitor.bytes, &expected);
    let terminal_category = if execution_owner == UsbExecutionOwner::Application {
        "complete"
    } else {
        "execution_owner_unknown"
    };
    let machine = serde_json::json!({
        "schema_version": "bitaxe-native-usb-rom-exit-private-v1",
        "source_commit": source_commit,
        "reference_commit": provenance.reference_commit(),
        "plan_sha256": ROM_EXIT_PLAN_SHA256,
        "manifest_sha256": sha256_bytes(&manifest_bytes),
        "restore_bundle_sha256": sha256_bytes(&bundle_bytes),
        "force_download_bit_set": capture.force_download_bit_set,
        "reset_adapter": "managed_esptool_hard_reset",
        "transport_profile": transport_label(capture.transport),
        "execution_owner": execution_owner_label(execution_owner),
        "application_marker_status": marker_status,
        "enumeration_changed": capture.reenumerated,
        "nvs_read_repeated": false,
        "device_write_observed": false,
        "host_network_effect": false,
        "cleanup_complete": true,
        "terminal_category": terminal_category,
        "redaction_status": "passed",
    });
    write_private_new_bytes(
        &root.join("machine-result.private.json"),
        &json_line(&machine)?,
    )?;
    emit_line("rom_exit", terminal_category)
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
        .with_context(|| format!("rom_exit=blocked reason=restore_identity_field field={key}"))
}

fn classify_execution_owner(
    transport: UsbProfile,
    bytes: &[u8],
    expected: &ExpectedRuntimeAttestationIdentity,
) -> (UsbExecutionOwner, &'static str) {
    if transport == UsbProfile::WorkerRuntime {
        return (UsbExecutionOwner::Application, "worker_descriptor");
    }
    if transport != UsbProfile::SerialJtagRuntime {
        return (UsbExecutionOwner::Unknown, "transport_unknown");
    }
    let text = String::from_utf8_lossy(bytes);
    let mut maybe_marker: Option<UsbBootProfileMarker> = None;
    let mut marker_invalid = false;
    let mut attestation = RuntimeAttestationAccumulator::default();
    for line in text.lines() {
        if let Some(start) = line.find(USB_BOOT_PROFILE_MARKER) {
            match UsbBootProfileMarker::parse(&line[start..]) {
                Ok(marker)
                    if maybe_marker
                        .as_ref()
                        .is_none_or(|previous| previous == &marker) =>
                {
                    maybe_marker.get_or_insert(marker);
                }
                Ok(_) | Err(_) => marker_invalid = true,
            }
        }
        if runtime_boot_attestation_marker_start(line.as_bytes()).is_some() {
            attestation.observe_line(line);
        }
    }
    if marker_invalid {
        return (UsbExecutionOwner::Unknown, "boot_profile_invalid");
    }
    if let Some(marker) = maybe_marker {
        return match admit_application_execution(
            transport,
            &marker,
            &expected.firmware_commit,
            &expected.app_elf_sha256,
        ) {
            Ok(identity) => (identity.execution_owner, "boot_profile_exact"),
            Err(_) => (UsbExecutionOwner::Unknown, "boot_profile_mismatch"),
        };
    }
    if attestation.status(expected) == RuntimeAttestationStatus::Trusted {
        (UsbExecutionOwner::Application, "runtime_attestation_exact")
    } else {
        (UsbExecutionOwner::Unknown, "marker_missing")
    }
}

fn transport_label(profile: UsbProfile) -> &'static str {
    match profile {
        UsbProfile::WorkerRuntime => "worker_runtime",
        UsbProfile::SerialJtagRuntime => "serial_jtag_runtime",
        UsbProfile::RomDownloader => "rom_downloader",
        UsbProfile::Unknown => "unknown",
    }
}

fn execution_owner_label(owner: UsbExecutionOwner) -> &'static str {
    match owner {
        UsbExecutionOwner::Unknown => "unknown",
        UsbExecutionOwner::Rom => "rom",
        UsbExecutionOwner::Application => "application",
    }
}

fn require_private_rom_exit_path(path: &Utf8Path, mode: u32, directory: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())?;
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        bail!("rom_exit=blocked reason=private_path_type");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != mode {
        bail!("rom_exit=blocked reason=private_path_mode");
    }
    Ok(())
}

fn json_line(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_download_parser_accepts_only_the_exact_register_read() {
        // Arrange / Act / Assert
        assert!(parse_force_download_bit(b"0x6000812c = 0x00000001\n").expect("set bit"));
        assert!(!parse_force_download_bit(b"0x6000812c = 0x00000000\n").expect("clear bit"));
        assert!(parse_force_download_bit(b"0x60008120 = 0x00000001\n").is_err());
        assert!(
            parse_force_download_bit(b"0x6000812c = 0x00000001\n0x6000812c = 0x00000001\n")
                .is_err()
        );
    }

    #[test]
    fn force_download_read_is_exact_and_read_only() {
        // Arrange / Act
        let args = super::force_download_read_args("admitted");

        // Assert
        assert_eq!(
            args,
            [
                "--chip",
                "esp32s3",
                "--port",
                "admitted",
                "--before",
                "no_reset",
                "--after",
                "no_reset",
                "--no-stub",
                "read_mem",
                "0x6000812c",
            ]
        );
        for forbidden in ["write_mem", "write_flash", "erase_flash"] {
            assert!(!args.iter().any(|argument| argument == forbidden));
        }
    }

    #[test]
    fn execution_owner_requires_worker_or_exact_serial_marker() {
        // Arrange
        let expected = ExpectedRuntimeAttestationIdentity {
            firmware_commit: "1".repeat(40),
            reference_commit: "3".repeat(40),
            app_elf_sha256: "2".repeat(64),
        };
        let marker = bitaxe_api::UsbBootProfileMarker::new(
            bitaxe_api::UsbBootTransport::SerialJtagRuntime,
            bitaxe_api::UsbBootProfileReason::BootBaselineUnconfirmed,
            bitaxe_api::UsbBootBaseline::Unconfirmed,
            expected.firmware_commit.clone(),
            expected.app_elf_sha256.clone(),
            4,
        )
        .expect("valid marker")
        .render();

        // Act / Assert
        assert_eq!(
            classify_execution_owner(UsbProfile::WorkerRuntime, b"", &expected).0,
            UsbExecutionOwner::Application
        );
        assert_eq!(
            classify_execution_owner(UsbProfile::SerialJtagRuntime, marker.as_bytes(), &expected).0,
            UsbExecutionOwner::Application
        );
        assert_eq!(
            classify_execution_owner(UsbProfile::SerialJtagRuntime, b"", &expected).0,
            UsbExecutionOwner::Unknown
        );
    }

    #[test]
    fn rom_exit_cli_requires_the_exact_no_write_contract() {
        // Arrange
        let args = [
            "bitaxe-flash",
            "rom-exit-diagnostic",
            "--port",
            "/dev/cu.usbmodem1101",
            "--package-manifest",
            ROM_EXIT_MANIFEST,
            "--restore-bundle",
            ROM_EXIT_BUNDLE,
            "--private-root",
            ROM_EXIT_ROOT,
            "--plan",
            ROM_EXIT_PLAN,
            "--observation-seconds",
            "30",
            "--redact-evidence",
        ];

        // Act
        let cli = parse_cli(args).expect("ROM exit CLI");

        // Assert
        let CliCommand::RomExitDiagnostic(command) = cli.command else {
            panic!("expected ROM exit command");
        };
        assert_eq!(command.board, BoardId::Ultra205);
        assert_eq!(command.private_root, Utf8Path::new(ROM_EXIT_ROOT));
        assert!(command.redact_evidence);
    }
}
