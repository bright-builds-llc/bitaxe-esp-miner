//! Gamma 601 safe boot (ADR-0041, `task-gamma-601-bring-up`).
//!
//! The 601 build boots with the ASIC held in reset and the TPS546 output off, and it never
//! mines. Nothing in this build releases the reset or turns the output on; only the test-only
//! `gamma601-vout-check` variant adds one bounded enable with the reset held (ADR-0041 item 6,
//! `gamma601/vout_check.rs`). A thread keeps the controller off, the fan running (full until the
//! output is confirmed off) and the readings current. Wi-Fi, HTTP and the display's startup screen run as on the Ultra 205; the 205's
//! safety, sensor, fan and mining owners are not started. With no Worker channel, the records
//! reach USB as log lines.
//!
//! Reference: `reference/esp-miner/main/main.c`, `main/asic_reset.c` and `main/power/TPS546.c`.

use crate::safety_adapter::{self, gamma601 as hardware, RuntimeI2cOwner};
use crate::{asic_adapter, bap_adapter, display_adapter, input_adapter, settings_adapter};
use bitaxe_core::StartupDebugText;
use bitaxe_safety::gamma601_safe_boot::{Controller, SafeBootReport};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::{modem::Modem, peripherals::Peripherals};
use esp_idf_svc::sys;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

#[cfg(feature = "gamma601-vout-check")]
mod vout_check;

/// Upstream reads the controller identity up to six times, 3 ms apart.
const IDENTIFY_ATTEMPTS: usize = 6;
const IDENTIFY_BACKOFF_MS: u32 = 3;
const SWEEP_PERIOD: Duration = Duration::from_secs(1);
/// One sweep in this many writes the records to the log. The 601 has no Worker channel, so the
/// USB replay does not run: the log lines are how a receive-only monitor sees them.
const LOG_EVERY_SWEEPS: u32 = 5;
const THREAD_NAME: &str = "gamma601-safe-boot";
#[cfg(not(feature = "gamma601-vout-check"))]
const THREAD_STACK_BYTES: usize = 6 * 1024;
/// The core-voltage check runs on this thread, deeper than the sweeps.
#[cfg(feature = "gamma601-vout-check")]
const THREAD_STACK_BYTES: usize = 12 * 1024;

static REPORT: Mutex<Option<SafeBootReport>> = Mutex::new(None);
static RESET_DRIVEN: AtomicBool = AtomicBool::new(false);

/// Drives the ASIC reset (GPIO1) low as the firmware's first act, before NVS, settings or any
/// peripheral; the HAL never takes the pin over. The level is set before the direction, so the
/// pin never drives high. Input stays enabled so the level can be read back.
pub(crate) fn hold_asic_reset_early() {
    let pin = sys::gpio_num_t_GPIO_NUM_1;
    let driven = unsafe {
        sys::gpio_set_level(pin, 0) == sys::ESP_OK
            && sys::gpio_set_direction(pin, sys::gpio_mode_t_GPIO_MODE_INPUT_OUTPUT) == sys::ESP_OK
    };
    RESET_DRIVEN.store(driven, Ordering::Release);
}

/// Whether the reset pin is driven and reads low.
fn reset_held() -> bool {
    RESET_DRIVEN.load(Ordering::Acquire)
        && unsafe { sys::gpio_get_level(sys::gpio_num_t_GPIO_NUM_1) } == 0
}

/// The latest safe-boot record, for the USB replay.
pub(crate) fn maybe_usb_marker() -> Option<String> {
    REPORT
        .lock()
        .ok()
        .and_then(|report| report.map(|report| report.usb_marker(&wifi_status())))
}

