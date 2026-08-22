use std::fmt;

use super::codec::Reader;
use super::MessageType;
use crate::v2::frame::Frame;
use crate::v2::{StratumV2Error, MAX_FRAME_PAYLOAD, MAX_MERKLE_BRANCHES, PROTOCOL_VERSION};

macro_rules! impl_redacted_channel_debug {
    ($type:ty, $name:literal) => {
        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct($name)
                    .field("request_id", &self.request_id)
                    .field("channel_id", &self.channel_id)
                    .field("target", &"redacted")
                    .field("extranonce", &"redacted")
                    .field("group_channel_id", &self.group_channel_id)
                    .finish()
            }
        }
    };
}

macro_rules! impl_redacted_error_debug {
    ($type:ty, $name:literal) => {
        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct($name)
                    .field("error_code", &"redacted")
                    .finish_non_exhaustive()
            }
        }
    };
}

macro_rules! impl_redacted_job_debug {
    ($type:ty, $name:literal) => {
        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct($name)
                    .field("job", &"redacted")
                    .finish()
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupConnectionSuccess {
    pub used_version: u16,
    pub flags: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SetupConnectionError {
    pub flags: u32,
    error_code: String,
}

impl SetupConnectionError {
    #[must_use]
    pub fn error_code(&self) -> &str {
        &self.error_code
    }
}

impl fmt::Debug for SetupConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetupConnectionError")
            .field("flags", &self.flags)
            .field("error_code", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct OpenStandardMiningChannelSuccess {
    pub request_id: u32,
    pub channel_id: u32,
    pub target: [u8; 32],
    pub extranonce_prefix: Vec<u8>,
    pub group_channel_id: u32,
}

impl_redacted_channel_debug!(
    OpenStandardMiningChannelSuccess,
    "OpenStandardMiningChannelSuccess"
);

#[derive(Clone, PartialEq, Eq)]
pub struct OpenExtendedMiningChannelSuccess {
    pub request_id: u32,
    pub channel_id: u32,
    pub target: [u8; 32],
    pub extranonce_size: u16,
    pub extranonce_prefix: Vec<u8>,
    pub group_channel_id: u32,
}

impl fmt::Debug for OpenExtendedMiningChannelSuccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenExtendedMiningChannelSuccess")
            .field("request_id", &self.request_id)
            .field("channel_id", &self.channel_id)
            .field("target", &"redacted")
            .field("extranonce", &"redacted")
            .field("extranonce_size", &self.extranonce_size)
            .field("group_channel_id", &self.group_channel_id)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct OpenMiningChannelError {
    pub request_id: u32,
    error_code: String,
}

impl OpenMiningChannelError {
    #[must_use]
    pub fn error_code(&self) -> &str {
        &self.error_code
    }
}

impl_redacted_error_debug!(OpenMiningChannelError, "OpenMiningChannelError");

#[derive(Clone, PartialEq, Eq)]
pub struct NewMiningJob {
    pub channel_id: u32,
    pub job_id: u32,
    pub maybe_min_ntime: Option<u32>,
    pub version: u32,
    pub merkle_root: [u8; 32],
}

impl_redacted_job_debug!(NewMiningJob, "NewMiningJob");

#[derive(Clone, PartialEq, Eq)]
pub struct NewExtendedMiningJob {
    pub channel_id: u32,
    pub job_id: u32,
    pub maybe_min_ntime: Option<u32>,
    pub version: u32,
    pub version_rolling_allowed: bool,
    pub merkle_path: Vec<[u8; 32]>,
    pub coinbase_prefix: Vec<u8>,
    pub coinbase_suffix: Vec<u8>,
}

impl_redacted_job_debug!(NewExtendedMiningJob, "NewExtendedMiningJob");

#[derive(Clone, PartialEq, Eq)]
pub struct SetNewPrevHash {
    pub channel_id: u32,
    pub job_id: u32,
    pub prev_hash: [u8; 32],
    pub min_ntime: u32,
    pub nbits: u32,
}

impl_redacted_job_debug!(SetNewPrevHash, "SetNewPrevHash");

#[derive(Clone, PartialEq, Eq)]
pub struct SetTarget {
    pub channel_id: u32,
    pub maximum_target: [u8; 32],
}

impl fmt::Debug for SetTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetTarget")
            .field("channel_id", &self.channel_id)
            .field("maximum_target", &"redacted")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubmitSharesSuccess {
    pub channel_id: u32,
    pub last_sequence_number: u32,
    pub accepted_count: u32,
    pub shares_sum: u64,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SubmitSharesError {
    pub channel_id: u32,
    pub sequence_number: u32,
    error_code: String,
}

impl SubmitSharesError {
    #[must_use]
    pub fn error_code(&self) -> &str {
        &self.error_code
    }
}

impl_redacted_error_debug!(SubmitSharesError, "SubmitSharesError");

#[derive(Clone, PartialEq, Eq)]
pub enum ServerMessage {
    SetupConnectionSuccess(SetupConnectionSuccess),
    SetupConnectionError(SetupConnectionError),
    OpenStandardMiningChannelSuccess(OpenStandardMiningChannelSuccess),
    OpenExtendedMiningChannelSuccess(OpenExtendedMiningChannelSuccess),
    OpenMiningChannelError(OpenMiningChannelError),
    NewMiningJob(NewMiningJob),
    NewExtendedMiningJob(NewExtendedMiningJob),
    SetNewPrevHash(SetNewPrevHash),
    SetTarget(SetTarget),
    SubmitSharesSuccess(SubmitSharesSuccess),
    SubmitSharesError(SubmitSharesError),
}

impl fmt::Debug for ServerMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SetupConnectionSuccess(_) => "ServerMessage::SetupConnectionSuccess",
            Self::SetupConnectionError(_) => "ServerMessage::SetupConnectionError",
            Self::OpenStandardMiningChannelSuccess(_) => {
                "ServerMessage::OpenStandardMiningChannelSuccess"
            }
            Self::OpenExtendedMiningChannelSuccess(_) => {
                "ServerMessage::OpenExtendedMiningChannelSuccess"
            }
            Self::OpenMiningChannelError(_) => "ServerMessage::OpenMiningChannelError",
            Self::NewMiningJob(_) => "ServerMessage::NewMiningJob",
            Self::NewExtendedMiningJob(_) => "ServerMessage::NewExtendedMiningJob",
            Self::SetNewPrevHash(_) => "ServerMessage::SetNewPrevHash",
            Self::SetTarget(_) => "ServerMessage::SetTarget",
            Self::SubmitSharesSuccess(_) => "ServerMessage::SubmitSharesSuccess",
            Self::SubmitSharesError(_) => "ServerMessage::SubmitSharesError",
        })
    }
}

impl ServerMessage {
    pub fn decode(frame: &Frame) -> Result<Self, StratumV2Error> {
        let message_type = MessageType::try_from(frame.header.message_type)?;
        let mut reader = Reader::new(frame.payload());
        let message = match message_type {
            MessageType::SetupConnectionSuccess => {
                let used_version = reader.u16("used_version")?;
                let flags = reader.u32("flags")?;
                if used_version != PROTOCOL_VERSION {
                    return Err(StratumV2Error::InvalidField {
                        field: "used_version",
                        reason: "does not select protocol version 2",
                    });
                }
                Self::SetupConnectionSuccess(SetupConnectionSuccess {
                    used_version,
                    flags,
                })
            }
            MessageType::SetupConnectionError => Self::SetupConnectionError(SetupConnectionError {
                flags: reader.u32("flags")?,
                error_code: reader.str0255("error_code")?,
            }),
            MessageType::OpenStandardMiningChannelSuccess => {
                Self::OpenStandardMiningChannelSuccess(parse_standard_channel(&mut reader)?)
            }
            MessageType::OpenExtendedMiningChannelSuccess => {
                Self::OpenExtendedMiningChannelSuccess(parse_extended_channel(&mut reader)?)
            }
            MessageType::OpenMiningChannelError => {
                Self::OpenMiningChannelError(OpenMiningChannelError {
                    request_id: reader.u32("request_id")?,
                    error_code: reader.str0255("error_code")?,
                })
            }
            MessageType::NewMiningJob => Self::NewMiningJob(parse_standard_job(&mut reader)?),
            MessageType::NewExtendedMiningJob => {
                Self::NewExtendedMiningJob(parse_extended_job(&mut reader)?)
            }
            MessageType::SetNewPrevHash => Self::SetNewPrevHash(SetNewPrevHash {
                channel_id: reader.u32("channel_id")?,
                job_id: reader.u32("job_id")?,
                prev_hash: reader.fixed("prev_hash")?,
                min_ntime: reader.u32("min_ntime")?,
                nbits: reader.u32("nbits")?,
            }),
            MessageType::SetTarget => Self::SetTarget(SetTarget {
                channel_id: reader.u32("channel_id")?,
                maximum_target: reader.fixed("maximum_target")?,
            }),
            MessageType::SubmitSharesSuccess => Self::SubmitSharesSuccess(SubmitSharesSuccess {
                channel_id: reader.u32("channel_id")?,
                last_sequence_number: reader.u32("last_sequence_number")?,
                accepted_count: reader.u32("accepted_count")?,
                shares_sum: reader.u64("shares_sum")?,
            }),
            MessageType::SubmitSharesError => Self::SubmitSharesError(SubmitSharesError {
                channel_id: reader.u32("channel_id")?,
                sequence_number: reader.u32("sequence_number")?,
                error_code: reader.str0255("error_code")?,
            }),
            unsupported => return Err(StratumV2Error::UnsupportedMessageType(unsupported as u8)),
        };
        reader.finish()?;
        Ok(message)
    }
}

fn parse_standard_channel(
    reader: &mut Reader<'_>,
) -> Result<OpenStandardMiningChannelSuccess, StratumV2Error> {
    Ok(OpenStandardMiningChannelSuccess {
        request_id: reader.u32("request_id")?,
        channel_id: reader.u32("channel_id")?,
        target: reader.fixed("target")?,
        extranonce_prefix: reader.bytes0255("extranonce_prefix", 32)?,
        group_channel_id: reader.u32("group_channel_id")?,
    })
}

fn parse_extended_channel(
    reader: &mut Reader<'_>,
) -> Result<OpenExtendedMiningChannelSuccess, StratumV2Error> {
    let request_id = reader.u32("request_id")?;
    let channel_id = reader.u32("channel_id")?;
    let target = reader.fixed("target")?;
    let extranonce_size = reader.u16("extranonce_size")?;
    if extranonce_size > 32 {
        return Err(StratumV2Error::InvalidField {
            field: "extranonce_size",
            reason: "exceeds 32 bytes",
        });
    }
    Ok(OpenExtendedMiningChannelSuccess {
        request_id,
        channel_id,
        target,
        extranonce_size,
        extranonce_prefix: reader.bytes0255("extranonce_prefix", 32)?,
        group_channel_id: reader.u32("group_channel_id")?,
    })
}

fn parse_standard_job(reader: &mut Reader<'_>) -> Result<NewMiningJob, StratumV2Error> {
    let channel_id = reader.u32("channel_id")?;
    let job_id = reader.u32("job_id")?;
    let maybe_min_ntime = read_option_u32(reader, "min_ntime")?;
    Ok(NewMiningJob {
        channel_id,
        job_id,
        maybe_min_ntime,
        version: reader.u32("version")?,
        merkle_root: reader.fixed("merkle_root")?,
    })
}

fn parse_extended_job(reader: &mut Reader<'_>) -> Result<NewExtendedMiningJob, StratumV2Error> {
    let channel_id = reader.u32("channel_id")?;
    let job_id = reader.u32("job_id")?;
    let maybe_min_ntime = read_option_u32(reader, "min_ntime")?;
    let version = reader.u32("version")?;
    let version_rolling_allowed = match reader.u8("version_rolling_allowed")? {
        0 => false,
        1 => true,
        _ => {
            return Err(StratumV2Error::InvalidField {
                field: "version_rolling_allowed",
                reason: "must be encoded as zero or one",
            })
        }
    };
    let branch_count = usize::from(reader.u8("merkle_path")?);
    if branch_count > MAX_MERKLE_BRANCHES {
        return Err(StratumV2Error::InvalidField {
            field: "merkle_path",
            reason: "exceeds 20 branches",
        });
    }
    let mut merkle_path = Vec::with_capacity(branch_count);
    for _ in 0..branch_count {
        merkle_path.push(reader.fixed("merkle_path")?);
    }
    Ok(NewExtendedMiningJob {
        channel_id,
        job_id,
        maybe_min_ntime,
        version,
        version_rolling_allowed,
        merkle_path,
        coinbase_prefix: reader.bytes064k("coinbase_prefix", MAX_FRAME_PAYLOAD)?,
        coinbase_suffix: reader.bytes064k("coinbase_suffix", MAX_FRAME_PAYLOAD)?,
    })
}

fn read_option_u32(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> Result<Option<u32>, StratumV2Error> {
    match reader.u8(field)? {
        0 => Ok(None),
        1 => reader.u32(field).map(Some),
        _ => Err(StratumV2Error::InvalidField {
            field,
            reason: "option flag must be zero or one",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(message_type: MessageType, payload: Vec<u8>) -> Frame {
        Frame::new(0, message_type as u8, payload).expect("bounded frame")
    }

    #[test]
    fn decodes_pinned_setup_and_submit_success_shapes() {
        // Arrange
        let mut setup = Vec::new();
        setup.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        setup.extend_from_slice(&1_u32.to_le_bytes());
        let mut submit = Vec::new();
        submit.extend_from_slice(&7_u32.to_le_bytes());
        submit.extend_from_slice(&8_u32.to_le_bytes());
        submit.extend_from_slice(&9_u32.to_le_bytes());
        submit.extend_from_slice(&10_u64.to_le_bytes());

        // Act
        let setup_result =
            ServerMessage::decode(&frame(MessageType::SetupConnectionSuccess, setup));
        let submit_result = ServerMessage::decode(&frame(MessageType::SubmitSharesSuccess, submit));

        // Assert
        assert_eq!(
            setup_result,
            Ok(ServerMessage::SetupConnectionSuccess(
                SetupConnectionSuccess {
                    used_version: 2,
                    flags: 1,
                }
            ))
        );
        assert_eq!(
            submit_result,
            Ok(ServerMessage::SubmitSharesSuccess(SubmitSharesSuccess {
                channel_id: 7,
                last_sequence_number: 8,
                accepted_count: 9,
                shares_sum: 10,
            }))
        );
    }

    #[test]
    fn rejects_malformed_option_branch_and_trailing_boundaries() {
        // Arrange
        let mut invalid_option = vec![0; 8];
        invalid_option.push(2);
        invalid_option.extend_from_slice(&[0; 36]);
        let mut too_many_branches = vec![0; 8];
        too_many_branches.push(0);
        too_many_branches.extend_from_slice(&0_u32.to_le_bytes());
        too_many_branches.push(0);
        too_many_branches.push(21);
        let mut trailing_success = PROTOCOL_VERSION.to_le_bytes().to_vec();
        trailing_success.extend_from_slice(&0_u32.to_le_bytes());
        trailing_success.push(0);

        // Act
        let option_result =
            ServerMessage::decode(&frame(MessageType::NewMiningJob, invalid_option));
        let branches_result =
            ServerMessage::decode(&frame(MessageType::NewExtendedMiningJob, too_many_branches));
        let trailing_result = ServerMessage::decode(&frame(
            MessageType::SetupConnectionSuccess,
            trailing_success,
        ));

        // Assert
        assert!(matches!(
            option_result,
            Err(StratumV2Error::InvalidField {
                field: "min_ntime",
                ..
            })
        ));
        assert!(matches!(
            branches_result,
            Err(StratumV2Error::InvalidField {
                field: "merkle_path",
                ..
            })
        ));
        assert_eq!(trailing_result, Err(StratumV2Error::TrailingPayload));
    }

    #[test]
    fn server_debug_output_never_renders_raw_job_target_or_error_values() {
        // Arrange
        let message = ServerMessage::SubmitSharesError(SubmitSharesError {
            channel_id: 1,
            sequence_number: 2,
            error_code: "private-error-canary".to_owned(),
        });
        let target = SetTarget {
            channel_id: 1,
            maximum_target: [0x44; 32],
        };

        // Act
        let rendered = format!("{message:?} {target:?}");

        // Assert
        assert!(!rendered.contains("private-error-canary"));
        assert!(!rendered.contains("44, 44"));
    }
}
