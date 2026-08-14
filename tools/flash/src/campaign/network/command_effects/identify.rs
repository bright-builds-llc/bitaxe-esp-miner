use std::fs;
use std::time::Instant;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use camino::Utf8Path;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::write_private_new_bytes;

const CHECKPOINT_SCHEMA: &str = "bitaxe-identify-checkpoint-v3";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentifyCheckpointKind {
    Ready,
    Rendered,
    Replayed,
    Cleared,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentifyCheckpointOutcome {
    Confirmed,
    Declined,
    Replay,
}

impl IdentifyCheckpointOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Declined => "declined",
            Self::Replay => "replay",
        }
    }
}

impl IdentifyCheckpointKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Rendered => "rendered",
            Self::Replayed => "replayed",
            Self::Cleared => "cleared",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IdentifyCheckpoint {
    schema: String,
    checkpoint: IdentifyCheckpointKind,
    status: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CheckpointResponse {
    Pending,
    Confirmed,
    Declined,
    Replay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RenderedCheckpointAction {
    Wait,
    Confirmed,
    Declined,
    ReplayAt(Instant),
    Expired,
}

pub(super) fn rendered_checkpoint_action(
    now: Instant,
    confirmation_expires_at: Instant,
    replay_not_before: Instant,
    response: CheckpointResponse,
    replay_allowed: bool,
) -> Result<RenderedCheckpointAction, ()> {
    match response {
        CheckpointResponse::Pending => Ok(RenderedCheckpointAction::Wait),
        CheckpointResponse::Confirmed if now < confirmation_expires_at => {
            Ok(RenderedCheckpointAction::Confirmed)
        }
        CheckpointResponse::Confirmed => Ok(RenderedCheckpointAction::Expired),
        CheckpointResponse::Declined => Ok(RenderedCheckpointAction::Declined),
        CheckpointResponse::Replay if replay_allowed => {
            let starts_at = if now < replay_not_before {
                replay_not_before
            } else {
                now
            };
            Ok(RenderedCheckpointAction::ReplayAt(starts_at))
        }
        CheckpointResponse::Replay => Err(()),
    }
}

fn checkpoint_path(
    root: &Utf8Path,
    checkpoint: IdentifyCheckpointKind,
    state: &str,
) -> camino::Utf8PathBuf {
    root.join(format!("identify-{}.{}.json", checkpoint.as_str(), state))
}

pub(super) fn write_required_checkpoint(
    root: &Utf8Path,
    checkpoint: IdentifyCheckpointKind,
) -> anyhow::Result<()> {
    let document = IdentifyCheckpoint {
        schema: CHECKPOINT_SCHEMA.to_owned(),
        checkpoint,
        status: "required".to_owned(),
    };
    let mut bytes = serde_json::to_vec_pretty(&document)?;
    bytes.push(b'\n');
    write_private_new_bytes(&checkpoint_path(root, checkpoint, "required"), &bytes)
}

pub(super) fn consume_checkpoint_response(
    root: &Utf8Path,
    checkpoint: IdentifyCheckpointKind,
) -> Result<CheckpointResponse, ()> {
    let response = checkpoint_path(root, checkpoint, "response");
    let consumed = checkpoint_path(root, checkpoint, "consumed");
    let metadata = match fs::symlink_metadata(response.as_std_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CheckpointResponse::Pending)
        }
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
        || fs::rename(response.as_std_path(), consumed.as_std_path()).is_err()
    {
        return Err(());
    }
    let bytes = fs::read(consumed.as_std_path()).map_err(|_| ())?;
    let document: IdentifyCheckpoint = serde_json::from_slice(&bytes).map_err(|_| ())?;
    if document.schema != CHECKPOINT_SCHEMA || document.checkpoint != checkpoint {
        return Err(());
    }
    match document.status.as_str() {
        "confirmed" => Ok(CheckpointResponse::Confirmed),
        "declined" => Ok(CheckpointResponse::Declined),
        "replay" => Ok(CheckpointResponse::Replay),
        _ => Err(()),
    }
}

pub(crate) fn respond_identify_checkpoint(
    root: &Utf8Path,
    checkpoint: IdentifyCheckpointKind,
    outcome: IdentifyCheckpointOutcome,
) -> anyhow::Result<()> {
    if outcome == IdentifyCheckpointOutcome::Replay
        && checkpoint != IdentifyCheckpointKind::Rendered
    {
        anyhow::bail!("identify_checkpoint=blocked reason=replay_checkpoint_invalid");
    }
    let required = checkpoint_path(root, checkpoint, "required");
    let response = checkpoint_path(root, checkpoint, "response");
    let consumed = checkpoint_path(root, checkpoint, "consumed");
    let metadata = fs::symlink_metadata(required.as_std_path())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        anyhow::bail!("identify_checkpoint=blocked reason=required_checkpoint_invalid");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        anyhow::bail!("identify_checkpoint=blocked reason=required_checkpoint_not_private");
    }
    if response.exists() || consumed.exists() {
        anyhow::bail!("identify_checkpoint=blocked reason=checkpoint_already_used");
    }
    let required_checkpoint: IdentifyCheckpoint =
        serde_json::from_slice(&fs::read(required.as_std_path())?)?;
    if required_checkpoint.schema != CHECKPOINT_SCHEMA
        || required_checkpoint.checkpoint != checkpoint
        || required_checkpoint.status != "required"
    {
        anyhow::bail!("identify_checkpoint=blocked reason=required_checkpoint_malformed");
    }
    let response_document = IdentifyCheckpoint {
        schema: CHECKPOINT_SCHEMA.to_owned(),
        checkpoint,
        status: outcome.as_str().to_owned(),
    };
    let mut bytes = serde_json::to_vec_pretty(&response_document)?;
    bytes.push(b'\n');
    write_private_new_bytes(&response, &bytes)?;
    Ok(())
}
