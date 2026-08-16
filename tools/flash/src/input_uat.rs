//! Exact-package, transcript-free physical BOOT-button verification.

use crate::*;

pub(crate) const INPUT_UAT_PLAN: &str = "docs/parity/work-plans/20260816T093555Z-UI-003/PLAN.md";
pub(crate) const INPUT_UAT_PRIVATE_ROOT: &str = "scratch/ui003-input/attempt-001";
pub(crate) const INPUT_UAT_PROJECTION: &str =
    "docs/parity/evidence/ui003-input/input-uat-projection.json";
const CHECKPOINT_SCHEMA: &str = "bitaxe-input-uat-checkpoint-v1";
const MAX_PENDING_LINE_BYTES: usize = 65_536;
pub(crate) const INPUT_CORE_SOURCE: &str = "crates/bitaxe-core/src/input.rs";
pub(crate) const INPUT_ADAPTER_SOURCE: &str = "firmware/bitaxe/src/input_adapter.rs";
pub(crate) const REFERENCE_INPUT_SOURCE: &str = "reference/esp-miner/main/input.c";
const STARTUP_DEADLINE: Duration = Duration::from_secs(90);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputUatAction {
    Continue,
    PublishCheckpoint,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputUatFailure {
    RuntimeAttestationInvalid,
    CheckpointWriteFailed,
    StartupTimedOut,
    UnexpectedShortRoute,
    LongPressObserved,
    DuplicateShortClick,
    SerialFramingInvalid,
}

impl InputUatFailure {
    const fn label(self) -> &'static str {
        match self {
            Self::RuntimeAttestationInvalid => "runtime_attestation_invalid",
            Self::CheckpointWriteFailed => "checkpoint_write_failed",
            Self::StartupTimedOut => "startup_timed_out",
            Self::UnexpectedShortRoute => "unexpected_short_route",
            Self::LongPressObserved => "long_press_observed",
            Self::DuplicateShortClick => "duplicate_short_click",
            Self::SerialFramingInvalid => "serial_framing_invalid",
        }
    }
}

#[derive(Debug)]
struct InputUatObserver {
    expected_runtime: ExpectedRuntimeAttestationIdentity,
    runtime_attestations: RuntimeAttestationAccumulator,
    pending_line: Vec<u8>,
    source_semantics_admitted: bool,
    reference_semantics_admitted: bool,
    checkpoint_published: bool,
    short_click_count: u8,
    screen_advance_observed: bool,
    long_press_observed: bool,
    maybe_failure: Option<InputUatFailure>,
}

impl InputUatObserver {
    fn new(
        expected_runtime: ExpectedRuntimeAttestationIdentity,
        source_semantics_admitted: bool,
        reference_semantics_admitted: bool,
    ) -> Self {
        Self {
            expected_runtime,
            runtime_attestations: RuntimeAttestationAccumulator::default(),
            pending_line: Vec::new(),
            source_semantics_admitted,
            reference_semantics_admitted,
            checkpoint_published: false,
            short_click_count: 0,
            screen_advance_observed: false,
            long_press_observed: false,
            maybe_failure: None,
        }
    }

    fn observe_chunk(&mut self, chunk: &[u8]) -> InputUatAction {
        if self.maybe_failure.is_some() || self.complete() {
            return InputUatAction::Stop;
        }
        let checkpoint_was_published = self.checkpoint_published;
        self.pending_line.extend_from_slice(chunk);
        while let Some(newline) = self.pending_line.iter().position(|byte| *byte == b'\n') {
            let mut bytes = self.pending_line.drain(..=newline).collect::<Vec<_>>();
            bytes.pop();
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
            self.observe_line(&bytes, checkpoint_was_published);
        }
        if self.pending_line.len() > MAX_PENDING_LINE_BYTES {
            self.pending_line.clear();
            self.maybe_failure
                .get_or_insert(InputUatFailure::SerialFramingInvalid);
        }
        if self.short_click_count > 1 {
            self.maybe_failure
                .get_or_insert(InputUatFailure::DuplicateShortClick);
        }
        self.refresh_runtime_failure();
        if self.maybe_failure.is_some() || self.complete() {
            return InputUatAction::Stop;
        }
        if !self.checkpoint_published && self.ready_for_checkpoint() {
            return InputUatAction::PublishCheckpoint;
        }
        InputUatAction::Continue
    }

