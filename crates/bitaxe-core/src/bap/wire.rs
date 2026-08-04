use std::fmt;

/// Reference BAP message-buffer size. Complete wire messages must be shorter.
pub const BAP_MAX_MESSAGE_LEN: usize = 256;
/// Window in which an identical complete message is suppressed.
pub const BAP_DUPLICATE_WINDOW_MS: u64 = 1_000;

/// Commands in the pinned BAP wire vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapCommand {
    Request,
    Response,
    Subscribe,
    Unsubscribe,
    Set,
    Acknowledge,
    Error,
    Command,
    Status,
    Log,
}

impl BapCommand {
    /// Complete vocabulary in reference enumeration order.
    pub const ALL: [Self; 10] = [
        Self::Request,
        Self::Response,
        Self::Subscribe,
        Self::Unsubscribe,
        Self::Set,
        Self::Acknowledge,
        Self::Error,
        Self::Command,
        Self::Status,
        Self::Log,
    ];

    /// Exact uppercase wire token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Request => "REQ",
            Self::Response => "RES",
            Self::Subscribe => "SUB",
            Self::Unsubscribe => "UNSUB",
            Self::Set => "SET",
            Self::Acknowledge => "ACK",
            Self::Error => "ERR",
            Self::Command => "CMD",
            Self::Status => "STA",
            Self::Log => "LOG",
        }
    }

    /// Parses one exact, case-sensitive wire token.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|command| command.token() == token)
    }
}

/// Parameters in the pinned BAP wire vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapParameter {
    SystemInfo,
    Hashrate,
    Temperature,
    Power,
    Voltage,
    Current,
    Shares,
    Frequency,
    AsicVoltage,
    Ssid,
    Password,
    FanSpeed,
    AutoFan,
    BestDifficulty,
    BlockHeight,
    Wifi,
    FoundBlock,
    ShowNewBlock,
}

impl BapParameter {
    /// Complete vocabulary in reference enumeration order.
    pub const ALL: [Self; 18] = [
        Self::SystemInfo,
        Self::Hashrate,
        Self::Temperature,
        Self::Power,
        Self::Voltage,
        Self::Current,
        Self::Shares,
        Self::Frequency,
        Self::AsicVoltage,
        Self::Ssid,
        Self::Password,
        Self::FanSpeed,
        Self::AutoFan,
        Self::BestDifficulty,
        Self::BlockHeight,
        Self::Wifi,
        Self::FoundBlock,
        Self::ShowNewBlock,
    ];

    /// Exact case-sensitive wire token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::SystemInfo => "systemInfo",
            Self::Hashrate => "hashrate",
            Self::Temperature => "temperature",
            Self::Power => "power",
            Self::Voltage => "voltage",
            Self::Current => "current",
            Self::Shares => "shares",
            Self::Frequency => "frequency",
            Self::AsicVoltage => "asic_voltage",
            Self::Ssid => "ssid",
            Self::Password => "password",
            Self::FanSpeed => "fan_speed",
            Self::AutoFan => "auto_fan",
            Self::BestDifficulty => "best_difficulty",
            Self::BlockHeight => "block_height",
            Self::Wifi => "wifi",
            Self::FoundBlock => "found_block",
            Self::ShowNewBlock => "show_new_block",
        }
    }

    /// Parses one exact, case-sensitive wire token.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|parameter| parameter.token() == token)
    }
}

/// How an admitted frame satisfied the reference checksum contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapChecksumDisposition {
    Verified,
    SubscriptionMissingCompatibility,
    SubscriptionMismatchCompatibility,
}

/// A parsed or response-planned BAP frame.
#[derive(Clone, PartialEq, Eq)]
pub struct BapFrame {
    command: BapCommand,
    parameter_token: String,
    maybe_value: Option<String>,
    checksum_disposition: BapChecksumDisposition,
}

impl BapFrame {
    /// Constructs a canonical outgoing frame.
    pub fn new(
        command: BapCommand,
        parameter: BapParameter,
        maybe_value: Option<String>,
    ) -> Result<Self, BapFrameError> {
        Self::new_token(command, parameter.token(), maybe_value)
    }

    pub(crate) fn new_token(
        command: BapCommand,
        parameter_token: &str,
        maybe_value: Option<String>,
    ) -> Result<Self, BapFrameError> {
        validate_field(parameter_token)?;
        if let Some(value) = maybe_value.as_deref() {
            validate_field(value)?;
        }
        Ok(Self {
            command,
            parameter_token: parameter_token.to_owned(),
            maybe_value,
            checksum_disposition: BapChecksumDisposition::Verified,
        })
    }

    /// Parses one complete bounded BAP message.
    pub fn parse(input: &[u8]) -> Result<Self, BapFrameError> {
        if input.is_empty() {
            return Err(BapFrameError::Empty);
        }
        if input.len() >= BAP_MAX_MESSAGE_LEN {
            return Err(BapFrameError::TooLong);
        }

        let input = std::str::from_utf8(input).map_err(|_| BapFrameError::InvalidUtf8)?;
        let message = strip_line_ending(input);
        let Some(payload) = message.strip_prefix('$') else {
            return Err(BapFrameError::MissingStart);
        };

        let (body, maybe_checksum) = match payload.split_once('*') {
            Some((body, checksum)) => {
                if checksum.len() != 2 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err(BapFrameError::MalformedChecksum);
                }
                let checksum = u8::from_str_radix(checksum, 16)
                    .map_err(|_| BapFrameError::MalformedChecksum)?;
                (body, Some(checksum))
            }
            None => (payload, None),
        };

