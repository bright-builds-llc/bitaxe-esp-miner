use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use bitaxe_api::SystemInfoWire;
use bitaxe_http_transport::{ExchangeObservation, StrictHttpClient};
use camino::Utf8Path;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use super::super::CampaignTerminalCategory;
use super::command_evidence::CommandEffectsEvidence;
use super::model::{CampaignNetworkEvidence, SharedSerialState, TrustedNetworkTarget};
use super::validation::{active_mining_state_valid, validate_identity_and_safety};
use crate::write_private_new_bytes;

const HTTP_DEADLINE: Duration = Duration::from_secs(3);
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const TERMINAL_DEADLINE: Duration = Duration::from_secs(15);
const CHECKPOINT_SCHEMA: &str = "bitaxe-identify-checkpoint-v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentifyObservation {
    Rendered,
    Cleared,
}

impl IdentifyObservation {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Rendered => "rendered",
            Self::Cleared => "cleared",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IdentifyCheckpoint {
    schema: String,
    observation: IdentifyObservation,
    status: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommandPhase {
    Notification,
    Pause,
    Resume,
    IdentifyRendered,
    IdentifyCleared,
    Dismiss,
    Terminal,
}

struct CommandProgress<'a> {
    phase: &'a mut CommandPhase,
    maybe_block_count: &'a mut Option<u64>,
    evidence: &'a mut CommandEffectsEvidence,
    maybe_failure: &'a mut Option<CampaignTerminalCategory>,
}

pub(super) fn observe_command_effects(
    target: TrustedNetworkTarget,
    shared: Arc<Mutex<SharedSerialState>>,
    evidence_root: &Utf8Path,
) -> CampaignNetworkEvidence {
    let Ok(http) = StrictHttpClient::new(&target.origin) else {
        return CampaignNetworkEvidence::from_unobserved(&shared);
    };
    let mut evidence = CommandEffectsEvidence::new();
    let mut maybe_failure = None;
    let mut phase = CommandPhase::Notification;
    let mut maybe_block_count = None;
    let mut maybe_terminal_deadline = None;
    let mut recovery_pause_request_count = 0;

    loop {
        let serial = shared_snapshot(&shared);
        if maybe_failure.is_none() {
            maybe_failure = serial.maybe_failure;
        }
        evidence.terminal_pool_persisted = serial.terminal_pool_persisted;
        if serial.terminal_consumed && maybe_terminal_deadline.is_none() {
            maybe_terminal_deadline = Some(Instant::now() + TERMINAL_DEADLINE);
        }

        if maybe_failure.is_none() {
            match fetch_system_info(&http) {
                Ok(Some(sample)) => {
                    if validate_identity_and_safety(&sample, &target).is_err() {
                        evidence.same_boot_and_package = false;
                        evidence.safety_valid = false;
                        maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
                    } else {
                        advance_commands(
                            &http,
                            &target,
                            evidence_root,
                            &sample,
                            CommandProgress {
                                phase: &mut phase,
                                maybe_block_count: &mut maybe_block_count,
                                evidence: &mut evidence,
                                maybe_failure: &mut maybe_failure,
                            },
                        );
                        if serial.terminal_consumed
                            && phase == CommandPhase::Terminal
                            && sample.mining_paused
                            && sample.mining_activity == "paused"
                            && !sample.start_mining_on_boot
                        {
                            evidence.terminal_http_valid = true;
                        }
                    }
                }
                Ok(None) => {}
                Err(category) => maybe_failure = Some(category),
            }
        }
        if take_recovery_pause_request(maybe_failure, &mut recovery_pause_request_count) {
            let _result = http.post_pause_once(Instant::now() + HTTP_DEADLINE);
        }

        if serial.terminal_consumed && evidence.terminal_http_valid {
            break;
        }
        if maybe_terminal_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            maybe_failure.get_or_insert(CampaignTerminalCategory::TerminalStateUnconfirmed);
            break;
        }
        if serial.serial_finished {
            if !serial.terminal_consumed {
                maybe_failure.get_or_insert(CampaignTerminalCategory::TerminalStateUnconfirmed);
            }
            break;
        }
        std::thread::sleep(POLL_INTERVAL);
    }

