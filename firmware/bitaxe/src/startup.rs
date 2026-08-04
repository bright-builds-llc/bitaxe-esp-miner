use bitaxe_core::{AsicTarget, BoardTarget, Phase1SafeState, StartupDebugText};
use esp_idf_svc::hal::{modem::Modem, peripherals::Peripherals};
use esp_idf_svc::sys;

use crate::{
    asic_adapter, boot_evidence, boot_validation, display_adapter, filesystem, http_api,
    operator_sensor_runtime, production_mining_session, runtime_snapshot, runtime_uptime,
    safety_adapter, scoreboard_adapter, settings_adapter, statistics_runtime, wifi_adapter,
    BOOT_LOG_LINE, RUST_TARGET, SAFE_STATE_LOG_LINE,
};

pub(crate) fn run() -> anyhow::Result<()> {
    let startup_debug_text = initialize_boot_identity_and_settings();
    let (startup_diagnostics, maybe_modem) = initialize_hardware(startup_debug_text);
    let boot_validation_ready = start_runtime_services(startup_diagnostics, maybe_modem)?;
    let (filesystem_status, route_shell_ready) = start_storage_and_http();
    publish_platform_readiness(boot_validation_ready, filesystem_status, route_shell_ready);
    Ok(())
}

fn initialize_boot_identity_and_settings() -> StartupDebugText {
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

    crate::info_retained(&boot_log_line);
    boot_evidence::record_booted();
    crate::info_retained(&safe_state_log_line);
    crate::retain_build_identity();
    if let Err(error) = settings_adapter::initialize_current_settings_snapshot() {
        log::warn!("axeos_settings_snapshot=startup_refresh_failed error={error}");
    }
    if let Err(error) = scoreboard_adapter::initialize() {
        log::warn!("scoreboard=unavailable category={}", error.category());
    }
    runtime_snapshot::apply_boot_mining_preference(settings_adapter::start_mining_on_boot());
    StartupDebugText::new(
        BoardTarget::Ultra205,
        AsicTarget::Bm1366,
        Some(crate::build_label()),
        crate::build_timestamp_utc(),
    )
}

fn initialize_hardware(
    startup_debug_text: StartupDebugText,
) -> (anyhow::Result<()>, Option<Modem<'static>>) {
    let peripherals = match Peripherals::take() {
        Ok(peripherals) => peripherals,
        Err(error) => {
            log::warn!("display_status=unavailable reason=peripherals_unavailable error={error}");
            display_adapter::publish_runtime_display_input_boundary(
                display_adapter::RuntimeDisplayMode::Unavailable,
            );
            return (
                asic_adapter::run_boot_gate_without_peripherals("peripherals_unavailable"),
                None,
            );
        }
    };
    let modem = peripherals.modem;
    let pins = peripherals.pins;
    let boot_peripherals = asic_adapter::AsicBootPeripherals {
        uart: peripherals.uart1,
        reset: pins.gpio1,
        enable: pins.gpio10,
        tx: pins.gpio17,
        rx: pins.gpio18,
    };
    let maybe_core_voltage_adc =
        match safety_adapter::Ultra205CoreVoltageAdc::new(peripherals.adc1, pins.gpio2) {
            Ok(adc) => {
                log::info!("core_voltage_adc=available calibration=curve");
                Some(adc)
            }
            Err(error) => {
                log::warn!(
                    "core_voltage_adc=unavailable reason=initialization_failed error={error:#}"
                );
                None
            }
        };
    let maybe_i2c_bus =
        match safety_adapter::BitaxeI2cBus::new(peripherals.i2c0, pins.gpio47, pins.gpio48) {
            Ok(bus) => Some(bus),
            Err(error) => {
                log::warn!("display_status=unavailable reason=i2c0_init_failed error={error:#}");
                display_adapter::publish_runtime_display_input_boundary(
                    display_adapter::RuntimeDisplayMode::Unavailable,
                );
                None
            }
        };
    initialize_operator_runtime(maybe_i2c_bus, maybe_core_voltage_adc, startup_debug_text);
    (
        asic_adapter::run_boot_gate_with_peripherals(boot_peripherals),
        Some(modem),
    )
}

