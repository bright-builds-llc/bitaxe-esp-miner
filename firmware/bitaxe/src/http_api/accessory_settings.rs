//! BAP setting effects, run on the HTTP server task through `httpd_queue_work`.
//! That task already persists settings PATCHes on its audited 16 KiB stack, so
//! the BAP owner, with less than 1 KiB of headroom, only queues a job and waits.
//! Values never reach a log: only setting categories and outcomes.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use bitaxe_runtime::reply::{reply, ReplySender};

use bitaxe_api::bap_settings::{bap_block_found_state, bap_settings_patch};
use bitaxe_api::plan_settings_patch_value;
use bitaxe_config::{reload_snapshot, LoadedValue};
use bitaxe_core::bap::{BapSettingIntent, BapSettingResult};

use super::*;

/// How long the BAP owner waits for a queued job to start; a PATCH completes in
/// well under a second. A job that has not started by then is abandoned and
/// never runs, so `set_failed` is final. A job that has started is awaited for
/// up to `ACCESSORY_JOB_RUNNING_TIMEOUT` more.
const ACCESSORY_JOB_TIMEOUT: Duration = Duration::from_secs(10);
const ACCESSORY_JOB_RUNNING_TIMEOUT: Duration = Duration::from_secs(30);
const JOB_QUEUED: u8 = 0;
const JOB_RUNNING: u8 = 1;
const JOB_ABANDONED: u8 = 2;

static ACCESSORY_SERVER: OnceLock<AccessoryServer> = OnceLock::new();

struct AccessoryServer(sys::httpd_handle_t);
// SAFETY: an `httpd_handle_t` is an opaque handle that ESP-IDF documents as
// usable from any task for `httpd_queue_work`. The server lives for the process.
unsafe impl Send for AccessoryServer {}
unsafe impl Sync for AccessoryServer {}

enum AccessoryRequest {
    Setting(BapSettingIntent),
    Restart,
}

struct AccessoryJob {
    request: AccessoryRequest,
    claim: Arc<AtomicU8>,
    reply: ReplySender<BapSettingResult>,
}

/// Records the running server so BAP jobs can be queued on its task.
pub(super) fn register_accessory_server(server: sys::httpd_handle_t) {
    if ACCESSORY_SERVER.set(AccessoryServer(server)).is_err() {
        log::warn!("bap_setting_owner=already_registered");
    }
}

/// Applies one BAP setting through the HTTP settings and command owners.
pub(crate) fn apply_accessory_setting(setting: BapSettingIntent) -> BapSettingResult {
    run_on_http_task(AccessoryRequest::Setting(setting))
}

/// Schedules the reference's post-credential restart through the HTTP restart
/// owner, which waits 1,000 ms before `esp_restart`. Returns whether it was
/// scheduled.
pub(crate) fn restart_for_accessory() -> bool {
    run_on_http_task(AccessoryRequest::Restart) != BapSettingResult::Failed
}

