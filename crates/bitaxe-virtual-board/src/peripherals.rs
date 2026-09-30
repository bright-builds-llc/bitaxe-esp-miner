//! Register behavior from pinned INA260/EMC2101/DS4432U reference drivers.
//! Dynamics are uncalibrated functional curves, independently configurable.
use crate::ModelError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dynamics {
    pub voltage_settle_mv_per_ms: u32,
    pub fan_accel_rpm_per_ms: u32,
    pub max_fan_rpm: u32,
    pub ambient_millicelsius: i32,
    pub heating_millicelsius_per_second: i32,
    pub cooling_millicelsius_per_second: i32,
    pub sample_period_ms: u64,
    pub i2c_transaction_ms: u64,
}
impl Default for Dynamics {
    fn default() -> Self {
        Self {
            voltage_settle_mv_per_ms: 10,
            fan_accel_rpm_per_ms: 10,
            max_fan_rpm: 6000,
            ambient_millicelsius: 25000,
            heating_millicelsius_per_second: 2000,
            cooling_millicelsius_per_second: 4000,
            sample_period_ms: 100,
            i2c_transaction_ms: 1,
        }
    }
}
impl Dynamics {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.sample_period_ms == 0
            || self.sample_period_ms > 60000
            || self.max_fan_rpm > 100000
            || self.heating_millicelsius_per_second < 0
            || self.cooling_millicelsius_per_second < 0
            || self.heating_millicelsius_per_second > 1_000_000
            || self.cooling_millicelsius_per_second > 1_000_000
            || !(-40_000..=200_000).contains(&self.ambient_millicelsius)
        {
            return Err(ModelError::Invalid("dynamics range"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensorSnapshot {
    pub boot_ordinal: u64,
    pub sampled_at_ms: u64,
    pub sequence: u64,
    pub core_mv: u32,
    pub input_mv: u32,
    pub input_ma: u32,
    pub power_mw: u32,
    pub fan_rpm: u32,
    pub temperature_millicelsius: i32,
    pub fan_percent: u8,
}

#[derive(Debug, Clone)]
pub struct Peripherals {
    pub dynamics: Dynamics,
    pub fan_stalled: bool,
    pub sensor_publication_blocked: bool,
    pub button_pressed: bool,
    pub power_enabled: bool,
    pub reset_asserted: bool,
    pub fan_percent: u8,
    pub target_core_mv: u32,
    core_mv: u32,
    fan_rpm: u32,
    temperature_millicelsius: i32,
    thermal_fraction: i64,
    last_advance_ms: u64,
    pub sample: SensorSnapshot,
    pub sample_valid: bool,
    active_boot_ordinal: u64,
    registers: [[u8; 256]; 3],
    pub display: [u8; 1024],
    pub display_enabled: bool,
    display_column: usize,
    display_page: usize,
    display_column_window: (usize, usize),
    display_page_window: (usize, usize),
    display_address_mode: u8,
    pub display_configuration: BTreeMap<u8, Vec<u8>>,
}
impl Peripherals {
    pub fn new(dynamics: Dynamics) -> Result<Self, ModelError> {
        dynamics.validate()?;
        let mut result = Self {
            dynamics,
            fan_stalled: false,
            sensor_publication_blocked: false,
            button_pressed: false,
            power_enabled: false,
            reset_asserted: true,
            fan_percent: 0,
            target_core_mv: 0,
            core_mv: 0,
            fan_rpm: 0,
            temperature_millicelsius: dynamics.ambient_millicelsius,
            thermal_fraction: 0,
            last_advance_ms: 0,
            sample_valid: false,
            active_boot_ordinal: 1,
            sample: SensorSnapshot {
                boot_ordinal: 1,
                sampled_at_ms: 0,
                sequence: 0,
                core_mv: 0,
                input_mv: 5000,
                input_ma: 0,
                power_mw: 0,
                fan_rpm: 0,
                temperature_millicelsius: dynamics.ambient_millicelsius,
                fan_percent: 0,
            },
            registers: [[0; 256]; 3],
            display: [0; 1024],
            display_enabled: false,
            display_column: 0,
            display_page: 0,
            display_column_window: (0, 127),
            display_page_window: (0, 7),
            display_address_mode: 2,
            display_configuration: BTreeMap::new(),
        };
        result.registers[0][0] = 0x61;
        result.registers[1][0xfd] = 0x16;
        result.registers[1][0xfe] = 0x5d;
        Ok(result)
    }
    pub fn advance_to(&mut self, now_ms: u64, hashing: bool) -> Result<(), ModelError> {
        if now_ms < self.last_advance_ms {
            return Err(ModelError::Invalid("peripheral clock regression"));
        }
        // Fixed 1ms integration makes changing scenario step sizes deterministic.
        for _ in self.last_advance_ms..now_ms {
            let target = if self.power_enabled {
                self.target_core_mv
            } else {
                0
            };
            self.core_mv = approach(self.core_mv, target, self.dynamics.voltage_settle_mv_per_ms);
            let fan_target = if self.fan_stalled {
                0
            } else {
                self.dynamics.max_fan_rpm * u32::from(self.fan_percent) / 100
            };
            self.fan_rpm = approach(self.fan_rpm, fan_target, self.dynamics.fan_accel_rpm_per_ms);
            let heating = if hashing && self.power_enabled && !self.reset_asserted {
                self.dynamics.heating_millicelsius_per_second
            } else {
                0
            };
            let cooling = i64::from(self.dynamics.cooling_millicelsius_per_second)
                * i64::from(self.fan_percent)
                / 100;
            self.thermal_fraction += i64::from(heating) - cooling;
            self.temperature_millicelsius = i32::try_from(
                i64::from(self.temperature_millicelsius) + self.thermal_fraction / 1000,
            )
            .map_err(|_| ModelError::Invalid("thermal model overflow"))?;
            self.thermal_fraction %= 1000;
            if self.temperature_millicelsius < self.dynamics.ambient_millicelsius {
                self.temperature_millicelsius = self.dynamics.ambient_millicelsius;
                self.thermal_fraction = 0;
            }
            let step = self.last_advance_ms + 1;
            self.last_advance_ms = step;
            if step.is_multiple_of(self.dynamics.sample_period_ms)
                && !self.sensor_publication_blocked
            {
                self.publish(step, hashing);
            }
        }
        Ok(())
    }
    fn publish(&mut self, now_ms: u64, hashing: bool) {
        let input_ma = if self.power_enabled {
            if hashing {
                2400
            } else {
                400
            }
        } else {
            0
        };
        self.sample_valid = true;
        self.sample = SensorSnapshot {
            boot_ordinal: self.active_boot_ordinal,
            sampled_at_ms: now_ms,
            sequence: self.sample.sequence.saturating_add(1),
            core_mv: self.core_mv,
            input_mv: 5000,
            input_ma,
            power_mw: input_ma * 5,
            fan_rpm: self.fan_rpm,
            temperature_millicelsius: self.temperature_millicelsius,
            fan_percent: self.fan_percent,
        };
    }
    /// A reset invalidates boot-owned publication state while retaining the
    /// physical fan speed, charge/voltage decay and temperature inertia.
    pub fn reset_boot(&mut self, boot_ordinal: u64) {
        self.active_boot_ordinal = boot_ordinal;
        self.sample_valid = false;
        self.power_enabled = false;
        self.reset_asserted = true;
        self.target_core_mv = 0;
    }
    pub fn write_register(
        &mut self,
        address: u8,
        register: u8,
        value: &[u8],
    ) -> Result<(), ModelError> {
        match address {
            0x40 if matches!(register, 0x00 | 0x06 | 0x07) && value.len() == 2 => {
                self.registers[0][usize::from(register)] = value[0];
                self.registers[0][usize::from(register) + 1] = value[1];
            }
            0x4c if value.len() == 1
                && matches!(register, 0x03 | 0x04 | 0x17 | 0x18 | 0x48..=0x5f) =>
            {
                if register == 0x4c && value[0] > 63 {
                    return Err(ModelError::Invalid("fan PWM code"));
                }
                self.registers[1][usize::from(register)] = value[0];
                if register == 0x4c {
                    self.fan_percent = ((u16::from(value[0]) * 100) / 63) as u8;
                }
            }
            0x48 if value.len() == 1 && matches!(register, 0xf8 | 0xf9) => {
                self.registers[2][usize::from(register)] = value[0];
                if register == 0xf8 {
                    // Reverse the documented current-DAC/resistor transfer relation.
                    let current = f64::from(value[0] & 0x7f) * 0.000098921 / 127.0;
                    let signed = if value[0] & 0x80 == 0 {
                        -current
                    } else {
                        current
                    };
                    self.target_core_mv =
                        ((0.6 + 4750.0 * (0.6 / 3320.0 - signed)) * 1000.0).round() as u32;
                }
            }
            _ => return Err(ModelError::Unsupported("I2C register write")),
        }
        Ok(())
    }
    pub fn read_register(
        &self,
        address: u8,
        register: u8,
        length: usize,
    ) -> Result<Vec<u8>, ModelError> {
        let measurement = (address == 0x40 && matches!(register, 0x01..=0x03))
            || (address == 0x4c && matches!(register, 0x00 | 0x01 | 0x10 | 0x46 | 0x47));
        if measurement && !self.sample_valid {
            return Err(ModelError::Unavailable("sensor publication unavailable"));
        }
        if address == 0x40 && length == 2 {
            let value = match register {
                0x01 => (self.sample.input_ma * 4 / 5) as u16,
                0x02 => (self.sample.input_mv * 4 / 5) as u16,
                0x03 => (self.sample.power_mw / 10) as u16,
                0xfe => 0x5449,
                0xff => 0x2270,
                0x00 | 0x06 | 0x07 => u16::from_be_bytes([
                    self.registers[0][usize::from(register)],
                    self.registers[0][usize::from(register) + 1],
                ]),
                _ => return Err(ModelError::Unsupported("INA260 register")),
            };
            return Ok(value.to_be_bytes().to_vec());
        }
        if address == 0x4c && length == 1 {
            let tach = if self.sample.fan_rpm == 0 {
                0xffff
            } else {
                (5400000 / self.sample.fan_rpm).min(0xffff) as u16
            };
            let temperature = self.sample.temperature_millicelsius / 125;
            let value = match register {
                // Internal driver adds the Ultra205 +5C calibration offset.
                0x00 => {
                    (self
                        .sample
                        .temperature_millicelsius
                        .checked_sub(5000)
                        .ok_or(ModelError::Invalid("temperature register range"))?
                        / 1000) as u8
                }
                // External data remains the explicitly modeled raw domain.
                0x01 => (temperature / 8) as u8,
                0x10 => ((temperature & 7) << 5) as u8,
                0x46 => tach as u8,
                0x47 => (tach >> 8) as u8,
                0x02..=0x04 | 0x17 | 0x18 | 0x48..=0x5f | 0xfd | 0xfe => {
                    self.registers[1][usize::from(register)]
                }
                _ => return Err(ModelError::Unsupported("EMC2101 register")),
            };
            return Ok(vec![value]);
        }
        if address == 0x48 && length == 1 && matches!(register, 0xf8 | 0xf9) {
            return Ok(vec![self.registers[2][usize::from(register)]]);
        }
        Err(ModelError::Unsupported("I2C register read"))
    }
    pub fn display_write(&mut self, control: u8, data: &[u8]) -> Result<(), ModelError> {
        if control == 0x40 {
            for byte in data {
                self.display[self.display_page * 128 + self.display_column] = *byte;
                self.advance_display_cursor();
            }
            return Ok(());
        }
        if control != 0 {
            return Err(ModelError::Unsupported("SSD1306 control"));
        }
        let mut cursor = 0;
        while cursor < data.len() {
            let command = data[cursor];
            cursor += 1;
            let parameters = match command {
                0x20 | 0x81 | 0x8d | 0xa8 | 0xd3 | 0xd5 | 0xd9 | 0xda | 0xdb => 1,
                0x21 | 0x22 => 2,
                0xae
                | 0xaf
                | 0xa0..=0xa7
                | 0xc0
                | 0xc8
                | 0x40..=0x7f
                | 0xb0..=0xb7
                | 0x00..=0x1f => 0,
                _ => return Err(ModelError::Unsupported("SSD1306 command")),
            };
            let arguments = data
                .get(cursor..cursor + parameters)
                .ok_or(ModelError::Invalid("SSD1306 parameters"))?;
            cursor += parameters;
            match command {
                0x20 if arguments[0] <= 2 => self.display_address_mode = arguments[0],
                0x20 => return Err(ModelError::Invalid("SSD1306 addressing mode")),
                0x21 => {
                    if arguments[0] > arguments[1] || arguments[1] > 127 {
                        return Err(ModelError::Invalid("SSD1306 columns"));
                    }
                    self.display_column_window =
                        (usize::from(arguments[0]), usize::from(arguments[1]));
                    self.display_column = usize::from(arguments[0]);
                }
                0x22 => {
                    if arguments[0] > arguments[1] || arguments[1] > 7 {
                        return Err(ModelError::Invalid("SSD1306 pages"));
                    }
                    self.display_page_window =
                        (usize::from(arguments[0]), usize::from(arguments[1]));
                    self.display_page = usize::from(arguments[0]);
                }
                0xae => self.display_enabled = false,
                0xaf => self.display_enabled = true,
                0xb0..=0xb7 => self.display_page = usize::from(command - 0xb0),
                0x00..=0x0f => {
                    self.display_column = (self.display_column & 0x70) | usize::from(command)
                }
                0x10..=0x17 => {
                    self.display_column =
                        (self.display_column & 0x0f) | (usize::from(command - 0x10) << 4)
                }
                0x18..=0x1f => return Err(ModelError::Invalid("SSD1306 column exceeds width")),
                _ => {}
            }
            self.display_configuration
                .insert(command, arguments.to_vec());
        }
        Ok(())
    }
    fn advance_display_cursor(&mut self) {
        if self.display_address_mode == 1 {
            self.display_page += 1;
            if self.display_page > self.display_page_window.1 {
                self.display_page = self.display_page_window.0;
                self.display_column += 1;
            }
        } else {
            self.display_column += 1;
            if self.display_column > self.display_column_window.1 {
                self.display_column = self.display_column_window.0;
                if self.display_address_mode == 0 {
                    self.display_page += 1;
                }
            }
        }
        if self.display_column > self.display_column_window.1 {
            self.display_column = self.display_column_window.0;
        }
        if self.display_page > self.display_page_window.1 {
            self.display_page = self.display_page_window.0;
        }
    }
}
fn approach(value: u32, target: u32, speed: u32) -> u32 {
    if value < target {
        value.saturating_add(speed).min(target)
    } else {
        value.saturating_sub(speed).max(target)
    }
}
