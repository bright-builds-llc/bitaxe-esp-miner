use std::io::{ErrorKind, Read, Write};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_FRAME_BYTES: usize = 16 * 1024;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SequencedFrame<T> {
    sequence: u64,
    payload: T,
}

#[derive(Debug)]
pub struct Phase36BrokerFrameReceiver {
    next_sequence: u64,
    closed: bool,
}

impl Phase36BrokerFrameReceiver {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_sequence: 1,
            closed: false,
        }
    }

    pub fn read_next<T: DeserializeOwned>(
        &mut self,
        reader: &mut impl Read,
    ) -> Result<T, Phase36BrokerIpcError> {
        if self.closed {
            return Err(Phase36BrokerIpcError::AfterClose);
        }
        let bytes = read_frame_bytes(reader)?;
        let frame = serde_json::from_slice::<SequencedFrame<T>>(&bytes)
            .map_err(|_| Phase36BrokerIpcError::Encoding)?;
        if frame.sequence < self.next_sequence {
            return Err(Phase36BrokerIpcError::Duplicate);
        }
        if frame.sequence != self.next_sequence {
            return Err(Phase36BrokerIpcError::OutOfOrder);
        }
        self.next_sequence += 1;
        Ok(frame.payload)
    }

    pub fn close(&mut self) {
        self.closed = true;
    }
}

impl Default for Phase36BrokerFrameReceiver {
    fn default() -> Self {
        Self::new()
    }
}

pub fn write_broker_frame<T: Serialize>(
    writer: &mut impl Write,
    sequence: u64,
    payload: &T,
) -> Result<(), Phase36BrokerIpcError> {
    if sequence == 0 {
        return Err(Phase36BrokerIpcError::OutOfOrder);
    }
    let bytes = serde_json::to_vec(&SequencedFrame { sequence, payload })
        .map_err(|_| Phase36BrokerIpcError::Encoding)?;
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(Phase36BrokerIpcError::Oversized);
    }
    let length = u32::try_from(bytes.len()).map_err(|_| Phase36BrokerIpcError::Oversized)?;
    writer
        .write_all(&length.to_be_bytes())
        .and_then(|()| writer.write_all(&bytes))
        .and_then(|()| writer.flush())
        .map_err(|_| Phase36BrokerIpcError::Io)
}

fn read_frame_bytes(reader: &mut impl Read) -> Result<Vec<u8>, Phase36BrokerIpcError> {
    let mut header = [0_u8; 4];
    if let Err(error) = reader.read_exact(&mut header) {
        return Err(if error.kind() == ErrorKind::UnexpectedEof {
            Phase36BrokerIpcError::Truncated
        } else {
            Phase36BrokerIpcError::Io
        });
    }
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(Phase36BrokerIpcError::Oversized);
    }
    let mut bytes = vec![0_u8; length];
    reader.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            Phase36BrokerIpcError::Truncated
        } else {
            Phase36BrokerIpcError::Io
        }
    })?;
    Ok(bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Phase36BrokerIpcError {
    #[error("phase36_broker_ipc_duplicate")]
    Duplicate,
    #[error("phase36_broker_ipc_out_of_order")]
    OutOfOrder,
    #[error("phase36_broker_ipc_truncated")]
    Truncated,
    #[error("phase36_broker_ipc_oversized")]
    Oversized,
    #[error("phase36_broker_ipc_encoding_failed")]
    Encoding,
    #[error("phase36_broker_ipc_after_close")]
    AfterClose,
    #[error("phase36_broker_ipc_io_failed")]
    Io,
}
