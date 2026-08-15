//! Durable physical-display UAT kept independent from programmatic command evidence.

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use bitaxe_api::{
    CommandStatusWire, DisplayFrameKind, DisplayRenderOutcome, COMMAND_STATUS_SCHEMA,
};
use bitaxe_http_transport::{ExchangeObservation, StrictHttpClient};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::evidence::{validate_empty_private_root, write_json_new};
use crate::{
    run_admitted_inspection, DeviceInspectionIntent, InspectionArtifacts, TerminalCategory,
    INSPECTION_INTENT_SCHEMA,
};

mod origin;
#[cfg(test)]
mod tests;

use origin::parse_runtime_origin_observation;

pub const DISPLAY_UAT_INTENT_SCHEMA: &str = "bitaxe-display-uat-intent-v2";
pub const DISPLAY_UAT_ADMISSION_SCHEMA: &str = "bitaxe-display-uat-admission-v1";
pub const DISPLAY_UAT_MACHINE_SCHEMA: &str = "bitaxe-display-uat-machine-v1";
pub const DISPLAY_UAT_PROJECTION_SCHEMA: &str = "bitaxe-display-uat-evidence-v1";
const CHECKPOINT_SCHEMA: &str = "bitaxe-identify-checkpoint-v3";
const POLL_INTERVAL: Duration = Duration::from_millis(200);
const RENDER_DEADLINE: Duration = Duration::from_secs(15);
const CLEAR_DEADLINE: Duration = Duration::from_secs(45);

/// Private package identity and sealed programmatic evidence binding.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayUatIntent {
    pub schema_version: String,
    pub board_category: String,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
    pub programmatic_evidence_sha256: String,
}

