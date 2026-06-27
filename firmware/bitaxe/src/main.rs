use std::ffi::CStr;

use bitaxe_core::{AsicTarget, BoardTarget, Phase1SafeState, StartupDebugText};
use esp_idf_svc::{hal::peripherals::Peripherals, sys};

mod asic_adapter;
mod display_adapter;
mod http_api;
mod runtime_snapshot;

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

    let safe_state = Phase1SafeState::default();

    let boot_log_line = format!(
        "bitaxe-rust boot: board={} asic={}",
        BoardTarget::Ultra205.display_name(),
        AsicTarget::Bm1366.display_name()
    );
    debug_assert_eq!(boot_log_line, BOOT_LOG_LINE);

    let safe_state_log_line = safe_state.log_line();
    debug_assert_eq!(safe_state_log_line, SAFE_STATE_LOG_LINE);

    log::info!("{boot_log_line}");
    log::info!("{safe_state_log_line}");
    let startup_debug_text = StartupDebugText::new(
        BoardTarget::Ultra205,
        AsicTarget::Bm1366,
        safe_state,
        Some(firmware_commit()),
    );
    match Peripherals::take() {
        Ok(peripherals) => {
            let pins = peripherals.pins;
            if let Err(error) = display_adapter::render_startup_debug_text(
                peripherals.i2c0,
                pins.gpio47,
                pins.gpio48,
                &startup_debug_text,
            ) {
                log::warn!(
                    "display_status=unavailable reason=startup_text_render_failed error={error:#}"
                );
            }
            asic_adapter::run_boot_gate_with_peripherals(asic_adapter::AsicBootPeripherals {
                uart: peripherals.uart1,
                reset: pins.gpio1,
                tx: pins.gpio17,
                rx: pins.gpio18,
            })?;
        }
        Err(error) => {
            log::warn!("display_status=unavailable reason=peripherals_unavailable error={error}");
            asic_adapter::run_boot_gate_without_peripherals("peripherals_unavailable")?;
        }
    }
    asic_adapter::publish_mining_loop_blocked_status("hardware_evidence_ack_missing");
    if let Err(error) = http_api::start_http_api() {
        log::warn!("axeos_api_route_shell=unavailable error={error:#}");
    }
    log::info!("reset_reason={}", reset_reason());
    log::info!("partition={}", partition_label());
    log::info!("psram_status={}", psram_status());
    log::info!("firmware_commit={}", firmware_commit());
    debug_assert_eq!(
        REFERENCE_COMMIT_LOG_LINE,
        format!("reference_commit={REFERENCE_COMMIT}")
    );

    log::info!("{REFERENCE_COMMIT_LOG_LINE}");
    log::info!("esp_idf_version={ESP_IDF_VERSION}");
    log::info!("rust_target={RUST_TARGET}");

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
