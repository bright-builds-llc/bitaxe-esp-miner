use std::ffi::CStr;

use bitaxe_core::{AsicTarget, BoardTarget, Phase1SafeState, StartupDebugText};
use esp_idf_svc::{hal::peripherals::Peripherals, sys};

mod asic_adapter;
mod boot_evidence;
mod boot_validation;
mod controlled_mining_runtime;
mod display_adapter;
mod filesystem;
mod http_api;
mod live_stratum_runtime;
mod log_buffer;
mod mining_evidence_mode;
mod network_stack;
mod ota_update;
mod runtime_snapshot;
mod runtime_uptime;
mod safety_adapter;
mod settings_adapter;
mod static_files;
mod websocket_api;
mod wifi_adapter;

const BOOT_LOG_LINE: &str = "bitaxe-rust boot: board=Ultra 205 asic=BM1366";
const ESP_IDF_VERSION: &str = "v5.5.4";
const REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const REFERENCE_COMMIT_LOG_LINE: &str = "reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50";
const RUST_TARGET: &str = "xtensa-esp32s3-espidf";
const SAFE_STATE_LOG_LINE: &str =
    "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";
const UNAVAILABLE: &str = "Unavailable";

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    boot_evidence::initialize_observer();

    let safe_state = Phase1SafeState::default();

    let boot_log_line = format!(
        "bitaxe-rust boot: board={} asic={}",
        BoardTarget::Ultra205.display_name(),
        AsicTarget::Bm1366.display_name()
    );
    debug_assert_eq!(boot_log_line, BOOT_LOG_LINE);

    let safe_state_log_line = safe_state.log_line();
    debug_assert_eq!(safe_state_log_line, SAFE_STATE_LOG_LINE);

    info_retained(&boot_log_line);
    boot_evidence::record_booted();
    info_retained(&safe_state_log_line);
    if let Err(error) = settings_adapter::initialize_current_settings_snapshot() {
        log::warn!("axeos_settings_snapshot=startup_refresh_failed error={error}");
    }
    let startup_debug_text = StartupDebugText::new(
        BoardTarget::Ultra205,
        AsicTarget::Bm1366,
        safe_state,
        Some(firmware_commit()),
    );
    let is_phase27_bridge =
        mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge();
    let (startup_diagnostics, maybe_modem) = match Peripherals::take() {
        Ok(peripherals) => {
            let modem = peripherals.modem;
            let pins = peripherals.pins;
            display_adapter::publish_runtime_display_input_boundary();
            let boot_peripherals = asic_adapter::AsicBootPeripherals {
                uart: peripherals.uart1,
                reset: pins.gpio1,
                tx: pins.gpio17,
                rx: pins.gpio18,
            };
            let startup_diagnostics = if is_phase27_bridge {
                log::warn!("display_status=deferred reason=phase27_safety_i2c0_in_use");
                asic_adapter::run_phase27_boot_gate_with_safety(
                    boot_peripherals,
                    asic_adapter::Phase27SafetyPeripherals {
                        i2c: peripherals.i2c0,
                        sda: pins.gpio47,
                        scl: pins.gpio48,
                        enable: pins.gpio10,
                    },
                )
            } else {
                match safety_adapter::BitaxeI2cBus::new(peripherals.i2c0, pins.gpio47, pins.gpio48)
                {
                    Ok(mut bus) => {
                        if let Err(error) = display_adapter::render_startup_debug_text(
                            &mut bus,
                            &startup_debug_text,
                        ) {
                            log::warn!(
                                "display_status=unavailable reason=startup_text_render_failed error={error:#}"
                            );
                        }
                    }
                    Err(error) => {
                        log::warn!(
                            "display_status=unavailable reason=i2c0_init_failed error={error:#}"
                        );
                    }
                }
                asic_adapter::run_boot_gate_with_peripherals(boot_peripherals)
            };
            (startup_diagnostics, Some(modem))
        }
        Err(error) => {
            log::warn!("display_status=unavailable reason=peripherals_unavailable error={error}");
            display_adapter::publish_runtime_display_input_boundary();
            (
                asic_adapter::run_boot_gate_without_peripherals("peripherals_unavailable"),
                None,
            )
        }
    };
    let startup_diagnostics_passed = startup_diagnostics.is_ok();
    if let Err(error) = boot_validation::validate_boot(startup_diagnostics_passed) {
        log::warn!("ota_boot_validation=error error={error:#}");
    }
    startup_diagnostics?;
    controlled_mining_runtime::maybe_start_after_asic_gate();
    safety_adapter::start_safety_supervisor();
    let network_ready = if let Some(modem) = maybe_modem {
        match wifi_adapter::start_wifi_sta(modem) {
            Ok(()) => true,
            Err(error) => {
                log::warn!("wifi_status=unavailable error={error:#}");
                false
            }
        }
    } else {
        log::warn!("wifi_status=unavailable reason=peripherals_unavailable");
        false
    };
    if !mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge() {
        live_stratum_runtime::maybe_start_after_network_setup(network_ready);
    }
    let filesystem_status = filesystem::mount_www_spiffs();
    if let Err(error) = http_api::start_http_api(filesystem_status) {
        log::warn!("axeos_api_route_shell=unavailable error={error:#}");
    }
    if mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge() {
        live_stratum_runtime::schedule_phase27_bridge_after_http_ready(network_ready);
    }
    info_retained(&format!("reset_reason={}", reset_reason()));
    info_retained(&format!("partition={}", partition_label()));
    info_retained(&format!("psram_status={}", psram_status()));
    info_retained(&format!("firmware_commit={}", firmware_commit()));
    debug_assert_eq!(
        REFERENCE_COMMIT_LOG_LINE,
        format!("reference_commit={REFERENCE_COMMIT}")
    );

    info_retained(REFERENCE_COMMIT_LOG_LINE);
    info_retained(&format!("esp_idf_version={ESP_IDF_VERSION}"));
    info_retained(&format!("rust_target={RUST_TARGET}"));

    Ok(())
}

fn reset_reason() -> i32 {
    // ESP-IDF owns the reset register interpretation at this boundary.
    unsafe { sys::esp_reset_reason() as i32 }
}

fn partition_label() -> String {
    let Some(label) = maybe_partition_label() else {
        return UNAVAILABLE.to_owned();
    };

    label
}

fn maybe_partition_label() -> Option<String> {
    // ESP-IDF returns a static partition pointer for the running image.
    let maybe_partition = unsafe { sys::esp_ota_get_running_partition() };
    if maybe_partition.is_null() {
        return None;
    }

    // The partition label is a null-terminated field owned by ESP-IDF.
    let label = unsafe { CStr::from_ptr((*maybe_partition).label.as_ptr()) };
    Some(label.to_string_lossy().into_owned())
}

fn psram_status() -> &'static str {
    // ESP-IDF heap capabilities expose whether external memory is present.
    let psram_bytes = unsafe { sys::heap_caps_get_total_size(sys::MALLOC_CAP_SPIRAM) };
    if psram_bytes > 0 {
        return "available";
    }

    "unavailable"
}

fn firmware_commit() -> &'static str {
    option_env!("BITAXE_FIRMWARE_COMMIT").unwrap_or(UNAVAILABLE)
}

fn info_retained(line: &str) {
    log::info!("{line}");
    log_buffer::append_runtime_log_line(line);
}
