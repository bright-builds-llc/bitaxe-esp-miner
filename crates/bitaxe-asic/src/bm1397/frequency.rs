//! Pure BM1397 PLL and frequency-write sequencing.

use crate::{
    bm1366::frequency_voltage::pll_parameters_with_bounds,
    bm1397::{
        protocol::{Bm1397Command, RegisterTarget},
        Bm1397ProtocolFault,
    },
};

const FB_DIVIDER_MIN: u16 = 60;
const FB_DIVIDER_MAX: u16 = 200;
const FREQ_MULT_MHZ: f32 = 25.0;
const PREFREQUENCY_REGISTER: u8 = 0x70;
const FREQUENCY_REGISTER: u8 = 0x08;

/// Exact BM1397 PLL register plan for one requested frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1397FrequencyPlan {
    pub fb_divider: u8,
    pub refdiv: u8,
    pub postdiv1: u8,
    pub postdiv2: u8,
}

impl Bm1397FrequencyPlan {
    pub fn for_quarter_mhz(frequency_quarter_mhz: u32) -> Result<Self, Bm1397ProtocolFault> {
        let maybe_pll =
            pll_parameters_with_bounds(frequency_quarter_mhz, FB_DIVIDER_MIN, FB_DIVIDER_MAX);
        let Some(pll) = maybe_pll else {
            return Err(Bm1397ProtocolFault::InvalidFrequency {
                frequency_quarter_mhz,
            });
        };

        Ok(Self {
            fb_divider: pll.fb_divider,
            refdiv: pll.refdiv,
            postdiv1: pll.postdiv1,
            postdiv2: pll.postdiv2,
        })
    }

    #[must_use]
    pub fn actual_frequency_mhz(self) -> f32 {
        let divider = u32::from(self.refdiv) * u32::from(self.postdiv1) * u32::from(self.postdiv2);
        FREQ_MULT_MHZ * f32::from(self.fb_divider) / divider as f32
    }

    #[must_use]
    pub const fn register_value(self) -> [u8; 4] {
        let postdiv = ((self.postdiv1 & 0x07) << 4) + (self.postdiv2 & 0x07);
        [0x40, self.fb_divider, self.refdiv, postdiv]
    }
}

/// Expands one BM1397 frequency change into its exact duplicated writes and
/// inter-write delays.
pub fn frequency_write_sequence(
    frequency_quarter_mhz: u32,
) -> Result<Vec<Bm1397Command>, Bm1397ProtocolFault> {
    let plan = Bm1397FrequencyPlan::for_quarter_mhz(frequency_quarter_mhz)?;
    let mut commands = Vec::with_capacity(9);

    for _ in 0..2 {
        commands.push(Bm1397Command::DelayMs(10));
        commands.push(Bm1397Command::WriteRegister {
            target: RegisterTarget::All,
            register: PREFREQUENCY_REGISTER,
            value: [0x0f, 0x0f, 0x0f, 0x00],
        });
    }
    for _ in 0..2 {
        commands.push(Bm1397Command::DelayMs(10));
        commands.push(Bm1397Command::WriteRegister {
            target: RegisterTarget::All,
            register: FREQUENCY_REGISTER,
            value: plan.register_value(),
        });
    }
    commands.push(Bm1397Command::DelayMs(10));
    Ok(commands)
}
