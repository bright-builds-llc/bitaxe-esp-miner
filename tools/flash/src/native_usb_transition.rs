use crate::*;

const TRANSITION_SCHEMA: &str = "bitaxe-native-usb-transition-projection-v1";
const TRANSITION_EVALUATOR_SOURCES: [(&str, &str); 4] = [
    (
        "tools/flash/src/native_usb_transition.rs",
        include_str!("native_usb_transition.rs"),
    ),
    ("tools/flash/src/cli.rs", include_str!("cli.rs")),
    (
        "tools/flash/src/environment.rs",
        include_str!("environment.rs"),
    ),
    ("tools/flash/src/main.rs", include_str!("main.rs")),
];
pub(crate) const TRANSITION_TASK: &str = "task-native-usb-recovery-transition-205";
pub(crate) const TRANSITION_PLAN: &str =
    "docs/parity/work-plans/20260830T142327Z-NATIVE-USB-RECOVERY-TRANSITION/PLAN.md";
const TRANSITION_PLAN_SHA256: &str =
    "cbc11639a51e67d24a04b33c05dd3dd2e570914be79f3a3d80b7326894e74eca";
pub(crate) const TRANSITION_MANIFEST: &str =
    "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
pub(crate) const TRANSITION_PRIVATE_ROOT: &str = "scratch/native-usb-transition/diagnostic-001";
pub(crate) const TRANSITION_PROJECTION: &str =
    "docs/parity/evidence/native-usb-transition/transition-projection-001.json";

#[derive(Debug, Serialize)]
struct NativeUsbTransitionProjection {
    schema_version: &'static str,
    source_commit: String,
    reference_commit: String,
    plan_sha256: &'static str,
    evaluator_sha256: String,
    manifest_sha256: String,
    app_elf_sha256: String,
    ready_received: bool,
    committed_received: bool,
    bus_reset_observed: bool,
    absent_count: u16,
    same_worker_count: u16,
    same_serial_jtag_count: u16,
    same_unknown_count: u16,
    physical_mismatch_count: u16,
    rom_admitted: bool,
    application_reappeared: bool,
    device_write_observed: bool,
    restoration_complete: bool,
    cleanup_complete: bool,
    redaction_status: &'static str,
    terminal_category: &'static str,
}

