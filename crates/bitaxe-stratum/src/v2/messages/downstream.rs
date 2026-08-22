use std::fmt;

use super::codec::Reader;
use super::{
    MessageType, OpenExtendedMiningChannel, OpenStandardMiningChannel, SetupConnection,
    SubmitSharesExtended, SubmitSharesStandard,
};
use crate::v2::frame::Frame;
use crate::v2::{StratumV2Error, MINING_PROTOCOL, PROTOCOL_VERSION};

#[derive(Clone, PartialEq)]
pub enum ClientMessage {
    SetupConnection(SetupConnection),
    OpenStandardMiningChannel(OpenStandardMiningChannel),
    OpenExtendedMiningChannel(OpenExtendedMiningChannel),
    SubmitSharesStandard(SubmitSharesStandard),
    SubmitSharesExtended(SubmitSharesExtended),
}

impl fmt::Debug for ClientMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SetupConnection(_) => "ClientMessage::SetupConnection",
            Self::OpenStandardMiningChannel(_) => "ClientMessage::OpenStandardMiningChannel",
            Self::OpenExtendedMiningChannel(_) => "ClientMessage::OpenExtendedMiningChannel",
            Self::SubmitSharesStandard(_) => "ClientMessage::SubmitSharesStandard",
            Self::SubmitSharesExtended(_) => "ClientMessage::SubmitSharesExtended",
        })
    }
}

impl ClientMessage {
    pub fn decode(frame: &Frame) -> Result<Self, StratumV2Error> {
        let message_type = MessageType::try_from(frame.header.message_type)?;
        let mut reader = Reader::new(frame.payload());
        let message = match message_type {
            MessageType::SetupConnection => Self::SetupConnection(parse_setup(&mut reader)?),
            MessageType::OpenStandardMiningChannel => {
                Self::OpenStandardMiningChannel(OpenStandardMiningChannel {
                    request_id: reader.u32("request_id")?,
                    user_identity: reader.str0255("user_identity")?,
                    nominal_hashrate: read_hashrate(&mut reader)?,
                    maximum_target: reader.fixed("maximum_target")?,
                })
            }
            MessageType::OpenExtendedMiningChannel => {
                let request_id = reader.u32("request_id")?;
                let user_identity = reader.str0255("user_identity")?;
                let nominal_hashrate = read_hashrate(&mut reader)?;
                let maximum_target = reader.fixed("maximum_target")?;
                let minimum_extranonce_size = reader.u16("minimum_extranonce_size")?;
                if minimum_extranonce_size > 32 {
                    return Err(StratumV2Error::InvalidField {
                        field: "minimum_extranonce_size",
                        reason: "exceeds 32 bytes",
                    });
                }
                Self::OpenExtendedMiningChannel(OpenExtendedMiningChannel {
                    request_id,
                    user_identity,
                    nominal_hashrate,
                    maximum_target,
                    minimum_extranonce_size,
                })
            }
            MessageType::SubmitSharesStandard => {
                Self::SubmitSharesStandard(parse_standard_submit(&mut reader)?)
            }
            MessageType::SubmitSharesExtended => {
                let standard = parse_standard_submit(&mut reader)?;
                Self::SubmitSharesExtended(SubmitSharesExtended {
                    channel_id: standard.channel_id,
                    sequence_number: standard.sequence_number,
                    job_id: standard.job_id,
                    nonce: standard.nonce,
                    ntime: standard.ntime,
                    version: standard.version,
                    extranonce: reader.bytes0255("extranonce", 32)?,
                })
            }
            unsupported => return Err(StratumV2Error::UnsupportedMessageType(unsupported as u8)),
        };
        reader.finish()?;
        Ok(message)
    }
}

fn parse_setup(reader: &mut Reader<'_>) -> Result<SetupConnection, StratumV2Error> {
    if reader.u8("protocol")? != MINING_PROTOCOL
        || reader.u16("minimum_version")? != PROTOCOL_VERSION
        || reader.u16("maximum_version")? != PROTOCOL_VERSION
    {
        return Err(StratumV2Error::InvalidField {
            field: "protocol_version",
            reason: "requires mining protocol version 2",
        });
    }
    Ok(SetupConnection {
        flags: reader.u32("flags")?,
        endpoint_host: reader.str0255("endpoint_host")?,
        endpoint_port: reader.u16("endpoint_port")?,
        vendor: reader.str0255("vendor")?,
        hardware_version: reader.str0255("hardware_version")?,
        firmware: reader.str0255("firmware")?,
        device_id: reader.str0255("device_id")?,
    })
}

fn parse_standard_submit(reader: &mut Reader<'_>) -> Result<SubmitSharesStandard, StratumV2Error> {
    Ok(SubmitSharesStandard {
        channel_id: reader.u32("channel_id")?,
        sequence_number: reader.u32("sequence_number")?,
        job_id: reader.u32("job_id")?,
        nonce: reader.u32("nonce")?,
        ntime: reader.u32("ntime")?,
        version: reader.u32("version")?,
    })
}

fn read_hashrate(reader: &mut Reader<'_>) -> Result<f32, StratumV2Error> {
    let hashrate = reader.f32("nominal_hashrate")?;
    if hashrate.is_finite() && hashrate > 0.0 {
        Ok(hashrate)
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
    use crate::v2::messages::ChannelKind;

    #[test]
    fn downstream_decoder_round_trips_each_pinned_client_message() {
        // Arrange
        let setup = SetupConnection {
            endpoint_host: "pool".to_owned(),
            endpoint_port: 3333,
            vendor: "bitaxe".to_owned(),
            hardware_version: "BM1366".to_owned(),
            firmware: String::new(),
            device_id: String::new(),
            flags: ChannelKind::Standard.setup_flags(),
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
        let setup_result = ClientMessage::decode(&setup.encode().expect("setup"));
        let submit_result = ClientMessage::decode(&submit.encode().expect("submit"));

        // Assert
        assert_eq!(setup_result, Ok(ClientMessage::SetupConnection(setup)));
        assert_eq!(
            submit_result,
            Ok(ClientMessage::SubmitSharesStandard(submit))
        );
    }
}