/// Holds the ASIC in reset, turns the TPS546 off and starts the safe-boot thread.
pub(crate) fn initialize_hardware(
    startup_debug_text: StartupDebugText,
    _maybe_thermal_fault_stimulus: Option<settings_adapter::ThermalFaultStimulusAdmission>,
) -> (
    anyhow::Result<asic_adapter::BootMiningBaseline>,
    Option<Modem<'static>>,
) {
    let peripherals = match Peripherals::take() {
        Ok(peripherals) => peripherals,
        Err(error) => {
            log::warn!(
                "gamma601_safe_boot=unavailable reason=peripherals_unavailable error={error}"
            );
            return (
                asic_adapter::run_boot_gate_without_peripherals("peripherals_unavailable"),
                None,
            );
        }
    };
    let pins = peripherals.pins;
    // GPIO1 was driven low by `hold_asic_reset_early`; its HAL token is never used.
    let reset_held = reset_held();
    if !reset_held {
        log::error!("gamma601_safe_boot=fault category=reset_pin");
    }
    if let Err(error) = bap_adapter::start(peripherals.uart2, pins.gpio39, pins.gpio40) {
        log::warn!("bap_status=unavailable reason=initialization_failed error={error:#}");
    }
    let input_available = match input_adapter::start(pins.gpio0) {
        Ok(()) => true,
        Err(error) => {
            log::warn!("input_status=unavailable reason=initialization_failed error={error:#}");
            false
        }
    };
    match safety_adapter::BitaxeI2cBus::new(peripherals.i2c0, pins.gpio47, pins.gpio48) {
        Ok(mut bus) => {
            // The controller goes off before the display or anything else on the bus.
            let startup = turn_off_at_startup(&mut bus);
            render_startup_screen(&mut bus, &startup_debug_text);
            start(bus.into_runtime(), reset_held, startup);
        }
        Err(error) => {
            log::error!("gamma601_safe_boot=fault category=i2c_unavailable error={error:#}");
            let report = SafeBootReport::initial(reset_held);
            log_records(&report);
            publish(report);
        }
    }
    display_adapter::publish_runtime_display_input_boundary(
        display_adapter::RuntimeDisplayMode::Unavailable,
        input_available,
    );
    (
        asic_adapter::run_boot_gate_without_peripherals("board_601_safe_boot"),
        Some(peripherals.modem),
    )
}

/// The 601 starts no safety, sensor, fan or mining owner: the safe-boot thread holds the board.
pub(crate) fn start_runtime_owners() -> anyhow::Result<bool> {
    log::info!("mining_owner=absent reason=board_601_safe_boot");
    Ok(false)
}

/// Shows the startup screen once; the safe boot has no runtime screen flow.
fn render_startup_screen(
    bus: &mut safety_adapter::BitaxeI2cBus<'static>,
    startup_debug_text: &StartupDebugText,
) {
    let now_ms = crate::runtime_uptime::millis();
    let settings = settings_adapter::current_settings_snapshot();
    let rendered = bitaxe_config::load_ultra205_display_configuration(&settings)
        .map_err(|error| anyhow::anyhow!("configuration_invalid category={error}"))
        .and_then(|configuration| {
            display_adapter::RuntimeDisplayOwner::initialize(
                bus,
                &startup_debug_text.frame_at(now_ms),
                configuration,
                now_ms,
            )
        });
    if let Err(error) = rendered {
        log::warn!("display_status=unavailable reason=startup_render_failed error={error:#}");
    }
}

/// What the startup read of the controller found, before any write to it.
struct StartupController {
    controller: Controller,
    maybe_pre_id_status: Option<u16>,
    maybe_pre_id_cml: Option<u8>,
    maybe_status: Option<u16>,
    maybe_cml: Option<u8>,
}

/// Records the controller's status before and after identifying it and, only when it is a
/// TPS546, commands its output off. The first status reads come before the identity read, so
/// the record shows whether a latched fault bit predates every transfer this boot makes; they
/// are reads only, harmless to any device at the address.
fn turn_off_at_startup(bus: &mut safety_adapter::BitaxeI2cBus<'static>) -> StartupController {
    let mut pre_id_budget = hardware::probe_budget();
    let (maybe_pre_id_status, maybe_pre_id_cml) =
        hardware::read_status(&mut bus.tps546(&mut pre_id_budget));
    let controller = identify(|| {
        let mut budget = hardware::budget();
        hardware::identify_controller(&mut bus.tps546(&mut budget))
    });
    let mut found = StartupController {
        controller,
        maybe_pre_id_status,
        maybe_pre_id_cml,
        maybe_status: None,
        maybe_cml: None,
    };
    if let Controller::Found(_) = controller {
        let mut status_budget = hardware::budget();
        (found.maybe_status, found.maybe_cml) =
            hardware::read_status(&mut bus.tps546(&mut status_budget));
        // The off writes get a budget of their own: slow status reads must not spend it.
        let mut off_budget = hardware::budget();
        if let Err(error) = hardware::command_off(&mut bus.tps546(&mut off_budget)) {
            log::error!("gamma601_safe_boot=fault category=off_command error={error:#}");
        }
    }
    found
}

