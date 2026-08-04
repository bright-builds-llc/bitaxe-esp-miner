//! Ultra 205 calibrated core-voltage ADC owner.
//!
//! Reference breadcrumb: `reference/esp-miner/main/adc.c`.

use std::sync::Arc;

use anyhow::{Context, Result};
use esp_idf_svc::hal::{
    adc::{
        attenuation,
        oneshot::{
            config::{AdcChannelConfig, Calibration, Resolution},
            AdcChannelDriver, AdcDriver,
        },
        ADC1, ADCCH1, ADCU1,
    },
    gpio::Gpio2,
};

type Ultra205AdcChannel = AdcChannelDriver<'static, ADCCH1<ADCU1>, Arc<AdcDriver<'static, ADCU1>>>;

/// Sole owner of the calibrated ADC1 channel-1 input used for Vcore telemetry.
pub(crate) struct Ultra205CoreVoltageAdc {
    channel: Ultra205AdcChannel,
}

impl Ultra205CoreVoltageAdc {
    pub(crate) fn new(adc: ADC1<'static>, pin: Gpio2<'static>) -> Result<Self> {
        let driver = Arc::new(AdcDriver::new(adc).context("create ADC1 oneshot unit")?);
        let config = AdcChannelConfig {
            attenuation: attenuation::DB_12,
            resolution: Resolution::new(),
            calibration: Calibration::Curve,
        };
        let channel = AdcChannelDriver::new(driver, pin, &config)
            .context("configure ADC1 channel 1 with curve calibration")?;

        Ok(Self { channel })
    }

    pub(super) fn read_millivolts(&mut self) -> Result<u16> {
        self.channel
            .read()
            .context("read calibrated ADC1 channel 1")
    }
}
