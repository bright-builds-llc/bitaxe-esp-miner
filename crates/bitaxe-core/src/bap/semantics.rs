use super::{BapCommand, BapFrame, BapFrameError, BapParameter};
use std::fmt;

/// Reference default subscription cadence.
pub const BAP_DEFAULT_SUBSCRIPTION_INTERVAL_MS: u32 = 3_000;
/// Reference subscription lease duration.
pub const BAP_SUBSCRIPTION_TIMEOUT_MS: u32 = 300_000;

/// Network mode visible to pure BAP command policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapConnectionMode {
    Connected,
    AccessPoint,
}

/// Exact public BAP error values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapErrorCode {
    ApModeNoSubscriptions,
    ApModeNoRequests,
    ApModeLimitedSettings,
    MissingParameter,
    SystemNotReady,
    InvalidRange,
    SetFailed,
    UnsupportedParameter,
    SubscriptionTimeout,
}

impl BapErrorCode {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::ApModeNoSubscriptions => "ap_mode_no_subscriptions",
            Self::ApModeNoRequests => "ap_mode_no_requests",
            Self::ApModeLimitedSettings => "ap_mode_limited_settings",
            Self::MissingParameter => "missing_parameter",
            Self::SystemNotReady => "system_not_ready",
            Self::InvalidRange => "invalid_range",
            Self::SetFailed => "set_failed",
            Self::UnsupportedParameter => "unsupported_parameter",
            Self::SubscriptionTimeout => "subscription_timeout",
        }
    }
}

/// Snapshot used only to project supported read requests.
#[derive(Clone, PartialEq, Eq)]
pub struct BapRequestSnapshot {
    /// Public device-family label projected by `systemInfo`.
    pub device_model: String,
    /// Public ASIC-family label projected by `systemInfo`.
    pub asic_model: String,
    /// Runtime pool endpoint. Debug output never renders this value.
    pub pool_endpoint: String,
    /// Runtime pool port.
    pub pool_port: u16,
    /// Runtime pool user. Debug output never renders this value.
    pub pool_user: String,
    /// Accepted share counter.
    pub shares_accepted: u64,
    /// Rejected share counter.
    pub shares_rejected: u64,
    /// Current block height.
    pub block_height: i32,
    /// Upstream-compatible found-block flag.
    pub found_block: i32,
    /// Whether the new-block notification is visible.
    pub show_new_block: bool,
}

impl fmt::Debug for BapRequestSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BapRequestSnapshot")
            .field("device_model_present", &!self.device_model.is_empty())
            .field("asic_model_present", &!self.asic_model.is_empty())
            .field("pool_endpoint_present", &!self.pool_endpoint.is_empty())
            .field("pool_user_present", &!self.pool_user.is_empty())
            .field("shares_accepted", &self.shares_accepted)
            .field("shares_rejected", &self.shares_rejected)
            .field("block_height", &self.block_height)
            .field("found_block", &self.found_block)
            .field("show_new_block", &self.show_new_block)
            .finish()
    }
}

/// Pure setting intent. Adapters must separately authorize and apply effects.
#[derive(Clone, PartialEq)]
pub enum BapSettingIntent {
    FrequencyMhz(f32),
    AsicVoltageMillivolts(u16),
    WifiSsid(String),
    WifiPassword(String),
    ManualFanPercent(u16),
    AutoFan(bool),
    FoundBlock(i32),
    ShowNewBlock(bool),
}