    CampaignNetworkEvidence::from_command_effects(
        evidence,
        recovery_pause_request_count,
        maybe_failure,
    )
}

fn take_recovery_pause_request(
    maybe_failure: Option<CampaignTerminalCategory>,
    request_count: &mut u64,
) -> bool {
    if maybe_failure.is_none() || *request_count > 0 {
        return false;
    }
    *request_count = 1;
    true
}

fn advance_commands(
    http: &StrictHttpClient,
    target: &TrustedNetworkTarget,
    evidence_root: &Utf8Path,
    sample: &SystemInfoWire,
    progress: CommandProgress<'_>,
) {
    let CommandProgress {
        phase,
        maybe_block_count,
        evidence,
        maybe_failure,
    } = progress;
    match phase {
        CommandPhase::Notification
            if active_mining_state_valid(sample)
                && sample.show_new_block
                && sample.block_found > 0 =>
        {
            evidence.active_before_pause = true;
            evidence.genuine_block_notification_observed = true;
            evidence.positive_block_count_observed = true;
            *maybe_block_count = Some(sample.block_found);
            if write_reboot_intent(evidence_root, target, sample).is_err() {
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
                return;
            }
            evidence.pause_request_count = 1;
            if post_succeeded(http.post_pause_once(Instant::now() + HTTP_DEADLINE)) {
                *phase = CommandPhase::Pause;
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
            }
        }
        CommandPhase::Pause if sample.mining_paused && sample.mining_activity == "paused" => {
            evidence.pause_confirmed = true;
            evidence.resume_request_count = 1;
            if post_succeeded(http.post_resume_once(Instant::now() + HTTP_DEADLINE)) {
                *phase = CommandPhase::Resume;
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
            }
        }
        CommandPhase::Resume if active_mining_state_valid(sample) => {
            evidence.resume_confirmed = true;
            evidence.active_after_resume = true;
            evidence.identify_request_count = 1;
            if post_succeeded(http.post_identify_once(Instant::now() + HTTP_DEADLINE))
                && write_required_checkpoint(evidence_root, IdentifyObservation::Rendered).is_ok()
            {
                *phase = CommandPhase::IdentifyRendered;
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
            }
        }
        CommandPhase::IdentifyRendered => {
            match consume_confirmation(evidence_root, IdentifyObservation::Rendered) {
                Ok(true) => {
                    evidence.identify_rendered_confirmed = true;
                    evidence.identify_request_count = 2;
                    if post_succeeded(http.post_identify_once(Instant::now() + HTTP_DEADLINE))
                        && write_required_checkpoint(evidence_root, IdentifyObservation::Cleared)
                            .is_ok()
                    {
                        *phase = CommandPhase::IdentifyCleared;
                    } else {
                        *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid);
                    }
                }
                Ok(false) => {}
                Err(()) => {
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
                }
            }
        }
        CommandPhase::IdentifyCleared => {
            match consume_confirmation(evidence_root, IdentifyObservation::Cleared) {
                Ok(true) => {
                    evidence.identify_cleared_confirmed = true;
                    evidence.dismiss_request_count = 1;
                    if post_succeeded(
                        http.post_block_found_dismiss_once(Instant::now() + HTTP_DEADLINE),
                    ) {
                        *phase = CommandPhase::Dismiss;
                    } else {
                        *maybe_failure = Some(CampaignTerminalCategory::CommandRequestFailed);
                    }
                }
                Ok(false) => {}
                Err(()) => {
                    *maybe_failure = Some(CampaignTerminalCategory::OperatorCheckpointInvalid)
                }
            }
        }
        CommandPhase::Dismiss if !sample.show_new_block => {
            evidence.dismiss_confirmed = true;
            evidence.block_count_preserved =
                maybe_block_count.is_some_and(|count| sample.block_found == count);
            if evidence.block_count_preserved {
                *phase = CommandPhase::Terminal;
            } else {
                *maybe_failure = Some(CampaignTerminalCategory::NetworkCorrelationFailed);
            }
        }
        _ => {}
    }
}

