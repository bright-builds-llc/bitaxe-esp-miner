use bitaxe_core::{AsicTarget, BoardTarget, Phase1SafeState, StartupDebugText};
use esp_idf_svc::hal::{modem::Modem, peripherals::Peripherals};
use esp_idf_svc::sys;

use crate::{
    asic_adapter, bap_adapter, boot_evidence, boot_validation, display_adapter,
    fan_controller_runtime, filesystem, http_api, input_adapter, operator_sensor_runtime,
    production_mining_session, runtime_snapshot, runtime_uptime, safety_adapter,
    scoreboard_adapter, self_test_runtime, settings_adapter, statistics_runtime,
    stratum_v2_noise_diagnostic, stratum_v2_session, stratum_v2_tcp_payload_diagnostic,
    wifi_adapter, BOOT_LOG_LINE, RUST_TARGET, SAFE_STATE_LOG_LINE,
};

pub(crate) struct BootMiningBaselineConfirmed(());

/// Starts firmware services while preserving evidence-before-network ordering.
///
/// Platform readiness must remain before Wi-Fi admission because ESP-IDF's
/// blocking Wi-Fi start can wait indefinitely for driver events. The HTTP route
/// shell can safely start first because it performs separate, idempotent
/// network-stack initialization.
pub(crate) fn run() -> anyhow::Result<()> {
    let (startup_debug_text, maybe_thermal_fault_stimulus) =
        initialize_boot_identity_and_settings()?;
    let (startup_diagnostics, maybe_modem) =
        initialize_hardware(startup_debug_text, maybe_thermal_fault_stimulus);
    let boot_validation_ready = start_runtime_services(startup_diagnostics)?;
    let (filesystem_status, route_shell_ready) = start_storage_and_http();
    publish_platform_readiness(boot_validation_ready, filesystem_status, route_shell_ready);
    start_network_services(maybe_modem);
    wifi_adapter::maybe_start_network_reconnect_probe(route_shell_ready);
    Ok(())
}

fn initialize_boot_identity_and_settings() -> anyhow::Result<(
    StartupDebugText,
    Option<settings_adapter::ThermalFaultStimulusAdmission>,
)> {
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
    settings_adapter::initialize_default_nvs_partition()?;
    if let Err(error) = settings_adapter::initialize_current_settings_snapshot() {
        log::warn!("axeos_settings_snapshot=startup_refresh_failed error={error}");
    }
    match settings_adapter::maybe_self_test_receipt() {
        Ok(Some((lease, receipt))) => {
            log::info!(
                "self_test_receipt outcome={} lease={lease:016x}",
                receipt.token()
            );
            boot_evidence::register_self_test_receipt(lease, receipt.token());
        }
        Ok(None) => {}
        Err(error) => log::warn!(
            "self_test_receipt=unavailable category={}",
            error.category()
        ),
    }
    match settings_adapter::current_ultra205_defaults_attestation() {
        Ok(attestation) => crate::info_retained(
            &attestation.retained_marker(!settings_adapter::start_mining_on_boot()),
        ),
        Err(error) => {
            log::warn!("ultra205_config_defaults=unavailable reason=settings_read error={error}")
        }
    }
    if let Err(error) = scoreboard_adapter::initialize() {
        log::warn!("scoreboard=unavailable category={}", error.category());
    }
    runtime_snapshot::apply_boot_mining_preference(settings_adapter::start_mining_on_boot());
    let maybe_thermal_fault_stimulus = match settings_adapter::load_thermal_fault_stimulus() {
        Ok(maybe_admission) => maybe_admission,
        Err(error) => {
            log::warn!(
                "thermal_fault_stimulus=unavailable reason={}",
                error.category()
            );
            None
        }
    };
    Ok((
        StartupDebugText::new(
            BoardTarget::Ultra205,
            AsicTarget::Bm1366,
            Some(crate::build_label()),
            crate::build_timestamp_utc(),
        ),
        maybe_thermal_fault_stimulus,
    ))
}