#[inline(never)]
fn run_on_http_task(request: AccessoryRequest) -> BapSettingResult {
    let Some(server) = ACCESSORY_SERVER.get() else {
        log::warn!("bap_setting=failed category=http_owner_unavailable");
        return BapSettingResult::Failed;
    };
    let (reply, receiver) = reply();
    let claim = Arc::new(AtomicU8::new(JOB_QUEUED));
    let job = Box::into_raw(Box::new(AccessoryJob {
        request,
        claim: Arc::clone(&claim),
        reply,
    }));
    // SAFETY: the server handle is live for the process; ownership of `job`
    // passes to `run_accessory_job` when queuing succeeds.
    let result = unsafe { sys::httpd_queue_work(server.0, Some(run_accessory_job), job.cast()) };
    if result != sys::ESP_OK {
        // SAFETY: the job was not queued, so this thread still owns it.
        drop(unsafe { Box::from_raw(job) });
        log::warn!("bap_setting=failed category=queue_unavailable error={result}");
        return BapSettingResult::Failed;
    }
    if let Ok(result) = receiver.recv_timeout(ACCESSORY_JOB_TIMEOUT) {
        return result;
    }
    let abandoned = claim
        .compare_exchange(
            JOB_QUEUED,
            JOB_ABANDONED,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok();
    if abandoned {
        log::warn!("bap_setting=failed category=owner_timeout");
        return BapSettingResult::Failed;
    }
    receiver
        .recv_timeout(ACCESSORY_JOB_RUNNING_TIMEOUT)
        .unwrap_or_else(|_| {
            log::warn!("bap_setting=failed category=owner_running_timeout");
            BapSettingResult::Failed
        })
}

/// # Safety
/// `argument` must be the `AccessoryJob` boxed by `run_on_http_task`, passed once.
unsafe extern "C" fn run_accessory_job(argument: *mut c_void) {
    if argument.is_null() {
        return;
    }
    // SAFETY: the caller contract above.
    let job = unsafe { Box::from_raw(argument.cast::<AccessoryJob>()) };
    let claimed = job
        .claim
        .compare_exchange(JOB_QUEUED, JOB_RUNNING, Ordering::AcqRel, Ordering::Acquire)
        .is_ok();
    if !claimed {
        // The BAP owner already answered `set_failed`; applying now would contradict it.
        return;
    }
    let result = match &job.request {
        AccessoryRequest::Setting(setting) => apply_setting(setting),
        AccessoryRequest::Restart => schedule_restart(),
    };
    // Only the BAP owner waits for this; it may have stopped waiting.
    let _ = job.reply.send(result);
}

fn apply_setting(setting: &BapSettingIntent) -> BapSettingResult {
    if let Some(result) = maybe_unchanged_credential(setting) {
        return result;
    }
    if let Some(patch) = bap_settings_patch(setting) {
        let companion_credential_present = matches!(setting, BapSettingIntent::WifiSsid(_))
            && !stored_string("wifipass").is_empty();
        return if persist(&patch) {
            BapSettingResult::Applied {
                companion_credential_present,
            }
        } else {
            BapSettingResult::Failed
        };
    }
    let Some(next_state) = bap_block_found_state(setting, block_found_notification_state()) else {
        log::warn!("bap_setting=failed category=found_block_negative");
        return BapSettingResult::Failed;
    };
    apply_block_found_dismiss_command(bitaxe_api::BlockFoundDismissEffect { next_state });
    BapSettingResult::Applied {
        companion_credential_present: false,
    }
}

/// The reference acknowledges an unchanged SSID or password without writing.
fn maybe_unchanged_credential(setting: &BapSettingIntent) -> Option<BapSettingResult> {
    let (key, value) = match setting {
        BapSettingIntent::WifiSsid(value) => ("wifissid", value),
        BapSettingIntent::WifiPassword(value) => ("wifipass", value),
        _ => return None,
    };
    (stored_string(key) == *value).then_some(BapSettingResult::Unchanged)
}

fn stored_string(key: &str) -> String {
    let loaded = reload_snapshot(&settings_adapter::current_settings_snapshot());
    match loaded.maybe_loaded_value(key) {
        Some(LoadedValue::Str(value)) => value.clone(),
        _ => String::new(),
    }
}

fn persist(patch: &serde_json::Value) -> bool {
    let Ok(accepted) = plan_settings_patch_value(patch) else {
        log::warn!("bap_setting=failed category=patch_rejected");
        return false;
    };
    let plan = SettingsPersistencePlan::from_accepted(&accepted);
    if plan.is_empty() {
        // Every BAP patch names a known setting, so an empty plan is a mapping fault.
        log::warn!("bap_setting=failed category=patch_empty");
        return false;
    }
    let Ok(mut adapter) = settings_adapter::FirmwareSettingsAdapter::open() else {
        log::warn!("bap_setting=failed category=adapter_open");
        return false;
    };
    let success = match execute_settings_persistence_plan(&plan, &mut adapter) {
        Ok(success) => success,
        Err(error) => {
            log::warn!(
                "bap_setting=failed category=persistence reason={}",
                settings_persistence_failure_label(error.reason())
            );
            return false;
        }
    };
    let maybe_effect_lease =
        success.maybe_acquire_best_effort_effect_lease(prepare_settings_effects);
    if maybe_effect_lease.is_some_and(|lease| lease.release_after_response().is_err()) {
        log::warn!("bap_setting=effects_degraded category=worker_disconnected");
    }
    true
}

fn schedule_restart() -> BapSettingResult {
    let plan = restart_plan();
    let maybe_deferred = match maybe_prepare_deferred_command_effect(&plan.effect) {
        Ok(maybe_deferred) => maybe_deferred,
        Err(error) => {
            log::warn!("bap_restart=failed error={error:#}");
            return BapSettingResult::Failed;
        }
    };
    if let Err(error) = apply_command_effect(plan.effect, maybe_deferred) {
        log::warn!("bap_restart=failed error={error:#}");
        return BapSettingResult::Failed;
    }
    log::info!("bap_restart=scheduled");
    BapSettingResult::Applied {
        companion_credential_present: false,
    }
}
