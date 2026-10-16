//! Single-owner ESP-IDF adapter for the Bitaxe Accessory Port.

use std::thread;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context};
use bitaxe_api::SystemInfoWire;
use bitaxe_core::bap::{
    complete_setting, BapCommand, BapConnectionMode, BapErrorCode, BapFrame, BapParameter,
    BapPendingSetting, BapRequestSnapshot, BapSettingIntent,
};
use esp_idf_svc::hal::{
    delay::TickType,
    gpio::{InputPin, OutputPin},
    uart::{config, Uart, UartDriver},
    units::Hertz,
};

use crate::bap_runtime::{BapFrameAccumulator, BapLastValues, BapRuntime, BapRuntimeAction};

pub const BAP_UART_BAUD: u32 = 115_200;
pub const BAP_UART_TX_PIN: i32 = 39;
pub const BAP_UART_RX_PIN: i32 = 40;
const BAP_UART_BUFFER_BYTES: usize = 1_024;
const BAP_READ_TIMEOUT_MS: u32 = 100;
const BAP_OWNER_STACK_BYTES: usize = 16 * 1_024;
/// The reference's `vTaskDelay(100)` between a credential acknowledgement and
/// `STA status restarting`.
const BAP_RESTART_STATUS_DELAY_MS: u64 = 100;
const BAP_INITIALIZATION_BANNER: &[u8] = b"BAP UART Interface Initialized\r\n";

pub fn start<UART, TX, RX>(uart: UART, tx: TX, rx: RX) -> anyhow::Result<()>
where
    UART: Uart + 'static,
    TX: OutputPin + 'static,
    RX: InputPin + 'static,
{
    debug_assert_eq!(BAP_UART_TX_PIN, 39);
    debug_assert_eq!(BAP_UART_RX_PIN, 40);
    let configuration = config::Config::new()
        .baudrate(Hertz(BAP_UART_BAUD))
        .data_bits(config::DataBits::DataBits8)
        .parity_none()
        .stop_bits(config::StopBits::STOP1)
        .flow_control(config::FlowControl::None)
        .rx_fifo_size(BAP_UART_BUFFER_BYTES);
    let driver = UartDriver::new(
        uart,
        tx,
        rx,
        Option::<RX>::None,
        Option::<TX>::None,
        &configuration,
    )?;
    thread::Builder::new()
        .name("bap-owner".to_owned())
        .stack_size(BAP_OWNER_STACK_BYTES)
        .spawn(move || run_owner(driver))
        .context("failed to spawn BAP owner")?;
    Ok(())
}

// Kept as its own frame so `just audit-bap-stack` can root its audit here.
#[inline(never)]
fn run_owner(mut driver: UartDriver<'static>) {
    if let Err(error) = write_all(&mut driver, BAP_INITIALIZATION_BANNER) {
        log::warn!("bap_status=unavailable reason=initialization_banner error={error:#}");
        return;
    }
    log::info!("bap_status=available owner=uart2");
    let started = Instant::now();
    let mut runtime = BapRuntime::default();
    let mut accumulator = BapFrameAccumulator::default();
    let mut last_values = BapLastValues::default();
    let mut buffer = [0_u8; 128];

    loop {
        let now_ms = elapsed_ms(started);
        runtime.set_mode(connection_mode(), now_ms);
        match driver.read(&mut buffer, ticks(BAP_READ_TIMEOUT_MS)) {
            Ok(read) if read > 0 => {
                for frame in accumulator.push(&buffer[..read]) {
                    // A (re)subscription resends its next values even when unchanged.
                    if let Some(parameter) = handle_frame(&mut driver, &mut runtime, &frame, now_ms)
                    {
                        last_values.reset(parameter);
                    }
                }
            }
            Ok(_) => {}
            Err(error) if error.code() == esp_idf_svc::sys::ESP_ERR_TIMEOUT => {}
            Err(error) => {
                log::warn!("bap_ingress=unavailable category=uart_read error={error}");
            }
        }
        for action in runtime.poll(elapsed_ms(started)) {
            handle_runtime_action(&mut driver, action, &mut last_values);
        }
    }
}

// Kept out of line so the owner loop's frame does not carry every handler's locals.
#[inline(never)]
/// Answers one frame; returns the parameter a `SUB` (re)subscribed.
fn handle_frame(
    driver: &mut UartDriver<'static>,
    runtime: &mut BapRuntime,
    input: &[u8],
    now_ms: u64,
) -> Option<BapParameter> {
    let dispatch = match runtime.admit(input, now_ms, bap_request_snapshot) {
        Ok(dispatch) => dispatch,
        Err(error) => {
            log::warn!("bap_ingress=rejected category={error:?}");
            return None;
        }
    };
    if let Some(pending) = dispatch.maybe_setting {
        apply_setting(driver, &pending);
        return None;
    }
    for response in dispatch.responses {
        write_frame(driver, &response);
    }
    dispatch.maybe_subscribed
}

