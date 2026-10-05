//! Task-gated loop that measures whether the native USB link survives repeated
//! USB-core resets of the running application.
//!
//! Each cycle repeats the native Serial/JTAG reset used to start an installed
//! application after flashing, then requires the exact application identity on
//! the receive-only observer. The loop never flashes, erases, writes NVS, sends
//! Worker records or touches the network, and it stops at the first cycle in
//! which the application is not observed because that state needs a replug.

use std::fs;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;

mod identity;
mod model;
mod runner;
mod session;

pub use model::{
    admit_reset_endurance_task, CycleRow, FirstFailure, ResetEnduranceConfig,
    ResetEnduranceProjection, ResetEnduranceStop, RESET_ENDURANCE_PRIVATE_SCHEMA,
    RESET_ENDURANCE_PROJECTION_SCHEMA,
};

use crate::evidence::write_json_new;
use crate::{UsbOperation, UsbSession};
use model::{CycleRun, FinalCleanup, ResetEndurancePrivateResult};
use session::{admit_espflash_version, create_supervisor_root, EspflashTool, SessionEffects};

/// Fully validated inputs for one supervised endurance run.
#[derive(Debug, Clone)]
pub struct ResetEnduranceRequest {
    pub config: ResetEnduranceConfig,
    pub port: String,
    pub workspace: Utf8PathBuf,
    pub private_root: Utf8PathBuf,
    pub projection_output: Utf8PathBuf,
}

/// Runs the loop after the task gate admits it, then writes private and redacted evidence.
pub fn run_usb_reset_endurance(request: ResetEnduranceRequest) -> Result<ResetEnduranceProjection> {
    let tasks = fs::read_to_string(request.workspace.join("TASKS.md").as_std_path())
        .context("usb_reset_endurance=blocked reason=tasks_unreadable")?;
    // The gate runs before any evidence root, tool resolution or device discovery.
    admit_reset_endurance_task(&tasks)?;
    if fs::symlink_metadata(request.projection_output.as_std_path()).is_ok() {
        bail!("usb_reset_endurance=blocked reason=projection_output_exists");
    }
    let espflash = EspflashTool::resolve()?;
    create_supervisor_root(&request.private_root)?;
    let started = Instant::now();
    let (run, cleanup) = supervise(&request, &espflash)?;
    let total_elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let private =
        ResetEndurancePrivateResult::build(&request.config, &run, &cleanup, total_elapsed_ms);
    write_json_new(&request.private_root.join("result.private.json"), &private)?;
    let projection =
        ResetEnduranceProjection::build(&request.config, &run, &cleanup, total_elapsed_ms);
    write_json_new(&request.projection_output, &projection)?;
    Ok(projection)
}

fn supervise(
    request: &ResetEnduranceRequest,
    espflash: &EspflashTool,
) -> Result<(CycleRun, FinalCleanup)> {
    let trace_root = request.private_root.join("usb-session");
    let mut session = match UsbSession::acquire(
        UsbOperation::ResetEndurance,
        &request.port,
        trace_root.as_std_path(),
    ) {
        Ok(session) => session,
        Err(error) => {
            let cleanup = FinalCleanup {
                proven: false,
                maybe_detail: Some("session_not_acquired".to_owned()),
            };
            return Ok((CycleRun::admission_failure(error.to_string()), cleanup));
        }
    };
    if let Err(error) = admit_espflash_version(&mut session, espflash) {
        let cleanup = finish(session);
        return Ok((CycleRun::admission_failure(error.to_string()), cleanup));
    }
    let mut effects = SessionEffects::new(
        session,
        espflash,
        request.config.expected_physical_sha256(),
        &request.private_root,
    )?;
    let result = runner::run_cycles(&request.config, &mut effects);
    // Final cleanup always runs and never replaces the loop's earliest failure.
    let cleanup = finish(effects.into_session());
    Ok((result?, cleanup))
}

fn finish(session: UsbSession) -> FinalCleanup {
    match session.finish() {
        Ok(_) => FinalCleanup {
            proven: true,
            maybe_detail: None,
        },
        Err(error) => FinalCleanup {
            proven: false,
            maybe_detail: Some(error.to_string()),
        },
    }
}

#[cfg(test)]
mod tests;
