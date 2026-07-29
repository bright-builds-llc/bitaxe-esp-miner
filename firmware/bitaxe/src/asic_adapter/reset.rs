use anyhow::Result;
use esp_idf_svc::hal::gpio::{Output, OutputPin, PinDriver};

pub const ASIC_RESET_GPIO: i32 = 1;
pub const ASIC_ENABLE_GPIO: i32 = 10;
pub const RESET_PULSE_LOW_MS: u32 = 100;
pub const RESET_PULSE_HIGH_MS: u32 = 100;

/// Active-low Ultra 205 ASIC power-enable owner.
pub struct AsicEnable<'d> {
    enable: PinDriver<'d, Output>,
}

impl<'d> AsicEnable<'d> {
    pub fn new<PIN>(enable_pin: PIN) -> Result<Self>
    where
        PIN: OutputPin + 'd,
    {
        debug_assert_eq!(ASIC_ENABLE_GPIO, 10);

        let mut enable = PinDriver::output(enable_pin)?;
        enable.set_high()?;
        Ok(Self { enable })
    }

    pub fn enable(&mut self) -> Result<()> {
        self.enable.set_low()?;
        log::info!("asic_enable_status=active gpio={ASIC_ENABLE_GPIO}");
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        self.enable.set_high()?;
        log::info!("asic_enable_status=inactive gpio={ASIC_ENABLE_GPIO}");
        Ok(())
    }
}

pub struct AsicReset<'d> {
    reset: PinDriver<'d, Output>,
}

impl<'d> AsicReset<'d> {
    pub fn new<PIN>(reset_pin: PIN) -> Result<Self>
    where
        PIN: OutputPin + 'd,
    {
        debug_assert_eq!(ASIC_RESET_GPIO, 1);
        debug_assert_eq!(ASIC_ENABLE_GPIO, 10);

        let reset = PinDriver::output(reset_pin)?;
        Ok(Self { reset })
    }

    pub fn reset_pulse(&mut self, low_ms: u32, high_ms: u32) -> Result<()> {
        debug_assert_eq!(RESET_PULSE_LOW_MS, 100);
        debug_assert_eq!(RESET_PULSE_HIGH_MS, 100);

        self.reset.set_low()?;
        std::thread::sleep(std::time::Duration::from_millis(u64::from(low_ms)));
        self.reset.set_high()?;
        std::thread::sleep(std::time::Duration::from_millis(u64::from(high_ms)));
        Ok(())
    }

    pub fn hold_reset_low(&mut self) -> Result<()> {
        self.reset.set_low()?;
        log::info!("asic_status=hold_reset_low gpio={ASIC_RESET_GPIO}");
        Ok(())
    }
}