#[inline(never)]
fn handle_runtime_action(
    driver: &mut UartDriver<'static>,
    action: BapRuntimeAction,
    last_values: &mut BapLastValues,
) {
    match action {
        BapRuntimeAction::AnnounceAccessPoint => {
            let frame =
                BapFrame::new_token(BapCommand::Command, "mode", Some("ap_mode".to_owned()));
            if let Ok(frame) = frame {
                write_frame(driver, &frame);
            }
        }
        BapRuntimeAction::SubscriptionTimedOut(parameter) => {
            let frame = BapFrame::new(
                BapCommand::Status,
                parameter,
                Some(BapErrorCode::SubscriptionTimeout.token().to_owned()),
            );
            if let Ok(frame) = frame {
                write_frame(driver, &frame);
            }
            last_values.reset(parameter);
        }
        BapRuntimeAction::PublishSubscription(parameter) => {
            let Some(frames) = current_subscription_frames(parameter) else {
                log::warn!("bap_subscription=unavailable category=runtime_snapshot");
                return;
            };
            for frame in last_values.changed(parameter, frames) {
                write_frame(driver, &frame);
            }
        }
    }
}

fn connection_mode() -> BapConnectionMode {
    if crate::wifi_adapter::current_wifi_snapshot().wifi_status == "connected" {
        BapConnectionMode::Connected
    } else {
        BapConnectionMode::AccessPoint
    }
}

/// The operator snapshot is projected inside its publication callback, so only the
/// few values BAP needs reach this thread's frames, never the whole system-info wire:
/// on the 16 KiB BAP stack that copy overflowed (`just audit-bap-stack`).
#[inline(never)]
fn current_subscription_frames(parameter: BapParameter) -> Option<Vec<BapFrame>> {
    crate::runtime_snapshot::publish_projected_system_info(0, |info| {
        Ok::<_, core::convert::Infallible>(subscription_frames(parameter, &info))
    })
    .ok()
}

#[inline(never)]
fn bap_request_snapshot() -> Option<BapRequestSnapshot> {
    crate::runtime_snapshot::publish_projected_system_info(0, |info| {
        Ok::<_, core::convert::Infallible>(request_snapshot(info))
    })
    .ok()
}

fn request_snapshot(info: SystemInfoWire) -> BapRequestSnapshot {
    bitaxe_api::bap_request_snapshot(info)
}

fn subscription_frames(parameter: BapParameter, info: &SystemInfoWire) -> Vec<BapFrame> {
    bitaxe_api::bap_subscription_values(parameter, info)
        .into_iter()
        .filter_map(|(token, value)| {
            BapFrame::new_token(BapCommand::Response, token, Some(value)).ok()
        })
        .collect()
}

/// Runs one setting through the HTTP-task owners, then sends the reference's
/// replies. The `restarting` status follows 100 ms after the acknowledgement,
/// and the restart owner waits 1,000 ms more, as the reference does.
#[inline(never)]
fn apply_setting(driver: &mut UartDriver<'static>, pending: &BapPendingSetting) {
    let category = setting_category(&pending.setting);
    let result = crate::http_api::apply_accessory_setting(pending.setting.clone());
    log::info!("bap_setting=handled setting={category} result={result:?}");
    let completion = match complete_setting(pending, result) {
        Ok(completion) => completion,
        Err(error) => {
            log::warn!("bap_egress=rejected category={error:?}");
            return;
        }
    };
    for frame in &completion.acknowledgement {
        write_frame(driver, frame);
    }
    let Some(restart_status) = completion.maybe_restart_status else {
        return;
    };
    std::thread::sleep(Duration::from_millis(BAP_RESTART_STATUS_DELAY_MS));
    write_frame(driver, &restart_status);
    if !crate::http_api::restart_for_accessory() {
        log::warn!("bap_restart=unavailable setting={category}");
    }
}

fn setting_category(setting: &BapSettingIntent) -> &'static str {
    match setting {
        BapSettingIntent::FrequencyMhz(_) => "frequency",
        BapSettingIntent::AsicVoltageMillivolts(_) => "asic_voltage",
        BapSettingIntent::WifiSsid(_) => "ssid",
        BapSettingIntent::WifiPassword(_) => "password",
        BapSettingIntent::ManualFanPercent(_) => "fan_speed",
        BapSettingIntent::AutoFan(_) => "auto_fan",
        BapSettingIntent::FoundBlock(_) => "found_block",
        BapSettingIntent::ShowNewBlock(_) => "show_new_block",
    }
}

fn write_frame(driver: &mut UartDriver<'static>, frame: &BapFrame) {
    let encoded = match frame.encode() {
        Ok(encoded) => encoded,
        Err(error) => {
            log::warn!("bap_egress=rejected category={error:?}");
            return;
        }
    };
    if let Err(error) = write_all(driver, encoded.as_bytes()) {
        log::warn!("bap_egress=unavailable category=uart_write error={error:#}");
    }
}

fn write_all(driver: &mut UartDriver<'static>, bytes: &[u8]) -> anyhow::Result<()> {
    let written = driver.write(bytes)?;
    ensure!(written == bytes.len(), "partial BAP UART write");
    Ok(())
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn ticks(timeout_ms: u32) -> esp_idf_svc::sys::TickType_t {
    TickType::new_millis(u64::from(timeout_ms)).ticks()
}
