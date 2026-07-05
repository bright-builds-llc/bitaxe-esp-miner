use anyhow::Result;
use esp_idf_svc::hal::gpio::{Output, OutputPin, PinDriver};

pub const ASIC_RESET_GPIO: i32 = 1;
pub const ASIC_ENABLE_GPIO: i32 = 10;
pub const RESET_PULSE_LOW_MS: u32 = 100;
pub const RESET_PULSE_HIGH_MS: u32 = 100;

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

impl crate::safety_adapter::Phase27BringUpReset for AsicReset<'_> {
    fn hold_reset_low(&mut self) -> Result<()> {
        AsicReset::hold_reset_low(self)
    }

    fn reset_pulse(&mut self, low_ms: u32, high_ms: u32) -> Result<()> {
        AsicReset::reset_pulse(self, low_ms, high_ms)
    }
}