    fn observe_line(&mut self, bytes: &[u8], checkpoint_was_published: bool) {
        let Ok(line) = std::str::from_utf8(bytes) else {
            return;
        };
        if bitaxe_api::runtime_boot_attestation_marker_start(line.as_bytes()).is_some() {
            self.runtime_attestations.observe_line(line);
        }
        if !checkpoint_was_published {
            return;
        }
        if log_message_matches(line, "input_event=short_click effect=screen_advance") {
            self.short_click_count = self.short_click_count.saturating_add(1);
            self.screen_advance_observed = true;
        } else if line.contains("input_event=short_click") {
            self.maybe_failure
                .get_or_insert(InputUatFailure::UnexpectedShortRoute);
        }
        if line.contains("input_event=long_press") {
            self.long_press_observed = true;
            self.maybe_failure
                .get_or_insert(InputUatFailure::LongPressObserved);
        }
    }

    fn ready_for_checkpoint(&self) -> bool {
        self.source_semantics_admitted
            && self.reference_semantics_admitted
            && self.runtime_status() == RuntimeAttestationStatus::Trusted
    }

    fn publish_checkpoint(&mut self) {
        self.checkpoint_published = true;
    }

    fn fail(&mut self, failure: InputUatFailure) {
        self.maybe_failure.get_or_insert(failure);
    }

    fn complete(&self) -> bool {
        self.checkpoint_published
            && self.short_click_count == 1
            && self.screen_advance_observed
            && !self.long_press_observed
            && self.maybe_failure.is_none()
    }

    fn runtime_status(&self) -> RuntimeAttestationStatus {
        self.runtime_attestations.status(&self.expected_runtime)
    }

    fn refresh_runtime_failure(&mut self) {
        match self.runtime_status() {
            RuntimeAttestationStatus::Trusted
            | RuntimeAttestationStatus::Missing
            | RuntimeAttestationStatus::InsufficientSamples => {}
            RuntimeAttestationStatus::Malformed
            | RuntimeAttestationStatus::MixedSessionOrOrdinal
            | RuntimeAttestationStatus::StaticFieldsMismatch
            | RuntimeAttestationStatus::NonMonotonicUptime
            | RuntimeAttestationStatus::PackageIdentityMismatch
            | RuntimeAttestationStatus::IncompleteReadiness => {
                self.maybe_failure
                    .get_or_insert(InputUatFailure::RuntimeAttestationInvalid);
            }
        }
    }
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct InputUatCheckpoint<'a> {
    schema_version: &'static str,
    status: &'static str,
    instruction: &'a str,
    maximum_hold_milliseconds: u16,
}

