//! BM1370 work payload, job identifiers and job-frame encoding.
//!
//! Reference: `reference/esp-miner/components/asic/bm1370.c` (`BM1370_send_work`) and
//! `components/asic/include/bm1370.h` (`BM1370_job`).

use crate::{
    bm1366::packet::{FrameBytes, JobFrame, CMD_WRITE, GROUP_SINGLE, JOB_HEADER_TYPE},
    bm1370::Bm1370ProtocolFault,
};

pub const BM1370_JOB_PAYLOAD_LEN: usize = 82;
pub const BM1370_JOB_FRAME_LEN: usize = 88;
pub const BM1370_NUM_MIDSTATES: u8 = 0x01;
/// Job ids advance by 24 modulo 128: sixteen ids, all multiples of eight.
pub const JOB_ID_STEP: u8 = 24;
pub const JOB_ID_MODULUS: u8 = 128;

/// Job identifier used in transmitted BM1370 work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bm1370JobId(u8);

impl Bm1370JobId {
    #[must_use]
    pub const fn new(raw: u8) -> Self {
        Self(raw % JOB_ID_MODULUS)
    }

    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// The id of the next job sent.
    #[must_use]
    pub const fn advance(self) -> Self {
        Self::new((self.0 + JOB_ID_STEP) % JOB_ID_MODULUS)
    }
}

/// Pool-derived work fields encoded into one BM1370 job, each already in wire byte order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1370WorkFields {
    pub starting_nonce: [u8; 4],
    pub nbits: [u8; 4],
    pub ntime: [u8; 4],
    pub merkle_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub version: [u8; 4],
}

/// Fixed-layout BM1370 work payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1370WorkPayload([u8; BM1370_JOB_PAYLOAD_LEN]);

impl Bm1370WorkPayload {
    #[must_use]
    pub fn new(job_id: Bm1370JobId, fields: Bm1370WorkFields) -> Self {
        let mut bytes = [0; BM1370_JOB_PAYLOAD_LEN];
        bytes[0] = job_id.raw();
        bytes[1] = BM1370_NUM_MIDSTATES;
        bytes[2..6].copy_from_slice(&fields.starting_nonce);
        bytes[6..10].copy_from_slice(&fields.nbits);
        bytes[10..14].copy_from_slice(&fields.ntime);
        bytes[14..46].copy_from_slice(&fields.merkle_root);
        bytes[46..78].copy_from_slice(&fields.prev_block_hash);
        bytes[78..82].copy_from_slice(&fields.version);
        Self(bytes)
    }

    #[must_use]
    pub const fn bytes(&self) -> &[u8; BM1370_JOB_PAYLOAD_LEN] {
        &self.0
    }
}

/// Encodes one complete BM1370 work frame.
pub fn encode_work_frame(
    job_id: Bm1370JobId,
    fields: Bm1370WorkFields,
) -> Result<FrameBytes, Bm1370ProtocolFault> {
    let payload = Bm1370WorkPayload::new(job_id, fields);
    let frame = JobFrame::new(JOB_HEADER_TYPE | GROUP_SINGLE | CMD_WRITE, payload.bytes())
        .map_err(|_| Bm1370ProtocolFault::InvalidLength {
            expected: BM1370_JOB_FRAME_LEN,
            actual: payload.bytes().len(),
        })?
        .into_bytes();
    if frame.as_slice().len() != BM1370_JOB_FRAME_LEN {
        return Err(Bm1370ProtocolFault::InvalidLength {
            expected: BM1370_JOB_FRAME_LEN,
            actual: frame.as_slice().len(),
        });
    }
    Ok(frame)
}
