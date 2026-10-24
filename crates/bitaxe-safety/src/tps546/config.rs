//! The Gamma 601's TPS546 configuration, as upstream writes it (ADR-0041, phase D).
//!
//! Upstream's ordered writes are data here, encoded as upstream encodes them, so they can be
//! pinned by a golden test and read back. Nothing here can store to the controller's NVM.
//!
//! Two deliberate deviations bound the voltage: `VOUT_COMMAND` is the requested target rather
//! than upstream's interim 1.2 V, and `VOUT_MAX` is 1.25 V rather than 2.0 V, so the controller
//! itself refuses a command above the check's ceiling.
//!
//! Reference: `reference/esp-miner/main/power/TPS546.c:214-332, 527-681` and
//! `main/power/vcore.c:63-78`.

/// `VOUT_MODE` the configuration is built for: linear, relative limits, exponent -9. Upstream
/// encodes the limits after `VOUT_COMMAND` as fractions of it, which only mean that under the
/// relative bit.
pub const EXPECTED_VOUT_MODE: u8 = 0x97;

/// The core-voltage targets the check may command.
pub const TARGET_MIN_VOLTS: f64 = 1.00;
pub const TARGET_MAX_VOLTS: f64 = 1.20;

/// `VOUT_MAX`, in volts; upstream writes 2.0 V.
pub const VOUT_MAX_VOLTS: f64 = 1.25;

/// Whether a command stores to, or restores from, NVM: PMBus `STORE_*`/`RESTORE_*`
/// (0x11-0x18) or a vendor command (0xF0 and up, including `NVM_CHECKSUM`). No write may use
/// one.
#[must_use]
pub const fn is_nvm_command(command: u8) -> bool {
    matches!(command, 0x11..=0x18 | 0xf0..=0xff)
}

/// The commands the Gamma plan writes, in its order. A bus that writes configuration writes
/// these and no other.
pub const PLAN_COMMANDS: [u8; 32] = [
    0x02, 0xec, 0xe4, 0x04, 0x33, 0x35, 0x36, 0x55, 0x56, 0x29, 0x21, 0x24, 0x2b, 0x40, 0x42, 0x25,
    0x26, 0x43, 0x44, 0x4a, 0x46, 0x47, 0x51, 0x4f, 0x50, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0xee,
];

/// Whether a command is one the Gamma plan writes.
#[must_use]
pub fn is_plan_command(command: u8) -> bool {
    PLAN_COMMANDS.contains(&command)
}

/// How a written value is encoded, and so how its readback is compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Raw,
    Linear11,
    /// ULINEAR16 under `VOUT_MODE`'s exponent.
    Vout,
}

/// One configuration write. Only [`gamma_config_plan`] and the live [`vout_command_write`] make
/// one, so a bus that takes it can write nothing outside the plan's registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigWrite {
    command: u8,
    word: bool,
    value: u16,
    encoding: Encoding,
}

impl ConfigWrite {
    #[must_use]
    pub const fn command(&self) -> u8 {
        self.command
    }

    /// `true` for a word, `false` for a byte.
    #[must_use]
    pub const fn word(&self) -> bool {
        self.word
    }

    #[must_use]
    pub const fn value(&self) -> u16 {
        self.value
    }
}

/// What reading the configuration back found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readback {
    Unread,
    Matched,
    /// The first register that did not read back as written.
    Mismatch {
        command: u8,
        wrote: u16,
        read: u16,
    },
}

/// Upstream's Gamma writes, in order; built only by [`gamma_config_plan`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GammaConfigPlan(Vec<ConfigWrite>);

impl GammaConfigPlan {
    #[must_use]
    pub fn writes(&self) -> &[ConfigWrite] {
        &self.0
    }
}

/// Why a configuration cannot be built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigRefusal {
    VoutModeUnexpected,
    TargetOutOfRange,
    Unencodable,
}

impl ConfigRefusal {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::VoutModeUnexpected => "vout_mode_unexpected",
            Self::TargetOutOfRange => "target_out_of_range",
            Self::Unencodable => "unencodable",
        }
    }
}

