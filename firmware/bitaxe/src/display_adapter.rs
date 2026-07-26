//! Minimal SSD1306 display adapter for Ultra 205 bring-up and runtime status.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/display.c`
//! - `reference/esp-miner/main/i2c_bitaxe.c`
//! - `reference/esp-miner/main/screen.c`
//! - parity checklist rows `IO-001`, `UI-001`, and `UI-002`

use anyhow::Result;
use bitaxe_core::{StartupDebugFrame, STARTUP_DEBUG_LINE_COUNT, STARTUP_DEBUG_LINE_STRIDE_PX};
use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::i2c::I2c;
use ssd1306::{command::AddrMode, prelude::*, I2CDisplayInterface, Ssd1306};

use crate::safety_adapter::{BitaxeI2cBus, RuntimeI2cOwner};

pub const DISPLAY_I2C_ADDRESS: u8 = 0x3c;
pub const DISPLAY_I2C_SDA_GPIO: i32 = 47;
pub const DISPLAY_I2C_SCL_GPIO: i32 = 48;
pub const DISPLAY_I2C_SPEED_HZ: u32 = 400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDisplayMode {
    AlternatingDebug,
    Deferred,
    Unavailable,
}

impl RuntimeDisplayMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AlternatingDebug => "alternating_debug",
            Self::Deferred => "deferred",
            Self::Unavailable => "unavailable",
        }
    }
}

pub fn publish_runtime_display_input_boundary(mode: RuntimeDisplayMode) {
    log::warn!(
        "display_input_status=runtime_gap reason=full_parity_pending display_runtime={} input_runtime=unavailable",
        mode.as_str()
    );
}

pub fn render_startup_debug_text(
    bus: &mut BitaxeI2cBus<'_>,
    frame: &StartupDebugFrame,
) -> Result<()> {
    debug_assert_eq!(DISPLAY_I2C_ADDRESS, 0x3c);
    debug_assert_eq!(DISPLAY_I2C_SDA_GPIO, 47);
    debug_assert_eq!(DISPLAY_I2C_SCL_GPIO, 48);
    debug_assert_eq!(DISPLAY_I2C_SPEED_HZ, 400_000);
    render_debug_text(bus.startup_display(), frame, true)?;
    log::info!("display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c");
    Ok(())
}

pub fn render_runtime_debug_text(
    owner: &mut RuntimeI2cOwner<'_>,
    frame: &StartupDebugFrame,
) -> Result<()> {
    render_debug_text(owner.display(), frame, false)
}

fn render_debug_text<I2C>(i2c: I2C, frame: &StartupDebugFrame, initialize: bool) -> Result<()>
where
    I2C: I2c,
{
    debug_assert_eq!(frame.lines().len(), STARTUP_DEBUG_LINE_COUNT);
    debug_assert!(frame.fits_ultra_205_display());

    let interface = I2CDisplayInterface::new_custom_address(i2c, DISPLAY_I2C_ADDRESS);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if initialize {
        display
            .init()
            .map_err(|error| anyhow::anyhow!("initialize SSD1306 display: {error:?}"))?;
    } else {
        display
            .set_addr_mode(AddrMode::Horizontal)
            .map_err(|error| anyhow::anyhow!("restore SSD1306 address mode: {error:?}"))?;
    }
    display.clear_buffer();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_5X7)
        .text_color(BinaryColor::On)
        .build();
    for (index, line) in frame.lines().into_iter().enumerate() {
        let y = (index * STARTUP_DEBUG_LINE_STRIDE_PX) as i32;
        Text::with_baseline(line, Point::new(0, y), text_style, Baseline::Top)
            .draw(&mut display)
            .map_err(|error| anyhow::anyhow!("draw debug display text: {error:?}"))?;
    }

    display
        .flush()
        .map_err(|error| anyhow::anyhow!("flush debug display text: {error:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;
    use core::convert::Infallible;
    use std::rc::Rc;

    use bitaxe_core::{AsicTarget, BoardTarget, StartupDebugText};
    use embedded_hal::i2c::{ErrorType, I2c, Operation};

    use super::render_debug_text;

    const DISPLAY_FRAMEBUFFER_BYTES: usize = 128 * 32 / 8;
    const ROW_THREE_SUFFIX_START: usize = 2 * 128 + 100;
    const ROW_THREE_SUFFIX_END: usize = 3 * 128;

    #[derive(Clone)]
    struct CapturingI2c {
        writes: Rc<RefCell<Vec<Vec<u8>>>>,
    }

    impl ErrorType for CapturingI2c {
        type Error = Infallible;
    }

    impl I2c for CapturingI2c {
        fn transaction(
            &mut self,
            _address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            for operation in operations {
                match operation {
                    Operation::Read(bytes) => bytes.fill(0),
                    Operation::Write(bytes) => self.writes.borrow_mut().push(bytes.to_vec()),
                }
            }
            Ok(())
        }
    }

    #[test]
    fn runtime_uptime_frame_transfers_cleared_full_framebuffer() {
        // Arrange
        let writes = Rc::new(RefCell::new(Vec::new()));
        let i2c = CapturingI2c {
            writes: Rc::clone(&writes),
        };
        let text = StartupDebugText::new(
            BoardTarget::Ultra205,
            AsicTarget::Bm1366,
            Some("abcdef123456-dev"),
            "2026-07-26T19:32:45Z",
        );
        let frame = text.frame_at(8_000);

        // Act
        render_debug_text(i2c, &frame, false).expect("runtime frame should render");
        let framebuffer = writes
            .borrow()
            .iter()
            .filter(|write| write.first() == Some(&0x40))
            .flat_map(|write| write[1..].iter().copied())
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(framebuffer.len(), DISPLAY_FRAMEBUFFER_BYTES);
        assert!(framebuffer[ROW_THREE_SUFFIX_START..ROW_THREE_SUFFIX_END]
            .iter()
            .all(|byte| *byte == 0));
    }
}
