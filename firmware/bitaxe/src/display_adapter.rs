//! Minimal startup-only SSD1306 display adapter for Ultra 205 bring-up.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/display.c`
//! - `reference/esp-miner/main/i2c_bitaxe.c`
//! - `reference/esp-miner/main/screen.c`
//! - parity checklist rows `IO-001`, `UI-001`, and `UI-002`

use anyhow::Result;
use bitaxe_core::{StartupDebugText, STARTUP_DEBUG_LINE_COUNT, STARTUP_DEBUG_LINE_STRIDE_PX};
use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use crate::safety_adapter::BitaxeI2cBus;

pub const DISPLAY_I2C_ADDRESS: u8 = 0x3c;
pub const DISPLAY_I2C_SDA_GPIO: i32 = 47;
pub const DISPLAY_I2C_SCL_GPIO: i32 = 48;
pub const DISPLAY_I2C_SPEED_HZ: u32 = 400_000;

pub fn publish_runtime_display_input_boundary() {
    log::warn!(
        "display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true"
    );
}

pub fn render_startup_debug_text(
    bus: &mut BitaxeI2cBus<'_>,
    text: &StartupDebugText,
) -> Result<()> {
    debug_assert_eq!(DISPLAY_I2C_ADDRESS, 0x3c);
    debug_assert_eq!(DISPLAY_I2C_SDA_GPIO, 47);
    debug_assert_eq!(DISPLAY_I2C_SCL_GPIO, 48);
    debug_assert_eq!(DISPLAY_I2C_SPEED_HZ, 400_000);
    debug_assert_eq!(text.lines().len(), STARTUP_DEBUG_LINE_COUNT);
    debug_assert!(text.fits_ultra_205_display());

    let interface =
        I2CDisplayInterface::new_custom_address(bus.startup_display(), DISPLAY_I2C_ADDRESS);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display
        .init()
        .map_err(|error| anyhow::anyhow!("initialize SSD1306 display: {error:?}"))?;

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_5X7)
        .text_color(BinaryColor::On)
        .build();
    for (index, line) in text.lines().into_iter().enumerate() {
        let y = (index * STARTUP_DEBUG_LINE_STRIDE_PX) as i32;
        Text::with_baseline(line, Point::new(0, y), text_style, Baseline::Top)
            .draw(&mut display)
            .map_err(|error| anyhow::anyhow!("draw startup debug display text: {error:?}"))?;
    }

    display
        .flush()
        .map_err(|error| anyhow::anyhow!("flush startup debug display text: {error:?}"))?;
    log::info!("display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c");
    Ok(())
}
