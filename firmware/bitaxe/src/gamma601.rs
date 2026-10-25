//! Gamma 601 safe boot (ADR-0041, `task-gamma-601-bring-up`).
//!
//! The 601 build boots with the ASIC held in reset and the TPS546 output off, and it never
//! mines. Nothing in this build releases the reset or turns the output on; only the test-only
//! variants add bounded checks: one enable with the reset held (`gamma601-vout-check`, ADR-0041
//! item 6), the chip detection (`gamma601-chip-check`, item 8) and the hash check
//! (`gamma601-hash-check`, item 9). A thread keeps the controller off, the fan running (full until the
//! output is confirmed off) and the readings current. Wi-Fi, HTTP and the display's startup
//! screen run as on the Ultra 205; the 205's safety, sensor, fan and mining owners are not
//! started. The records reach a receive-only USB console through the USB replay's board slot
//! (`board_identity::maybe_board_runtime_marker`); their log lines feed the general log only.
//!
//! Reference: `reference/esp-miner/main/main.c`, `main/asic_reset.c` and `main/power/TPS546.c`.

use crate::safety_adapter::gamma601::ThermalAcquisitions;
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

mod actuation;
#[cfg(any(
    feature = "gamma601-vout-check",
    feature = "gamma601-chip-check",
    feature = "gamma601-hash-check"
))]
mod check_guard;
#[cfg(any(feature = "gamma601-chip-check", feature = "gamma601-hash-check"))]
mod check_power;
#[cfg(feature = "gamma601-chip-check")]
mod chip_check;
#[cfg(feature = "gamma601-hash-check")]
mod hash_check;
mod observations;
#[cfg(any(
    feature = "gamma601-vout-check",
    feature = "gamma601-mining",
    feature = "gamma601-chip-check",
    feature = "gamma601-hash-check"
))]
mod regulator;
#[cfg(feature = "gamma601-vout-check")]
mod vout_check;

/// The Gamma 601 test variants are separate builds.
const TEST_VARIANTS_BUILT: usize = cfg!(feature = "gamma601-vout-check") as usize
    + cfg!(feature = "gamma601-mining") as usize
    + cfg!(feature = "gamma601-chip-check") as usize
    + cfg!(feature = "gamma601-hash-check") as usize;
const _: () = assert!(
    TEST_VARIANTS_BUILT <= 1,
    "the Gamma 601 test variants are separate builds"
);

/// The boot line only the test-only mining variant compiles (ADR-0041); the packager and the
/// flash tool check it against the recorded variant.
#[cfg(feature = "gamma601-mining")]
const MINING_VARIANT_MARKER: &str = "firmware_variant=gamma601-mining";

/// How long the boot waits for the safe-boot thread to confirm the output off before it
/// confirms the baseline; its first sweeps run within this.
const BASELINE_OFF_WAIT_MS: u64 = 3_000;
const BASELINE_OFF_POLL_MS: u32 = 50;
/// Upstream reads the controller identity up to six times, 3 ms apart.
const IDENTIFY_ATTEMPTS: usize = 6;
const IDENTIFY_BACKOFF_MS: u32 = 3;
/// Twice per stale window, so the published observations stay fresh between sweeps.
const SWEEP_PERIOD_MS: u64 = 500;
/// One sweep in this many writes the records to the general log, every 5 s. A receive-only
/// console sees them through the USB replay, not these lines.
const LOG_EVERY_SWEEPS: u32 = 10;
const THREAD_NAME: &str = "gamma601-safe-boot";
/// The sweeps publish the shared observations through the same verdict and store as the 205's
/// 8 KiB sensor owner, which the 601 does not start.
#[cfg(not(any(
    feature = "gamma601-vout-check",
    feature = "gamma601-chip-check",
    feature = "gamma601-hash-check"
)))]
const THREAD_STACK_BYTES: usize = 8 * 1024;
/// The core-voltage check runs on this thread, deeper than the sweeps.
#[cfg(feature = "gamma601-vout-check")]
const THREAD_STACK_BYTES: usize = 12 * 1024;
/// The chip check runs on this thread and through the executor, deeper than the sweeps.
#[cfg(feature = "gamma601-chip-check")]
const THREAD_STACK_BYTES: usize = 12 * 1024;
/// The hash check adds the init, the result reads and the hashing that verifies them.
#[cfg(feature = "gamma601-hash-check")]
const THREAD_STACK_BYTES: usize = 16 * 1024;