/// LINEAR11 for a positive value, as upstream's `float_2_slinear11`: the largest exponent that
/// keeps the truncated mantissa under 1024. Upstream silently overflows the mantissa from 2048;
/// that has no encoding here.
#[must_use]
pub fn encode_linear11(value: f64) -> Option<u16> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let exponent =
        (0..=15_u32).find(|&shift| (value * f64::from(1_u32 << shift)) as i64 >= 1024)?;
    let exponent = i32::try_from(exponent).ok()? - 1;
    let mantissa = (value * 2_f64.powi(exponent)) as u16;
    if mantissa > 0x03ff {
        return None;
    }
    let encoded_exponent = ((-exponent) as u16) << 11 & 0xf800;
    Some(encoded_exponent | mantissa)
}

/// LINEAR11 for a non-negative integer, as upstream's `int_2_slinear11`.
#[must_use]
pub fn encode_linear11_integer(value: u32) -> Option<u16> {
    let exponent = (0..=15_u32).find(|&exponent| value >> exponent < 1024)?;
    Some(((exponent as u16) << 11) | (value >> exponent) as u16)
}

/// ULINEAR16 under `VOUT_MODE`'s exponent, truncated, as upstream's `float_2_ulinear16`.
#[must_use]
pub fn encode_vout(volts: f64, vout_mode: u8) -> Option<u16> {
    if !volts.is_finite() || volts < 0.0 {
        return None;
    }
    let exponent = ((i32::from(vout_mode & 0x1f)) << 27) >> 27;
    let raw = volts / 2_f64.powi(exponent);
    (raw <= f64::from(u16::MAX)).then_some(raw as u16)
}

/// Upstream's Gamma writes, in order, with `VOUT_COMMAND` set to `target_volts`. Built only
/// for the expected `VOUT_MODE`, a target within 1.00-1.20 V and an output confirmed off: the
/// first write lets the output follow `OPERATION` alone.
pub fn gamma_config_plan(
    _off: super::ConfirmedOff,
    vout_mode: u8,
    target_volts: f64,
) -> Result<GammaConfigPlan, ConfigRefusal> {
    if vout_mode != EXPECTED_VOUT_MODE {
        return Err(ConfigRefusal::VoutModeUnexpected);
    }
    if !(TARGET_MIN_VOLTS..=TARGET_MAX_VOLTS).contains(&target_volts) {
        return Err(ConfigRefusal::TargetOutOfRange);
    }
    let raw_byte = |command, value: u8| Ok(write(command, false, u16::from(value), Encoding::Raw));
    let raw_word = |command, value| Ok(write(command, true, value, Encoding::Raw));
    let linear = |command, value: f64| {
        encode_linear11(value)
            .map(|encoded| write(command, true, encoded, Encoding::Linear11))
            .ok_or(ConfigRefusal::Unencodable)
    };
    let integer = |command, value: u32| {
        encode_linear11_integer(value)
            .map(|encoded| write(command, true, encoded, Encoding::Linear11))
            .ok_or(ConfigRefusal::Unencodable)
    };
    let vout = |command, value: f64| {
        encode_vout(value, vout_mode)
            .map(|encoded| write(command, true, encoded, Encoding::Vout))
            .ok_or(ConfigRefusal::Unencodable)
    };
    [
        raw_byte(0x02, 0x1b),       // ON_OFF_CONFIG: the output follows OPERATION only
        raw_word(0xec, 0x0000),     // STACK_CONFIG: one module
        raw_byte(0xe4, 0x10),       // SYNC_CONFIG: disabled
        raw_byte(0x04, 0x00),       // PHASE: single
        integer(0x33, 650),         // FREQUENCY_SWITCH, kHz
        linear(0x35, 4.8),          // VIN_ON
        linear(0x36, 4.5),          // VIN_OFF
        linear(0x55, 6.5),          // VIN_OV_FAULT_LIMIT
        raw_byte(0x56, 0xb7),       // VIN_OV_FAULT_RESPONSE
        linear(0x29, 0.25),         // VOUT_SCALE_LOOP
        vout(0x21, target_volts),   // VOUT_COMMAND
        vout(0x24, VOUT_MAX_VOLTS), // VOUT_MAX (upstream 2.0 V)
        vout(0x2b, 1.0),            // VOUT_MIN
        vout(0x40, 1.25),           // VOUT_OV_FAULT_LIMIT (relative)
        vout(0x42, 1.16),           // VOUT_OV_WARN_LIMIT (relative)
        vout(0x25, 1.10),           // VOUT_MARGIN_HIGH (relative)
        vout(0x26, 0.90),           // VOUT_MARGIN_LOW (relative)
        vout(0x43, 0.90),           // VOUT_UV_WARN_LIMIT (relative)
        vout(0x44, 0.75),           // VOUT_UV_FAULT_LIMIT (relative)
        linear(0x4a, 25.0),         // IOUT_OC_WARN_LIMIT, A
        linear(0x46, 30.0),         // IOUT_OC_FAULT_LIMIT, A
        raw_byte(0x47, 0xc0),       // IOUT_OC_FAULT_RESPONSE: shut down, no retries
        integer(0x51, 105),         // OT_WARN_LIMIT, C
        integer(0x4f, 145),         // OT_FAULT_LIMIT, C
        raw_byte(0x50, 0xff),       // OT_FAULT_RESPONSE
        integer(0x60, 0),           // TON_DELAY
        integer(0x61, 3),           // TON_RISE, ms
        integer(0x62, 0),           // TON_MAX_FAULT_LIMIT
        raw_byte(0x63, 0x3b),       // TON_MAX_FAULT_RESPONSE
        integer(0x64, 0),           // TOFF_DELAY
        integer(0x65, 0),           // TOFF_FALL
        raw_word(0xee, 0xffff),     // PIN_DETECT_OVERRIDE
    ]
    .into_iter()
    .collect::<Result<_, _>>()
    .map(GammaConfigPlan)
}

