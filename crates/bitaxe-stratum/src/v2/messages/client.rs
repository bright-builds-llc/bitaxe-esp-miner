use std::fmt;

use super::codec::Writer;
use super::MessageType;
use crate::v2::frame::Frame;
use crate::v2::{StratumV2Error, CHANNEL_MESSAGE_FLAG, MINING_PROTOCOL, PROTOCOL_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    Standard,
    Extended,
}

impl ChannelKind {
    #[must_use]
    pub const fn setup_flags(self) -> u32 {
        match self {
            Self::Standard => 0x01,
            Self::Extended => 0,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SetupConnection {
    pub endpoint_host: String,
    pub endpoint_port: u16,
    pub vendor: String,
    pub hardware_version: String,
    pub firmware: String,
    pub device_id: String,
    pub flags: u32,
}

impl fmt::Debug for SetupConnection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetupConnection")
            .field("endpoint", &"redacted")
            .field("identity", &"redacted")
            .field("flags", &self.flags)
            .finish()
    }
}

impl SetupConnection {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u8(MINING_PROTOCOL);
        payload.u16(PROTOCOL_VERSION);
        payload.u16(PROTOCOL_VERSION);
        payload.u32(self.flags);
        payload.str0255("endpoint_host", &self.endpoint_host)?;
        payload.u16(self.endpoint_port);
        payload.str0255("vendor", &self.vendor)?;
        payload.str0255("hardware_version", &self.hardware_version)?;
        payload.str0255("firmware", &self.firmware)?;
        payload.str0255("device_id", &self.device_id)?;
        Frame::new(0, MessageType::SetupConnection as u8, payload.finish())
    }
}

#[derive(Clone, PartialEq)]
pub struct OpenStandardMiningChannel {
    pub request_id: u32,
    pub user_identity: String,
    pub nominal_hashrate: f32,
    pub maximum_target: [u8; 32],
}

impl fmt::Debug for OpenStandardMiningChannel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenStandardMiningChannel")
            .field("request_id", &self.request_id)
            .field("user_identity", &"redacted")
            .field("nominal_hashrate", &self.nominal_hashrate)
            .field("maximum_target", &"redacted")
            .finish()
    }
}

impl OpenStandardMiningChannel {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        require_hashrate(self.nominal_hashrate)?;
        let mut payload = Writer::new();
        payload.u32(self.request_id);
        payload.str0255("user_identity", &self.user_identity)?;
        payload.f32(self.nominal_hashrate);
        payload.fixed(&self.maximum_target);
        Frame::new(
            0,
            MessageType::OpenStandardMiningChannel as u8,
            payload.finish(),
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct OpenExtendedMiningChannel {
    pub request_id: u32,
    pub user_identity: String,
    pub nominal_hashrate: f32,
    pub maximum_target: [u8; 32],
    pub minimum_extranonce_size: u16,
}

impl fmt::Debug for OpenExtendedMiningChannel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenExtendedMiningChannel")
            .field("request_id", &self.request_id)
            .field("user_identity", &"redacted")
            .field("nominal_hashrate", &self.nominal_hashrate)
            .field("maximum_target", &"redacted")
            .field("minimum_extranonce_size", &self.minimum_extranonce_size)
            .finish()
    }
}

impl OpenExtendedMiningChannel {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        require_hashrate(self.nominal_hashrate)?;
        if self.minimum_extranonce_size > 32 {
            return Err(StratumV2Error::InvalidField {
                field: "minimum_extranonce_size",
                reason: "exceeds 32 bytes",
            });
        }
        let mut payload = Writer::new();
        payload.u32(self.request_id);
        payload.str0255("user_identity", &self.user_identity)?;
        payload.f32(self.nominal_hashrate);
        payload.fixed(&self.maximum_target);
        payload.u16(self.minimum_extranonce_size);
        Frame::new(
            0,
            MessageType::OpenExtendedMiningChannel as u8,
            payload.finish(),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SubmitSharesStandard {
    pub channel_id: u32,
    pub sequence_number: u32,
    pub job_id: u32,
    pub nonce: u32,
    pub ntime: u32,
    pub version: u32,
}

impl fmt::Debug for SubmitSharesStandard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SubmitSharesStandard")
            .field("channel_id", &self.channel_id)
            .field("sequence_number", &self.sequence_number)
            .field("share", &"redacted")
            .finish()
    }
}