pub(crate) fn run_input_uat(
    command: &InputUatCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    validate_input_uat_paths(command)?;
    let private_root = environment.workspace_path(&command.private_root);
    let projection = environment.workspace_path(&command.projection);
    let plan = environment.workspace_path(&command.plan);
    environment.approve_private_evidence_root(&private_root)?;
    create_private_attempt_root(&private_root)?;
    preflight_public_projection(&projection, environment)?;

    let plan_bytes = environment.read_bytes(&plan)?;
    validate_plan(&plan_bytes)?;
    let plan_sha256 = sha256_bytes(&plan_bytes);
    admit_input_semantics(environment)?;
    let source_semantics_admitted = true;
    let reference_semantics_admitted = true;
    let manifest_path = environment.workspace_path(&command.manifest);
    let manifest_before = environment.read_bytes(&manifest_path)?;
    let manifest_sha256 = sha256_bytes(&manifest_before);

    let flash_command = FlashCommand {
        common: CommonArgs {
            board: command.board,
            port: Some(command.port.clone()),
            dry_run: false,
            redact_evidence: false,
            evidence_mode: None,
            evidence_dir: None,
        },
        image: None,
        manifest: Some(command.manifest.clone()),
        wifi_credentials: None,
    };
    let flash_outcome = run_flash(&flash_command, environment)?;
    if environment.device_effect_state() != UsbDeviceEffectState::Completed {
        bail!("input_uat=failed reason=exact_package_flash_not_completed");
    }
    let expected_runtime = flash_outcome
        .runtime_identity
        .context("input_uat=failed reason=runtime_identity_unavailable")?;
    if environment.read_bytes(&manifest_path)? != manifest_before {
        bail!("input_uat=failed reason=manifest_changed_after_admission");
    }

    let checkpoint_path = private_root.join("short-click.required.json");
    let started = Instant::now();
    let mut observer = InputUatObserver::new(
        expected_runtime.clone(),
        source_semantics_admitted,
        reference_semantics_admitted,
    );
    let transport = environment.receive_input_uat(&mut |chunk| {
        if !observer.checkpoint_published && started.elapsed() >= STARTUP_DEADLINE {
            observer.fail(InputUatFailure::StartupTimedOut);
            return true;
        }
        if observer.observe_chunk(chunk) == InputUatAction::PublishCheckpoint {
            if write_checkpoint(&checkpoint_path).is_err()
                || emit_line(
                    "input_uat_checkpoint",
                    "ready - briefly press and release BOOT once; do not hold for two seconds",
                )
                .is_err()
            {
                observer.fail(InputUatFailure::CheckpointWriteFailed);
                return true;
            }
            observer.publish_checkpoint();
        }
        observer.maybe_failure.is_some() || observer.complete()
    });
    let operation_result = validate_input_observation(transport, &observer);
    let cleanup_result = environment.finish_usb_session();
    combine_operation_and_cleanup(operation_result, cleanup_result)?;
    if environment.read_bytes(&plan)? != plan_bytes {
        bail!("input_uat=failed reason=plan_changed_after_admission");
    }
    admit_input_semantics(environment)
        .context("input_uat=failed reason=source_changed_after_admission")?;

    let evidence = InputUatEvidence {
        schema_version: INPUT_UAT_EVIDENCE_SCHEMA.to_owned(),
        board: 205,
        source_commit: expected_runtime.firmware_commit,
        reference_commit: expected_runtime.reference_commit,
        app_elf_sha256: expected_runtime.app_elf_sha256,
        package_manifest_sha256: manifest_sha256,
        plan_sha256,
        input: InputUatObservationEvidence {
            gpio: 0,
            active_low: true,
            pull_up_enabled: true,
            sampling_ms: bitaxe_core::input::BUTTON_SAMPLE_MS,
            debounce_ms: bitaxe_core::input::BUTTON_DEBOUNCE_MS,
            long_press_ms: bitaxe_core::input::BUTTON_LONG_PRESS_MS,
            checkpoint_published_before_input: observer.checkpoint_published,
            physical_short_click_count: observer.short_click_count,
            screen_advance_observed: observer.screen_advance_observed,
            long_press_observed: observer.long_press_observed,
        },
        exact_package_flash_completed: true,
        runtime_attestation_trusted: observer.runtime_status() == RuntimeAttestationStatus::Trusted,
        source_semantics_admitted,
        reference_semantics_admitted,
        usb_admission_confirmed: true,
        cleanup_complete: true,
        mining_state: "disabled".to_owned(),
        hardware_control_state: "disabled".to_owned(),
        serial_transcript_retained: false,
        redaction_status: "passed".to_owned(),
    };
    evidence
        .validate()
        .map_err(|error| anyhow::anyhow!("input_uat=failed reason={error}"))?;
    write_public_projection(&projection, &evidence)?;
    emit_line("input_uat", "verified")
}

fn validate_input_observation(
    transport: Result<MonitorOutput>,
    observer: &InputUatObserver,
) -> Result<()> {
    if let Some(failure) = observer.maybe_failure {
        bail!("input_uat=failed reason={}", failure.label());
    }
    let output = transport?;
    if output.interrupted_by.is_some() {
        bail!("input_uat=stopped reason=operator_interrupted");
    }
    if !observer.complete() {
        bail!("input_uat=failed reason=physical_short_click_not_observed");
    }
    if !output.bytes.is_empty() {
        bail!("input_uat=failed reason=serial_transcript_retained");
    }
    Ok(())
}

fn validate_input_uat_paths(command: &InputUatCommand) -> Result<()> {
    if command.private_root != Utf8Path::new(INPUT_UAT_PRIVATE_ROOT)
        || command.plan != Utf8Path::new(INPUT_UAT_PLAN)
        || command.projection != Utf8Path::new(INPUT_UAT_PROJECTION)
    {
        bail!("input_uat=blocked reason=path_contract_mismatch");
    }
    Ok(())
}

fn validate_plan(bytes: &[u8]) -> Result<()> {
    let plan = std::str::from_utf8(bytes).context("input_uat=blocked reason=plan_invalid")?;
    for marker in [
        "- Run ID: `20260816T093555Z-UI-003`",
        "- Parity row: `UI-003`",
        "`attempt-001` is the only authorized effectful attempt",
    ] {
        if plan.matches(marker).count() != 1 {
            bail!("input_uat=blocked reason=plan_contract_mismatch");
        }
    }
    Ok(())
}

