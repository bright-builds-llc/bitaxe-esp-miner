//! BM1368 work payload and job-frame encoding.

use crate::bm1368::{
    protocol::{job_frame, Bm1368FrameBytes, CMD_WRITE, GROUP_SINGLE, JOB_HEADER_TYPE},
    Bm1368ProtocolFault,
};

pub const BM1368_JOB_PAYLOAD_LEN: usize = 82;
pub const BM1368_JOB_FRAME_LEN: usize = 88;
pub const BM1368_NUM_MIDSTATES: u8 = 0x01;
pub const JOB_ID_STEP: u8 = 24;
pub const JOB_ID_MODULUS: u8 = 128;

/// Job identifier used in transmitted BM1368 work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bm1368JobId(u8);

impl Bm1368JobId {
    #[must_use]
    pub const fn new(raw: u8) -> Self {
        Self(raw % JOB_ID_MODULUS)
    }

    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn advance(self) -> Self {
        Self::new((self.0 + JOB_ID_STEP) % JOB_ID_MODULUS)
    }
}

/// Pool-derived work fields encoded into one BM1368 job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1368WorkFields {
    pub starting_nonce: [u8; 4],
    pub nbits: [u8; 4],
    pub ntime: [u8; 4],
    pub merkle_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub version: [u8; 4],
}

/// Fixed-layout BM1368 work payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1368WorkPayload([u8; BM1368_JOB_PAYLOAD_LEN]);

impl Bm1368WorkPayload {
    #[must_use]
    pub fn new(job_id: Bm1368JobId, fields: Bm1368WorkFields) -> Self {
        let mut bytes = [0; BM1368_JOB_PAYLOAD_LEN];
        bytes[0] = job_id.raw();
        bytes[1] = BM1368_NUM_MIDSTATES;
        bytes[2..6].copy_from_slice(&fields.starting_nonce);
        bytes[6..10].copy_from_slice(&fields.nbits);
        bytes[10..14].copy_from_slice(&fields.ntime);
        bytes[14..46].copy_from_slice(&fields.merkle_root);
        bytes[46..78].copy_from_slice(&fields.prev_block_hash);
        bytes[78..82].copy_from_slice(&fields.version);
        Self(bytes)
    }

    #[must_use]
    pub const fn bytes(&self) -> &[u8; BM1368_JOB_PAYLOAD_LEN] {
        &self.0
    }
}

/// Encodes one complete BM1368 work frame.
pub fn encode_work_frame(
    job_id: Bm1368JobId,
    fields: Bm1368WorkFields,
) -> Result<Bm1368FrameBytes, Bm1368ProtocolFault> {
    let payload = Bm1368WorkPayload::new(job_id, fields);
    let frame = job_frame(JOB_HEADER_TYPE | GROUP_SINGLE | CMD_WRITE, payload.bytes())?;
    if frame.as_slice().len() != BM1368_JOB_FRAME_LEN {
        return Err(Bm1368ProtocolFault::InvalidLength {
            expected: BM1368_JOB_FRAME_LEN,
            actual: frame.as_slice().len(),
        });
    }
    Ok(frame)
}