impl SubmitSharesStandard {
    pub fn encode(self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        encode_submit_fields(&mut payload, self.into());
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::SubmitSharesStandard as u8,
            payload.finish(),
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SubmitSharesExtended {
    pub channel_id: u32,
    pub sequence_number: u32,
    pub job_id: u32,
    pub nonce: u32,
    pub ntime: u32,
    pub version: u32,
    pub extranonce: Vec<u8>,
}

impl fmt::Debug for SubmitSharesExtended {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SubmitSharesExtended")
            .field("channel_id", &self.channel_id)
            .field("sequence_number", &self.sequence_number)
            .field("share", &"redacted")
            .finish()
    }
}

impl SubmitSharesExtended {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        if self.extranonce.len() > 32 {
            return Err(StratumV2Error::InvalidField {
                field: "extranonce",
                reason: "exceeds 32 bytes",
            });
        }
        let mut payload = Writer::new();
        encode_submit_fields(&mut payload, self.into());
        payload.bytes0255("extranonce", &self.extranonce)?;
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::SubmitSharesExtended as u8,
            payload.finish(),
        )
    }
}

#[derive(Clone, Copy)]
struct SubmitFields {
    channel_id: u32,
    sequence_number: u32,
    job_id: u32,
    nonce: u32,
    ntime: u32,
    version: u32,
}

impl From<SubmitSharesStandard> for SubmitFields {
    fn from(value: SubmitSharesStandard) -> Self {
        Self {
            channel_id: value.channel_id,
            sequence_number: value.sequence_number,
            job_id: value.job_id,
            nonce: value.nonce,
            ntime: value.ntime,
            version: value.version,
        }
    }
}

impl From<&SubmitSharesExtended> for SubmitFields {
    fn from(value: &SubmitSharesExtended) -> Self {
        Self {
            channel_id: value.channel_id,
            sequence_number: value.sequence_number,
            job_id: value.job_id,
            nonce: value.nonce,
            ntime: value.ntime,
            version: value.version,
        }
    }
}

fn encode_submit_fields(payload: &mut Writer, fields: SubmitFields) {
    payload.u32(fields.channel_id);
    payload.u32(fields.sequence_number);
    payload.u32(fields.job_id);
    payload.u32(fields.nonce);
    payload.u32(fields.ntime);
    payload.u32(fields.version);
}

fn require_hashrate(value: f32) -> Result<(), StratumV2Error> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(StratumV2Error::InvalidField {
            field: "nominal_hashrate",
            reason: "must be finite and positive",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_connection_matches_pinned_reference_layout() {
        // Arrange
        let message = SetupConnection {
            endpoint_host: "pool".to_owned(),
            endpoint_port: 3333,
            vendor: "bitaxe".to_owned(),
            hardware_version: "BM1366".to_owned(),
            firmware: String::new(),
            device_id: String::new(),
            flags: ChannelKind::Standard.setup_flags(),
        };

        // Act
        let frame = message.encode().expect("setup must encode");

        // Assert
        assert_eq!(frame.header.message_type, 0x00);
        assert_eq!(&frame.payload()[..9], &[0, 2, 0, 2, 0, 1, 0, 0, 0]);
        assert_eq!(&frame.payload()[9..14], &[4, b'p', b'o', b'o', b'l']);
    }

    #[test]
    fn standard_submit_matches_fixed_reference_payload() {
        // Arrange
        let submit = SubmitSharesStandard {
            channel_id: 1,
            sequence_number: 2,
            job_id: 3,
            nonce: 4,
            ntime: 5,
            version: 6,
        };

        // Act
        let frame = submit.encode().expect("submit must encode");

        // Assert
        assert_eq!(frame.header.extension_type, CHANNEL_MESSAGE_FLAG);
        assert_eq!(frame.header.message_type, 0x1a);
        assert_eq!(frame.payload().len(), 24);
        assert_eq!(&frame.payload()[0..4], &1_u32.to_le_bytes());
        assert_eq!(&frame.payload()[20..24], &6_u32.to_le_bytes());
    }

    #[test]
    fn client_debug_output_redacts_pool_identity_and_share_fields() {
        // Arrange
        let setup = SetupConnection {
            endpoint_host: "private-pool-canary".to_owned(),
            endpoint_port: 1,
            vendor: "private-vendor-canary".to_owned(),
            hardware_version: String::new(),
            firmware: String::new(),
            device_id: "private-device-canary".to_owned(),
            flags: 0,
        };
        let submit = SubmitSharesStandard {
            channel_id: 1,
            sequence_number: 2,
            job_id: 3,
            nonce: 4,
            ntime: 5,
            version: 6,
        };

        // Act
        let rendered = format!("{setup:?} {submit:?}");

        // Assert
        assert!(!rendered.contains("private-pool-canary"));
        assert!(!rendered.contains("private-vendor-canary"));
        assert!(!rendered.contains("private-device-canary"));
        assert!(!rendered.contains("nonce: 4"));
    }
}