fn initialize_hardware(
    startup_debug_text: StartupDebugText,
    maybe_thermal_fault_stimulus: Option<settings_adapter::ThermalFaultStimulusAdmission>,
) -> (
    anyhow::Result<asic_adapter::BootMiningBaseline>,
    Option<Modem<'static>>,
) {
    let peripherals = match Peripherals::take() {
        Ok(peripherals) => peripherals,
        Err(error) => {
            log::warn!("display_status=unavailable reason=peripherals_unavailable error={error}");
            display_adapter::publish_runtime_display_input_boundary(
                display_adapter::RuntimeDisplayMode::Unavailable,
                false,
            );
            return (
                asic_adapter::run_boot_gate_without_peripherals("peripherals_unavailable"),
                None,
            );
        }
    };
    let modem = peripherals.modem;
    let pins = peripherals.pins;
    if let Err(error) = bap_adapter::start(peripherals.uart2, pins.gpio39, pins.gpio40) {
        log::warn!("bap_status=unavailable reason=initialization_failed error={error:#}");
    }
    let boot_peripherals = asic_adapter::AsicBootPeripherals {
        uart: peripherals.uart1,
        reset: pins.gpio1,
        enable: pins.gpio10,
        tx: pins.gpio17,
        rx: pins.gpio18,
    };
    let input_available = match input_adapter::start(pins.gpio0) {
        Ok(()) => true,
        Err(error) => {
            log::warn!("input_status=unavailable reason=initialization_failed error={error:#}");
            false
        }
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
                None
            }
        };
    let display_mode = initialize_operator_runtime(
        maybe_i2c_bus,
        maybe_core_voltage_adc,
        startup_debug_text,
        maybe_thermal_fault_stimulus,
    );
    display_adapter::publish_runtime_display_input_boundary(display_mode, input_available);
    (
        asic_adapter::run_boot_gate_with_peripherals(boot_peripherals),
        Some(modem),
    )
}

fn initialize_operator_runtime(
    mut maybe_bus: Option<safety_adapter::BitaxeI2cBus<'static>>,
    maybe_core_voltage_adc: Option<safety_adapter::Ultra205CoreVoltageAdc>,
    startup_debug_text: StartupDebugText,
    maybe_thermal_fault_stimulus: Option<settings_adapter::ThermalFaultStimulusAdmission>,
) -> display_adapter::RuntimeDisplayMode {
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
        maybe_thermal_fault_stimulus,
    ) {
        log::warn!(
            "operator_sensor_runtime=unavailable reason=thread_spawn_failed error={error:#}"
        );
        return display_adapter::RuntimeDisplayMode::Unavailable;
    }
    if runtime_display_enabled {
        display_adapter::RuntimeDisplayMode::ScreenFlow
    } else {
        display_adapter::RuntimeDisplayMode::Unavailable
    }
}