fn initialize_operator_runtime(
    mut maybe_bus: Option<safety_adapter::BitaxeI2cBus<'static>>,
    maybe_core_voltage_adc: Option<safety_adapter::Ultra205CoreVoltageAdc>,
    startup_debug_text: StartupDebugText,
) {
    let display_started_at_ms = runtime_uptime::millis();
    let startup_frame = startup_debug_text.frame_at(display_started_at_ms);
    let confirmed_settings = settings_adapter::current_settings_snapshot();
    let display_configuration =
        bitaxe_config::load_ultra205_display_configuration(&confirmed_settings);
    let maybe_runtime_display = if let (Some(bus), Ok(configuration)) =
        (maybe_bus.as_mut(), display_configuration.as_ref())
    {
        match display_adapter::RuntimeDisplayOwner::initialize(
            bus,
            &startup_frame,
            *configuration,
            display_started_at_ms,
        ) {
            Ok(owner) => {
                log::info!("operator_sensor_display=rendered");
                Some(owner)
            }
            Err(error) => {
                log::warn!(
                    "display_status=unavailable reason=startup_text_render_failed error={error:#}"
                );
                None
            }
        }
    } else {
        if let Err(error) = display_configuration {
            log::warn!("display_status=unavailable reason=configuration_invalid category={error}");
        }
        None
    };
    let runtime_display_enabled = maybe_runtime_display.is_some();
    let maybe_runtime_owner = maybe_bus.map(safety_adapter::BitaxeI2cBus::into_runtime);
    if let Err(error) = operator_sensor_runtime::start(
        maybe_runtime_owner,
        maybe_core_voltage_adc,
        maybe_runtime_display,
    ) {
        log::warn!(
            "operator_sensor_runtime=unavailable reason=thread_spawn_failed error={error:#}"
        );
        display_adapter::publish_runtime_display_input_boundary(
            display_adapter::RuntimeDisplayMode::Unavailable,
        );
        return;
    }
    let mode = if runtime_display_enabled {
        display_adapter::RuntimeDisplayMode::ScreenFlow
    } else {
        display_adapter::RuntimeDisplayMode::Unavailable
    };
    display_adapter::publish_runtime_display_input_boundary(mode);
}

fn start_runtime_services(
    startup_diagnostics: anyhow::Result<()>,
    maybe_modem: Option<Modem<'static>>,
) -> anyhow::Result<bool> {
    let startup_diagnostics_passed = startup_diagnostics.is_ok();
    let boot_validation_ready = match boot_validation::validate_boot(startup_diagnostics_passed) {
        Ok(()) => true,
        Err(error) => {
            log::warn!("ota_boot_validation=error error={error:#}");
            false
        }
    };
    startup_diagnostics?;
    safety_adapter::start_safety_supervisor();
    if let Err(error) = production_mining_session::start() {
        log::warn!(
            "production_mining_session=unavailable reason=thread_spawn_failed error={error:#}"
        );
    }
    if let Err(error) = statistics_runtime::start() {
        log::warn!("statistics_runtime=unavailable reason=thread_spawn_failed error={error:#}");
    }
    let _network_ready = if let Some(modem) = maybe_modem {
        match wifi_adapter::start_wifi(modem) {
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
    let _ = production_mining_session::notify(
        bitaxe_stratum::v1::production_session::ProductionSessionWakeup::NetworkChanged,
    );
    Ok(boot_validation_ready)
}

fn start_storage_and_http() -> (filesystem::FilesystemStatus, bool) {
    let filesystem_status = filesystem::mount_www_spiffs();
    let route_shell_ready = match http_api::start_http_api(filesystem_status) {
        Ok(()) => true,
        Err(error) => {
            log::warn!("axeos_api_route_shell=unavailable error={error:#}");
            false
        }
    };
    (filesystem_status, route_shell_ready)
}

fn publish_platform_readiness(
    boot_validation_ready: bool,
    filesystem_status: filesystem::FilesystemStatus,
    route_shell_ready: bool,
) {
    let platform_snapshot = runtime_snapshot::collect_api_snapshot();
    crate::info_retained(&format!(
        "reset_reason={}",
        platform_snapshot.platform.reset_reason
    ));
    crate::info_retained(&format!(
        "partition={}",
        platform_snapshot.platform.running_partition
    ));
    let psram_status = if platform_snapshot.platform.psram_available {
        "available"
    } else {
        "unavailable"
    };
    crate::info_retained(&format!("psram_status={psram_status}"));
    crate::info_retained(&format!(
        "esp_idf_version={}",
        platform_snapshot.platform.idf_version
    ));
    crate::info_retained(&format!("rust_target={RUST_TARGET}"));
    let spiffs_ready = matches!(
        filesystem_status,
        filesystem::FilesystemStatus::Available { .. }
    );
    if boot_validation_ready && spiffs_ready && route_shell_ready {
        boot_evidence::publish_runtime_boot_attestation(
            crate::firmware_commit(),
            crate::reference_commit(),
            &crate::app_elf_sha256(),
            &platform_snapshot.platform.idf_version,
        );
        return;
    }
    log::warn!(
        "runtime_boot_attestation=deferred reason=readiness_incomplete ota_boot_validation={} spiffs_mount={} api_route_shell={}",
        readiness_label(boot_validation_ready),
        readiness_label(spiffs_ready),
        readiness_label(route_shell_ready),
    );
}

const fn readiness_label(ready: bool) -> &'static str {
    if ready {
        "complete"
    } else {
        "incomplete"
    }
}
