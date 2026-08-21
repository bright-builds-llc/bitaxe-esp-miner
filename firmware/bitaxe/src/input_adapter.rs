//! Retained active-low Ultra 205 boot-button owner.

use std::{
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
    thread,
    time::Duration,
};

use bitaxe_core::input::{
    route_button_event, ButtonEvent, ButtonInput, ButtonLevel, ButtonRoute, ButtonSelfTestState,
    BUTTON_SAMPLE_MS,
};
use esp_idf_svc::hal::gpio::{InputPin, PinDriver, Pull};

const INPUT_THREAD_NAME: &str = "boot-button";
const INPUT_THREAD_STACK_BYTES: usize = 4 * 1024;
const MAX_PENDING_SCREEN_ADVANCES: u8 = 12;

static PENDING_SCREEN_ADVANCES: AtomicU8 = AtomicU8::new(0);
static INPUT_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Starts the sole GPIO0 input owner with an internal pull-up.
pub fn start(pin: impl InputPin + 'static) -> anyhow::Result<()> {
    let driver = PinDriver::input(pin, Pull::Up)?;
    INPUT_AVAILABLE.store(true, Ordering::Release);
    if let Err(error) = thread::Builder::new()
        .name(INPUT_THREAD_NAME.to_owned())
        .stack_size(INPUT_THREAD_STACK_BYTES)
        .spawn(move || run(driver))
    {
        INPUT_AVAILABLE.store(false, Ordering::Release);
        return Err(anyhow::anyhow!(
            "failed to start boot-button owner: {error}"
        ));
    }
    log::info!(
        "input_status=active owner=boot_button sampling_ms={BUTTON_SAMPLE_MS} active_low=true"
    );
    Ok(())
}

/// Returns whether the retained input owner is currently running.
pub fn is_available() -> bool {
    INPUT_AVAILABLE.load(Ordering::Acquire)
}

/// Takes the bounded number of screen advances admitted since the last frame.
pub fn take_pending_screen_advances() -> u8 {
    PENDING_SCREEN_ADVANCES.swap(0, Ordering::AcqRel)
}

fn run(driver: PinDriver<'static, esp_idf_svc::hal::gpio::Input>) {
    let started_at_ms = crate::runtime_uptime::millis();
    let mut input = ButtonInput::new(started_at_ms, sampled_level(&driver));
    loop {
        thread::sleep(Duration::from_millis(BUTTON_SAMPLE_MS));
        let now_ms = crate::runtime_uptime::millis();
        match input.update(now_ms, sampled_level(&driver)) {
            Ok(Some(event)) => apply_event(event, now_ms),
            Ok(None) => {}
            Err(error) => {
                INPUT_AVAILABLE.store(false, Ordering::Release);
                log::warn!("input_status=disabled reason=classifier_failed category={error}");
                return;
            }
        }
    }
}

fn sampled_level(driver: &PinDriver<'_, esp_idf_svc::hal::gpio::Input>) -> ButtonLevel {
    if driver.is_low() {
        ButtonLevel::PressedLow
    } else {
        ButtonLevel::ReleasedHigh
    }
}

fn apply_event(event: ButtonEvent, now_ms: u64) {
    let identify_active = if event == ButtonEvent::ShortClick {
        match crate::runtime_snapshot::cancel_identify_if_active_at(now_ms) {
            crate::runtime_snapshot::ButtonIdentifyCancellation::Cancelled => true,
            crate::runtime_snapshot::ButtonIdentifyCancellation::Inactive => false,
            crate::runtime_snapshot::ButtonIdentifyCancellation::StateUnavailable => {
                log::warn!("input_event=short_click effect=unavailable category=runtime_state");
                return;
            }
        }
    } else {
        false
    };
    let self_test_state = if crate::self_test_runtime::is_active() {
        ButtonSelfTestState::Active
    } else {
        ButtonSelfTestState::Inactive
    };
    let route = route_button_event(event, self_test_state, identify_active);
    match route {
        ButtonRoute::AdvanceScreen => {
            enqueue_screen_advance();
            log::info!("input_event=short_click effect=screen_advance");
        }
        ButtonRoute::CancelIdentify => {
            log::info!("input_event=short_click effect=identify_cancelled");
        }
        ButtonRoute::ToggleConfigurationAp => {
            match crate::wifi_adapter::toggle_configuration_ap() {
                Ok(enabled) => log::info!(
                    "input_event=long_press effect=configuration_ap_toggle ap_enabled={enabled}"
                ),
                Err(error) => log::warn!(
                "input_event=long_press effect=configuration_ap_toggle status=failed category={}",
                error.category()
            ),
            }
        }
        ButtonRoute::ResetSelfTest => {
            if crate::self_test_runtime::request_cancel() {
                log::info!("input_event=long_press effect=self_test_cancel_requested");
            } else {
                log::warn!("input_event=long_press effect=self_test_cancel status=not_ready");
            }
        }
        ButtonRoute::SelfTestResetUnavailable => {
            log::warn!("input_event=long_press effect=self_test_reset_unavailable");
        }
        ButtonRoute::IgnoreShortDuringSelfTest => {
            log::info!("input_event=short_click effect=ignored_self_test");
        }
    }
}

fn enqueue_screen_advance() {
    PENDING_SCREEN_ADVANCES
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |pending| {
            Some(pending.saturating_add(1).min(MAX_PENDING_SCREEN_ADVANCES))
        })
        .expect("screen-advance update closure always returns a value");
}
