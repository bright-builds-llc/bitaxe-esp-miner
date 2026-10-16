//! What the reference sends once a setting has been handled
//! (`bap_handlers.c:BAP_handle_settings`), given what the firmware's owners did.

use super::semantics::{BapErrorCode, BapRestartPolicy, BapSettingIntent};
use super::wire::{BapCommand, BapFrame, BapFrameError};

/// A planned setting awaiting its effect, with the acknowledgement the planner
/// computed for success.
#[derive(Clone, Debug, PartialEq)]
pub struct BapPendingSetting {
    pub setting: BapSettingIntent,
    pub restart: BapRestartPolicy,
    pub acknowledgement: Vec<BapFrame>,
}

/// What the firmware's settings owner did with one setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapSettingResult {
    /// Stored or applied. For an SSID, says whether a Wi-Fi password is stored,
    /// which decides the reference's restart.
    Applied {
        companion_credential_present: bool,
    },
    /// The stored Wi-Fi credential already had this value, so nothing was written.
    Unchanged,
    Failed,
}

/// The replies to send. When `maybe_restart_status` is present the device
/// restarts: the reference waits 100 ms after the acknowledgement, sends that
/// `STA status restarting` frame, then waits 1,000 ms before `esp_restart`.
#[derive(Clone, Debug, PartialEq)]
pub struct BapSettingCompletion {
    pub acknowledgement: Vec<BapFrame>,
    pub maybe_restart_status: Option<BapFrame>,
}

/// The reference's replies for one handled setting.
pub fn complete_setting(
    pending: &BapPendingSetting,
    result: BapSettingResult,
) -> Result<BapSettingCompletion, BapFrameError> {
    let parameter = pending.setting.parameter();
    let (acknowledgement, restart) = match result {
        BapSettingResult::Failed => {
            let error = BapErrorCode::SetFailed.token().to_owned();
            let frame = BapFrame::new(BapCommand::Error, parameter, Some(error))?;
            return Ok(BapSettingCompletion {
                acknowledgement: vec![frame],
                maybe_restart_status: None,
            });
        }
        BapSettingResult::Unchanged => (unchanged_acknowledgement(pending)?, false),
        BapSettingResult::Applied {
            companion_credential_present,
        } => {
            let restart = match pending.restart {
                BapRestartPolicy::Never => false,
                BapRestartPolicy::Always => true,
                BapRestartPolicy::WhenCompanionCredentialPresent => companion_credential_present,
            };
            (pending.acknowledgement.clone(), restart)
        }
    };
    let maybe_restart_status = restart
        .then(|| BapFrame::new_token(BapCommand::Status, "status", Some("restarting".to_owned())))
        .transpose()?;
    Ok(BapSettingCompletion {
        acknowledgement,
        maybe_restart_status,
    })
}

fn unchanged_acknowledgement(pending: &BapPendingSetting) -> Result<Vec<BapFrame>, BapFrameError> {
    if !matches!(pending.setting, BapSettingIntent::WifiPassword(_)) {
        return Ok(pending.acknowledgement.clone());
    }
    let frame = BapFrame::new(
        BapCommand::Acknowledge,
        pending.setting.parameter(),
        Some("password_already_set".to_owned()),
    )?;
    Ok(vec![frame])
}