static REPORT: Mutex<Option<SafeBootReport>> = Mutex::new(None);
/// Set by the safe-boot thread once it runs; the boot's own startup reading is not enough for
/// the Worker baseline.
static SAFE_BOOT_RUNNING: AtomicBool = AtomicBool::new(false);
static RESET_DRIVEN: AtomicBool = AtomicBool::new(false);
#[cfg(any(
    feature = "gamma601-vout-check",
    feature = "gamma601-chip-check",
    feature = "gamma601-hash-check"
))]
static CHECK_REPLAY_TURN: bitaxe_safety::gamma601_vout_check::ReplayTurn =
    bitaxe_safety::gamma601_vout_check::ReplayTurn::new();

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

/// Whether the ASIC is held in reset: by the executor's reset owner once the boot gate retains
/// the pin (output-only, so its level cannot be read back), and before that by the early drive,
/// read back from the pin.
pub(crate) fn reset_held() -> bool {
    if let Some(held) = asic_adapter::production::executor_holds_reset() {
        return held;
    }
    RESET_DRIVEN.load(Ordering::Acquire)
        && unsafe { sys::gpio_get_level(sys::gpio_num_t_GPIO_NUM_1) } == 0
}

/// The latest safe-boot record, for the USB replay. The core-voltage check variant alternates
/// it with the check's record in the same replay slot, so the replay cycle keeps its length.
pub(crate) fn maybe_usb_marker() -> Option<String> {
    #[cfg(feature = "gamma601-vout-check")]
    if CHECK_REPLAY_TURN.next() == bitaxe_safety::gamma601_vout_check::ReplayRecord::Check {
        return Some(vout_check::usb_marker());
    }
    #[cfg(feature = "gamma601-chip-check")]
    if CHECK_REPLAY_TURN.next() == bitaxe_safety::gamma601_vout_check::ReplayRecord::Check {
        return Some(chip_check::usb_marker());
    }
    #[cfg(feature = "gamma601-hash-check")]
    if CHECK_REPLAY_TURN.next() == bitaxe_safety::gamma601_vout_check::ReplayRecord::Check {
        return Some(hash_check::usb_marker());
    }
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
    // GPIO1 was driven low by `hold_asic_reset_early`; the boot gate below hands it, still low,
    // to the executor's reset owner.
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
    let baseline = asic_adapter::retain_board_601_peripherals(
        peripherals.uart1,
        pins.gpio1,
        pins.gpio17,
        pins.gpio18,
    )
    .map(confirm_output_off);
    (baseline, Some(peripherals.modem))
}

/// The 601's baseline also needs the TPS546 output confirmed off, by the startup reading or the
/// first sweeps, with the safe-boot thread running to keep it so. Without both the baseline
/// stays unconfirmed and no Worker starts (ADR-0041 item 10).
fn confirm_output_off(
    baseline: asic_adapter::BootMiningBaseline,
) -> asic_adapter::BootMiningBaseline {
    if baseline != asic_adapter::BootMiningBaseline::Confirmed {
        return baseline;
    }
    let deadline_ms = crate::runtime_uptime::millis().saturating_add(BASELINE_OFF_WAIT_MS);
    loop {
        let off_confirmed = SAFE_BOOT_RUNNING.load(Ordering::Acquire)
            && REPORT
                .lock()
                .ok()
                .and_then(|slot| slot.as_ref().map(SafeBootReport::off_confirmed))
                .unwrap_or(false);
        if off_confirmed {
            return baseline;
        }
        if crate::runtime_uptime::millis() >= deadline_ms {
            log::warn!("gamma601_safe_boot=fault category=baseline_off_unconfirmed");
            return asic_adapter::BootMiningBaseline::Unconfirmed;
        }
        FreeRtos::delay_ms(BASELINE_OFF_POLL_MS);
    }
}

