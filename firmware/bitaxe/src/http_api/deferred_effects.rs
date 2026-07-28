use super::deferred_effect_queue::{
    spawn_deferred_effect_worker, DeferredEffectLease, DeferredEffectQueue,
};
use super::*;

static DEFERRED_EFFECT_QUEUE: OnceLock<DeferredEffectQueue<DeferredFirmwareEffect>> =
    OnceLock::new();

enum DeferredFirmwareEffect {
    Settings(Vec<SettingsPersistenceEffect>),
    Restart,
}

pub(super) fn initialize_deferred_effect_worker() -> anyhow::Result<()> {
    if DEFERRED_EFFECT_QUEUE.get().is_some() {
        return Ok(());
    }

    let queue = spawn_deferred_effect_worker(
        DEFERRED_EFFECT_QUEUE_CAPACITY,
        |worker| {
            std::thread::Builder::new()
                .name("deferred-effects".to_owned())
                .stack_size(DEFERRED_EFFECT_THREAD_STACK_BYTES)
                .spawn(worker)
                .map(|_| ())
                .map_err(|_| ())
        },
        execute_deferred_firmware_effect,
    )
    .map_err(|_| anyhow::anyhow!("deferred effect worker spawn failed"))?;

    DEFERRED_EFFECT_QUEUE
        .set(queue)
        .map_err(|_| anyhow::anyhow!("deferred effect worker already initialized"))
}

fn deferred_effect_queue() -> anyhow::Result<&'static DeferredEffectQueue<DeferredFirmwareEffect>> {
    DEFERRED_EFFECT_QUEUE
        .get()
        .ok_or_else(|| anyhow::anyhow!("deferred effect worker unavailable"))
}

pub(super) fn prepare_settings_effects(
    effects: Vec<SettingsPersistenceEffect>,
) -> anyhow::Result<DeferredEffectLease> {
    let effect_lease = deferred_effect_queue()?
        .acquire(DeferredFirmwareEffect::Settings(effects))
        .map_err(|_| anyhow::anyhow!("settings effect worker unavailable"))?;
    settings_patch_retained("axeos_settings_patch=effects_scheduled");
    Ok(effect_lease)
}

fn execute_deferred_firmware_effect(effect: DeferredFirmwareEffect) {
    match effect {
        DeferredFirmwareEffect::Settings(effects) => {
            std::thread::sleep(Duration::from_millis(
                SETTINGS_EFFECTS_POST_RESPONSE_DELAY_MS,
            ));
            apply_settings_effects(&effects);
            settings_patch_retained("axeos_settings_patch=effects_applied");
        }
        DeferredFirmwareEffect::Restart => {
            std::thread::sleep(Duration::from_millis(RESTART_POST_RESPONSE_DELAY_MS));
            log::info!("axeos_command_effect=restart_after_response");
            unsafe { sys::esp_restart() };
        }
    }
}

pub(super) fn apply_settings_effects(effects: &[SettingsPersistenceEffect]) {
    for effect in effects {
        match effect {
            SettingsPersistenceEffect::BestEffortApplyHostname { hostname } => {
                apply_hostname_effect(hostname);
            }
        }
    }
}

pub(super) fn apply_hostname_effect(hostname: &str) {
    const NETIF_KEYS: [&[u8]; 2] = [b"WIFI_STA_DEF\0", b"WIFI_AP_DEF\0"];

    let Ok(hostname_cstr) = CString::new(hostname) else {
        log::warn!("axeos_settings_effect=hostname_failed reason=interior_nul");
        return;
    };

    let mut applied = false;
    for key in NETIF_KEYS {
        let netif = unsafe { sys::esp_netif_get_handle_from_ifkey(key.as_ptr().cast()) };
        if netif.is_null() {
            continue;
        }

        let result = unsafe { sys::esp_netif_set_hostname(netif, hostname_cstr.as_ptr()) };
        if result == sys::ESP_OK {
            applied = true;
            continue;
        }

        log::warn!("axeos_settings_effect=hostname_failed esp_err={result}");
    }

    if applied {
        log::info!("axeos_settings_effect=hostname_applied");
        return;
    }

    log::warn!("axeos_settings_effect=hostname_skipped reason=netif_unavailable");
}

pub(super) fn maybe_prepare_deferred_command_effect(
    effect: &CommandEffect,
) -> anyhow::Result<Option<DeferredEffectLease>> {
    match effect {
        CommandEffect::RestartAfterResponse => prepare_restart_after_response().map(Some),
        _ => Ok(None),
    }
}

pub(super) fn apply_command_effect(
    effect: CommandEffect,
    maybe_deferred_effect: Option<DeferredEffectLease>,
) -> anyhow::Result<()> {
    match effect {
        CommandEffect::MiningOperatorIntent(effect) => {
            apply_mining_operator_intent_command(effect);
            let _ = crate::production_mining_session::notify(
                bitaxe_stratum::v1::production_session::ProductionSessionWakeup::OperatorIntentChanged,
            );
            log::info!(
                "axeos_command_effect=mining_operator_intent next_intent={:?}",
                effect.next_intent
            );
        }
        CommandEffect::RestartAfterResponse => {
            let Some(deferred_effect) = maybe_deferred_effect else {
                return Err(anyhow::anyhow!("restart effect ownership missing"));
            };
            deferred_effect.release_after_response().map_err(|_| {
                anyhow::anyhow!("restart effect worker unavailable after ownership")
            })?;
        }
        CommandEffect::Identify(effect) => match effect {
            IdentifyModeEffect::Enable { duration_ms } => {
                apply_identify_mode_command(effect);
                log::info!("axeos_command_effect=identify_enable duration_ms={duration_ms}");
            }
            IdentifyModeEffect::Disable => {
                apply_identify_mode_command(effect);
                log::info!("axeos_command_effect=identify_disable");
            }
        },
        CommandEffect::BlockFoundDismiss(effect) => {
            apply_block_found_dismiss_command(effect);
            log::info!(
                "axeos_command_effect=block_found_dismiss block_found={} show_new_block={}",
                effect.next_state.block_found,
                effect.next_state.show_new_block
            );
        }
    }

    Ok(())
}

pub(super) fn prepare_restart_after_response() -> anyhow::Result<DeferredEffectLease> {
    // The process-lifetime worker owns the restart before success is serialized.
    // Its delay begins only after the handler schedules the public response.
    deferred_effect_queue()?
        .acquire(DeferredFirmwareEffect::Restart)
        .map_err(|_| anyhow::anyhow!("restart effect worker unavailable"))
}

pub(super) fn record_firmware_ota_status(status: FirmwareOtaStatus) {
    let text = status.status_text();
    log::info!("firmware_ota_status={text}");
    log_buffer::append_runtime_log_line(&format!("firmware_ota_status={text}"));
}

pub(super) fn schedule_firmware_ota_restart() {
    let result = std::thread::Builder::new()
        .name("firmware-ota-restart".to_owned())
        .spawn(|| {
            std::thread::sleep(Duration::from_millis(1000));
            log::info!("firmware_ota_update=restart_now");
            unsafe { sys::esp_restart() };
        });

    if let Err(error) = result {
        log::warn!("firmware_ota_update=restart_thread_failed error={error}");
        unsafe { sys::esp_restart() };
    }
}