        let mut fields = body.split(',');
        if fields.next() != Some("BAP") {
            return Err(BapFrameError::InvalidTalker);
        }
        let command = fields
            .next()
            .and_then(BapCommand::from_token)
            .ok_or(BapFrameError::UnknownCommand)?;
        let parameter_token = fields.next().ok_or(BapFrameError::UnknownParameter)?;
        validate_field(parameter_token)?;
        let maybe_value = fields.next().map(str::to_owned);
        if fields.next().is_some() {
            return Err(BapFrameError::TooManyFields);
        }
        if let Some(value) = maybe_value.as_deref() {
            validate_field(value)?;
        }

        let expected_checksum = bap_checksum(body.as_bytes());
        let checksum_disposition = match (command, maybe_checksum) {
            (_, Some(received)) if received == expected_checksum => {
                BapChecksumDisposition::Verified
            }
            (BapCommand::Subscribe, Some(_)) => {
                BapChecksumDisposition::SubscriptionMismatchCompatibility
            }
            (BapCommand::Subscribe | BapCommand::Unsubscribe, None) => {
                BapChecksumDisposition::SubscriptionMissingCompatibility
            }
            (_, Some(_)) => return Err(BapFrameError::ChecksumMismatch),
            (_, None) => return Err(BapFrameError::MissingChecksum),
        };

        Ok(Self {
            command,
            parameter_token: parameter_token.to_owned(),
            maybe_value,
            checksum_disposition,
        })
    }

    /// Encodes this frame in canonical checksum-bearing form.
    pub fn encode(&self) -> Result<String, BapFrameError> {
        let mut body = format!("BAP,{},{}", self.command.token(), self.parameter_token);
        if let Some(value) = self.maybe_value.as_deref() {
            validate_field(value)?;
            body.push(',');
            body.push_str(value);
        }
        let checksum = bap_checksum(body.as_bytes());
        let message = format!("${body}*{checksum:02X}\r\n");
        if message.len() >= BAP_MAX_MESSAGE_LEN {
            return Err(BapFrameError::TooLong);
        }
        Ok(message)
    }

    #[must_use]
    pub const fn command(&self) -> BapCommand {
        self.command
    }

    #[must_use]
    /// Exact parameter token, including response-only tokens outside the fixed
    /// request vocabulary.
    pub fn parameter_token(&self) -> &str {
        &self.parameter_token
    }

    /// Returns the parameter when it belongs to the fixed request vocabulary.
    #[must_use]
    pub fn known_parameter(&self) -> Option<BapParameter> {
        BapParameter::from_token(&self.parameter_token)
    }

    #[must_use]
    /// Optional value. Callers must treat credential and network values as
    /// sensitive runtime data.
    pub fn value(&self) -> Option<&str> {
        self.maybe_value.as_deref()
    }

    #[must_use]
    pub const fn checksum_disposition(&self) -> BapChecksumDisposition {
        self.checksum_disposition
    }
}

impl fmt::Debug for BapFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BapFrame")
            .field("command", &self.command)
            .field("known_parameter", &self.known_parameter())
            .field("value_present", &self.maybe_value.is_some())
            .field(
                "value_bytes",
                &self.maybe_value.as_deref().map_or(0, str::len),
            )
            .field("checksum_disposition", &self.checksum_disposition)
            .finish()
    }
}

/// Closed parse/encoding failure categories with no raw wire values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BapFrameError {
    Empty,
    TooLong,
    InvalidUtf8,
    MissingStart,
    InvalidTalker,
    UnknownCommand,
    UnknownParameter,
    TooManyFields,
    InvalidField,
    MissingChecksum,
    MalformedChecksum,
    ChecksumMismatch,
}

/// XOR checksum over the sentence body, excluding `$`, `*`, and terminators.
#[must_use]
pub fn bap_checksum(body: &[u8]) -> u8 {
    body.iter().fold(0, |checksum, byte| checksum ^ byte)
}

fn strip_line_ending(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or_else(|| input.strip_suffix('\r'))
        .or_else(|| input.strip_suffix('\n'))
        .unwrap_or(input)
}

fn validate_field(field: &str) -> Result<(), BapFrameError> {
    if field.is_empty()
        || field
            .chars()
            .any(|character| matches!(character, ',' | '*' | '\r' | '\n' | '\0'))
    {
        return Err(BapFrameError::InvalidField);
    }
    Ok(())
}

#[derive(Clone, PartialEq, Eq)]
struct LastAdmission {
    wire: Vec<u8>,
    observed_at_ms: u64,
}

/// Stateful one-second duplicate admission matching the reference handler.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BapIngress {
    maybe_last: Option<LastAdmission>,
}

impl BapIngress {
    /// Parses and admits a complete frame, or reports a duplicate.
    pub fn admit(
        &mut self,
        input: &[u8],
        observed_at_ms: u64,
    ) -> Result<BapAdmission, BapFrameError> {
        let frame = BapFrame::parse(input)?;
        let duplicate = self.maybe_last.as_ref().is_some_and(|last| {
            input == last.wire
                && observed_at_ms >= last.observed_at_ms
                && observed_at_ms - last.observed_at_ms < BAP_DUPLICATE_WINDOW_MS
        });
        if duplicate {
            return Ok(BapAdmission::Duplicate);
        }

        self.maybe_last = Some(LastAdmission {
            wire: input.to_vec(),
            observed_at_ms,
        });
        Ok(BapAdmission::Accepted(frame))
    }
}

impl fmt::Debug for BapIngress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BapIngress")
            .field("has_last", &self.maybe_last.is_some())
            .field(
                "last_wire_bytes",
                &self.maybe_last.as_ref().map_or(0, |last| last.wire.len()),
            )
            .field(
                "last_observed_at_ms",
                &self.maybe_last.as_ref().map(|last| last.observed_at_ms),
            )
            .finish()
    }
}

/// Result of stateful BAP ingress admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BapAdmission {
    Accepted(BapFrame),
    Duplicate,
}