impl fmt::Debug for BapSettingIntent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrequencyMhz(value) => {
                formatter.debug_tuple("FrequencyMhz").field(value).finish()
            }
            Self::AsicVoltageMillivolts(value) => formatter
                .debug_tuple("AsicVoltageMillivolts")
                .field(value)
                .finish(),
            Self::WifiSsid(value) => formatter
                .debug_struct("WifiSsid")
                .field("value_bytes", &value.len())
                .finish(),
            Self::WifiPassword(value) => formatter
                .debug_struct("WifiPassword")
                .field("value_bytes", &value.len())
                .finish(),
            Self::ManualFanPercent(value) => formatter
                .debug_tuple("ManualFanPercent")
                .field(value)
                .finish(),
            Self::AutoFan(value) => formatter.debug_tuple("AutoFan").field(value).finish(),
            Self::FoundBlock(value) => formatter.debug_tuple("FoundBlock").field(value).finish(),
            Self::ShowNewBlock(value) => {
                formatter.debug_tuple("ShowNewBlock").field(value).finish()
            }
        }
    }
}

/// Restart requirement attached to a pure setting intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapRestartPolicy {
    Never,
    WhenCompanionCredentialPresent,
    Always,
}

/// Imperative work described but never executed by the pure planner.
#[derive(Clone, Debug, PartialEq)]
pub enum BapEffect {
    Subscribe {
        parameter: BapParameter,
        interval_ms: u32,
        timeout_ms: u32,
    },
    Unsubscribe {
        parameter: BapParameter,
    },
    ApplySetting {
        setting: BapSettingIntent,
        restart: BapRestartPolicy,
    },
}

/// Closed command planning result.
#[derive(Clone, Debug, PartialEq)]
pub struct BapPlan {
    responses: Vec<BapFrame>,
    maybe_effect: Option<BapEffect>,
}

impl BapPlan {
    /// Frames to send immediately for a rejection or after a planned effect
    /// succeeds.
    #[must_use]
    pub fn responses(&self) -> &[BapFrame] {
        &self.responses
    }

    /// Optional work for an imperative adapter. The planner never applies it.
    #[must_use]
    pub const fn effect(&self) -> Option<&BapEffect> {
        self.maybe_effect.as_ref()
    }

    fn no_response() -> Self {
        Self {
            responses: Vec::new(),
            maybe_effect: None,
        }
    }

    fn with_effect(effect: BapEffect, responses: Vec<BapFrame>) -> Self {
        Self {
            responses,
            maybe_effect: Some(effect),
        }
    }
}

/// Closed planning failures that do not contain request values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapPlanError {
    NoRegisteredHandler,
    MissingRequestSnapshot,
    InvalidGeneratedFrame(BapFrameError),
}

/// Plans one admitted BAP command without performing I/O or hardware effects.
pub fn plan_command(
    frame: &BapFrame,
    mode: BapConnectionMode,
    maybe_snapshot: Option<&BapRequestSnapshot>,
) -> Result<BapPlan, BapPlanError> {
    match frame.command() {
        BapCommand::Request => plan_request(frame, mode, maybe_snapshot),
        BapCommand::Subscribe => plan_subscribe(frame, mode),
        BapCommand::Unsubscribe => plan_unsubscribe(frame),
        BapCommand::Set => plan_setting(frame, mode),
        BapCommand::Response
        | BapCommand::Acknowledge
        | BapCommand::Error
        | BapCommand::Command
        | BapCommand::Status
        | BapCommand::Log => Err(BapPlanError::NoRegisteredHandler),
    }
}