impl DisplayUatIntent {
    #[must_use]
    pub fn schema_is_valid(&self) -> bool {
        self.schema_version == DISPLAY_UAT_INTENT_SCHEMA
            && self.board_category == "205"
            && is_lower_hex(&self.source_commit, 40)
            && is_lower_hex(&self.reference_commit, 40)
            && is_lower_hex(&self.app_elf_sha256, 64)
            && is_lower_hex(&self.programmatic_evidence_sha256, 64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DisplayUatMachineResult {
    schema_version: String,
    boot_session: String,
    identify_generation: u64,
    identify_request_count: u8,
    machine_render_confirmed: bool,
    machine_clear_confirmed: bool,
    build_identity_matches: bool,
    usb_admission_confirmed: bool,
    programmatic_evidence_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct DisplayUatAdmissionResult {
    schema_version: &'static str,
    terminal_category: &'static str,
    identify_request_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandStatusFailure {
    Unavailable,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DisplayUatOutcome {
    category: TerminalCategory,
    identify_request_count: u8,
}

impl DisplayUatOutcome {
    const fn before_effect(category: TerminalCategory) -> Self {
        Self {
            category,
            identify_request_count: 0,
        }
    }

    const fn after_identify(category: TerminalCategory) -> Self {
        Self {
            category,
            identify_request_count: 1,
        }
    }

    const fn from_status_failure(failure: CommandStatusFailure, identify_sent: bool) -> Self {
        let category = match failure {
            CommandStatusFailure::Unavailable => TerminalCategory::ServiceRecoveryTimeout,
            CommandStatusFailure::Invalid => TerminalCategory::EvidenceInvalid,
        };
        if identify_sent {
            Self::after_identify(category)
        } else {
            Self::before_effect(category)
        }
    }
}

/// Aggregate-only final physical-display attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayUatProjection {
    pub schema_version: &'static str,
    pub board: u16,
    pub identify_request_count: u8,
    pub machine_render_confirmed: bool,
    pub machine_clear_confirmed: bool,
    pub operator_render_confirmed: bool,
    pub operator_clear_confirmed: bool,
    pub build_identity_matches: bool,
    pub usb_admission_confirmed: bool,
    pub programmatic_evidence_sha256: String,
    pub redaction_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CheckpointDocument {
    schema: String,
    checkpoint: String,
    status: String,
}

/// Runs the bounded machine portion and leaves durable human checkpoints behind.
pub fn run_display_uat_live(
    intent: DisplayUatIntent,
    admitted_port: String,
    runtime_observation: &[u8],
    programmatic_evidence: &[u8],
    root: &Utf8Path,
) -> Result<TerminalCategory> {
    validate_empty_private_root(root)?;
    let outcome = run_display_uat_admitted(
        intent,
        admitted_port,
        runtime_observation,
        programmatic_evidence,
        root,
    )?;
    write_json_new(
        &root.join("display-uat-admission.private.json"),
        &DisplayUatAdmissionResult {
            schema_version: DISPLAY_UAT_ADMISSION_SCHEMA,
            terminal_category: outcome.category.as_str(),
            identify_request_count: outcome.identify_request_count,
        },
    )?;
    Ok(outcome.category)
}

fn run_display_uat_admitted(
    intent: DisplayUatIntent,
    admitted_port: String,
    runtime_observation: &[u8],
    programmatic_evidence: &[u8],
    root: &Utf8Path,
) -> Result<DisplayUatOutcome> {
    if !intent.schema_is_valid()
        || digest(programmatic_evidence) != intent.programmatic_evidence_sha256
    {
        return Ok(DisplayUatOutcome::before_effect(
            TerminalCategory::EvidenceInvalid,
        ));
    }
    let Ok(programmatic) = serde_json::from_slice::<Value>(programmatic_evidence) else {
        return Ok(DisplayUatOutcome::before_effect(
            TerminalCategory::EvidenceInvalid,
        ));
    };
    if programmatic.get("schema_version").and_then(Value::as_str)
        != Some("bitaxe-api-command-effects-evidence-v1")
        || programmatic.get("redaction_status").and_then(Value::as_str) != Some("passed")
    {
        return Ok(DisplayUatOutcome::before_effect(
            TerminalCategory::EvidenceInvalid,
        ));
    }
    let Some(runtime) = parse_runtime_origin_observation(runtime_observation) else {
        return Ok(DisplayUatOutcome::before_effect(
            TerminalCategory::EvidenceInvalid,
        ));
    };

    let Ok(client) = StrictHttpClient::new(&runtime.origin) else {
        return Ok(DisplayUatOutcome::before_effect(
            TerminalCategory::EvidenceInvalid,
        ));
    };
    let initial_status = match command_status(&client, Instant::now() + Duration::from_secs(5)) {
        Ok(status) => status,
        Err(failure) => {
            return Ok(DisplayUatOutcome::from_status_failure(failure, false));
        }
    };
    let boot_session = initial_status.boot_session.to_string();
    if boot_session != runtime.boot_session {
        return Ok(DisplayUatOutcome::before_effect(
            TerminalCategory::BootIdentityInvalid,
        ));
    }

    let inspection_root = root.join("inspection");
    fs::create_dir(inspection_root.as_std_path())?;
    #[cfg(unix)]
    fs::set_permissions(
        inspection_root.as_std_path(),
        fs::Permissions::from_mode(0o700),
    )?;
    let inspection_projection = root.join("inspection-projection.private.json");
    let inspection = run_admitted_inspection(
        DeviceInspectionIntent {
            schema_version: INSPECTION_INTENT_SCHEMA.to_owned(),
            board_category: "205".to_owned(),
            trusted_origin: runtime.origin,
            boot_session: boot_session.clone(),
            source_commit: intent.source_commit,
            reference_commit: intent.reference_commit,
            app_elf_sha256: intent.app_elf_sha256,
        },
        admitted_port,
        InspectionArtifacts::create(&inspection_root, &inspection_projection)?,
        Duration::from_secs(15),
    )?;
    if inspection != TerminalCategory::Ready {
        return Ok(DisplayUatOutcome::before_effect(inspection));
    }

    let baseline = match command_status(&client, Instant::now() + Duration::from_secs(5)) {
        Ok(status) => status,
        Err(failure) => {
            return Ok(DisplayUatOutcome::from_status_failure(failure, false));
        }
    };
    if baseline.boot_session.to_string() != boot_session
        || baseline.identify.active
        || !baseline.display.available
    {
        return Ok(DisplayUatOutcome::before_effect(
            TerminalCategory::ObserverUnqualified,
        ));
    }
    let generation = baseline.identify.generation.saturating_add(1);
    let identify = match client.post_identify_once(Instant::now() + Duration::from_secs(5)) {
        Ok(observation) => observation,
        Err(_) => {
            return Ok(DisplayUatOutcome::after_identify(
                TerminalCategory::PostconditionMismatch,
            ));
        }
    };
    if !request_succeeded(&identify) {
        return Ok(DisplayUatOutcome::after_identify(
            TerminalCategory::PostconditionMismatch,
        ));
    }
    let rendered = match poll_status(&client, RENDER_DEADLINE, |status| {
        status.boot_session.to_string() == boot_session
            && status.identify.active
            && status.identify.generation == generation
            && successful_receipt(status, DisplayFrameKind::Identify, generation)
    }) {
        Ok(rendered) => rendered,
        Err(failure) => {
            return Ok(DisplayUatOutcome::from_status_failure(failure, true));
        }
    };
    if !rendered {
        return Ok(DisplayUatOutcome::after_identify(
            TerminalCategory::PostconditionMismatch,
        ));
    }
    let cleared = match poll_status(&client, CLEAR_DEADLINE, |status| {
        status.boot_session.to_string() == boot_session
            && !status.identify.active
            && status.identify.generation == generation
            && successful_receipt(status, DisplayFrameKind::NonIdentify, generation)
    }) {
        Ok(cleared) => cleared,
        Err(failure) => {
            return Ok(DisplayUatOutcome::from_status_failure(failure, true));
        }
    };
    if !cleared {
        return Ok(DisplayUatOutcome::after_identify(
            TerminalCategory::PostconditionMismatch,
        ));
    }

    write_json_new(
        &root.join("display-uat-machine.private.json"),
        &DisplayUatMachineResult {
            schema_version: DISPLAY_UAT_MACHINE_SCHEMA.to_owned(),
            boot_session,
            identify_generation: generation,
            identify_request_count: 1,
            machine_render_confirmed: true,
            machine_clear_confirmed: true,
            build_identity_matches: true,
            usb_admission_confirmed: true,
            programmatic_evidence_sha256: intent.programmatic_evidence_sha256,
        },
    )?;
    write_required_checkpoint(root, "rendered")?;
    write_required_checkpoint(root, "cleared")?;
    Ok(DisplayUatOutcome::after_identify(TerminalCategory::Ready))
}

/// Consumes durable human attestations and seals aggregate-only UAT evidence.
pub fn finalize_display_uat(root: &Utf8Path, projection_output: &Utf8Path) -> Result<()> {
    let machine: DisplayUatMachineResult = read_private_json(
        &root.join("display-uat-machine.private.json"),
        "display UAT machine result",
    )?;
    if machine.schema_version != DISPLAY_UAT_MACHINE_SCHEMA
        || !machine.machine_render_confirmed
        || !machine.machine_clear_confirmed
        || !machine.build_identity_matches
        || !machine.usb_admission_confirmed
        || machine.identify_request_count != 1
    {
        bail!("display UAT machine result is incomplete");
    }
    require_confirmed_checkpoint(root, "rendered")?;
    require_confirmed_checkpoint(root, "cleared")?;
    write_json_new(
        projection_output,
        &DisplayUatProjection {
            schema_version: DISPLAY_UAT_PROJECTION_SCHEMA,
            board: 205,
            identify_request_count: machine.identify_request_count,
            machine_render_confirmed: true,
            machine_clear_confirmed: true,
            operator_render_confirmed: true,
            operator_clear_confirmed: true,
            build_identity_matches: true,
            usb_admission_confirmed: true,
            programmatic_evidence_sha256: machine.programmatic_evidence_sha256,
            redaction_status: "passed",
        },
    )
}

fn command_status(
    client: &StrictHttpClient,
    deadline: Instant,
) -> std::result::Result<CommandStatusWire, CommandStatusFailure> {
    let observation = client
        .get_command_status(deadline)
        .map_err(|_| CommandStatusFailure::Unavailable)?;
    let Some(response) = observation
        .maybe_http_response()
        .filter(|response| matches!(response.status(), 200..=299))
    else {
        return Err(CommandStatusFailure::Unavailable);
    };
    let status: CommandStatusWire =
        serde_json::from_slice(response.body()).map_err(|_| CommandStatusFailure::Invalid)?;
    if status.schema != COMMAND_STATUS_SCHEMA {
        return Err(CommandStatusFailure::Invalid);
    }
    Ok(status)
}

fn request_succeeded(observation: &ExchangeObservation) -> bool {
    observation
        .maybe_http_response()
        .is_some_and(|response| matches!(response.status(), 200..=299))
}

fn poll_status(
    client: &StrictHttpClient,
    timeout: Duration,
    accepted: impl Fn(&CommandStatusWire) -> bool,
) -> std::result::Result<bool, CommandStatusFailure> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if accepted(&command_status(
            client,
            Instant::now() + Duration::from_secs(3),
        )?) {
            return Ok(true);
        }
        thread::sleep(POLL_INTERVAL);
    }
    Ok(false)
}

fn successful_receipt(status: &CommandStatusWire, kind: DisplayFrameKind, generation: u64) -> bool {
    status
        .display
        .maybe_last_success
        .as_ref()
        .is_some_and(|receipt| {
            receipt.frame_kind == kind
                && receipt.identify_generation == generation
                && receipt.outcome == DisplayRenderOutcome::Rendered
        })
}

fn write_required_checkpoint(root: &Utf8Path, checkpoint: &str) -> Result<()> {
    write_json_new(
        &root.join(format!("identify-{checkpoint}.required.json")),
        &CheckpointDocument {
            schema: CHECKPOINT_SCHEMA.to_owned(),
            checkpoint: checkpoint.to_owned(),
            status: "required".to_owned(),
        },
    )
}

fn require_confirmed_checkpoint(root: &Utf8Path, checkpoint: &str) -> Result<()> {
    let response = root.join(format!("identify-{checkpoint}.response.json"));
    let document: CheckpointDocument = read_private_json(&response, "display UAT checkpoint")?;
    if document.schema != CHECKPOINT_SCHEMA
        || document.checkpoint != checkpoint
        || document.status != "confirmed"
    {
        bail!("display UAT checkpoint is not a confirmation");
    }
    Ok(())
}

fn read_private_json<T: for<'value> Deserialize<'value>>(
    path: &Utf8Path,
    context: &str,
) -> Result<T> {
    validate_private_file(path)?;
    serde_json::from_slice(&fs::read(path.as_std_path())?)
        .with_context(|| format!("{context} is malformed"))
}

fn validate_private_file(path: &Utf8Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("display UAT artifact must be a regular non-symlink file");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        bail!("display UAT artifact must be mode 0600");
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex_lower(&digest.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
