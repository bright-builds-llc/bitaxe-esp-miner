use esp_idf_svc::sys;

mod asic_adapter;
mod boot_evidence;
mod boot_validation;
mod boot_validation_plan;
mod display_adapter;
mod fan_controller_plan;
mod fan_controller_runtime;
mod filesystem;
mod http_api;
mod input_adapter;
mod log_buffer;
mod mining_actuation;
mod mining_actuation_adapter;
mod network_stack;
mod operator_sensor_runtime;
mod operator_snapshot_publication;
mod operator_snapshot_retention;
mod ota_update;
mod platform_identity;
mod production_mining_session;
mod rtc_boot_ordinal;
mod runtime_health_adapter;
mod runtime_snapshot;
mod runtime_uptime;
mod safety_adapter;
mod scoreboard_adapter;
mod settings_adapter;
mod settings_snapshot_store;
mod startup;
mod static_files;
mod statistics_runtime;
mod task_watchdog_observation;
mod websocket_api;
mod wifi_adapter;

const BOOT_LOG_LINE: &str = "bitaxe-rust boot: board=Ultra 205 asic=BM1366";
const RUST_TARGET: &str = "xtensa-esp32s3-espidf";
const SAFE_STATE_LOG_LINE: &str =
    "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";

fn main() -> anyhow::Result<()> {
    startup::run()
}

fn firmware_commit() -> &'static str {
    env!("BITAXE_FIRMWARE_COMMIT")
}

fn semantic_version() -> &'static str {
    env!("BITAXE_SEMANTIC_VERSION")
}

fn build_label() -> &'static str {
    env!("BITAXE_BUILD_LABEL")
}

fn build_channel() -> &'static str {
    env!("BITAXE_BUILD_CHANNEL")
}

fn build_timestamp_utc() -> &'static str {
    env!("BITAXE_BUILD_TIMESTAMP_UTC")
}

fn source_dirty() -> bool {
    env!("BITAXE_SOURCE_DIRTY") == "true"
}

fn maybe_release_tag() -> Option<&'static str> {
    let release_tag = env!("BITAXE_RELEASE_TAG");
    (release_tag != "unavailable").then_some(release_tag)
}

fn reference_commit() -> &'static str {
    env!("BITAXE_REFERENCE_COMMIT")
}

fn app_elf_sha256() -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let maybe_description = unsafe { sys::esp_app_get_description().as_ref() };
    let Some(description) = maybe_description else {
        return "unavailable".to_owned();
    };
    let mut digest = String::with_capacity(description.app_elf_sha256.len() * 2);
    for byte in description.app_elf_sha256 {
        digest.push(char::from(HEX[usize::from(byte >> 4)]));
        digest.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    digest
}

fn retain_build_identity() {
    info_retained(&format!("firmware_commit={}", firmware_commit()));
    info_retained(&format!("reference_commit={}", reference_commit()));
    info_retained(&format!("app_elf_sha256={}", app_elf_sha256()));
    info_retained(&format!(
        "firmware_build_timestamp_utc={}",
        build_timestamp_utc()
    ));
    info_retained(env!("BITAXE_RUNTIME_BUILD_IDENTITY"));
}

fn info_retained(line: &str) {
    log::info!("{line}");
    log_buffer::append_runtime_log_line(line);
}
