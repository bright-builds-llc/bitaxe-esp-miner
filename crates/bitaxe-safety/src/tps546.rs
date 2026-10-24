//! TPS546 PMBus power-controller facts for the Gamma 601 safe boot (ADR-0041).
//!
//! The safe boot reads the controller and can only command its output off: `OffWrite` has no
//! way to express an output-on or voltage write. Faults are judged with the deliberate `OFF`
//! status bit masked.
//!
//! Reference: `reference/esp-miner/main/power/TPS546.c`, `TPS546.h` and `pmbus_commands.h`.

pub mod config;
pub mod enable;

pub const MODULE_NAME: &str = "tps546";

/// The controller's 7-bit I2C address on the Gamma 601.
pub const I2C_ADDRESS: u8 = 0x24;

/// A residual output above this after an off command means the output is not confirmed off.
pub const OFF_VOUT_LIMIT_VOLTS: f64 = 0.2;

/// The PMBus reads the safe boot performs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadCommand {
    /// `IC_DEVICE_ID`: a block read, a length byte then six identity bytes.
    DeviceId,
    StatusWord,
    /// `STATUS_CML`: why the communication, memory or logic summary bit is set.
    StatusCml,
    VoutMode,
    ReadVin,
    ReadVout,
    ReadIout,
    ReadTemperature,
}

impl ReadCommand {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::DeviceId => 0xad,
            Self::StatusWord => 0x79,
            Self::StatusCml => 0x7e,
            Self::VoutMode => 0x20,
            Self::ReadVin => 0x88,
            Self::ReadVout => 0x8b,
            Self::ReadIout => 0x8c,
            Self::ReadTemperature => 0x8d,
        }
    }

    /// Bytes the read returns; words are little-endian.
    #[must_use]
    pub const fn length(self) -> usize {
        match self {
            Self::DeviceId => 7,
            Self::VoutMode | Self::StatusCml => 1,
            Self::StatusWord
            | Self::ReadVin
            | Self::ReadVout
            | Self::ReadIout
            | Self::ReadTemperature => 2,
        }
    }
}

/// The only writes the safe boot can make. Each turns the output off or keeps it off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffWrite {
    /// `OPERATION` = off.
    OperationOff,
    /// `ON_OFF_CONFIG`: the output needs both the `OPERATION` command and the control pin.
    OnOffConfigRequireCommandAndPin,
}

impl OffWrite {
    /// The command code and value, as written on the bus.
    #[must_use]
    pub const fn bytes(self) -> [u8; 2] {
        match self {
            Self::OperationOff => [0x01, 0x00],
            Self::OnOffConfigRequireCommandAndPin => [0x02, 0x1f],
        }
    }
}

/// Upstream's order: the output off first, then the on/off configuration.
pub const OFF_SEQUENCE: [OffWrite; 2] = [
    OffWrite::OperationOff,
    OffWrite::OnOffConfigRequireCommandAndPin,
];

/// The TPS546 variants the Gamma 601 is built with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceModel {
    D24a,
    D24s,
}

impl DeviceModel {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::D24a => "d24a",
            Self::D24s => "d24s",
        }
    }
}

const DEVICE_ID_D24A: [u8; 6] = [0x54, 0x49, 0x54, 0x6d, 0x24, 0x41];
const DEVICE_ID_D24S: [u8; 6] = [0x54, 0x49, 0x54, 0x6d, 0x24, 0x62];

/// Names the controller from its `IC_DEVICE_ID` block; the first byte is the block length.
#[must_use]
pub fn classify_device_id(block: &[u8; 7]) -> Option<DeviceModel> {
    let identity = &block[1..];
    if identity == DEVICE_ID_D24A {
        Some(DeviceModel::D24a)
    } else if identity == DEVICE_ID_D24S {
        Some(DeviceModel::D24s)
    } else {
        None
    }
}

