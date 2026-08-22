//! Pinned-reference Stratum V2 mining messages.

mod client;
mod codec;
mod downstream;
mod pool;
mod server;

pub use client::{
    ChannelKind, OpenExtendedMiningChannel, OpenStandardMiningChannel, SetupConnection,
    SubmitSharesExtended, SubmitSharesStandard,
};
pub use downstream::ClientMessage;
pub use server::{
    NewExtendedMiningJob, NewMiningJob, OpenExtendedMiningChannelSuccess, OpenMiningChannelError,
    OpenStandardMiningChannelSuccess, ServerMessage, SetNewPrevHash, SetTarget,
    SetupConnectionError, SetupConnectionSuccess, SubmitSharesError, SubmitSharesSuccess,
};

use super::StratumV2Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    SetupConnection = 0x00,
    SetupConnectionSuccess = 0x01,
    SetupConnectionError = 0x02,
    OpenStandardMiningChannel = 0x10,
    OpenStandardMiningChannelSuccess = 0x11,
    OpenMiningChannelError = 0x12,
    OpenExtendedMiningChannel = 0x13,
    OpenExtendedMiningChannelSuccess = 0x14,
    NewMiningJob = 0x15,
    SubmitSharesStandard = 0x1a,
    SubmitSharesExtended = 0x1b,
    SubmitSharesSuccess = 0x1c,
    SubmitSharesError = 0x1d,
    NewExtendedMiningJob = 0x1f,
    SetNewPrevHash = 0x20,
    SetTarget = 0x21,
}

impl TryFrom<u8> for MessageType {
    type Error = StratumV2Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::SetupConnection),
            0x01 => Ok(Self::SetupConnectionSuccess),
            0x02 => Ok(Self::SetupConnectionError),
            0x10 => Ok(Self::OpenStandardMiningChannel),
            0x11 => Ok(Self::OpenStandardMiningChannelSuccess),
            0x12 => Ok(Self::OpenMiningChannelError),
            0x13 => Ok(Self::OpenExtendedMiningChannel),
            0x14 => Ok(Self::OpenExtendedMiningChannelSuccess),
            0x15 => Ok(Self::NewMiningJob),
            0x1a => Ok(Self::SubmitSharesStandard),
            0x1b => Ok(Self::SubmitSharesExtended),
            0x1c => Ok(Self::SubmitSharesSuccess),
            0x1d => Ok(Self::SubmitSharesError),
            0x1f => Ok(Self::NewExtendedMiningJob),
            0x20 => Ok(Self::SetNewPrevHash),
            0x21 => Ok(Self::SetTarget),
            other => Err(StratumV2Error::UnsupportedMessageType(other)),
        }
    }
}

#[cfg(test)]
mod golden_tests;