fn plan_request(
    frame: &BapFrame,
    mode: BapConnectionMode,
    maybe_snapshot: Option<&BapRequestSnapshot>,
) -> Result<BapPlan, BapPlanError> {
    if mode == BapConnectionMode::AccessPoint {
        return error_plan(frame.parameter_token(), BapErrorCode::ApModeNoRequests);
    }
    let Some(parameter) = frame.known_parameter() else {
        return Ok(BapPlan::no_response());
    };
    let snapshot = maybe_snapshot.ok_or(BapPlanError::MissingRequestSnapshot)?;
    let mut responses = Vec::new();
    match parameter {
        BapParameter::SystemInfo => {
            responses.push(response("deviceModel", &snapshot.device_model)?);
            responses.push(response("asicModel", &snapshot.asic_model)?);
            responses.push(response("pool", &snapshot.pool_endpoint)?);
            responses.push(response("poolPort", &snapshot.pool_port.to_string())?);
            responses.push(response("poolUser", &snapshot.pool_user)?);
        }
        BapParameter::Shares => responses.push(frame_with_value(
            BapCommand::Response,
            BapParameter::Shares,
            format!("{}/{}", snapshot.shares_accepted, snapshot.shares_rejected),
        )?),
        BapParameter::BlockHeight => responses.push(frame_with_value(
            BapCommand::Response,
            BapParameter::BlockHeight,
            snapshot.block_height.to_string(),
        )?),
        BapParameter::FoundBlock => responses.push(frame_with_value(
            BapCommand::Response,
            BapParameter::FoundBlock,
            snapshot.found_block.to_string(),
        )?),
        BapParameter::ShowNewBlock => responses.push(frame_with_value(
            BapCommand::Response,
            BapParameter::ShowNewBlock,
            u8::from(snapshot.show_new_block).to_string(),
        )?),
        _ => return Ok(BapPlan::no_response()),
    }
    Ok(BapPlan {
        responses,
        maybe_effect: None,
    })
}

fn response(response_parameter: &str, value: &str) -> Result<BapFrame, BapPlanError> {
    frame_with_token(BapCommand::Response, response_parameter, value.to_owned())
}

fn plan_subscribe(frame: &BapFrame, mode: BapConnectionMode) -> Result<BapPlan, BapPlanError> {
    if mode == BapConnectionMode::AccessPoint {
        return error_plan(frame.parameter_token(), BapErrorCode::ApModeNoSubscriptions);
    }
    let Some(parameter) = frame.known_parameter() else {
        return Ok(BapPlan::no_response());
    };
    let interval_ms = frame
        .value()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|interval| *interval > 0)
        .unwrap_or(BAP_DEFAULT_SUBSCRIPTION_INTERVAL_MS);
    let response = frame_with_value(BapCommand::Acknowledge, parameter, "subscribed".to_owned())?;
    Ok(BapPlan::with_effect(
        BapEffect::Subscribe {
            parameter,
            interval_ms,
            timeout_ms: BAP_SUBSCRIPTION_TIMEOUT_MS,
        },
        vec![response],
    ))
}

fn plan_unsubscribe(frame: &BapFrame) -> Result<BapPlan, BapPlanError> {
    let Some(parameter) = frame.known_parameter() else {
        return Ok(BapPlan::no_response());
    };
    let response = frame_with_value(
        BapCommand::Acknowledge,
        parameter,
        "unsubscribed".to_owned(),
    )?;
    Ok(BapPlan::with_effect(
        BapEffect::Unsubscribe { parameter },
        vec![response],
    ))
}

