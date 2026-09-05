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
        crate::production_mining_session::revocation::note_asic_halted(
            crate::runtime_uptime::millis(),
        );
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
        self.reset_pulse_cancellable(low_ms, high_ms, &mut || Ok(()))
    }

    pub fn reset_pulse_cancellable(
        &mut self,
        low_ms: u32,
        high_ms: u32,
        check: &mut dyn FnMut() -> Result<()>,
    ) -> Result<()> {
        debug_assert_eq!(RESET_PULSE_LOW_MS, 100);
        debug_assert_eq!(RESET_PULSE_HIGH_MS, 100);

        check()?;
        self.reset.set_low()?;
        cancellable_delay(low_ms, check)?;
        check()?;
        self.reset.set_high()?;
        cancellable_delay(high_ms, check)?;
        Ok(())
    }

    pub fn hold_reset_low(&mut self) -> Result<()> {
        self.reset.set_low()?;
        crate::production_mining_session::revocation::note_asic_halted(
            crate::runtime_uptime::millis(),
        );
        log::info!("asic_status=hold_reset_low gpio={ASIC_RESET_GPIO}");
        Ok(())
    }
}

fn cancellable_delay(duration_ms: u32, check: &mut dyn FnMut() -> Result<()>) -> Result<()> {
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_millis(u64::from(duration_ms));
    loop {
        check()?;
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return Ok(());
        }
        std::thread::sleep(remaining.min(std::time::Duration::from_millis(50)));
    }
}
