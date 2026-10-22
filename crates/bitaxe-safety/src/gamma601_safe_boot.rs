//! What the Gamma 601 safe boot observed, and its periodic USB record (ADR-0041).
//!
//! The safe boot holds the ASIC in reset, keeps the TPS546 output off and drives the fan at a
//! fixed duty. Each reading is either a value or `unavailable`; nothing private is recorded.

use crate::tps546::{DeviceModel, OffState};

pub const MODULE_NAME: &str = "gamma601_safe_boot";

/// The fan duty once the output is confirmed off: upstream's duty when mining is paused.
pub const FAN_DUTY_PERCENT: u8 = 30;
/// The fan duty until then: the output may still be on, inherited from the previous firmware.
pub const FAN_DUTY_UNCONFIRMED_PERCENT: u8 = 100;

/// What the controller identification found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controller {
    /// Not read yet, or the read failed.
    Unread,
    /// A device answered with an identity that is not a TPS546; it is never written.
    Unrecognised,
    Found(DeviceModel),
}

impl Controller {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unread => "unread",
            Self::Unrecognised => "unrecognised",
            Self::Found(model) => model.label(),
        }
    }
}

/// One sweep of the safe boot's readings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SafeBootReport {
    pub reset_held: bool,
    pub controller: Controller,
    /// `None` until the status and output have both been read.
    pub maybe_off: Option<OffState>,
    pub maybe_faults: Option<u16>,
    pub maybe_vin_volts: Option<f64>,
    pub maybe_vout_volts: Option<f64>,
    pub maybe_iout_amps: Option<f64>,
    pub maybe_vr_temperature_celsius: Option<f64>,
    pub maybe_board_temperature_celsius: Option<f64>,
    pub fan_duty_percent: u8,
    pub maybe_fan_rpm: Option<u32>,
    pub wifi_joined: bool,
}

impl SafeBootReport {
    /// Before any reading.
    #[must_use]
    pub const fn initial(reset_held: bool) -> Self {
        Self {
            reset_held,
            controller: Controller::Unread,
            maybe_off: None,
            maybe_faults: None,
            maybe_vin_volts: None,
            maybe_vout_volts: None,
            maybe_iout_amps: None,
            maybe_vr_temperature_celsius: None,
            maybe_board_temperature_celsius: None,
            fan_duty_percent: FAN_DUTY_UNCONFIRMED_PERCENT,
            maybe_fan_rpm: None,
            wifi_joined: false,
        }
    }

    /// Whether the output is confirmed off.
    #[must_use]
    pub fn off_confirmed(&self) -> bool {
        self.maybe_off == Some(OffState::Confirmed)
    }

    /// The fan duty for the current state: full until the output is confirmed off.
    #[must_use]
    pub fn fan_duty_for_state(&self) -> u8 {
        if self.off_confirmed() {
            FAN_DUTY_PERCENT
        } else {
            FAN_DUTY_UNCONFIRMED_PERCENT
        }
    }

    /// The closed USB record.
    #[must_use]
    pub fn usb_marker(&self) -> String {
        format!(
            "usb_gamma601_safe_boot schema=v1 reset_held={} tps546={} off={} faults={} vin_mv={} vout_mv={} iout_ma={} vr_temp_c={} board_temp_c={} fan_duty_pct={} fan_rpm={} wifi_joined={} redacted=true",
            self.reset_held,
            self.controller.label(),
            self.maybe_off.map_or("unread", OffState::label),
            self.maybe_faults
                .map_or_else(|| "unavailable".to_owned(), |faults| format!("0x{faults:04x}")),
            milli(self.maybe_vin_volts),
            milli(self.maybe_vout_volts),
            milli(self.maybe_iout_amps),
            whole(self.maybe_vr_temperature_celsius),
            whole(self.maybe_board_temperature_celsius),
            self.fan_duty_percent,
            self.maybe_fan_rpm
                .map_or_else(|| "unavailable".to_owned(), |rpm| rpm.to_string()),
            self.wifi_joined,
        )
    }
}

fn milli(maybe_value: Option<f64>) -> String {
    maybe_value.filter(|value| value.is_finite()).map_or_else(
        || "unavailable".to_owned(),
        |value| format!("{:.0}", value * 1000.0),
    )
}

fn whole(maybe_value: Option<f64>) -> String {
    maybe_value
        .filter(|value| value.is_finite())
        .map_or_else(|| "unavailable".to_owned(), |value| format!("{value:.0}"))
}

#[cfg(test)]
mod tests;