fn plan_setting(frame: &BapFrame, mode: BapConnectionMode) -> Result<BapPlan, BapPlanError> {
    let Some(value) = frame.value() else {
        return error_plan(frame.parameter_token(), BapErrorCode::MissingParameter);
    };
    let maybe_parameter = frame.known_parameter();
    if mode == BapConnectionMode::AccessPoint
        && !matches!(
            maybe_parameter,
            Some(BapParameter::Ssid | BapParameter::Password)
        )
    {
        return error_plan(frame.parameter_token(), BapErrorCode::ApModeLimitedSettings);
    }

    let (setting, restart, maybe_ack) = match maybe_parameter {
        Some(BapParameter::Frequency) => {
            let Some(frequency) = value
                .parse::<f32>()
                .ok()
                .filter(|value| value.is_finite() && (100.0..=800.0).contains(value))
            else {
                return error_plan(frame.parameter_token(), BapErrorCode::InvalidRange);
            };
            (
                BapSettingIntent::FrequencyMhz(frequency),
                BapRestartPolicy::Never,
                Some(format!("{frequency:.2}")),
            )
        }
        Some(BapParameter::AsicVoltage) => {
            let Some(voltage) = parse_bounded_u16(value, 700, 1_400) else {
                return error_plan(frame.parameter_token(), BapErrorCode::InvalidRange);
            };
            (
                BapSettingIntent::AsicVoltageMillivolts(voltage),
                BapRestartPolicy::Never,
                Some(voltage.to_string()),
            )
        }
        Some(BapParameter::Ssid) => (
            BapSettingIntent::WifiSsid(value.to_owned()),
            BapRestartPolicy::WhenCompanionCredentialPresent,
            Some(value.to_owned()),
        ),
        Some(BapParameter::Password) => (
            BapSettingIntent::WifiPassword(value.to_owned()),
            BapRestartPolicy::Always,
            Some("password_set".to_owned()),
        ),
        Some(BapParameter::FanSpeed) => {
            let Some(fan_speed) = parse_bounded_u16(value, 0, 100) else {
                return error_plan(frame.parameter_token(), BapErrorCode::InvalidRange);
            };
            (
                BapSettingIntent::ManualFanPercent(fan_speed),
                BapRestartPolicy::Never,
                None,
            )
        }
        Some(BapParameter::AutoFan) => {
            let Some(auto_fan) = parse_bounded_u16(value, 0, 1) else {
                return error_plan(frame.parameter_token(), BapErrorCode::InvalidRange);
            };
            (
                BapSettingIntent::AutoFan(auto_fan == 1),
                BapRestartPolicy::Never,
                Some("auto_fan_speed_set".to_owned()),
            )
        }
        Some(BapParameter::FoundBlock) => {
            let Some(found_block) = value.parse::<i32>().ok() else {
                return error_plan(frame.parameter_token(), BapErrorCode::InvalidRange);
            };
            (
                BapSettingIntent::FoundBlock(found_block),
                BapRestartPolicy::Never,
                Some(value.to_owned()),
            )
        }
        Some(BapParameter::ShowNewBlock) => {
            let Some(show_new_block) = value.parse::<i32>().ok() else {
                return error_plan(frame.parameter_token(), BapErrorCode::InvalidRange);
            };
            (
                BapSettingIntent::ShowNewBlock(show_new_block != 0),
                BapRestartPolicy::Never,
                Some(value.to_owned()),
            )
        }
        _ => return error_plan(frame.parameter_token(), BapErrorCode::UnsupportedParameter),
    };

    let responses = maybe_ack
        .map(|ack| frame_with_token(BapCommand::Acknowledge, frame.parameter_token(), ack))
        .transpose()?
        .into_iter()
        .collect();
    Ok(BapPlan::with_effect(
        BapEffect::ApplySetting { setting, restart },
        responses,
    ))
}

fn parse_bounded_u16(value: &str, minimum: u16, maximum: u16) -> Option<u16> {
    value
        .parse::<u16>()
        .ok()
        .filter(|parsed| (minimum..=maximum).contains(parsed))
}

fn error_plan(parameter_token: &str, error: BapErrorCode) -> Result<BapPlan, BapPlanError> {
    Ok(BapPlan {
        responses: vec![frame_with_token(
            BapCommand::Error,
            parameter_token,
            error.token().to_owned(),
        )?],
        maybe_effect: None,
    })
}

fn frame_with_token(
    command: BapCommand,
    parameter_token: &str,
    value: String,
) -> Result<BapFrame, BapPlanError> {
    BapFrame::new_token(command, parameter_token, Some(value))
        .map_err(BapPlanError::InvalidGeneratedFrame)
}

fn frame_with_value(
    command: BapCommand,
    parameter: BapParameter,
    value: String,
) -> Result<BapFrame, BapPlanError> {
    BapFrame::new(command, parameter, Some(value)).map_err(BapPlanError::InvalidGeneratedFrame)
}