/// Starts the 601's runtime owners: the safety supervisor, the production mining owner and the
/// fan controller (ADR-0041 item 5, amended). Its sensors and actuation stay the safe-boot
/// thread's, so it starts no sensor owner; the 205-only self-test and diagnostic owners never
/// start. Only the mining variant admits dispatch (`MINING_DISPATCH_ADMITTED`); a production 601
/// refuses every run, and keeps its safe boot's fan, before its fan or regulator moves. Never a
/// diagnostic Serial/JTAG owner. A failure is logged and boot continues to HTTP, so the owner can
/// still repair the settings that caused it; the safe-boot thread keeps the board safe meanwhile.
pub(crate) fn start_runtime_owners() -> anyhow::Result<bool> {
    if let Err(error) = start_owners() {
        log::error!("mining_owner=unavailable board=601 error={error:#}");
    }
    Ok(false)
}

fn start_owners() -> anyhow::Result<()> {
    crate::safety_adapter::start_safety_supervisor()?;
    if let Err(decision) = settings_adapter::configured_protocol_plan() {
        return Err(anyhow::anyhow!(
            "production_protocol_owner_unavailable:{}",
            decision.label()
        ));
    }
    crate::production_mining_session::start()?;
    crate::fan_controller_runtime::start()?;
    log::info!(
        "mining_owner=started board=601 dispatch={}",
        if crate::board_identity::MINING_DISPATCH_ADMITTED {
            "admitted"
        } else {
            "deferred"
        }
    );
    Ok(())
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
    #[cfg(feature = "gamma601-mining")]
    log::info!("{MINING_VARIANT_MARKER}");
    #[cfg(feature = "gamma601-chip-check")]
    log::info!("{}", chip_check::VARIANT_MARKER);
    #[cfg(feature = "gamma601-hash-check")]
    log::info!("{}", hash_check::VARIANT_MARKER);
    let mut report = SafeBootReport::initial(reset_held);
    report.controller = startup.controller;
    report.maybe_pre_id_status = startup.maybe_pre_id_status;
    report.maybe_pre_id_cml = startup.maybe_pre_id_cml;
    report.maybe_start_status = startup.maybe_status;
    report.maybe_start_cml = startup.maybe_cml;
    // This thread owns the bus, so it is the safety actuation owner (ADR-0041).
    let (registration, inbox) = safety_adapter::prepare_safety_actuation_owner();
    if let Err(error) = safety_adapter::publish_safety_actuation_owner(registration) {
        log::error!("gamma601_safe_boot=fault category=actuation_owner error={error:#}");
    }
    let mut producer = observations::Producer::new();
    let state = actuation::ActuationState::default();
    refresh(&mut owner, &mut report, &mut producer, &state);
    log_records(&report);
    let spawned = std::thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .stack_size(THREAD_STACK_BYTES)
        .spawn(move || run(owner, report, producer, inbox));
    if let Err(error) = spawned {
        log::error!("gamma601_safe_boot=fault category=thread_spawn error={error}");
    }
}

// Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
#[inline(never)]
fn run(
    mut owner: RuntimeI2cOwner<'static>,
    mut report: SafeBootReport,
    mut producer: observations::Producer,
    inbox: safety_adapter::SafetyActuationOwnerInbox,
) -> ! {
    SAFE_BOOT_RUNNING.store(true, Ordering::Release);
    let mut sweeps: u32 = 0;
    let mut state = actuation::ActuationState::default();
    #[cfg(feature = "gamma601-vout-check")]
    let mut check = vout_check::VoutCheck::pending();
    #[cfg(feature = "gamma601-chip-check")]
    let mut check = chip_check::ChipCheck::pending();
    #[cfg(feature = "gamma601-hash-check")]
    let mut check = hash_check::HashCheck::pending();
    loop {
        let next_sweep_ms = crate::runtime_uptime::millis().saturating_add(SWEEP_PERIOD_MS);
        service_until(
            &mut owner,
            &mut report,
            &mut producer,
            &mut state,
            &inbox,
            next_sweep_ms,
        );
        if !matches!(report.controller, Controller::Found(_)) {
            report.controller = identify(|| {
                let mut budget = hardware::budget();
                hardware::identify_controller(&mut owner.tps546(&mut budget))
            });
        }
        // Re-assert off whenever it is not confirmed, unless an owner turned the output on; the
        // writes can only turn the output off.
        if matches!(report.controller, Controller::Found(_))
            && !report.off_confirmed()
            && !state.regulator_on
        {
            let mut budget = hardware::budget();
            if let Err(error) = hardware::command_off(&mut owner.tps546(&mut budget)) {
                log::warn!("gamma601_safe_boot=fault category=off_command error={error:#}");
            }
        }
        refresh(&mut owner, &mut report, &mut producer, &state);
        // The check reads the fresh sweep; the next sweep restores the fan duty.
        #[cfg(any(
            feature = "gamma601-vout-check",
            feature = "gamma601-chip-check",
            feature = "gamma601-hash-check"
        ))]
        if check.poll(&mut owner, &report) {
            // The check ended with its own off; the next sweep re-asserts off once more.
            report.maybe_off = None;
        }
        sweeps = sweeps.wrapping_add(1);
        if sweeps % LOG_EVERY_SWEEPS == 0 {
            log_records(&report);
            // SAFETY: a null handle names this thread; the call only reads its stack watermark.
            let free_bytes = unsafe { sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
            log::info!("gamma601_safe_boot_stack free_bytes={free_bytes}");
        }
    }
}

