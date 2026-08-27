//! Private boot-time Stratum V2 Noise diagnostic owner.

use std::thread;
use std::time::{Duration, Instant};

use crate::settings_adapter::NoiseDiagnosticAdmission;
use crate::stratum_v2_session::transport::{
    run_noise_diagnostic, NoiseDiagnosticFailure, NoiseDiagnosticStage,
};

const OWNER_THREAD_NAME: &str = "stratum-v2-noise-diag";
const OWNER_STACK_BYTES: usize = 24 * 1_024;
const WIFI_TIMEOUT: Duration = Duration::from_secs(60);
const WIFI_POLL: Duration = Duration::from_millis(100);
const MONITOR_ARM_DELAY: Duration = Duration::from_secs(10);

pub(crate) fn start(admission: NoiseDiagnosticAdmission) -> anyhow::Result<()> {
    thread::Builder::new()
        .name(OWNER_THREAD_NAME.to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(move || run(admission))?;
    Ok(())
}

fn run(admission: NoiseDiagnosticAdmission) {
    if admission.lease() == 0 {
        publish_terminal("state", false);
        return;
    }
    if !wait_for_wifi() {
        publish_terminal("connect", false);
        return;
    }
    thread::sleep(MONITOR_ARM_DELAY);
    crate::info_retained(
        "stratum_v2_noise_diagnostic={\"schema\":\"bitaxe-stratum-v2-noise-diagnostic-v1\",\"stage\":\"monitor_armed\"}",
    );
    let settings = match exact_primary_settings() {
        Ok(settings) => settings,
        Err(failure) => {
            publish_terminal(failure.label(), false);
            return;
        }
    };
    match run_noise_diagnostic(settings, publish_stage) {
        Ok(()) => publish_terminal("accepted", true),
        Err(failure) => publish_terminal(failure.label(), false),
    }
}

fn exact_primary_settings(
) -> Result<crate::settings_adapter::V2PoolSettings, NoiseDiagnosticFailure> {
    let pool_set = crate::settings_adapter::read_stratum_v2_pool_set()
        .map_err(|_| NoiseDiagnosticFailure::Configure)?
        .ok_or(NoiseDiagnosticFailure::Configure)?;
    if pool_set.prefer_fallback || pool_set.fallback.is_some() {
        return Err(NoiseDiagnosticFailure::Configure);
    }
    pool_set.primary.ok_or(NoiseDiagnosticFailure::Configure)
}

fn wait_for_wifi() -> bool {
    let deadline = Instant::now() + WIFI_TIMEOUT;
    while Instant::now() < deadline {
        if crate::wifi_adapter::current_wifi_snapshot().wifi_status == "connected" {
            return true;
        }
        thread::sleep(WIFI_POLL);
    }
    false
}

fn publish_stage(stage: NoiseDiagnosticStage) {
    crate::info_retained(&format!(
        "stratum_v2_noise_diagnostic={{\"schema\":\"bitaxe-stratum-v2-noise-diagnostic-v1\",\"stage\":\"{}\"}}",
        stage.label()
    ));
}

fn publish_terminal(category: &'static str, accepted: bool) {
    crate::info_retained(&format!(
        "stratum_v2_noise_terminal={{\"schema\":\"bitaxe-stratum-v2-noise-terminal-v1\",\"category\":\"{category}\",\"accepted\":{accepted},\"lease_present\":true,\"mining_started\":false,\"asic_touched\":false,\"fan_touched\":false,\"voltage_touched\":false}}"
    ));
}