fn admit_input_semantics(environment: &impl FlashEnvironment) -> Result<()> {
    let core = environment
        .read_to_string(&environment.workspace_path(Utf8Path::new(INPUT_CORE_SOURCE)))?;
    let adapter = environment
        .read_to_string(&environment.workspace_path(Utf8Path::new(INPUT_ADAPTER_SOURCE)))?;
    let reference = environment
        .read_to_string(&environment.workspace_path(Utf8Path::new(REFERENCE_INPUT_SOURCE)))?;
    for marker in [
        "pub const BUTTON_SAMPLE_MS: u64 = 10;",
        "pub const BUTTON_DEBOUNCE_MS: u64 = 30;",
        "pub const BUTTON_LONG_PRESS_MS: u64 = 2_000;",
    ] {
        require_unique_source_marker(&core, marker)?;
    }
    for marker in [
        "PinDriver::input(pin, Pull::Up)?",
        "input_status=active owner=boot_button sampling_ms={BUTTON_SAMPLE_MS} active_low=true",
        "input_event=short_click effect=screen_advance",
    ] {
        require_unique_source_marker(&adapter, marker)?;
    }
    for marker in [
        "#define LONG_PRESS_DURATION_MS 2000",
        ".pull_up_en = GPIO_PULLUP_ENABLE",
        "gpio_get_level(GPIO_BUTTON_BOOT) == 0",
        "LV_EVENT_SHORT_CLICKED",
    ] {
        require_unique_source_marker(&reference, marker)?;
    }
    Ok(())
}

fn require_unique_source_marker(source: &str, marker: &str) -> Result<()> {
    if source.matches(marker).count() != 1 {
        bail!("input_uat=blocked reason=source_semantics_mismatch");
    }
    Ok(())
}

fn create_private_attempt_root(root: &Utf8Path) -> Result<()> {
    let parent = root
        .parent()
        .context("input_uat=blocked reason=private_root_parent_invalid")?;
    fs::create_dir_all(parent.as_std_path())?;
    fs::create_dir(root.as_std_path())
        .with_context(|| format!("input_uat=blocked reason=private_root_not_fresh path={root}"))?;
    set_private_directory_mode(root)?;
    #[cfg(unix)]
    if fs::metadata(root.as_std_path())?.permissions().mode() & 0o777 != 0o700 {
        bail!("input_uat=blocked reason=private_root_mode_invalid");
    }
    Ok(())
}

fn preflight_public_projection(
    projection: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let expected = environment.workspace_path(Utf8Path::new(INPUT_UAT_PROJECTION));
    if projection != expected {
        bail!("input_uat=blocked reason=projection_path_invalid");
    }
    let Some(parent) = projection.parent() else {
        bail!("input_uat=blocked reason=projection_parent_invalid");
    };
    fs::create_dir_all(parent.as_std_path())?;
    let workspace = fs::canonicalize(environment.workspace_path(Utf8Path::new(".")).as_std_path())?;
    let canonical_parent = fs::canonicalize(parent.as_std_path())?;
    if !canonical_parent.starts_with(&workspace) {
        bail!("input_uat=blocked reason=projection_parent_escape");
    }
    match fs::symlink_metadata(projection.as_std_path()) {
        Ok(_) => bail!("input_uat=blocked reason=projection_already_exists"),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn write_checkpoint(path: &Utf8Path) -> Result<()> {
    let checkpoint = InputUatCheckpoint {
        schema_version: CHECKPOINT_SCHEMA,
        status: "required",
        instruction: "briefly press and release BOOT exactly once",
        maximum_hold_milliseconds: 1_999,
    };
    let mut bytes = serde_json::to_vec_pretty(&checkpoint)?;
    bytes.push(b'\n');
    write_private_new_bytes(path, &bytes)?;
    set_private_file_mode(path)
}

fn write_public_projection(path: &Utf8Path, evidence: &InputUatEvidence) -> Result<()> {
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
            bail!("input_uat=failed reason=projection_mode_invalid");
        }
    }
    Ok(())
}

fn log_message_matches(line: &str, marker: &str) -> bool {
    let line = line.trim();
    line == marker || line.ends_with(&format!(": {marker}"))
}

#[cfg(test)]
mod tests;