/// Decodes a PMBus LINEAR11 word: a signed 5-bit exponent over a signed 11-bit mantissa.
#[must_use]
pub fn decode_linear11(raw: u16) -> f64 {
    let exponent = sign_extend(raw >> 11, 5);
    let mantissa = sign_extend(raw & 0x07ff, 11);
    f64::from(mantissa) * 2_f64.powi(exponent)
}

/// Decodes `READ_VOUT` under `VOUT_MODE`; only the linear mode (bits 6:5 clear) is accepted.
/// Bit 7, PMBus 1.3's relative flag, does not change how `READ_VOUT` is read.
#[must_use]
pub fn decode_vout(raw: u16, vout_mode: u8) -> Option<f64> {
    if (vout_mode >> 5) & 0b11 != 0 {
        return None;
    }
    let exponent = sign_extend(u16::from(vout_mode & 0x1f), 5);
    Some(f64::from(raw) * 2_f64.powi(exponent))
}

fn sign_extend(value: u16, bits: u32) -> i32 {
    let shift = 32 - bits;
    (i32::from(value) << shift) >> shift
}

/// A `STATUS_WORD` reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusWord(pub u16);

const STATUS_OFF: u16 = 0x0040;
/// Fault summary bits: VOUT_OV, IOUT_OC, VIN_UV, TEMP, CML, and the VOUT, IOUT, INPUT, MFR,
/// OTHER and UNKNOWN summaries. `OFF`, `POWER_GOOD#` and `NONE_OF_THE_ABOVE` are left out: they
/// may be set whenever the output is deliberately off. Latched bits stay set until a
/// `CLEAR_FAULTS`, which the safe boot never sends, so a fault may predate this boot.
const STATUS_FAULTS: u16 = 0x0020 | 0x0010 | 0x0008 | 0x0004 | 0x0002 | 0xf000 | 0x0200 | 0x0100;

/// The bits upstream's `TPS546_check_status` treats as a power fault besides `OFF`: VOUT_OV,
/// IOUT_OC, VIN_UV and TEMP.
const STATUS_POWER_FAULTS: u16 = 0x0020 | 0x0010 | 0x0008 | 0x0004;

impl StatusWord {
    #[must_use]
    pub const fn output_off(self) -> bool {
        self.0 & STATUS_OFF != 0
    }

    /// Whether upstream would report a power fault: one of its fault bits, or the output off
    /// while it should be on. The latched `CML` bit is not one of them.
    #[must_use]
    pub const fn power_fault(self, output_expected_on: bool) -> bool {
        self.0 & STATUS_POWER_FAULTS != 0 || (output_expected_on && self.output_off())
    }

    /// The fault bits that are set, with the deliberate `OFF` masked.
    #[must_use]
    pub const fn faults(self) -> u16 {
        self.0 & STATUS_FAULTS
    }
}

/// Whether the output is confirmed off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffState {
    Confirmed,
    /// The controller does not report its output off.
    ReportsOn,
    /// The controller reports off, but the output still measures above the limit.
    VoltagePresent,
}

impl OffState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::ReportsOn => "reports_on",
            Self::VoltagePresent => "voltage_present",
        }
    }

    #[must_use]
    pub const fn confirmed(self) -> Option<ConfirmedOff> {
        match self {
            Self::Confirmed => Some(ConfirmedOff(())),
            Self::ReportsOn | Self::VoltagePresent => None,
        }
    }
}

/// That a reading confirmed the output off: [`OffState::confirmed`] makes one only from
/// `Confirmed`. Writes that need the output off take it, so a caller must pass through an off
/// reading; it does not prove the reading was fresh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfirmedOff(());

/// Confirms off only when the controller reports off and the output measures below the limit.
#[must_use]
pub fn confirm_off(status: StatusWord, vout_volts: f64) -> OffState {
    if !status.output_off() {
        OffState::ReportsOn
    } else if vout_volts.is_nan() || vout_volts >= OFF_VOUT_LIMIT_VOLTS {
        OffState::VoltagePresent
    } else {
        OffState::Confirmed
    }
}

#[cfg(test)]
mod tests;