fn write_reboot_intent(
    root: &Utf8Path,
    target: &TrustedNetworkTarget,
    sample: &SystemInfoWire,
) -> anyhow::Result<()> {
    let intent = serde_json::json!({
        "schema_version": "esp-device-session-reboot-intent-v1",
        "board_category": "205",
        "trusted_origin": target.origin,
        "baseline": {
            "boot_session": target.boot_session,
            "boot_ordinal": target.boot_ordinal,
            "source_commit": target.expected.firmware_commit,
            "reference_commit": target.expected.reference_commit,
            "app_elf_sha256": target.expected.app_elf_sha256,
        },
        "expected_postcondition": {
            "hostname_sha256": crate::sha256_bytes(sample.hostname.as_bytes()),
        },
    });
    let mut bytes = serde_json::to_vec_pretty(&intent)?;
    bytes.push(b'\n');
    write_private_new_bytes(
        &root.join("command-effects-reboot-intent.private.json"),
        &bytes,
    )
}

fn fetch_system_info(
    http: &StrictHttpClient,
) -> Result<Option<SystemInfoWire>, CampaignTerminalCategory> {
    let observation = http
        .get_system_info(Instant::now() + HTTP_DEADLINE)
        .map_err(|_| CampaignTerminalCategory::NetworkCorrelationFailed)?;
    let Some(response) = observation
        .maybe_http_response()
        .filter(|response| response.status() == 200)
    else {
        return Ok(None);
    };
    serde_json::from_slice(response.body())
        .map(Some)
        .map_err(|_| CampaignTerminalCategory::NetworkCorrelationFailed)
}

fn post_succeeded(result: anyhow::Result<ExchangeObservation>) -> bool {
    result.ok().and_then(|observation| {
        observation
            .maybe_http_response()
            .map(|response| response.status())
    }) == Some(200)
}

fn checkpoint_path(
    root: &Utf8Path,
    observation: IdentifyObservation,
    state: &str,
) -> camino::Utf8PathBuf {
    root.join(format!("identify-{}.{}.json", observation.as_str(), state))
}

fn write_required_checkpoint(
    root: &Utf8Path,
    observation: IdentifyObservation,
) -> anyhow::Result<()> {
    let checkpoint = IdentifyCheckpoint {
        schema: CHECKPOINT_SCHEMA.to_owned(),
        observation,
        status: "required".to_owned(),
    };
    let mut bytes = serde_json::to_vec_pretty(&checkpoint)?;
    bytes.push(b'\n');
    write_private_new_bytes(&checkpoint_path(root, observation, "required"), &bytes)
}

