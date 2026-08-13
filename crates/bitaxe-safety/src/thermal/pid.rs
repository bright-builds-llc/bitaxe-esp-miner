//! Pure state transition for the pinned ESP-Miner fan PID.

use serde::Serialize;

pub const PID_KP: f32 = 5.0;
pub const PID_KI: f32 = 0.1;
pub const PID_KD: f32 = 2.0;
pub const PID_SAMPLE_TIME_MS: u32 = 100;
pub const PID_EMA_ALPHA: f32 = 0.2;

const INITIAL_OUTPUT_MIN_PERCENT: f32 = 0.0;
const INITIAL_OUTPUT_MAX_PERCENT: f32 = 255.0;
const FAN_OUTPUT_MAX_PERCENT: f32 = 100.0;

/// Retained state shared by the input filter and PID controller.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PidState {
    pub automatic: bool,
    pub maybe_filtered_input_celsius: Option<f32>,
    pub output_percent: f32,
    pub output_sum_percent: f32,
    pub last_input_celsius: f32,
    pub output_min_percent: f32,
    pub output_max_percent: f32,
}

impl Default for PidState {
    fn default() -> Self {
        Self {
            automatic: false,
            maybe_filtered_input_celsius: None,
            output_percent: 0.0,
            output_sum_percent: 0.0,
            last_input_celsius: 0.0,
            output_min_percent: INITIAL_OUTPUT_MIN_PERCENT,
            output_max_percent: INITIAL_OUTPUT_MAX_PERCENT,
        }
    }
}

/// One scheduled PID computation and its retained successor state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PidStep {
    pub filtered_input_celsius: f32,
    pub output_percent: f32,
    pub next_state: PidState,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PidController {
    pub state: PidState,
}

impl PidController {
    #[must_use]
    pub const fn new(state: PidState) -> Self {
        Self { state }
    }

    /// Advances the upstream reverse P-on-error controller by one 100 ms slot.
    #[must_use]
    pub fn step(
        self,
        target_temp_celsius: f64,
        raw_input_celsius: f64,
        min_fan_percent: u8,
    ) -> PidStep {
        let mut state = self.state;
        update_output_limits(&mut state, f32::from(min_fan_percent));
        let target_temp_celsius = target_temp_celsius as f32;
        let raw_input_celsius = raw_input_celsius as f32;

        let filtered_input_celsius = state
            .maybe_filtered_input_celsius
            .map_or(raw_input_celsius, |prior| {
                PID_EMA_ALPHA * raw_input_celsius + (1.0 - PID_EMA_ALPHA) * prior
            });
        state.maybe_filtered_input_celsius = Some(filtered_input_celsius);

        if !state.automatic {
            state.output_sum_percent = state
                .output_percent
                .clamp(state.output_min_percent, state.output_max_percent);
            state.last_input_celsius = filtered_input_celsius;
            state.automatic = true;
        }

        let sample_time_seconds = PID_SAMPLE_TIME_MS as f32 / 1_000.0;
        let proportional_gain = -PID_KP;
        let integral_gain = -(PID_KI * sample_time_seconds);
        let derivative_gain = -(PID_KD / sample_time_seconds);
        let error = target_temp_celsius - filtered_input_celsius;
        let input_delta = filtered_input_celsius - state.last_input_celsius;

        state.output_sum_percent += integral_gain * error;
        state.output_sum_percent = state
            .output_sum_percent
            .clamp(state.output_min_percent, state.output_max_percent);

        let mut output_percent =
            proportional_gain * error + state.output_sum_percent - derivative_gain * input_delta;
        if output_percent > state.output_max_percent {
            state.output_sum_percent -= output_percent - state.output_max_percent;
            output_percent = state.output_max_percent;
        } else if output_percent < state.output_min_percent {
            state.output_sum_percent += state.output_min_percent - output_percent;
            output_percent = state.output_min_percent;
        }

        state.output_percent = output_percent;
        state.last_input_celsius = filtered_input_celsius;

        PidStep {
            filtered_input_celsius,
            output_percent,
            next_state: state,
        }
    }
}

fn update_output_limits(state: &mut PidState, min_fan_percent: f32) {
    if state.output_min_percent == min_fan_percent {
        return;
    }

    state.output_min_percent = min_fan_percent;
    state.output_max_percent = FAN_OUTPUT_MAX_PERCENT;
    if !state.automatic {
        return;
    }

    state.output_percent = state
        .output_percent
        .clamp(state.output_min_percent, state.output_max_percent);
    state.output_sum_percent = state
        .output_sum_percent
        .clamp(state.output_min_percent, state.output_max_percent);
}
