//! ASIC core power enable GPIO adapter (GPIO10, active-low).
//!
//! Reference: `reference/esp-miner/main/power/vcore.c`

use anyhow::Result;
use esp_idf_svc::hal::gpio::{Output, OutputPin, PinDriver};

pub const ASIC_ENABLE_GPIO: i32 = 10;

pub struct AsicEnable<'d> {
    enable: PinDriver<'d, Output>,
}

impl<'d> AsicEnable<'d> {
    pub fn new<PIN>(enable_pin: PIN) -> Result<Self>
    where
        PIN: OutputPin + 'd,
    {
        debug_assert_eq!(ASIC_ENABLE_GPIO, 10);
        let enable = PinDriver::output(enable_pin)?;
        Ok(Self { enable })
    }

    /// Enable ASIC core power (active-low pin driven low).
    pub fn enable_power(&mut self) -> Result<()> {
        self.enable.set_low()?;
        log::info!("asic_enable_status=active gpio={ASIC_ENABLE_GPIO}");
        Ok(())
    }

    /// Disable ASIC core power (active-low pin driven high).
    pub fn disable_power(&mut self) -> Result<()> {
        self.enable.set_high()?;
        log::info!("asic_enable_status=disabled gpio={ASIC_ENABLE_GPIO}");
        Ok(())
    }
}