pub(crate) fn run_verify_native_usb_transition(
    command: &VerifyNativeUsbTransitionCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    validate_transition_invocation(command)?;
    let plan_path = environment.workspace_path(&command.plan);
    let plan_document = environment.read_bytes(&plan_path)?;
    if sha256_bytes(&plan_document) != TRANSITION_PLAN_SHA256 {
        bail!("native_usb_transition=blocked reason=plan_identity");
    }
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    if !tasks.contains(&format!("### {TRANSITION_TASK}")) {
        bail!("native_usb_transition=blocked reason=task_identity");
    }
    let manifest_path = environment.workspace_path(&command.manifest);
    let manifest_document = environment.read_bytes(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_document)?;
    let source_commit = environment.firmware_commit();
    let reference_commit = environment.reference_commit();
    if environment.pushed_firmware_commit() != source_commit
        || manifest
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            != Some(3)
        || manifest
            .get("source_commit")
            .and_then(serde_json::Value::as_str)
            != Some(source_commit.as_str())
        || manifest
            .get("reference_commit")
            .and_then(serde_json::Value::as_str)
            != Some(reference_commit.as_str())
    {
        bail!("native_usb_transition=blocked reason=source_identity");
    }
    let app_elf_sha256 = manifest
        .get("app_elf_sha256")
        .and_then(serde_json::Value::as_str)
        .filter(|value| is_sha256(value))
        .context("native_usb_transition=blocked reason=manifest_identity")?
        .to_owned();
    validate_transition_artifacts(&manifest, &manifest_path, environment)?;
    let private_root = environment.workspace_path(&command.private_root);
    require_absent_path(&private_root)?;
    environment.approve_private_evidence_root(&command.private_root)?;
    let private_parent = private_root
        .parent()
        .context("native_usb_transition=blocked reason=private_root_parent")?;
    fs::create_dir_all(private_parent.as_std_path())?;
    set_private_directory_mode(private_parent)?;
    fs::create_dir(private_root.as_std_path())?;
    set_private_directory_mode(&private_root)?;
    let intent = serde_json::json!({
        "schema_version": "bitaxe-native-usb-transition-intent-v1",
        "source_commit": source_commit,
        "reference_commit": reference_commit,
        "plan_sha256": TRANSITION_PLAN_SHA256,
        "manifest_sha256": sha256_bytes(&manifest_document),
        "ordinal": 1,
        "write_allowed": false,
    });
    let mut intent_bytes = serde_json::to_vec_pretty(&intent)?;
    intent_bytes.push(b'\n');
    write_private_new_bytes(&private_root.join("intent.private.json"), &intent_bytes)?;

    let transition_result = environment
        .begin_usb_session(UsbOperation::VerifyTransition, &command.port)
        .and_then(|()| environment.verify_native_usb_transition(&command.port));
    let device_write_observed = environment.device_effect_state() != UsbDeviceEffectState::None;
    let failure_profile_counts = environment.native_usb_profile_counts();
    let cleanup_result = environment.finish_usb_session();
    let (outcome, terminal_category) = match &transition_result {
        Ok(outcome) if !device_write_observed && cleanup_result.is_ok() => (*outcome, "complete"),
        Ok(outcome) if device_write_observed => (*outcome, "recovery_required"),
        Ok(outcome) => (*outcome, "cleanup_failed"),
        Err(error) => (
            transition_failure_outcome(closed_transition_category(error), failure_profile_counts),
            closed_transition_category(error),
        ),
    };
    let counts = outcome.profile_counts;
    let projection = NativeUsbTransitionProjection {
        schema_version: TRANSITION_SCHEMA,
        source_commit: source_commit.clone(),
        reference_commit: reference_commit.clone(),
        plan_sha256: TRANSITION_PLAN_SHA256,
        evaluator_sha256: transition_evaluator_sha256(),
        manifest_sha256: sha256_bytes(&manifest_document),
        app_elf_sha256,
        ready_received: outcome.ready_received,
        committed_received: outcome.committed_received,
        bus_reset_observed: outcome.bus_reset_observed,
        absent_count: counts.absent,
        same_worker_count: counts.same_worker,
        same_serial_jtag_count: counts.same_serial_jtag,
        same_unknown_count: counts.same_unknown,
        physical_mismatch_count: counts.physical_mismatch,
        rom_admitted: outcome.rom_admitted,
        application_reappeared: outcome.application_reappeared,
        device_write_observed,
        restoration_complete: false,
        cleanup_complete: cleanup_result.is_ok(),
        redaction_status: "passed",
        terminal_category,
    };
    let mut private_result = serde_json::to_vec_pretty(&projection)?;
    private_result.push(b'\n');
    write_private_new_bytes(
        &private_root.join("transition-result.private.json"),
        &private_result,
    )?;
    if let Err(error) = transition_result {
        return match cleanup_result {
            Ok(()) => Err(error),
            Err(_) => Err(error.context("cleanup_failure=secondary")),
        };
    }
    if device_write_observed {
        bail!("native_usb_transition=blocked reason=device_write_observed");
    }
    cleanup_result?;
    emit_line("native_usb_transition", "complete")?;
    Ok(())
}

