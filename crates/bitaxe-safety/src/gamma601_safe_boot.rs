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
    /// `STATUS_WORD` at this sweep.
    pub maybe_status: Option<u16>,
    pub maybe_faults: Option<u16>,
    /// `STATUS_CML` at this sweep.
    pub maybe_cml: Option<u8>,
    /// `STATUS_WORD` and `STATUS_CML` read before the identity read, the first transfers to
    /// the controller in this boot: whether a fault bit predates the firmware's first read.
    pub maybe_pre_id_status: Option<u16>,
    pub maybe_pre_id_cml: Option<u8>,
    /// `STATUS_WORD` and `STATUS_CML` as first read at boot, before the off writes.
    pub maybe_start_status: Option<u16>,
    pub maybe_start_cml: Option<u8>,
    pub maybe_vin_volts: Option<f64>,
    pub maybe_vout_volts: Option<f64>,
    pub maybe_iout_amps: Option<f64>,
    pub maybe_vr_temperature_celsius: Option<f64>,
    pub maybe_board_temperature_celsius: Option<f64>,
    pub fan_duty_percent: u8,
    pub maybe_fan_rpm: Option<u32>,
}

impl SafeBootReport {
    /// Before any reading.
    #[must_use]
    pub const fn initial(reset_held: bool) -> Self {
        Self {
            reset_held,
            controller: Controller::Unread,
            maybe_off: None,
            maybe_status: None,
            maybe_faults: None,
            maybe_cml: None,
            maybe_pre_id_status: None,
            maybe_pre_id_cml: None,
            maybe_start_status: None,
            maybe_start_cml: None,
            maybe_vin_volts: None,
            maybe_vout_volts: None,
            maybe_iout_amps: None,
            maybe_vr_temperature_celsius: None,
            maybe_board_temperature_celsius: None,
            fan_duty_percent: FAN_DUTY_UNCONFIRMED_PERCENT,
            maybe_fan_rpm: None,
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
        self.fan_duty_with(None, false)
    }

    /// The fan duty once an owner may command it. While the output is deliberately on, the
    /// owner's duty applies, full until it commands one. Otherwise an output not confirmed off
    /// keeps the fan full, whatever was commanded, and a confirmed-off output takes the owner's
    /// duty or the safe boot's.
    #[must_use]
    pub fn fan_duty_with(&self, maybe_commanded: Option<u8>, output_on: bool) -> u8 {
        if output_on {
            maybe_commanded.unwrap_or(FAN_DUTY_UNCONFIRMED_PERCENT)
        } else if self.off_confirmed() {
            maybe_commanded.unwrap_or(FAN_DUTY_PERCENT)
        } else {
            FAN_DUTY_UNCONFIRMED_PERCENT
        }
    }

    /// The closed USB record. `wifi_status` is the Wi-Fi owner's status category; anything
    /// other than a short lowercase token is reported as `other`, so no network value leaks.
    #[must_use]
    pub fn usb_marker(&self, wifi_status: &str) -> String {
        format!(
            "usb_gamma601_safe_boot schema=v3 reset_held={} tps546={} off={} faults={} cml={} pre_id_status={} pre_id_cml={} start_status={} start_cml={} vin_mv={} vout_mv={} iout_ma={} vr_temp_c={} board_temp_c={} fan_duty_pct={} fan_rpm={} wifi={} redacted=true",
            self.reset_held,
            self.controller.label(),
            self.maybe_off.map_or("unread", OffState::label),
            hex(self.maybe_faults.map(u32::from), 4),
            hex(self.maybe_cml.map(u32::from), 2),
            hex(self.maybe_pre_id_status.map(u32::from), 4),
            hex(self.maybe_pre_id_cml.map(u32::from), 2),
            hex(self.maybe_start_status.map(u32::from), 4),
            hex(self.maybe_start_cml.map(u32::from), 2),
            milli(self.maybe_vin_volts),
            milli(self.maybe_vout_volts),
            milli(self.maybe_iout_amps),
            whole(self.maybe_vr_temperature_celsius),
            whole(self.maybe_board_temperature_celsius),
            self.fan_duty_percent,
            self.maybe_fan_rpm
                .map_or_else(|| "unavailable".to_owned(), |rpm| rpm.to_string()),
            closed_category(wifi_status),
        )
    }
}

fn hex(maybe_value: Option<u32>, digits: usize) -> String {
    maybe_value.map_or_else(
        || "unavailable".to_owned(),
        |value| format!("0x{value:0digits$x}"),
    )
}

/// A status category is a short lowercase token; anything else is `other`.
fn closed_category(status: &str) -> &str {
    let closed = !status.is_empty()
        && status.len() <= 48
        && status
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_');
    if closed {
        status
    } else {
        "other"
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