fn consume_confirmation(root: &Utf8Path, observation: IdentifyObservation) -> Result<bool, ()> {
    let confirmed = checkpoint_path(root, observation, "confirmed");
    let consumed = checkpoint_path(root, observation, "consumed");
    let metadata = match fs::symlink_metadata(confirmed.as_std_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err(()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(());
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(());
    }
    if fs::symlink_metadata(consumed.as_std_path()).is_ok()
        || fs::rename(confirmed.as_std_path(), consumed.as_std_path()).is_err()
    {
        return Err(());
    }
    let bytes = fs::read(consumed.as_std_path()).map_err(|_| ())?;
    let checkpoint: IdentifyCheckpoint = serde_json::from_slice(&bytes).map_err(|_| ())?;
    if checkpoint.schema != CHECKPOINT_SCHEMA
        || checkpoint.observation != observation
        || checkpoint.status != "confirmed"
    {
        return Err(());
    }
    Ok(true)
}

fn shared_snapshot(shared: &Arc<Mutex<SharedSerialState>>) -> SharedSerialState {
    shared.lock().map_or_else(
        |_| SharedSerialState {
            serial_finished: true,
            maybe_failure: Some(CampaignTerminalCategory::NetworkCorrelationFailed),
            ..SharedSerialState::default()
        },
        |state| state.clone(),
    )
}

pub(crate) fn confirm_identify_observation(
    root: &Utf8Path,
    observation: IdentifyObservation,
) -> anyhow::Result<()> {
    let required = checkpoint_path(root, observation, "required");
    let confirmed = checkpoint_path(root, observation, "confirmed");
    let consumed = checkpoint_path(root, observation, "consumed");
    let metadata = fs::symlink_metadata(required.as_std_path())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        anyhow::bail!("identify_checkpoint=blocked reason=required_checkpoint_invalid");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        anyhow::bail!("identify_checkpoint=blocked reason=required_checkpoint_not_private");
    }
    if confirmed.exists() || consumed.exists() {
        anyhow::bail!("identify_checkpoint=blocked reason=checkpoint_already_used");
    }
    let required_checkpoint: IdentifyCheckpoint =
        serde_json::from_slice(&fs::read(required.as_std_path())?)?;
    if required_checkpoint.schema != CHECKPOINT_SCHEMA
        || required_checkpoint.observation != observation
        || required_checkpoint.status != "required"
    {
        anyhow::bail!("identify_checkpoint=blocked reason=required_checkpoint_malformed");
    }
    let confirmation = IdentifyCheckpoint {
        schema: CHECKPOINT_SCHEMA.to_owned(),
        observation,
        status: "confirmed".to_owned(),
    };
    let mut bytes = serde_json::to_vec_pretty(&confirmation)?;
    bytes.push(b'\n');
    write_private_new_bytes(&confirmed, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use camino::Utf8PathBuf;

    use super::{
        confirm_identify_observation, consume_confirmation, take_recovery_pause_request,
        write_required_checkpoint, IdentifyObservation,
    };
    use crate::set_private_directory_mode;

    #[test]
    fn confirmation_is_consumed_once() {
        // Arrange
        let temp = tempfile::tempdir().expect("tempdir");
        let root =
            Utf8PathBuf::from_path_buf(temp.path().join("attempt")).expect("utf8 attempt path");
        fs::create_dir(&root).expect("create attempt");
        set_private_directory_mode(&root).expect("private attempt");
        write_required_checkpoint(&root, IdentifyObservation::Rendered)
            .expect("required checkpoint");

        // Act
        confirm_identify_observation(&root, IdentifyObservation::Rendered).expect("confirmation");
        let accepted = consume_confirmation(&root, IdentifyObservation::Rendered)
            .expect("consume confirmation");

        // Assert
        assert!(accepted);
        assert!(!consume_confirmation(&root, IdentifyObservation::Rendered)
            .expect("second observation is absent"));
        assert!(confirm_identify_observation(&root, IdentifyObservation::Rendered).is_err());
    }

    #[test]
    fn malformed_confirmation_fails_closed() {
        // Arrange
        let temp = tempfile::tempdir().expect("tempdir");
        let root =
            Utf8PathBuf::from_path_buf(temp.path().join("attempt")).expect("utf8 attempt path");
        fs::create_dir(&root).expect("create attempt");
        set_private_directory_mode(&root).expect("private attempt");
        write_required_checkpoint(&root, IdentifyObservation::Cleared)
            .expect("required checkpoint");
        let confirmed = root.join("identify-cleared.confirmed.json");
        crate::write_private_new_bytes(
            &confirmed,
            br#"{"schema":"wrong","observation":"cleared","status":"confirmed"}"#,
        )
        .expect("malformed confirmation");

        // Act
        let result = consume_confirmation(&root, IdentifyObservation::Cleared);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn confirmation_without_a_required_checkpoint_is_rejected() {
        // Arrange
        let temp = tempfile::tempdir().expect("tempdir");
        let root =
            Utf8PathBuf::from_path_buf(temp.path().join("attempt")).expect("utf8 attempt path");
        fs::create_dir(&root).expect("create attempt");
        set_private_directory_mode(&root).expect("private attempt");

        // Act
        let result = confirm_identify_observation(&root, IdentifyObservation::Rendered);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn recovery_pause_is_claimed_once_without_replacing_the_primary_failure() {
        // Arrange
        let primary = Some(super::CampaignTerminalCategory::CommandRequestFailed);
        let mut request_count = 0;

        // Act
        let first = take_recovery_pause_request(primary, &mut request_count);
        let second = take_recovery_pause_request(primary, &mut request_count);

        // Assert
        assert!(first);
        assert!(!second);
        assert_eq!(request_count, 1);
        assert_eq!(
            primary,
            Some(super::CampaignTerminalCategory::CommandRequestFailed)
        );
    }
}