fn start_runtime_services(
    startup_diagnostics: anyhow::Result<asic_adapter::BootMiningBaseline>,
) -> anyhow::Result<bool> {
    let startup_diagnostics_passed = startup_diagnostics.is_ok();
    let boot_validation_ready = match boot_validation::validate_boot(startup_diagnostics_passed) {
        Ok(ready) => ready,
        Err(error) => {
            log::warn!("ota_boot_validation=error error={error:#}");
            false
        }
    };
    let boot_mining_baseline = startup_diagnostics?;
    let maybe_bwg_recovery = match boot_mining_baseline {
        asic_adapter::BootMiningBaseline::Confirmed => Some(
            crate::bwg_worker_usb::recover_interrupted_effect(BootMiningBaselineConfirmed(()))?,
        ),
        asic_adapter::BootMiningBaseline::Unconfirmed => None,
    };
    safety_adapter::start_safety_supervisor();
    let maybe_tcp_payload_admission =
        match settings_adapter::load_tcp_payload_diagnostic_admission() {
            Ok(maybe_admission) => maybe_admission,
            Err(error) => {
                log::warn!(
                    "stratum_v2_tcp_payload_admission=rejected category={}",
                    error.category()
                );
                None
            }
        };
    let maybe_noise_diagnostic_admission = if maybe_tcp_payload_admission.is_some() {
        None
    } else {
        match settings_adapter::load_noise_diagnostic_admission() {
            Ok(maybe_admission) => maybe_admission,
            Err(error) => {
                log::warn!(
                    "stratum_v2_noise_admission=rejected category={}",
                    error.category()
                );
                None
            }
        }
    };
    let maybe_self_test_admission = if maybe_noise_diagnostic_admission.is_some() {
        None
    } else {
        match settings_adapter::load_self_test_admission() {
            Ok(maybe_admission) => maybe_admission,
            Err(error) => {
                log::warn!("self_test_admission=rejected category={}", error.category());
                None
            }
        }
    };
    let serial_jtag_runtime = maybe_tcp_payload_admission.is_some()
        || maybe_noise_diagnostic_admission.is_some()
        || maybe_self_test_admission.is_some();
    if let Some(admission) = maybe_tcp_payload_admission {
        if let Err(error) = stratum_v2_tcp_payload_diagnostic::start(admission) {
            log::warn!(
                "stratum_v2_tcp_payload_diagnostic=unavailable reason=thread_spawn_failed error={error:#}"
            );
        }
    } else if let Some(admission) = maybe_noise_diagnostic_admission {
        if let Err(error) = stratum_v2_noise_diagnostic::start(admission) {
            log::warn!(
                "stratum_v2_noise_diagnostic=unavailable reason=thread_spawn_failed error={error:#}"
            );
        }
    } else if let Some(admission) = maybe_self_test_admission {
        if let Err(error) = self_test_runtime::start(admission) {
            log::warn!("self_test_runtime=unavailable reason=thread_spawn_failed error={error:#}");
        }
    } else {
        match settings_adapter::configured_protocol_plan() {
            Ok(plan) if plan.initial() == settings_adapter::ConfiguredStratumProtocol::V2 => {
                if let Err(error) = stratum_v2_session::start() {
                    log::warn!(
                        "stratum_v2_session=unavailable reason=thread_spawn_failed error={error:#}"
                    );
                }
            }
            Ok(_) => {
                if let Err(error) = production_mining_session::start() {
                    log::warn!(
                        "production_mining_session=unavailable reason=thread_spawn_failed error={error:#}"
                    );
                }
            }
            Err(decision) => {
                log::warn!(
                    "production_protocol_owner=unavailable category={}",
                    decision.label()
                );
            }
        }
        if let Err(error) = fan_controller_runtime::start() {
            log::warn!("fan_controller=unavailable reason=thread_spawn_failed error={error:#}");
        }
    }
    if let Err(error) = statistics_runtime::start() {
        log::warn!("statistics_runtime=unavailable reason=thread_spawn_failed error={error:#}");
    }
    if serial_jtag_runtime {
        log::info!("usb_runtime=serial_jtag reason=diagnostic_owner");
    } else if let Some(bwg_recovery) = maybe_bwg_recovery {
        if let Err(error) = crate::bwg_worker_usb::start(bwg_recovery) {
            log::warn!("bwg_worker_control=unavailable category=startup_failed error={error:#}");
        }
    } else {
        log::warn!("bwg_worker_control=unavailable category=boot_baseline_unconfirmed");
    }
    Ok(boot_validation_ready)
}

fn start_network_services(maybe_modem: Option<Modem<'static>>) {
    if let Some(modem) = maybe_modem {
        if let Err(error) = wifi_adapter::start_wifi(modem) {
            log::warn!("wifi_status=unavailable error={error:#}");
        }
    } else {
        log::warn!("wifi_status=unavailable reason=peripherals_unavailable");
    }
    let _ = production_mining_session::notify(
        bitaxe_stratum::v1::production_session::ProductionSessionWakeup::NetworkChanged,
    );
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
    let platform_snapshot = runtime_snapshot::collect_platform_readiness_snapshot();
    crate::info_retained(&format!("reset_reason={}", platform_snapshot.reset_reason));
    crate::info_retained(&format!(
        "partition={}",
        platform_snapshot.running_partition
    ));
    let psram_status = if platform_snapshot.psram_available {
        "available"
    } else {
        "unavailable"
    };
    crate::info_retained(&format!("psram_status={psram_status}"));
    crate::info_retained(&format!(
        "esp_idf_version={}",
        platform_snapshot.idf_version
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
            &platform_snapshot.idf_version,
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