pub(crate) fn transition_failure_outcome(
    category: &'static str,
    profile_counts: ProfileObservationCounts,
) -> NativeUsbTransitionOutcome {
    let ready_received = !matches!(
        category,
        "handoff_ready_timeout"
            | "handoff_unsupported"
            | "handoff_rejected_unsafe_state"
            | "runtime_profile_unknown"
            | "foreign_holder"
    );
    let committed_received = ready_received && category != "handoff_commit_timeout";
    let bus_reset_observed = matches!(
        category,
        "rom_admission_failed" | "application_reappearance_timeout" | "physical_identity_drift"
    );
    let rom_admitted = category == "application_reappearance_timeout";
    NativeUsbTransitionOutcome {
        ready_received,
        committed_received,
        bus_reset_observed,
        profile_counts,
        rom_admitted,
        application_reappeared: false,
    }
}

pub(crate) fn closed_transition_category(error: &anyhow::Error) -> &'static str {
    let detail = error.to_string();
    let candidate = detail.split(':').next().unwrap_or_default();
    match candidate {
        "runtime_profile_unknown" => "runtime_profile_unknown",
        "handoff_unsupported" => "handoff_unsupported",
        "handoff_rejected_unsafe_state" => "handoff_rejected_unsafe_state",
        "handoff_ready_timeout" => "handoff_ready_timeout",
        "handoff_commit_timeout" => "handoff_commit_timeout",
        "bus_reset_timeout" => "bus_reset_timeout",
        "same_worker_after_commit" => "same_worker_after_commit",
        "handoff_transition_timeout" => "handoff_transition_timeout",
        "bootloader_ambiguous" => "bootloader_ambiguous",
        "physical_identity_drift" => "physical_identity_drift",
        "rom_admission_failed" => "rom_admission_failed",
        "application_reappearance_timeout" => "application_reappearance_timeout",
        "foreign_holder" => "foreign_holder",
        "cleanup_failed" => "cleanup_failed",
        _ => "recovery_required",
    }
}

pub(crate) fn transition_evaluator_sha256() -> String {
    let mut input = Vec::new();
    input.extend_from_slice(native_usb_transition_module_sha256().as_bytes());
    input.push(0xfe);
    for (path, source) in TRANSITION_EVALUATOR_SOURCES {
        input.extend_from_slice(path.as_bytes());
        input.push(0);
        input.extend_from_slice(source.as_bytes());
        input.push(0xff);
    }
    sha256_bytes(&input)
}

fn validate_transition_invocation(command: &VerifyNativeUsbTransitionCommand) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if command.manifest != Utf8Path::new(TRANSITION_MANIFEST)
        || command.plan != Utf8Path::new(TRANSITION_PLAN)
        || command.private_root != Utf8Path::new(TRANSITION_PRIVATE_ROOT)
        || command.projection != Utf8Path::new(TRANSITION_PROJECTION)
        || !command.redact_evidence
    {
        bail!("native_usb_transition=blocked reason=invocation");
    }
    Ok(())
}

fn validate_transition_artifacts(
    manifest: &serde_json::Value,
    manifest_path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let artifacts = manifest
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .context("native_usb_transition=blocked reason=manifest_artifacts")?;
    let manifest_dir = manifest_path
        .parent()
        .context("native_usb_transition=blocked reason=manifest_path")?;
    for artifact in artifacts {
        let relative = artifact
            .get("path")
            .and_then(serde_json::Value::as_str)
            .context("native_usb_transition=blocked reason=manifest_artifacts")?;
        let digest = artifact
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .filter(|value| is_sha256(value))
            .context("native_usb_transition=blocked reason=manifest_artifacts")?;
        let path = if relative == "firmware/bitaxe/partitions-ultra205.csv" {
            environment.workspace_path(Utf8Path::new(relative))
        } else {
            manifest_dir.join(relative)
        };
        if sha256_bytes(&environment.read_bytes(&path)?) != digest {
            bail!("native_usb_transition=blocked reason=artifact_digest");
        }
    }
    Ok(())
}

fn require_absent_path(path: &Utf8Path) -> Result<()> {
    match fs::symlink_metadata(path.as_std_path()) {
        Ok(_) => bail!("native_usb_transition=blocked reason=private_root_exists"),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
