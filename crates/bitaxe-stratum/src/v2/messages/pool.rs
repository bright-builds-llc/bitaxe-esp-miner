use super::codec::Writer;
use super::{
    MessageType, NewExtendedMiningJob, NewMiningJob, OpenExtendedMiningChannelSuccess,
    OpenMiningChannelError, OpenStandardMiningChannelSuccess, SetNewPrevHash, SetTarget,
    SetupConnectionError, SetupConnectionSuccess, SubmitSharesError, SubmitSharesSuccess,
};
use crate::v2::frame::Frame;
use crate::v2::{StratumV2Error, CHANNEL_MESSAGE_FLAG, MAX_MERKLE_BRANCHES, PROTOCOL_VERSION};

impl SetupConnectionSuccess {
    pub fn encode(self) -> Result<Frame, StratumV2Error> {
        if self.used_version != PROTOCOL_VERSION {
            return Err(StratumV2Error::InvalidField {
                field: "used_version",
                reason: "must select protocol version 2",
            });
        }
        let mut payload = Writer::new();
        payload.u16(self.used_version);
        payload.u32(self.flags);
        Frame::new(
            0,
            MessageType::SetupConnectionSuccess as u8,
            payload.finish(),
        )
    }
}

impl SetupConnectionError {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.flags);
        payload.str0255("error_code", self.error_code())?;
        Frame::new(0, MessageType::SetupConnectionError as u8, payload.finish())
    }
}

impl OpenStandardMiningChannelSuccess {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.request_id);
        payload.u32(self.channel_id);
        payload.fixed(&self.target);
        payload.bytes0255("extranonce_prefix", &self.extranonce_prefix)?;
        payload.u32(self.group_channel_id);
        Frame::new(
            0,
            MessageType::OpenStandardMiningChannelSuccess as u8,
            payload.finish(),
        )
    }
}

impl OpenExtendedMiningChannelSuccess {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        if self.extranonce_size > 32 {
            return Err(StratumV2Error::InvalidField {
                field: "extranonce_size",
                reason: "exceeds 32 bytes",
            });
        }
        let mut payload = Writer::new();
        payload.u32(self.request_id);
        payload.u32(self.channel_id);
        payload.fixed(&self.target);
        payload.u16(self.extranonce_size);
        payload.bytes0255("extranonce_prefix", &self.extranonce_prefix)?;
        payload.u32(self.group_channel_id);
        Frame::new(
            0,
            MessageType::OpenExtendedMiningChannelSuccess as u8,
            payload.finish(),
        )
    }
}

impl OpenMiningChannelError {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.request_id);
        payload.str0255("error_code", self.error_code())?;
        Frame::new(
            0,
            MessageType::OpenMiningChannelError as u8,
            payload.finish(),
        )
    }
}

impl NewMiningJob {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.channel_id);
        payload.u32(self.job_id);
        write_option_u32(&mut payload, self.maybe_min_ntime);
        payload.u32(self.version);
        payload.fixed(&self.merkle_root);
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::NewMiningJob as u8,
            payload.finish(),
        )
    }
}

impl NewExtendedMiningJob {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        if self.merkle_path.len() > MAX_MERKLE_BRANCHES {
            return Err(StratumV2Error::InvalidField {
                field: "merkle_path",
                reason: "exceeds 20 branches",
            });
        }
        let mut payload = Writer::new();
        payload.u32(self.channel_id);
        payload.u32(self.job_id);
        write_option_u32(&mut payload, self.maybe_min_ntime);
        payload.u32(self.version);
        payload.u8(u8::from(self.version_rolling_allowed));
        payload.u8(self.merkle_path.len() as u8);
        for branch in &self.merkle_path {
            payload.fixed(branch);
        }
        payload.bytes064k("coinbase_prefix", &self.coinbase_prefix)?;
        payload.bytes064k("coinbase_suffix", &self.coinbase_suffix)?;
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::NewExtendedMiningJob as u8,
            payload.finish(),
        )
    }
}

impl SetNewPrevHash {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.channel_id);
        payload.u32(self.job_id);
        payload.fixed(&self.prev_hash);
        payload.u32(self.min_ntime);
        payload.u32(self.nbits);
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::SetNewPrevHash as u8,
            payload.finish(),
        )
    }
}

impl SetTarget {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.channel_id);
        payload.fixed(&self.maximum_target);
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::SetTarget as u8,
            payload.finish(),
        )
    }
}

impl SubmitSharesSuccess {
    pub fn encode(self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.channel_id);
        payload.u32(self.last_sequence_number);
        payload.u32(self.accepted_count);
        payload.u64(self.shares_sum);
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::SubmitSharesSuccess as u8,
            payload.finish(),
        )
    }
}

impl SubmitSharesError {
    pub fn encode(&self) -> Result<Frame, StratumV2Error> {
        let mut payload = Writer::new();
        payload.u32(self.channel_id);
        payload.u32(self.sequence_number);
        payload.str0255("error_code", self.error_code())?;
        Frame::new(
            CHANNEL_MESSAGE_FLAG,
            MessageType::SubmitSharesError as u8,
            payload.finish(),
        )
    }
}

fn write_option_u32(payload: &mut Writer, maybe_value: Option<u32>) {
    match maybe_value {
        Some(value) => {
            payload.u8(1);
            payload.u32(value);
        }
        None => payload.u8(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::messages::ServerMessage;

    #[test]
    fn pool_encoders_round_trip_pinned_success_job_target_and_share_messages() {
        // Arrange
        let messages = [
            SetupConnectionSuccess {
                used_version: 2,
                flags: 1,
            }
            .encode()
            .expect("setup"),
            NewMiningJob {
                channel_id: 1,
                job_id: 2,
                maybe_min_ntime: None,
                version: 3,
                merkle_root: [4; 32],
            }
            .encode()
            .expect("job"),
            SetTarget {
                channel_id: 1,
                maximum_target: [0xff; 32],
            }
            .encode()
            .expect("target"),
            SubmitSharesSuccess {
                channel_id: 1,
                last_sequence_number: 2,
                accepted_count: 1,
                shares_sum: 1,
            }
            .encode()
            .expect("share"),
        ];

        // Act
        let decoded = messages
            .iter()
            .map(ServerMessage::decode)
            .collect::<Result<Vec<_>, _>>();

        // Assert
        assert_eq!(decoded.expect("decode").len(), messages.len());
    }
}