/// Applies queued actuation requests until `deadline_ms`. Around a regulator command the thread
/// sweeps and publishes too, so the observations stay fresh for the owner waiting on it.
fn service_until(
    owner: &mut RuntimeI2cOwner<'_>,
    report: &mut SafeBootReport,
    producer: &mut observations::Producer,
    state: &mut actuation::ActuationState,
    inbox: &safety_adapter::SafetyActuationOwnerInbox,
    deadline_ms: u64,
) {
    loop {
        actuation::enforce_authority(owner, state, *report);
        let now_ms = crate::runtime_uptime::millis();
        if now_ms >= deadline_ms {
            return;
        }
        let mut wait_ms = deadline_ms - now_ms;
        if state.regulator_on {
            wait_ms = wait_ms.min(actuation::ON_AUTHORITY_CHECK_MS);
        }
        let waited = safety_adapter::service_next_board_actuation_request(
            inbox,
            Duration::from_millis(wait_ms),
            |command| {
                let regulator = actuation::drives_regulator(command);
                if regulator {
                    refresh(owner, report, producer, state);
                }
                let reply = actuation::apply(owner, state, *report, command);
                if regulator {
                    actuation::enforce_authority(owner, state, *report);
                    refresh(owner, report, producer, state);
                }
                reply
            },
        );
        if waited == safety_adapter::SafetyActuationOwnerWait::Disconnected {
            std::thread::sleep(Duration::from_millis(
                deadline_ms.saturating_sub(crate::runtime_uptime::millis()),
            ));
            return;
        }
    }
}

/// One sweep, published as the observations and the safe-boot record.
fn refresh(
    owner: &mut RuntimeI2cOwner<'_>,
    report: &mut SafeBootReport,
    producer: &mut observations::Producer,
    state: &actuation::ActuationState,
) {
    let thermal = sweep(owner, report, state);
    producer.publish(report, thermal, state.output_verified_on());
    publish(*report);
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

/// Reads everything, then sets the fan: full until the output is confirmed off, unless an owner
/// turned it on; otherwise the owner's duty, or the safe boot's.
fn sweep(
    owner: &mut RuntimeI2cOwner<'_>,
    report: &mut SafeBootReport,
    state: &actuation::ActuationState,
) -> ThermalAcquisitions {
    if matches!(report.controller, Controller::Found(_)) {
        hardware::read_controller(owner, report);
    }
    report.fan_duty_percent = state.fan_duty(report);
    if let Err(error) = hardware::configure_thermal(owner, report.fan_duty_percent) {
        log::warn!("gamma601_safe_boot=fault category=fan_configuration error={error:#}");
    }
    hardware::read_thermal(owner, report)
}

fn log_records(report: &SafeBootReport) {
    log::info!("{}", report.usb_marker(&wifi_status()));
    log::info!("{}", crate::board_identity::marker());
    #[cfg(feature = "gamma601-vout-check")]
    log::info!("{}", vout_check::usb_marker());
    #[cfg(feature = "gamma601-chip-check")]
    log::info!("{}", chip_check::usb_marker());
    #[cfg(feature = "gamma601-hash-check")]
    log::info!("{}", hash_check::usb_marker());
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