/// The `VOUT_COMMAND` write for a new target while the output is on (upstream
/// `TPS546_set_vout`), within the configuration's 1.00-1.20 V clamp and only for the expected
/// `VOUT_MODE`.
pub fn vout_command_write(vout_mode: u8, target_volts: f64) -> Result<ConfigWrite, ConfigRefusal> {
    if vout_mode != EXPECTED_VOUT_MODE {
        return Err(ConfigRefusal::VoutModeUnexpected);
    }
    if !(TARGET_MIN_VOLTS..=TARGET_MAX_VOLTS).contains(&target_volts) {
        return Err(ConfigRefusal::TargetOutOfRange);
    }
    encode_vout(target_volts, vout_mode)
        .map(|encoded| write(0x21, true, encoded, Encoding::Vout))
        .ok_or(ConfigRefusal::Unencodable)
}

/// The registers a live voltage change requires to still hold this firmware's configuration:
/// `VOUT_MAX` and `VOUT_SCALE_LOOP`, as the plan writes them. A controller that lost its power
/// since, and so its volatile configuration, reads back otherwise.
pub fn live_guard_writes(vout_mode: u8) -> Result<[ConfigWrite; 2], ConfigRefusal> {
    if vout_mode != EXPECTED_VOUT_MODE {
        return Err(ConfigRefusal::VoutModeUnexpected);
    }
    let vout_max = encode_vout(VOUT_MAX_VOLTS, vout_mode).ok_or(ConfigRefusal::Unencodable)?;
    let scale_loop = encode_linear11(0.25).ok_or(ConfigRefusal::Unencodable)?;
    Ok([
        write(0x24, true, vout_max, Encoding::Vout),
        write(0x29, true, scale_loop, Encoding::Linear11),
    ])
}

const fn write(command: u8, word: bool, value: u16, encoding: Encoding) -> ConfigWrite {
    ConfigWrite {
        command,
        word,
        value,
        encoding,
    }
}

/// Whether a register read back as written. Raw values must match exactly and VOUT values to
/// one count. A LINEAR11 value may come back re-encoded, so it is compared by value: within
/// half a step of the written encoding, never negative, and exactly zero when zero was written.
/// The read's own exponent never widens the tolerance.
#[must_use]
pub fn readback_matches(write: &ConfigWrite, read: u16, vout_mode: u8) -> bool {
    match write.encoding {
        Encoding::Raw => {
            let mask = if write.word { 0xffff } else { 0x00ff };
            read & mask == write.value & mask
        }
        Encoding::Linear11 => {
            let written = super::decode_linear11(write.value);
            let decoded = super::decode_linear11(read);
            if written == 0.0 {
                return decoded == 0.0;
            }
            decoded >= 0.0 && (decoded - written).abs() <= linear11_step(write.value) / 2.0
        }
        Encoding::Vout => {
            read.abs_diff(write.value) <= 1 && super::decode_vout(read, vout_mode).is_some()
        }
    }
}

fn linear11_step(raw: u16) -> f64 {
    let exponent = ((i32::from(raw >> 11)) << 27) >> 27;
    2_f64.powi(exponent)
}

#[cfg(test)]
mod tests;