/// Starts the safe-boot thread, which keeps the controller off and the fan running.
fn start(mut owner: RuntimeI2cOwner<'static>, reset_held: bool, startup: StartupController) {
    #[cfg(feature = "gamma601-vout-check")]
    log::info!("{}", vout_check::VARIANT_MARKER);
    let mut report = SafeBootReport::initial(reset_held);
    report.controller = startup.controller;
    report.maybe_pre_id_status = startup.maybe_pre_id_status;
    report.maybe_pre_id_cml = startup.maybe_pre_id_cml;
    report.maybe_start_status = startup.maybe_status;
    report.maybe_start_cml = startup.maybe_cml;
    sweep(&mut owner, &mut report);
    log_records(&report);
    publish(report);
    let spawned = std::thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .stack_size(THREAD_STACK_BYTES)
        .spawn(move || run(owner, report));
    if let Err(error) = spawned {
        log::error!("gamma601_safe_boot=fault category=thread_spawn error={error}");
    }
}

// Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
#[inline(never)]
fn run(mut owner: RuntimeI2cOwner<'static>, mut report: SafeBootReport) -> ! {
    let mut sweeps: u32 = 0;
    #[cfg(feature = "gamma601-vout-check")]
    let mut check = vout_check::VoutCheck::pending();
    loop {
        std::thread::sleep(SWEEP_PERIOD);
        if !matches!(report.controller, Controller::Found(_)) {
            report.controller = identify(|| {
                let mut budget = hardware::budget();
                hardware::identify_controller(&mut owner.tps546(&mut budget))
            });
        }
        // Re-assert off whenever it is not confirmed; the writes can only turn the output off.
        if matches!(report.controller, Controller::Found(_)) && !report.off_confirmed() {
            let mut budget = hardware::budget();
            if let Err(error) = hardware::command_off(&mut owner.tps546(&mut budget)) {
                log::warn!("gamma601_safe_boot=fault category=off_command error={error:#}");
            }
        }
        sweep(&mut owner, &mut report);
        publish(report);
        // The check reads the fresh sweep; the next sweep restores the fan duty.
        #[cfg(feature = "gamma601-vout-check")]
        if check.poll(&mut owner, &report) {
            // The check ended with its own off; the next sweep re-asserts off once more.
            report.maybe_off = None;
        }
        sweeps = sweeps.wrapping_add(1);
        if sweeps % LOG_EVERY_SWEEPS == 0 {
            log_records(&report);
        }
    }
}

fn identify(mut read: impl FnMut() -> Controller) -> Controller {
    let mut controller = Controller::Unread;
    for _ in 0..IDENTIFY_ATTEMPTS {
        controller = read();
        if matches!(controller, Controller::Found(_)) {
            break;
        }
        FreeRtos::delay_ms(IDENTIFY_BACKOFF_MS);
    }
    controller
}

/// Reads everything, then sets the fan: full until the output is confirmed off.
fn sweep(owner: &mut RuntimeI2cOwner<'_>, report: &mut SafeBootReport) {
    if matches!(report.controller, Controller::Found(_)) {
        hardware::read_controller(owner, report);
    }
    report.fan_duty_percent = report.fan_duty_for_state();
    if let Err(error) = hardware::configure_thermal(owner, report.fan_duty_percent) {
        log::warn!("gamma601_safe_boot=fault category=fan_configuration error={error:#}");
    }
    hardware::read_thermal(owner, report);
}

fn log_records(report: &SafeBootReport) {
    log::info!("{}", report.usb_marker(&wifi_status()));
    log::info!("{}", crate::board_identity::marker());
    #[cfg(feature = "gamma601-vout-check")]
    log::info!("{}", vout_check::usb_marker());
}

/// The Wi-Fi owner's status category; the record admits only closed tokens of it.
fn wifi_status() -> String {
    crate::wifi_adapter::current_wifi_snapshot().wifi_status
}

fn publish(report: SafeBootReport) {
    if let Ok(mut slot) = REPORT.lock() {
        *slot = Some(report);
    }
}
